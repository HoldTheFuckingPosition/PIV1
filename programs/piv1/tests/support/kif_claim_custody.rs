#![allow(dead_code)]
//! Four-account isolated host composition. No pool, round, registry or unrelated
//! custody is instantiated or checked here. AccountInfo signer flags are supplied
//! host evidence, not a transaction signature or proof of runtime invocation.

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent},
    solana_program::system_program,
    AnchorDeserialize, AnchorSerialize,
};
use piv1::{
    accounts::{CONFIG_DISCRIMINATOR, STAKE_PROGRAM_ID},
    constants::*,
    errors::{Piv1Error, Piv1Result},
    kif_claim_accounts::*,
    state::*,
};

pub const CONFIG: usize = 0;
pub const REWARD: usize = 1;
pub const KIF: usize = 2;
pub const GUARDIAN: usize = 3;
// Non-deployed public fixture bytes; no keypair or signature is created.
pub const PROGRAM: Pubkey = Pubkey::new_from_array([217; 32]);
pub fn key(tag: u8) -> Pubkey { Pubkey::new_from_array([tag; 32]) }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackingAccount {
    pub key: Pubkey,
    pub owner: Pubkey,
    pub executable: bool,
    pub signer: bool,
    pub writable: bool,
    pub lamports: u64,
    pub data: Vec<u8>,
}

impl BackingAccount {
    pub fn info(&mut self) -> AccountInfo<'_> {
        AccountInfo::new(&self.key, self.signer, self.writable, &mut self.lamports,
                        &mut self.data, &self.owner, self.executable, 0)
    }
}

