//! Independent settlement oracle: fixed arithmetic, full accounts and exact
//! state fields. Previous preparation runs its existing host boundary; withdrawal
//! setup reuses the independently encoded initiation/finalization fixture oracles.
use anchor_lang::{prelude::{Clock, Pubkey}, AnchorDeserialize,
    solana_program::{instruction::{Instruction, AccountMeta}, system_program}};
use piv1::{accounts::{CONFIG_DISCRIMINATOR, DISTRIBUTION_DISCRIMINATOR},
    constants::RECOVERY_FLAG_RESIDUAL_HWM,
    events::{DistributionSettled, DistributionSettlementRecovery},
    kif_claim_accounts::GUARDIAN_REWARD_DISCRIMINATOR,
    settlement_execution::SettlementEvent,
    state::{ActiveDistribution, DistributionLifecycle, GuardianReward, PivConfig}};
use super::{custody::{self, *}, preparation_custody, withdrawal_custody, leg_custody::{self, Call},
    final_custody, oracle, bootstrap_custody::POOL,
    support::kif_claim_custody::{BackingAccount, envelope, PROGRAM}};

#[derive(Clone, Debug, PartialEq)]
pub struct Fixture { pub custody: custody::Fixture, pub clock: Clock, pub base: usize }
impl Fixture {
    pub fn new(shared: bool, bitmap: u8, withdrawal: bool, cooldown: u64) -> Self {
        if withdrawal {
            let mut w = withdrawal_custody::Fixture::new(321, 117, shared);
            let b = w.inner.guardian_start;
            for i in 0..6 {
                let mut reward = w.inner.rewards[i]; reward.last_active_period = if bitmap & (1 << i) != 0 { Some(0) } else { None };
                w.inner.base.custody.accounts[b + 1 + i].data = envelope(&reward, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE);
            }
            let rewards = w.inner.base.custody.accounts[b + 1..b + 7].to_vec();
            let mut f = final_custody::Fixture::from_leg(leg_custody::Fixture::from_preparation(w));
            f.reward(cooldown); let f = final_custody::expected(&f).after;
            let clock_account = f.custody.accounts[b].clone();
            Self::finish(f.custody, f.clock, b, clock_account, rewards)
        } else { Self::liquid_gross(shared, bitmap, None) }
    }
    pub fn liquid_gross(shared: bool, bitmap: u8, gross: Option<u64>) -> Self {
            let mut f = preparation_custody::Fixture::new(100_000, 117, shared, bitmap);
            if let Some(gross) = gross {
                let p = &f.base.pool;
                let c = f.base.custody.config();
                let value = c.accounted_historical_sol_lamports +
                    (u128::from(c.accounted_historical_jitosol_units) * u128::from(p.total_lamports) / u128::from(p.pool_token_supply)) as u64;
                f.base.custody.edit_config(|c| c.protected_principal_hwm_lamports = value + c.next_cycle_yield_lamports - gross);
            }
            let b = f.guardian_start; let clock = f.base.clock.clone(); let rent = f.base.custody.rent.clone();
            let mut data = b"PIV1PD01".to_vec(); data.push(1);
            f.base.custody.with_infos(|a| piv1::distribution_preparation_execution::process_instruction_with_host_callbacks(
                &PROGRAM, a, &data, || Ok(clock.clone()), || Ok(rent), |ix, infos, _| {
                    assert_eq!(ix.program_id, system_program::ID); assert_eq!(ix.data[..4], 2_u32.to_le_bytes());
                    let amount = u64::from_le_bytes(ix.data[4..].try_into().unwrap());
                    **infos[0].try_borrow_mut_lamports()? -= amount; **infos[1].try_borrow_mut_lamports()? += amount; Ok(())
                }, |_| {})).unwrap();
            let rewards = f.base.custody.accounts[b + 1..b + 7].to_vec();
            let clock_account = f.base.custody.accounts[b + 7].clone();
            Self::finish(f.base.custody, clock, b, clock_account, rewards)
    }
    fn finish(mut custody: custody::Fixture, clock: Clock, base: usize, clock_account: BackingAccount,
        rewards: Vec<BackingAccount>) -> Self {
        custody.accounts.truncate(base); custody.accounts.push(clock_account);
        let c = custody.config();
        for key in [c.htfp_recipient, c.team_owner_recipient] {
            custody.accounts.push(BackingAccount { key, owner: system_program::ID, executable: false,
                signer: false, writable: true, lamports: custody.rent.minimum_balance(0) + 37, data: vec![] });
        }
        custody.accounts.extend(rewards);
        for i in [CONFIG, ROUND, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL, base + 3, base + 4, base + 5, base + 6, base + 7, base + 8] {
            custody.accounts[i].writable = true;
        }
        Self { custody, clock, base }
    }
    pub fn data(&self) -> Vec<u8> { b"PIV1SD01\x01".to_vec() }
    pub fn round(&self) -> ActiveDistribution { ActiveDistribution::deserialize(&mut &self.custody.accounts[ROUND].data[8..]).unwrap() }
    pub fn set_round(&mut self, r: &ActiveDistribution) { self.custody.accounts[ROUND].data = envelope(r, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SPACE); }
    pub fn reward(&self, i: usize) -> GuardianReward { GuardianReward::deserialize(&mut &self.custody.accounts[self.base + 3 + i].data[8..]).unwrap() }
    pub fn set_reward(&mut self, i: usize, r: &GuardianReward) { self.custody.accounts[self.base + 3 + i].data = envelope(r, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE); }
    pub fn pool(&self) -> oracle::StakePool { borsh1::BorshDeserialize::deserialize(&mut &self.custody.accounts[POOL].data[..]).unwrap() }
    pub fn set_pool(&mut self, p: &oracle::StakePool) { let bytes = borsh1::to_vec(p).unwrap(); self.custody.accounts[POOL].data[..bytes.len()].copy_from_slice(&bytes); }
}
#[derive(Debug)]
pub struct Amounts { pub net: u64, pub htfp: u64, pub team: u64, pub kif: u64, pub total: u64,
    pub dust: u64, pub conservative: u64, pub per_guardian: u64, pub liability: u64, pub carry: u64,
    pub zero: u64, pub hwm_delta: u64, pub hwm: u64, pub protected: u64, pub recovery: bool }
