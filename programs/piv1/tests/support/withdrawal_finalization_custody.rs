//! Independent byte/lamport accounting oracle; initiation uses its prior fixture
//! oracle, and finalization never calls production transition/proof/persistence.
use anchor_lang::{prelude::{Clock, Pubkey}, AnchorDeserialize,
    solana_program::{instruction::{AccountMeta, Instruction}, system_program, sysvar}};
use solana_stake_interface::state::StakeStateV2;
use piv1::{accounts::{CONFIG_DISCRIMINATOR, DISTRIBUTION_DISCRIMINATOR, STAKE_PROGRAM_ID},
    events::WithdrawalLegFinalized, state::{ActiveDistribution, DistributionLifecycle, WithdrawalLeg},
    constants::{RECOVERY_FLAG_COOLDOWN_LOSS, RECOVERY_FLAG_RESIDUAL_HWM}};
use super::{custody::{self, *}, leg_custody::{self, Call}, oracle,
    bootstrap_custody::POOL, support::kif_claim_custody::{BackingAccount, envelope, PROGRAM}};

#[derive(Clone, Debug, PartialEq)]
pub struct Fixture { pub custody: custody::Fixture, pub clock: Clock, pub base: usize, pub index: u64 }
impl Fixture {
    pub fn new(shared: bool) -> Self { Self::from_leg(leg_custody::Fixture::new(shared)) }
    pub fn from_leg(f: leg_custody::Fixture) -> Self {
        let mut f = leg_custody::expected(&f).after;
        let b = f.base;
        // Keep Clock and Stake program; replace source/withdraw authority with
        // History and retain the initiated metadata/Stake accounts.
        f.custody.accounts.remove(b + 3);
        f.custody.accounts[b + 2] = BackingAccount { key: sysvar::stake_history::ID,
            owner: sysvar::ID, lamports: 1, data: vec![0; 8], signer: false, writable: false, executable: false };
        let index = f.round().next_leg_index - 1;
        let mut out = Self { custody: f.custody, clock: f.clock, base: b, index };
        out.set_epoch(out.clock.epoch + 1);
        for i in [CONFIG, ROUND, PENDING_SOL, OPERATIONAL_SOL, ESCROW_SOL, b + 3, b + 4] {
            out.custody.accounts[i].writable = true;
        }
        out
    }
    pub fn leg(&self) -> WithdrawalLeg { WithdrawalLeg::deserialize(&mut &self.custody.accounts[self.base + 3].data[8..]).unwrap() }
    pub fn round(&self) -> ActiveDistribution { ActiveDistribution::deserialize(&mut &self.custody.accounts[ROUND].data[8..]).unwrap() }
    pub fn set_round(&mut self, r: &ActiveDistribution) { self.custody.accounts[ROUND].data = envelope(r, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SPACE); }
    pub fn set_leg(&mut self, leg: &WithdrawalLeg) {
        self.custody.accounts[self.base + 3].data = envelope(leg, [23, 0, 86, 144, 250, 142, 73, 170], WithdrawalLeg::SPACE);
    }
    pub fn pool(&self) -> oracle::StakePool { borsh1::BorshDeserialize::deserialize(&mut &self.custody.accounts[POOL].data[..]).unwrap() }
    pub fn set_pool(&mut self, p: &oracle::StakePool) {
        let bytes = borsh1::to_vec(p).unwrap(); self.custody.accounts[POOL].data[..bytes.len()].copy_from_slice(&bytes);
    }
    pub fn stake(&self) -> StakeStateV2 { StakeStateV2::deserialize(&mut &self.custody.accounts[self.base + 4].data[..]).unwrap() }
    pub fn set_stake(&mut self, s: &StakeStateV2) {
        let data = borsh1::to_vec(s).unwrap(); self.custody.accounts[self.base + 4].data[..data.len()].copy_from_slice(&data);
    }
    pub fn set_epoch(&mut self, epoch: u64) {
        self.clock.epoch = epoch;
        self.custody.accounts[self.base].data[16..24].copy_from_slice(&epoch.to_le_bytes());
        let mut p = self.pool(); p.last_update_epoch = epoch; self.set_pool(&p);
        let mut history = 1_u64.to_le_bytes().to_vec(); history.extend((epoch - 1).to_le_bytes());
        history.extend(1_000_000_u64.to_le_bytes()); history.extend(0_u64.to_le_bytes()); history.extend(100_u64.to_le_bytes());
        self.custody.accounts[self.base + 2].data = history;
    }
    pub fn reward(&mut self, reward: u64) {
        let StakeStateV2::Stake(meta, mut stake, flags) = self.stake() else { panic!() };
        stake.delegation.stake += reward; stake.credits_observed += 1;
        self.set_stake(&StakeStateV2::Stake(meta, stake, flags));
        self.custody.accounts[self.base + 4].lamports += reward;
    }
    pub fn data(&self) -> Vec<u8> { let mut d = b"PIV1FL01".to_vec(); d.push(1); d.extend(self.index.to_le_bytes()); d }
}