pub fn envelope<T: AnchorSerialize>(value: &T, discriminator: [u8; 8], space: usize) -> Vec<u8> {
    let mut data = discriminator.to_vec();
    data.extend(value.try_to_vec().unwrap());
    assert!(data.len() <= space);
    data.resize(space, 0);
    data
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Failure {
    AfterPrepare,
    AfterDebit,
    MissingCredit,
    WrongCredit,
    AfterCredit,
    StateChanged,
    AfterState,
    InvalidPostEnvelope,
    BeforeCommit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error { State(Piv1Error), Injected, Conservation }
impl From<Piv1Error> for Error { fn from(error: Piv1Error) -> Self { Self::State(error) } }
pub type Result<T> = core::result::Result<T, Error>;

/// Original baseline established exactly once; later credits/receipts are flows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Audit {
    initial_native: u128,
    initial_source: u64,
    initial_destination: u64,
    initial_global_credited: u64,
    initial_global_claimed: u64,
    initial_global_liability: u64,
    initial_reward: GuardianReward,
    initial_carry: u64,
    pub credited: u128,
    pub external_excess: u128,
    pub paid: u128,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fixture {
    pub program_id: Pubkey,
    pub rent: Rent,
    pub accounts: [BackingAccount; 4],
    pub audit: Audit,
    pub failure: Option<Failure>,
}

impl Fixture {
    pub fn new(excess: u64) -> Self {
        let config = configuration();
        let reward = GuardianReward {
            version: STATE_LAYOUT_VERSION, bump: 0, guardian_index: 2,
            registry_revision: 7, guardian: key(91), last_active_period: None,
            claimable_lamports: 300, cumulative_earned: 310, cumulative_claimed: 10,
        };
        Self::from_earned(config, reward, excess)
    }

    /// Supplemental compatibility imports already-earned state from a healthy
    /// lifecycle into a newly funded/baselined isolated fixture. This is not
    /// continuous cross-World custody evidence; unrelated accounts are omitted.
    pub fn from_earned(mut config: PivConfig, mut reward: GuardianReward, excess: u64) -> Self {
        let rent = Rent::default();
        let (config_key, bump) = Pubkey::find_program_address(&[b"config"], &PROGRAM);
        config.bumps.config = bump;
        let (source, bump) = Pubkey::find_program_address(&[b"kif-sol"], &PROGRAM);
        config.kif_sol_vault = source;
        config.bumps.kif_sol_vault = bump;
        config.system_program = system_program::ID;
        config.token_program = spl_token::ID;
        config.stake_program = STAKE_PROGRAM_ID;
        let (reward_key, bump) = Pubkey::find_program_address(&[
            b"guardian-reward", reward.guardian.as_ref(),
            &reward.registry_revision.to_le_bytes(), &[reward.guardian_index],
        ], &PROGRAM);
        reward.bump = bump;
        let state = |key, data: Vec<u8>| BackingAccount {
            key, owner: PROGRAM, executable: false, signer: false, writable: true,
            lamports: rent.minimum_balance(data.len()), data,
        };
        let source_lamports = rent.minimum_balance(0)
            .checked_add(config.kif_claim_liability_lamports).unwrap()
            .checked_add(config.collective_kif_carry_lamports).unwrap()
            .checked_add(excess).unwrap();
        let accounts = [
            state(config_key, envelope(&config, CONFIG_DISCRIMINATOR, PivConfig::SPACE)),
            state(reward_key, envelope(&reward, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE)),
            BackingAccount { key: source, owner: system_program::ID, executable: false,
                signer: false, writable: true, lamports: source_lamports, data: vec![] },
            BackingAccount { key: reward.guardian, owner: system_program::ID, executable: false,
                signer: true, writable: true, lamports: 11, data: vec![] },
        ];
        let audit = Audit {
            initial_native: accounts.iter().map(|a| u128::from(a.lamports)).sum(),
            initial_source: source_lamports, initial_destination: 11,
            initial_global_credited: config.cumulative_kif_credited_lamports,
            initial_global_claimed: config.cumulative_kif_claimed_lamports,
            initial_global_liability: config.kif_claim_liability_lamports,
            initial_reward: reward, initial_carry: config.collective_kif_carry_lamports,
            credited: 0, external_excess: 0, paid: 0,
        };
        let mut fixture = Self { program_id: PROGRAM, rent, accounts, audit, failure: None };
        fixture.authenticate().unwrap();
        fixture.validate_audit().unwrap();
        fixture
    }

    pub fn with_infos<T>(&mut self, action: impl FnOnce(KifClaimAccountInfos<'_, '_>) -> T) -> T {
        let [c, r, k, g] = &mut self.accounts;
        let c = c.info(); let r = r.info(); let k = k.info(); let g = g.info();
        action(KifClaimAccountInfos { config: &c, guardian_reward: &r, kif_sol: &k, guardian: &g })
    }

    pub fn authenticate(&mut self) -> Piv1Result<AuthenticatedKifClaimAccounts> {
        let program = self.program_id;
        let rent = self.rent.clone();
        self.with_infos(|accounts| authenticate_kif_claim_accounts(&program, &rent, accounts))
    }

    pub fn config(&self) -> PivConfig {
        PivConfig::deserialize(&mut &self.accounts[CONFIG].data[8..]).unwrap()
    }
    pub fn reward(&self) -> GuardianReward {
        GuardianReward::deserialize(&mut &self.accounts[REWARD].data[8..]).unwrap()
    }
    pub fn update_config(&mut self, edit: impl FnOnce(&mut PivConfig)) {
        let mut c = self.config(); edit(&mut c);
        self.accounts[CONFIG].data = envelope(&c, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    }
    pub fn update_reward(&mut self, edit: impl FnOnce(&mut GuardianReward)) {
        let mut r = self.reward(); edit(&mut r);
        self.accounts[REWARD].data = envelope(&r, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE);
    }

    pub fn claim(&mut self, amount: u64, expected_claimed: u64) -> Result<KifClaimTransfer> {
        let mut next = self.clone();
        let before = next.authenticate()?;
        let plan = prepare_kif_claim(before.config(), before.reward(), KifClaimRequest {
            amount_lamports: amount, expected_cumulative_claimed: expected_claimed,
        }, before.custody())?;
        if next.failure == Some(Failure::AfterPrepare) { return Err(Error::Injected); }
        let transfer = plan.transfer();
        if transfer.source != next.accounts[KIF].key || transfer.destination != next.accounts[GUARDIAN].key {
            return Err(Error::Conservation);
        }
        next.accounts[KIF].lamports = next.accounts[KIF].lamports.checked_sub(transfer.amount_lamports)
            .ok_or(Error::Conservation)?;
        if next.failure == Some(Failure::AfterDebit) { return Err(Error::Injected); }
        if next.failure != Some(Failure::MissingCredit) {
            let destination = if next.failure == Some(Failure::WrongCredit) { CONFIG } else { GUARDIAN };
            next.accounts[destination].lamports = next.accounts[destination].lamports
                .checked_add(transfer.amount_lamports).ok_or(Piv1Error::ArithmeticOverflow)?;
        }
        if next.failure == Some(Failure::AfterCredit) { return Err(Error::Injected); }
        if next.failure == Some(Failure::StateChanged) { next.update_config(|c| c.paused = !c.paused); }
        // Structural post-transfer auth deliberately precedes the new state writes.
        // Complete post backing belongs to plan.commit's staged new liabilities.
        let after = next.authenticate()?;
        let mut config = after.config().clone();
        let mut reward = *after.reward();
        let result = plan.commit(&mut config, &mut reward, after.custody())?;
        next.accounts[CONFIG].data = envelope(&config, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
        next.accounts[REWARD].data = envelope(&reward, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE);
        if next.failure == Some(Failure::AfterState) { return Err(Error::Injected); }
        if next.failure == Some(Failure::InvalidPostEnvelope) { next.accounts[REWARD].data[0] ^= 1; }
        let committed = next.authenticate()?;
        if *committed.config() != config || *committed.reward() != reward
            || committed.custody() != after.custody()
        {
            return Err(Error::Conservation);
        }
        let paid = before.custody().kif_sol.lamports.checked_sub(after.custody().kif_sol.lamports)
            .ok_or(Error::Conservation)?;
        next.audit.paid = next.audit.paid.checked_add(u128::from(paid)).ok_or(Error::Conservation)?;
        next.validate_audit()?;
        if next.failure == Some(Failure::BeforeCommit) { return Err(Error::Injected); }
        *self = next;
        Ok(result)
    }

    /// Modeled old-tuple credit and explicit native funding for replay tests.
    /// Account ownership is authenticated, but earning eligibility and an external
    /// earning snapshot are not. This is not a production credit handler.
    pub fn credit_snapshot(&mut self, amount: u64) -> Result<()> {
        let mut next = self.clone();
        let state = next.authenticate()?;
        let mut c = state.config().clone();
        let mut r = *state.reward();
        r.credit_snapshot(r.guardian, r.guardian_index, r.registry_revision, amount)?;
        c.kif_claim_liability_lamports = c.kif_claim_liability_lamports.checked_add(amount)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        c.cumulative_kif_credited_lamports = c.cumulative_kif_credited_lamports.checked_add(amount)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.accounts[KIF].lamports = next.accounts[KIF].lamports.checked_add(amount)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.accounts[CONFIG].data = envelope(&c, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
        next.accounts[REWARD].data = envelope(&r, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE);
        next.audit.credited += u128::from(amount);
        next.authenticate()?;
        next.validate_audit()?;
        *self = next;
        Ok(())
    }

    pub fn unsolicited_excess(&mut self, amount: u64) -> Result<()> {
        let mut next = self.clone();
        next.accounts[KIF].lamports = next.accounts[KIF].lamports.checked_add(amount)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.audit.external_excess += u128::from(amount);
        next.validate_audit()?;
        *self = next;
        Ok(())
    }

    pub fn validate_audit(&self) -> Result<()> {
        let c = self.config(); let r = self.reward();
        let a = &self.audit;
        let total: u128 = self.accounts.iter().map(|account| u128::from(account.lamports)).sum();
        if a.initial_native + a.credited + a.external_excess != total
            || u128::from(a.initial_source) + a.credited + a.external_excess
                != u128::from(self.accounts[KIF].lamports) + a.paid
            || u128::from(a.initial_destination) + a.paid != u128::from(self.accounts[GUARDIAN].lamports)
            || u128::from(a.initial_global_claimed) + a.paid != u128::from(c.cumulative_kif_claimed_lamports)
            || u128::from(a.initial_global_credited) + a.credited != u128::from(c.cumulative_kif_credited_lamports)
            || u128::from(a.initial_global_liability) + a.credited != u128::from(c.kif_claim_liability_lamports) + a.paid
            || u128::from(a.initial_reward.cumulative_claimed) + a.paid != u128::from(r.cumulative_claimed)
            || u128::from(a.initial_reward.cumulative_earned) + a.credited != u128::from(r.cumulative_earned)
            || u128::from(a.initial_reward.claimable_lamports) + a.credited != u128::from(r.claimable_lamports) + a.paid
            || c.collective_kif_carry_lamports != a.initial_carry
        {
            return Err(Error::Conservation);
        }
        Ok(())
    }
}

fn configuration() -> PivConfig {
    PivConfig {
        version: STATE_LAYOUT_VERSION, is_initialized: true, paused: false,
        bumps: PivConfigBumps { config: 1, piv_authority: 2, active_distribution: 3,
            principal_jito_vault: 4, pending_jito_vault: 5, pending_sol_vault: 6,
            principal_sol_queue: 7, operational_sol_vault: 8, distribution_escrow: 9,
            kif_sol_vault: 10, guardian_registry: 11 },
        stake_pool_program: key(1), stake_pool: key(2), validator_list: key(3),
        reserve_stake: key(4), jitosol_mint: key(5), token_program: spl_token::ID,
        stake_program: STAKE_PROGRAM_ID, system_program: system_program::ID,
        manager_fee_account: key(9), referrer_token_account: key(10),
        piv_authority: key(11), active_distribution: key(12), principal_jito_vault: key(13),
        pending_jito_vault: key(14), pending_sol_vault: key(15), principal_sol_queue: key(16),
        operational_sol_vault: key(17), distribution_escrow: key(18), kif_sol_vault: key(19),
        htfp_recipient: key(20), team_owner_recipient: key(21), guardian_registry: key(22),
        basis_points_denominator: 10_000, htfp_reserve_bps: 5_900, permanent_compound_bps: 1_950,
        team_owner_pool_bps: 1_950, kif_bps: 200, configured_slippage_bps: 1, slippage_hard_cap_bps: 1,
        minimum_distribution_interval_seconds: MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        insufficient_retry_cooldown_seconds: INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        last_successful_preparation_at: Some(900_000), last_valid_insufficient_attempt_at: Some(800_000),
        next_distribution_sequence: 42, protected_principal_hwm_lamports: 1_000_000,
        accounted_historical_jitosol_units: 1_000_000, accounted_historical_sol_lamports: 707,
        accounted_pending_jitosol_units: 31, accounted_pending_sol_lamports: 53,
        next_cycle_yield_lamports: 13, kif_claim_liability_lamports: 960,
        collective_kif_carry_lamports: 73, cumulative_contribution_value_lamports: 10_000,
        cumulative_gross_yield_lamports: 5_000, cumulative_htfp_paid_lamports: 2_950,
        cumulative_team_owner_paid_lamports: 975, cumulative_kif_credited_lamports: 1_000,
        cumulative_kif_claimed_lamports: 40, cumulative_permanent_compound_lamports: 975,
        cumulative_retained_dust_lamports: 3, cumulative_zero_active_kif_compound_lamports: 17,
        cumulative_cooldown_yield_recorded_lamports: 13, kif_anchor_timestamp: 0,
        kif_period_seconds: KIF_PERIOD_SECONDS, guardian_registry_revision: 99,
        migration_reserve: [0; CONFIG_MIGRATION_RESERVE_BYTES],
    }
}
