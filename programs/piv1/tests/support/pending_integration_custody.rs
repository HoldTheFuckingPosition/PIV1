//! Independent integration oracle: full contribution P vs physical P-U, exact
//! token/SOL transfers, current floor valuation, normalized Idle summary.
use anchor_lang::{prelude::{Clock, Pubkey, SolanaSysvar}, AnchorDeserialize,
    solana_program::{instruction::{Instruction, AccountMeta}, system_program}};
use piv1::{accounts::{CONFIG_DISCRIMINATOR, DISTRIBUTION_DISCRIMINATOR},
    events::PendingContributionsIntegrated,
    state::{ActiveDistribution, CompletedDistributionSummary, PivConfig}};
use super::{custody::{self, *}, settlement_custody, leg_custody::Call,
    bootstrap_custody::POOL, oracle, support::kif_claim_custody::{envelope, PROGRAM}};
#[derive(Clone, Debug, PartialEq)]
pub struct Fixture { pub custody: custody::Fixture, pub clock: Clock, pub base: usize }
impl Fixture {
    pub fn new(shared: bool, bitmap: u8, withdrawal: bool, cooldown: u64) -> Self {
        Self::from_settlement(settlement_custody::Fixture::new(shared, bitmap, withdrawal, cooldown))
    }
    pub fn from_settlement(f: settlement_custody::Fixture) -> Self {
        let mut f = settlement_custody::expected(&f).after;
        f.custody.accounts.truncate(f.base + 1);
        for i in [CONFIG, ROUND, PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, PRINCIPAL_JITO, PENDING_JITO] {
            f.custody.accounts[i].writable = true;
        }
        Self { custody: f.custody, clock: f.clock, base: f.base }
    }
    pub fn data(&self) -> Vec<u8> { b"PIV1IP01\x01".to_vec() }
    pub fn round(&self) -> ActiveDistribution { ActiveDistribution::deserialize(&mut &self.custody.accounts[ROUND].data[8..]).unwrap() }
    pub fn set_round(&mut self, r: &ActiveDistribution) { self.custody.accounts[ROUND].data = envelope(r, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SPACE); }
    pub fn pool(&self) -> oracle::StakePool { borsh1::BorshDeserialize::deserialize(&mut &self.custody.accounts[POOL].data[..]).unwrap() }
    pub fn set_pool(&mut self, p: &oracle::StakePool) { let bytes = borsh1::to_vec(p).unwrap(); self.custody.accounts[POOL].data[..bytes.len()].copy_from_slice(&bytes); }
    pub fn sync_clock(&mut self) { self.clock.to_account_info(&mut self.custody.accounts[self.base].info()).unwrap(); }
}
pub struct Expected { pub after: Fixture, pub calls: Vec<Call>, pub event: PendingContributionsIntegrated }
pub fn expected(f: &Fixture) -> Expected {
    let mut out = f.clone(); let old = f.round(); let mut c = f.custody.config(); let p = f.pool();
    let floor = f.custody.rent.minimum_balance(0);
    let full_sol = c.accounted_pending_sol_lamports; let tokens = c.accounted_pending_jitosol_units;
    let value = |q: u64| if p.pool_token_supply == 0 { 0 } else {
        (u128::from(q) * u128::from(p.total_lamports) / u128::from(p.pool_token_supply)) as u64 };
    let contribution = full_sol + value(tokens);
    let mut calls = vec![];
    for (source, seed) in [(PENDING_SOL, b"pending-sol".as_slice()), (ESCROW_SOL, b"distribution-escrow".as_slice())] {
        let amount = out.custody.accounts[source].lamports - floor;
        if amount == 0 { continue; }
        out.custody.accounts[source].lamports -= amount; out.custody.accounts[PRINCIPAL_SOL].lamports += amount;
        let mut data = vec![2, 0, 0, 0]; data.extend(amount.to_le_bytes());
        calls.push(Call { ix: Instruction { program_id: system_program::ID, data, accounts: vec![
            AccountMeta::new(f.custody.accounts[source].key, true), AccountMeta::new(f.custody.accounts[PRINCIPAL_SOL].key, false)] },
            infos: vec![source, PRINCIPAL_SOL, SYSTEM], seeds: vec![vec![seed.to_vec(), vec![Pubkey::find_program_address(&[seed], &PROGRAM).1]]],
            after: out.custody.accounts.clone() });
    }
    if tokens != 0 {
        out.custody.set_token_units(PENDING_JITO, 0);
        out.custody.set_token_units(PRINCIPAL_JITO, out.custody.token_units(PRINCIPAL_JITO) + tokens);
        let mut data = vec![12]; data.extend(tokens.to_le_bytes()); data.push(f.custody.accounts[MINT].data[44]);
        calls.push(Call { ix: Instruction { program_id: spl_token::ID, data, accounts: vec![
            AccountMeta::new(f.custody.accounts[PENDING_JITO].key, false),
            AccountMeta::new_readonly(f.custody.accounts[MINT].key, false),
            AccountMeta::new(f.custody.accounts[PRINCIPAL_JITO].key, false),
            AccountMeta::new_readonly(f.custody.accounts[AUTHORITY].key, true)] },
            infos: vec![PENDING_JITO, MINT, PRINCIPAL_JITO, AUTHORITY, TOKEN],
            seeds: vec![vec![b"authority".to_vec(), vec![Pubkey::find_program_address(&[b"authority"], &PROGRAM).1]]],
            after: out.custody.accounts.clone() });
    }
    c.accounted_pending_sol_lamports = 0; c.accounted_pending_jitosol_units = 0;
    c.accounted_historical_sol_lamports = out.custody.accounts[PRINCIPAL_SOL].lamports - floor - c.next_cycle_yield_lamports;
    c.accounted_historical_jitosol_units = out.custody.token_units(PRINCIPAL_JITO);
    c.protected_principal_hwm_lamports += contribution; c.cumulative_contribution_value_lamports += contribution;
    let summary = CompletedDistributionSummary { sequence: old.active_sequence, completed_at: f.clock.unix_timestamp,
        gross_yield_lamports: old.gross_yield_lamports, actual_allocated_outgoing_lamports: old.actual_allocated_outgoing_lamports,
        integrated_contribution_value_lamports: contribution, final_protected_hwm_lamports: c.protected_principal_hwm_lamports,
        fixed_jitosol_withdrawal_target_units: old.fixed_jitosol_withdrawal_target_units, successful_leg_count: old.successful_leg_count,
        cumulative_cooldown_rewards_lamports: old.cumulative_cooldown_rewards_lamports,
        actual_kif_liability_lamports: old.actual_kif_liability_lamports, actual_kif_carry_next_lamports: old.actual_kif_carry_next_lamports };
    let mut idle = ActiveDistribution::new_idle(old.bump); idle.last_completed = Some(summary);
    out.set_round(&idle); out.custody.accounts[CONFIG].data = envelope(&c, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    Expected { after: out, calls, event: PendingContributionsIntegrated { config: f.custody.accounts[CONFIG].key,
        sequence: old.active_sequence, integrated_sol_lamports: full_sol, integrated_jitosol_units: tokens,
        contribution_value_lamports: contribution, protected_hwm_lamports: c.protected_principal_hwm_lamports } }
}