pub struct Expected { pub after: Fixture, pub calls: Vec<Call>, pub event: WithdrawalLegFinalized }
fn signer(seed: &[u8]) -> Vec<Vec<Vec<u8>>> {
    vec![vec![seed.to_vec(), vec![Pubkey::find_program_address(&[seed], &PROGRAM).1]]]
}
pub fn expected(f: &Fixture) -> Expected {
    let mut out = f.clone(); let b = f.base; let leg = f.leg(); let mut r = f.round();
    let mut c = f.custody.config(); let mut calls = vec![];
    let native = f.custody.accounts[b + 4].lamports;
    let net = native - leg.stake_rent_advanced_lamports;
    let metadata_excess = f.custody.accounts[b + 3].lamports - leg.metadata_rent_advanced_lamports;
    let reward = net.saturating_sub(leg.observed_delegated_native_lamports);
    let loss = leg.observed_delegated_native_lamports.saturating_sub(net);
    let p = f.pool(); let unused_carry = r.prior_next_cycle_yield_lamports - r.prior_next_cycle_yield_used_lamports().unwrap();
    let residual = c.accounted_historical_sol_lamports + unused_carry +
        (u128::from(c.accounted_historical_jitosol_units - r.fixed_jitosol_withdrawal_target_units)
            * u128::from(p.total_lamports) / u128::from(p.pool_token_supply)) as u64;
    out.custody.accounts[b + 4].lamports = 0; out.custody.accounts[b + 4].data.clear();
    out.custody.accounts[ESCROW_SOL].lamports += native;
    let mut data = vec![4, 0, 0, 0]; data.extend(native.to_le_bytes());
    calls.push(Call { ix: Instruction { program_id: STAKE_PROGRAM_ID, data, accounts: vec![
        AccountMeta::new(f.custody.accounts[b + 4].key, false), AccountMeta::new(c.distribution_escrow, false),
        AccountMeta::new_readonly(sysvar::clock::ID, false), AccountMeta::new_readonly(sysvar::stake_history::ID, false),
        AccountMeta::new_readonly(c.piv_authority, true)] }, infos: vec![b + 4, ESCROW_SOL, b, b + 2, AUTHORITY, b + 1],
        seeds: signer(b"authority"), after: out.custody.accounts.clone() });
    out.custody.accounts[ESCROW_SOL].lamports -= leg.stake_rent_advanced_lamports;
    out.custody.accounts[OPERATIONAL_SOL].lamports += leg.stake_rent_advanced_lamports;
    let mut data = vec![2, 0, 0, 0]; data.extend(leg.stake_rent_advanced_lamports.to_le_bytes());
    calls.push(Call { ix: Instruction { program_id: system_program::ID, data, accounts: vec![
        AccountMeta::new(c.distribution_escrow, true), AccountMeta::new(c.operational_sol_vault, false)] },
        infos: vec![ESCROW_SOL, OPERATIONAL_SOL, SYSTEM], seeds: signer(b"distribution-escrow"), after: out.custody.accounts.clone() });
    out.custody.accounts[b + 3].lamports = 0; out.custody.accounts[b + 3].data.fill(0);
    out.custody.accounts[OPERATIONAL_SOL].lamports += leg.metadata_rent_advanced_lamports;
    out.custody.accounts[PENDING_SOL].lamports += metadata_excess;
    c.accounted_pending_sol_lamports += metadata_excess;
    let mut flags = 0;
    if loss > 0 { flags |= RECOVERY_FLAG_COOLDOWN_LOSS; }
    if residual < r.stored_residual_hwm_floor_lamports { flags |= RECOVERY_FLAG_RESIDUAL_HWM; }
    r.cumulative_finalized_delegated_native_lamports += leg.observed_delegated_native_lamports;
    r.cumulative_finalized_native_lamports += native;
    r.cumulative_recovered_stake_rent_lamports += leg.stake_rent_advanced_lamports;
    r.cumulative_recovered_metadata_rent_lamports += leg.metadata_rent_advanced_lamports;
    r.cumulative_cooldown_rewards_lamports += reward; r.cumulative_cooldown_losses_lamports += loss;
    r.finalized_leg_count += 1; r.recorded_escrow_available_lamports += net;
    if flags != 0 { r.lifecycle = DistributionLifecycle::RecoveryRequired; r.recovery_flags |= flags; }
    else if r.cumulative_jitosol_assigned_units == r.fixed_jitosol_withdrawal_target_units && r.finalized_leg_count == r.successful_leg_count {
        r.lifecycle = DistributionLifecycle::EscrowFunded;
    }
    out.set_round(&r);
    out.custody.accounts[CONFIG].data = envelope(&c, CONFIG_DISCRIMINATOR, piv1::state::PivConfig::SPACE);
    Expected { after: out, calls, event: WithdrawalLegFinalized { config: f.custody.accounts[CONFIG].key,
        sequence: leg.sequence, leg_index: leg.leg_index, finalized_native_lamports: native,
        recovered_stake_rent_lamports: leg.stake_rent_advanced_lamports,
        recovered_metadata_rent_lamports: leg.metadata_rent_advanced_lamports,
        cooldown_reward_lamports: reward, cooldown_loss_lamports: loss,
        normalized_metadata_excess_lamports: metadata_excess, recovery_flags: flags } }
}