pub fn amounts(f: &Fixture) -> Amounts {
    let r = f.round(); let c = f.custody.config(); let p = f.pool();
    let prior_used = r.prior_next_cycle_yield_lamports.min(r.outgoing_gross_obligation_lamports - r.pending_sol_used_lamports);
    let eligible = r.pending_sol_used_lamports + prior_used + r.cumulative_finalized_delegated_native_lamports;
    let net = eligible.min(r.outgoing_gross_obligation_lamports);
    let portion = |weight, cap| ((u128::from(net) * weight / 8050) as u64).min(cap);
    let htfp = portion(5900, r.htfp_gross_obligation_lamports);
    let team = portion(1950, r.team_owner_gross_obligation_lamports);
    let kif = portion(200, r.kif_gross_obligation_lamports); let total = htfp + team + kif;
    let dust = net - total; let conservative = eligible - net;
    let available = kif + r.kif_carry_input_lamports;
    let per_guardian = if r.kif_active_guardian_count == 0 { 0 } else { available / u64::from(r.kif_active_guardian_count) };
    let liability = per_guardian * u64::from(r.kif_active_guardian_count);
    let zero = if r.kif_active_guardian_count == 0 { available / 2 } else { 0 };
    let carry = available - liability - zero;
    let hwm_delta = r.proposed_hwm_delta_lamports + dust + conservative + zero;
    let hwm = r.old_protected_principal_lamports + hwm_delta;
    let floor = f.custody.rent.minimum_balance(0);
    let physical_principal = f.custody.accounts[PRINCIPAL_SOL].lamports - floor + zero;
    let units = f.custody.token_units(PRINCIPAL_JITO);
    let token_value = (u128::from(units) * u128::from(p.total_lamports) / u128::from(p.pool_token_supply)) as u64;
    let escrow = f.custody.accounts[ESCROW_SOL].lamports - floor - total - r.cumulative_cooldown_rewards_lamports;
    let retained = physical_principal + token_value + escrow;
    let protected = retained.saturating_sub(r.pending_sol_used_lamports);
    assert_eq!(c.protected_principal_hwm_lamports, r.old_protected_principal_lamports);
    Amounts { net, htfp, team, kif, total, dust, conservative, per_guardian, liability, carry,
        zero, hwm_delta, hwm, protected, recovery: protected < hwm }
}
pub struct Expected { pub after: Fixture, pub calls: Vec<Call>, pub event: SettlementEvent }
pub fn expected(f: &Fixture) -> Expected {
    let n = amounts(f); let mut out = f.clone(); let mut r = f.round(); let mut c = f.custody.config();
    let mut calls = vec![];
    if n.recovery {
        r.lifecycle = DistributionLifecycle::RecoveryRequired; r.recovery_flags |= RECOVERY_FLAG_RESIDUAL_HWM;
        out.set_round(&r);
        return Expected { after: out, calls, event: SettlementEvent::Recovery(DistributionSettlementRecovery {
            config: f.custody.accounts[CONFIG].key, sequence: r.active_sequence,
            observed_protected_value_lamports: n.protected, recovery_flags: RECOVERY_FLAG_RESIDUAL_HWM }) };
    }
    for (source, destination, amount, seed) in [(ESCROW_SOL, f.base + 1, n.htfp, b"distribution-escrow".as_slice()),
        (ESCROW_SOL, f.base + 2, n.team, b"distribution-escrow".as_slice()), (ESCROW_SOL, KIF_SOL, n.kif, b"distribution-escrow".as_slice()),
        (KIF_SOL, PRINCIPAL_SOL, n.zero, b"kif-sol".as_slice())] {
        if amount == 0 { continue; }
        out.custody.accounts[source].lamports -= amount; out.custody.accounts[destination].lamports += amount;
        let mut data = vec![2, 0, 0, 0]; data.extend(amount.to_le_bytes());
        calls.push(Call { ix: Instruction { program_id: system_program::ID, data, accounts: vec![
            AccountMeta::new(f.custody.accounts[source].key, true), AccountMeta::new(f.custody.accounts[destination].key, false)] },
            infos: vec![source, destination, SYSTEM], seeds: vec![vec![seed.to_vec(), vec![Pubkey::find_program_address(&[seed], &PROGRAM).1]]],
            after: out.custody.accounts.clone() });
    }
    for i in 0..6 { let mut reward = out.reward(i);
        if r.kif_eligibility_bitmap & (1 << i) != 0 { reward.claimable_lamports += n.per_guardian; reward.cumulative_earned += n.per_guardian; }
        out.set_reward(i, &reward);
    }
    r.actual_net_available_lamports = n.net; r.actual_htfp_lamports = n.htfp; r.actual_team_owner_lamports = n.team;
    r.actual_kif_allocation_lamports = n.kif; r.actual_net_allocation_dust_lamports = n.dust;
    r.actual_allocated_outgoing_lamports = n.total; r.actual_escrow_remainder_lamports = r.recorded_escrow_available_lamports - n.total;
    r.actual_retained_conservative_dust_lamports = n.conservative; r.actual_kif_liability_lamports = n.liability;
    r.actual_kif_carry_next_lamports = n.carry; r.actual_zero_active_kif_compound_lamports = n.zero;
    r.actual_hwm_delta_lamports = n.hwm_delta; r.settled_protected_hwm_lamports = n.hwm;
    r.outstanding_active_round_liability_lamports = 0; r.settlement_recorded = true; r.lifecycle = DistributionLifecycle::Settled;
    c.protected_principal_hwm_lamports = n.hwm; c.cumulative_gross_yield_lamports += r.gross_yield_lamports;
    c.cumulative_htfp_paid_lamports += n.htfp; c.cumulative_team_owner_paid_lamports += n.team;
    c.cumulative_kif_credited_lamports += n.liability; c.kif_claim_liability_lamports += n.liability;
    c.collective_kif_carry_lamports = n.carry; c.cumulative_permanent_compound_lamports += r.permanent_compound_lamports;
    c.cumulative_retained_dust_lamports += r.split_dust_lamports + r.snapshot_conversion_dust_lamports + n.dust + n.conservative;
    c.cumulative_zero_active_kif_compound_lamports += n.zero;
    c.cumulative_cooldown_yield_recorded_lamports += r.cumulative_cooldown_rewards_lamports;
    c.next_cycle_yield_lamports += r.cumulative_cooldown_rewards_lamports;
    out.set_round(&r); out.custody.accounts[CONFIG].data = envelope(&c, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    Expected { after: out, calls, event: SettlementEvent::Settled(DistributionSettled {
        config: f.custody.accounts[CONFIG].key, sequence: r.active_sequence, htfp_lamports: n.htfp,
        team_owner_lamports: n.team, kif_allocation_lamports: n.kif, kif_liability_lamports: n.liability,
        kif_carry_lamports: n.carry, zero_active_compound_lamports: n.zero, protected_hwm_lamports: n.hwm }) }
}
