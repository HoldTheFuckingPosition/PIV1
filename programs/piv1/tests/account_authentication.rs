mod support;

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent},
    solana_program::{program_option::COption, program_pack::Pack, system_program},
    AnchorDeserialize, AnchorSerialize,
};
use piv1::{
    accounts::{
        authenticate_fixed_accounts, seeds, AuthenticatedFixedAccounts, FixedAccountInfos,
        CONFIG_DISCRIMINATOR, DISTRIBUTION_DISCRIMINATOR, STAKE_PROGRAM_ID,
    },
    errors::{Piv1Error, Piv1Result},
    integrations::FeeFraction,
    state::{reconciliation::*, ActiveDistribution, CompletedDistributionSummary, PivConfig},
};
use spl_token::state::{Account as TokenAccount, AccountState};
use support::vault_custody_model::{
    World, ESCROW, KIF, OPERATIONS, PENDING, PENDING_TOKEN, PRINCIPAL, PRINCIPAL_TOKEN,
};

const CONFIG: usize = 0;
const ROUND: usize = 1;
const PENDING_SOL: usize = 2;
const PRINCIPAL_SOL: usize = 3;
const OPERATIONAL_SOL: usize = 4;
const ESCROW_SOL: usize = 5;
const KIF_SOL: usize = 6;
const PRINCIPAL_JITO: usize = 7;
const PENDING_JITO: usize = 8;
// Non-deployed deterministic fixture identity; no signer/keypair is created.
const FIXTURE_PROGRAM: Pubkey = Pubkey::new_from_array([213; 32]);
fn key(tag: u8) -> Pubkey { Pubkey::new_from_array([tag; 32]) }

#[derive(Clone, Debug, Eq, PartialEq)]
struct BackingAccount {
    key: Pubkey,
    owner: Pubkey,
    executable: bool,
    lamports: u64,
    data: Vec<u8>,
}

impl BackingAccount {
    fn info(&mut self) -> AccountInfo<'_> {
        // Success is required without signer or writable privileges.
        AccountInfo::new(&self.key, false, false, &mut self.lamports, &mut self.data,
            &self.owner, self.executable, 0)
    }
}

fn envelope<T: AnchorSerialize>(value: &T, discriminator: [u8; 8], space: usize) -> Vec<u8> {
    let mut bytes = discriminator.to_vec();
    bytes.extend(value.try_to_vec().unwrap());
    assert!(bytes.len() <= space);
    bytes.resize(space, 0);
    bytes
}

#[derive(Clone, Debug, PartialEq)]
struct Fixture {
    program_id: Pubkey,
    rent: Rent,
    accounts: [BackingAccount; 9],
}

impl Fixture {
    fn new() -> Self {
        Self::from_world(World::new(10_000, 50, 123, 9, 3, FeeFraction::ZERO, 100_000), Rent::default())
    }

    fn from_world(w: World, rent: Rent) -> Self {
        let mut config = w.config.clone();
        let mut round = w.round;
        let (config_key, config_bump) = Pubkey::find_program_address(&[b"config"], &FIXTURE_PROGRAM);
        config.bumps.config = config_bump;
        macro_rules! bind {
            ($field:ident, $seed:literal) => {{
                let (address, bump) = Pubkey::find_program_address(&[$seed], &FIXTURE_PROGRAM);
                config.$field = address;
                config.bumps.$field = bump;
            }};
        }
        // Literals independently pin the accepted Phase 0 seed mapping.
        bind!(piv_authority, b"authority");
        bind!(active_distribution, b"distribution");
        bind!(pending_sol_vault, b"pending-sol");
        bind!(principal_sol_queue, b"principal-sol");
        bind!(operational_sol_vault, b"operational-sol");
        bind!(distribution_escrow, b"distribution-escrow");
        bind!(kif_sol_vault, b"kif-sol");
        bind!(principal_jito_vault, b"principal-jito-vault");
        bind!(pending_jito_vault, b"pending-jito-vault");
        round.bump = config.bumps.active_distribution;
        config.system_program = system_program::ID;
        config.token_program = spl_token::ID;
        config.stake_program = STAKE_PROGRAM_ID;
        let state = |key, data: Vec<u8>| BackingAccount {
            key, owner: FIXTURE_PROGRAM, executable: false,
            lamports: rent.minimum_balance(data.len()), data,
        };
        let native = |key, world_index| BackingAccount {
            key, owner: system_program::ID, executable: false, data: vec![],
            lamports: rent.minimum_balance(0) + w.spendable(world_index).unwrap(),
        };
        let token = |key, amount| {
            let token = TokenAccount { mint: config.jitosol_mint, owner: config.piv_authority,
                amount, delegate: COption::None, state: AccountState::Initialized,
                is_native: COption::None, delegated_amount: 0, close_authority: COption::None };
            let mut data = vec![0; TokenAccount::LEN];
            TokenAccount::pack(token, &mut data).unwrap();
            BackingAccount { key, owner: spl_token::ID, executable: false,
                lamports: rent.minimum_balance(TokenAccount::LEN), data }
        };
        let accounts = [
            state(config_key, envelope(&config, CONFIG_DISCRIMINATOR, PivConfig::SPACE)),
            state(config.active_distribution, envelope(&round, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SPACE)),
            native(config.pending_sol_vault, PENDING),
            native(config.principal_sol_queue, PRINCIPAL),
            native(config.operational_sol_vault, OPERATIONS),
            native(config.distribution_escrow, ESCROW),
            native(config.kif_sol_vault, KIF),
            token(config.principal_jito_vault, w.tokens[PRINCIPAL_TOKEN]),
            token(config.pending_jito_vault, w.tokens[PENDING_TOKEN]),
        ];
        Self { program_id: FIXTURE_PROGRAM, rent, accounts }
    }

    fn with_infos<T>(&mut self, action: impl FnOnce(FixedAccountInfos<'_, '_>) -> T) -> T {
        let [c, d, p, h, o, e, k, pt, pn] = &mut self.accounts;
        let c = c.info(); let d = d.info(); let p = p.info(); let h = h.info();
        let o = o.info(); let e = e.info(); let k = k.info();
        let pt = pt.info(); let pn = pn.info();
        action(FixedAccountInfos { config: &c, active_distribution: &d,
            pending_sol: &p, principal_sol: &h, operational_sol: &o,
            distribution_escrow: &e, kif_sol: &k, principal_jito: &pt, pending_jito: &pn })
    }

    fn authenticate(&mut self) -> Piv1Result<AuthenticatedFixedAccounts> {
        let id = self.program_id;
        let rent = self.rent.clone();
        self.with_infos(|infos| authenticate_fixed_accounts(&id, &rent, infos))
    }

    fn update_config(&mut self, edit: impl FnOnce(&mut PivConfig)) {
        let mut config = PivConfig::deserialize(&mut &self.accounts[CONFIG].data[8..]).unwrap();
        edit(&mut config);
        self.accounts[CONFIG].data = envelope(&config, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    }

    fn update_round(&mut self, edit: impl FnOnce(&mut ActiveDistribution)) {
        let mut round = ActiveDistribution::deserialize(&mut &self.accounts[ROUND].data[8..]).unwrap();
        edit(&mut round);
        self.accounts[ROUND].data = envelope(&round, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SPACE);
    }

    fn update_token(&mut self, index: usize, edit: impl FnOnce(&mut TokenAccount)) {
        let mut token = TokenAccount::unpack(&self.accounts[index].data).unwrap();
        edit(&mut token);
        TokenAccount::pack(token, &mut self.accounts[index].data).unwrap();
    }

    fn reject(&mut self, error: Piv1Error) {
        let before = self.clone();
        assert_eq!(self.authenticate(), Err(error));
        assert_eq!(*self, before, "authentication must preserve every byte and lamport");
    }

    fn success(&mut self) -> AuthenticatedFixedAccounts {
        let before = self.clone();
        let result = self.authenticate().unwrap();
        assert_eq!(*self, before, "authentication must preserve every byte and lamport");
        result
    }
}

#[test]
fn complete_real_account_info_set_is_read_only_and_keeps_all_categories_separate() {
    let mut f = Fixture::new();
    let authenticated = f.success();
    let observed = authenticated.economic_observation().unwrap();
    assert_eq!(observed.amounts().unwrap(), EconomicVaultAmounts {
        pending_sol_lamports: 10_000, principal_sol_lamports: 123,
        escrow_sol_lamports: 0, kif_sol_lamports: 99,
        pending_jitosol_units: 50, principal_jitosol_units: 1_000_000,
    });
    assert_eq!(observed.pending_sol.non_economic_floor_lamports, f.rent.minimum_balance(0));
    assert_eq!(authenticated.principal_jito().native.lamports, f.rent.minimum_balance(165));
    assert_eq!(authenticated.pending_jito().token_units, 50);
    assert_eq!(authenticated.operational_sol().economic_lamports().unwrap(), 100_000);
    assert_eq!(authenticated.operational_surplus().unwrap(), OperationalSurplusAssessment::UnsupportedFundingBaseline);
}

#[test]
fn pause_and_host_privilege_flags_do_not_block_authentication() {
    let mut f = Fixture::new();
    f.update_config(|c| c.paused = true);
    assert!(f.success().config().paused);
    let id = f.program_id;
    let rent = f.rent.clone();
    let before = f.clone();
    f.with_infos(|infos| {
        let mut pending = infos.pending_sol.clone();
        pending.is_signer = true;
        pending.is_writable = true;
        let result = authenticate_fixed_accounts(&id, &rent, FixedAccountInfos { pending_sol: &pending, ..infos }).unwrap();
        result.economic_observation().unwrap();
    });
    assert_eq!(f, before);
}

#[test]
fn exact_discriminators_sizes_and_fixed_seed_bytes_are_pinned() {
    // Independently obtained SHA-256("account:<type>") first-eight-byte values.
    assert_eq!(CONFIG_DISCRIMINATOR, [98, 115, 11, 164, 170, 207, 163, 20]);
    assert_eq!(DISTRIBUTION_DISCRIMINATOR, [104, 51, 125, 187, 226, 55, 209, 99]);
    assert_ne!(CONFIG_DISCRIMINATOR, DISTRIBUTION_DISCRIMINATOR);
    assert_eq!(PivConfig::SPACE, 1_014);
    assert_eq!(ActiveDistribution::SPACE, 891);
    assert_eq!(seeds::CONFIG, b"config");
    assert_eq!(seeds::AUTHORITY, b"authority");
    assert_eq!(seeds::DISTRIBUTION, b"distribution");
    assert_eq!(seeds::PENDING_SOL, b"pending-sol");
    assert_eq!(seeds::PRINCIPAL_SOL, b"principal-sol");
    assert_eq!(seeds::OPERATIONAL_SOL, b"operational-sol");
    assert_eq!(seeds::DISTRIBUTION_ESCROW, b"distribution-escrow");
    assert_eq!(seeds::KIF_SOL, b"kif-sol");
    assert_eq!(seeds::PRINCIPAL_JITO, b"principal-jito-vault");
    assert_eq!(seeds::PENDING_JITO, b"pending-jito-vault");
}

#[test]
fn timestamp_options_use_consumed_borsh_length_and_require_zero_tail() {
    for prepared in [None, Some(100)] {
        for insufficient in [None, Some(200)] {
            let mut f = Fixture::new();
            f.update_config(|c| {
                c.last_successful_preparation_at = prepared;
                c.last_valid_insufficient_attempt_at = insufficient;
            });
            let authenticated = f.success();
            let c = authenticated.config();
            assert_eq!(c.last_successful_preparation_at, prepared);
            assert_eq!(c.last_valid_insufficient_attempt_at, insufficient);
            let used = 8 + c.try_to_vec().unwrap().len();
            if used < PivConfig::SPACE {
                f.accounts[CONFIG].data[used] = 1;
                f.reject(Piv1Error::InvalidAccountData);
            }
        }
    }
}

#[test]
fn completed_summary_option_and_its_idle_sequence_binding_are_checked() {
    let mut f = Fixture::new();
    f.update_round(|r| r.last_completed = Some(CompletedDistributionSummary {
        sequence: 7, completed_at: 20, gross_yield_lamports: 30,
        actual_allocated_outgoing_lamports: 24, integrated_contribution_value_lamports: 10,
        final_protected_hwm_lamports: 1_000_010, fixed_jitosol_withdrawal_target_units: 0,
        successful_leg_count: 0, cumulative_cooldown_rewards_lamports: 0,
        actual_kif_liability_lamports: 0, actual_kif_carry_next_lamports: 0,
    }));
    f.update_config(|c| c.next_distribution_sequence = 8);
    assert_eq!(f.success().distribution().last_completed.unwrap().sequence, 7);
    f.update_config(|c| c.next_distribution_sequence = 7);
    f.reject(Piv1Error::SequenceMismatch);
    let mut f = Fixture::new();
    let round = *f.success().distribution();
    let used = 8 + round.try_to_vec().unwrap().len();
    assert!(used < ActiveDistribution::SPACE);
    f.accounts[ROUND].data[used] = 1;
    f.reject(Piv1Error::InvalidAccountData);
}

#[test]
fn every_fixed_role_rejects_wrong_owner_executable_size_and_key() {
    for index in 0..9 {
        let mut f = Fixture::new();
        f.accounts[index].owner = key(231);
        f.reject(Piv1Error::InvalidAccountOwner);
        let mut f = Fixture::new();
        f.accounts[index].executable = true;
        f.reject(Piv1Error::ExecutableAccount);
        for delta in [1, -1] {
            let mut f = Fixture::new();
            if delta == -1 && f.accounts[index].data.is_empty() { continue; }
            if delta == 1 { f.accounts[index].data.push(0); }
            else { f.accounts[index].data.pop(); }
            f.reject(Piv1Error::InvalidAccountSize);
        }
        let mut f = Fixture::new();
        f.accounts[index].key = key(232);
        f.reject(Piv1Error::InvalidAccountPda);
    }
}

#[test]
fn canonical_pda_checks_include_every_stored_bump_and_the_address_only_authority() {
    for role in 0..10 {
        let mut f = Fixture::new();
        f.update_config(|c| {
            let bump = match role {
                0 => &mut c.bumps.config, 1 => &mut c.bumps.piv_authority,
                2 => &mut c.bumps.active_distribution, 3 => &mut c.bumps.pending_sol_vault,
                4 => &mut c.bumps.principal_sol_queue, 5 => &mut c.bumps.operational_sol_vault,
                6 => &mut c.bumps.distribution_escrow, 7 => &mut c.bumps.kif_sol_vault,
                8 => &mut c.bumps.principal_jito_vault, _ => &mut c.bumps.pending_jito_vault,
            };
            *bump ^= 1;
        });
        f.reject(Piv1Error::InvalidAccountPda);
    }
    let mut f = Fixture::new();
    f.update_round(|r| r.bump ^= 1);
    f.reject(Piv1Error::InvalidAccountPda);
    let mut f = Fixture::new();
    f.update_config(|c| c.piv_authority = key(233));
    f.reject(Piv1Error::InvalidAccountPda);
}

#[test]
fn alternate_valid_off_curve_bump_is_not_canonical_even_with_matching_config_and_account() {
    let mut f = Fixture::new();
    let (_, canonical) = Pubkey::find_program_address(&[b"pending-sol"], &FIXTURE_PROGRAM);
    let (key, bump) = (0..canonical).find_map(|bump| {
        Pubkey::create_program_address(&[b"pending-sol", &[bump]], &FIXTURE_PROGRAM)
            .ok().map(|key| (key, bump))
    }).unwrap();
    f.accounts[PENDING_SOL].key = key;
    f.update_config(|c| { c.pending_sol_vault = key; c.bumps.pending_sol_vault = bump; });
    f.reject(Piv1Error::InvalidAccountPda);
}

#[test]
fn swapped_and_aliased_accounts_and_config_roles_are_rejected() {
    for (left, right) in [(PENDING_SOL, PRINCIPAL_SOL), (PRINCIPAL_JITO, PENDING_JITO),
        (CONFIG, ROUND), (ESCROW_SOL, KIF_SOL)] {
        let mut f = Fixture::new();
        f.accounts.swap(left, right);
        assert!(f.authenticate().is_err());
        let mut f = Fixture::new();
        f.accounts[right].key = f.accounts[left].key;
        f.reject(Piv1Error::AccountAlias);
    }
    let mut f = Fixture::new();
    f.update_config(|c| c.pending_jito_vault = c.principal_jito_vault);
    f.reject(Piv1Error::InvalidAddress);
    let mut f = Fixture::new();
    let config_key = f.accounts[CONFIG].key;
    f.update_config(|c| c.htfp_recipient = config_key);
    f.reject(Piv1Error::AccountAlias);
    let mut f = Fixture::new();
    let id = f.program_id; let rent = f.rent.clone(); let before = f.clone();
    f.with_infos(|infos| assert_eq!(authenticate_fixed_accounts(&id, &rent,
        FixedAccountInfos { pending_jito: infos.principal_jito, ..infos }), Err(Piv1Error::AccountAlias)));
    assert_eq!(f, before);
}

#[test]
fn forged_program_bindings_and_untrusted_program_context_are_rejected() {
    for role in 0..3 {
        let mut f = Fixture::new();
        f.update_config(|c| match role {
            0 => c.system_program = key(230), 1 => c.token_program = key(230),
            _ => c.stake_program = key(230),
        });
        f.reject(Piv1Error::InvalidProgramIdentity);
    }
    let mut f = Fixture::new();
    f.program_id = key(229);
    // Even making both state owners agree cannot rebase the fixed PDA identities.
    f.accounts[CONFIG].owner = f.program_id;
    f.accounts[ROUND].owner = f.program_id;
    f.reject(Piv1Error::InvalidAccountPda);
    for invalid in [system_program::ID, spl_token::ID, STAKE_PROGRAM_ID] {
        let mut f = Fixture::new(); f.program_id = invalid;
        f.reject(Piv1Error::InvalidProgramIdentity);
    }
}

#[test]
fn all_zero_key_is_valid_only_for_system_program_role_and_manager_referrer_alias_survives() {
    let mut f = Fixture::new();
    assert_eq!(f.success().config().system_program, Pubkey::default());
    for role in 0..21 {
        let mut f = Fixture::new();
        f.update_config(|c| {
            let fields = [&mut c.stake_pool_program, &mut c.stake_pool, &mut c.validator_list,
                &mut c.reserve_stake, &mut c.jitosol_mint, &mut c.token_program,
                &mut c.stake_program, &mut c.manager_fee_account, &mut c.referrer_token_account,
                &mut c.piv_authority, &mut c.active_distribution, &mut c.principal_jito_vault,
                &mut c.pending_jito_vault, &mut c.pending_sol_vault, &mut c.principal_sol_queue,
                &mut c.operational_sol_vault, &mut c.distribution_escrow, &mut c.kif_sol_vault,
                &mut c.htfp_recipient, &mut c.team_owner_recipient, &mut c.guardian_registry];
            *fields.into_iter().nth(role).unwrap() = Pubkey::default();
        });
        f.reject(Piv1Error::InvalidAddress);
    }
    let mut f = Fixture::new();
    f.update_config(|c| c.referrer_token_account = c.manager_fee_account);
    f.success().economic_observation().unwrap();
}

#[test]
fn state_discriminator_version_initialization_reserve_and_malformed_borsh_fail() {
    for index in [CONFIG, ROUND] {
        let mut f = Fixture::new(); f.accounts[index].data[0] ^= 1;
        f.reject(Piv1Error::InvalidAccountDiscriminator);
        let mut f = Fixture::new(); f.accounts[index].data[8] = 2;
        f.reject(Piv1Error::InvalidVersion);
        let initialized = if index == CONFIG { 9 } else { 10 };
        let mut f = Fixture::new(); f.accounts[index].data[initialized] = 0;
        f.reject(Piv1Error::InvalidInitialization);
        let mut f = Fixture::new(); f.accounts[index].data[initialized] = 2;
        f.reject(Piv1Error::InvalidAccountData);
        let mut f = Fixture::new(); f.accounts[index].data.truncate(7);
        f.reject(Piv1Error::InvalidAccountSize);
    }
    let mut f = Fixture::new();
    f.update_config(|c| c.migration_reserve[12] = 1);
    f.reject(Piv1Error::InvalidInitialization);
    let mut f = Fixture::new(); f.accounts[ROUND].data[11] = u8::MAX;
    f.reject(Piv1Error::InvalidAccountData);
    let mut f = Fixture::new(); f.accounts[ROUND].data[22] = 2; // last_completed option tag
    f.reject(Piv1Error::InvalidAccountData);
}

#[test]
fn initialized_legacy_token_state_rejects_foreign_mint_authority_and_extra_powers() {
    for index in [PRINCIPAL_JITO, PENDING_JITO] {
        for case in 0..9 {
            let mut f = Fixture::new();
            f.update_token(index, |t| match case {
                0 => t.mint = key(230), 1 => t.owner = key(230),
                2 => t.state = AccountState::Uninitialized, 3 => t.state = AccountState::Frozen,
                4 => t.delegate = COption::Some(key(230)), 5 => t.delegated_amount = 1,
                6 => t.close_authority = COption::Some(key(230)),
                7 => t.close_authority = COption::Some(t.owner),
                _ => t.is_native = COption::Some(2_039_280),
            });
            f.reject(Piv1Error::InvalidTokenCustody);
        }
        for offset in [72, 108, 109, 129] {
            let mut f = Fixture::new(); f.accounts[index].data[offset] = 255;
            f.reject(Piv1Error::InvalidTokenCustody);
        }
    }
}

#[test]
fn all_nine_accounts_require_their_exact_runtime_rent_floor() {
    for index in 0..9 {
        let mut f = Fixture::new();
        f.accounts[index].lamports = f.rent.minimum_balance(f.accounts[index].data.len()) - 1;
        f.reject(Piv1Error::AccountRentDeficit);
    }
    for rent in [Rent::default(), Rent::free(), Rent {
        lamports_per_byte_year: 5, exemption_threshold: 2.5, burn_percent: 3,
    }] {
        let mut f = Fixture::from_world(World::new(10_000, 50, 0, 9, 3, FeeFraction::ZERO, 100_000), rent.clone());
        let authenticated = f.success();
        assert_eq!(authenticated.economic_observation().unwrap().pending_sol.non_economic_floor_lamports,
            rent.minimum_balance(0));
        assert_eq!(authenticated.principal_jito().native.non_economic_floor_lamports,
            rent.minimum_balance(TokenAccount::LEN));
    }
}

#[test]
fn invalid_or_overflowing_runtime_rent_inputs_fail_without_panicking() {
    for threshold in [f64::NAN, f64::INFINITY, -1.0] {
        let mut f = Fixture::new(); f.rent.exemption_threshold = threshold;
        // NaN is not equal to itself, so compare backing accounts directly.
        let before = f.accounts.clone();
        assert_eq!(f.authenticate(), Err(Piv1Error::InvalidRent));
        assert_eq!(f.accounts, before);
    }
    let mut f = Fixture::new(); f.rent.burn_percent = 101;
    f.reject(Piv1Error::InvalidRent);
    let mut f = Fixture::new(); f.rent.lamports_per_byte_year = u64::MAX;
    f.reject(Piv1Error::ArithmeticOverflow);
    let mut f = Fixture::new(); f.rent.exemption_threshold = f64::MAX;
    f.reject(Piv1Error::ArithmeticOverflow);
}

#[test]
fn unsolicited_token_native_lamport_is_observed_but_economic_normalization_is_unsupported() {
    for index in [PRINCIPAL_JITO, PENDING_JITO] {
        let mut f = Fixture::new(); f.accounts[index].lamports += 1;
        let authenticated = f.success();
        let balance = if index == PRINCIPAL_JITO { authenticated.principal_jito() }
            else { authenticated.pending_jito() };
        assert_eq!(balance.native.economic_lamports().unwrap(), 1);
        assert_eq!(authenticated.economic_observation(), Err(Piv1Error::UnsupportedTokenNativeExcess));
        assert_eq!(authenticated.config().accounted_pending_sol_lamports, 10_000);
        assert_eq!(authenticated.config().next_cycle_yield_lamports, 123);
    }
}

#[test]
fn operational_native_excess_is_always_separate_and_has_no_inferred_funding_baseline() {
    let mut f = Fixture::new();
    let expected = f.success().economic_observation().unwrap();
    f.accounts[OPERATIONAL_SOL].lamports += 999;
    let authenticated = f.success();
    assert_eq!(authenticated.economic_observation().unwrap(), expected);
    assert_eq!(authenticated.operational_sol().economic_lamports().unwrap(), 100_999);
    assert_eq!(authenticated.operational_surplus().unwrap(), OperationalSurplusAssessment::UnsupportedFundingBaseline);
}

#[test]
fn borrow_conflicts_in_any_data_or_lamports_fail_read_only() {
    for index in 0..9 {
        for data in [false, true] {
            let mut f = Fixture::new();
            let before = f.clone(); let id = f.program_id; let rent = f.rent.clone();
            f.with_infos(|infos| {
                let all = [infos.config, infos.active_distribution, infos.pending_sol,
                    infos.principal_sol, infos.operational_sol, infos.distribution_escrow,
                    infos.kif_sol, infos.principal_jito, infos.pending_jito];
                if data {
                    let _borrow = all[index].try_borrow_mut_data().unwrap();
                    assert_eq!(authenticate_fixed_accounts(&id, &rent, infos), Err(Piv1Error::AccountBorrowFailed));
                } else {
                    let _borrow = all[index].try_borrow_mut_lamports().unwrap();
                    assert_eq!(authenticate_fixed_accounts(&id, &rent, infos), Err(Piv1Error::AccountBorrowFailed));
                }
            });
            assert_eq!(f, before);
        }
    }
}

#[test]
fn active_pending_offsets_match_actual_host_opening_movements_and_do_not_create_false_deficits() {
    for initial in [0, 4_000, 8_050, 10_000] {
        let mut w = World::new(initial, 50, 0, 9, 3, FeeFraction::ZERO, 100_000);
        w.open(900_000).unwrap();
        assert_eq!(w.round.pending_sol_used_lamports, initial.min(8_050));
        let mut f = Fixture::from_world(w, Rent::default());
        let authenticated = f.success();
        let observation = authenticated.economic_observation().unwrap();
        assert_eq!(observation.pending_sol.economic_lamports().unwrap(), initial - initial.min(8_050));
        assert_eq!(observation.distribution_escrow.economic_lamports().unwrap(), initial.min(8_050));
        assert_eq!(authenticated.config().accounted_pending_sol_lamports, initial);
        let mut config = authenticated.config().clone();
        assert!(piv1::state::reconcile_pending_contributions(&mut config,
            authenticated.distribution(), observation.pending()).unwrap().is_no_change());
        assert_eq!(&config, authenticated.config());
    }
}

#[test]
fn active_binding_rejects_individually_valid_mismatched_config() {
    for case in 0..4 {
        let mut w = World::new(10_000, 50, 0, 9, 3, FeeFraction::ZERO, 100_000);
        w.open(900_000).unwrap();
        let mut f = Fixture::from_world(w, Rent::default());
        f.update_config(|c| match case {
            0 => c.next_distribution_sequence += 1,
            1 => c.last_successful_preparation_at = Some(900_001),
            2 => c.accounted_historical_jitosol_units -= 1,
            _ => c.protected_principal_hwm_lamports -= 1,
        });
        f.reject(if case == 0 { Piv1Error::SequenceMismatch }
            else { Piv1Error::CumulativeReconciliationMismatch });
    }
}

#[test]
fn every_economic_vault_deficit_rejects_observation_even_with_unrelated_surplus() {
    for index in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL, PRINCIPAL_JITO, PENDING_JITO] {
        let mut f = if index == ESCROW_SOL {
            let mut w = World::new(10_000, 50, 0, 9, 3, FeeFraction::ZERO, 100_000);
            w.open(900_000).unwrap(); Fixture::from_world(w, Rent::default())
        } else { Fixture::new() };
        if index >= PRINCIPAL_JITO { f.update_token(index, |t| t.amount -= 1); }
        else { f.accounts[index].lamports -= 1; }
        f.accounts[OPERATIONAL_SOL].lamports += 1_000_000;
        if index != PENDING_SOL { f.accounts[PENDING_SOL].lamports += 1_000_000; }
        else { f.accounts[PRINCIPAL_SOL].lamports += 1_000_000; }
        let authenticated = f.success();
        assert_eq!(authenticated.economic_observation(), Err(Piv1Error::EconomicCustodyDeficit));
    }
}

#[test]
fn accounting_overflow_and_sequence_overflow_reject_before_observation() {
    let mut f = Fixture::new();
    f.update_config(|c| c.collective_kif_carry_lamports = u64::MAX);
    f.reject(Piv1Error::ArithmeticOverflow);
    let mut f = Fixture::new();
    f.update_round(|r| r.last_completed = Some(CompletedDistributionSummary {
        sequence: u64::MAX, completed_at: 0, gross_yield_lamports: 0,
        actual_allocated_outgoing_lamports: 0, integrated_contribution_value_lamports: 0,
        final_protected_hwm_lamports: 0, fixed_jitosol_withdrawal_target_units: 0,
        successful_leg_count: 0, cumulative_cooldown_rewards_lamports: 0,
        actual_kif_liability_lamports: 0, actual_kif_carry_next_lamports: 0,
    }));
    f.reject(Piv1Error::ArithmeticOverflow);
}

#[test]
fn owned_snapshot_requires_reauthentication_after_account_mutation() {
    let mut f = Fixture::new();
    let before = f.success();
    f.accounts[PENDING_SOL].lamports += 7;
    let after = f.success();
    assert_eq!(before.economic_observation().unwrap().pending_sol.economic_lamports().unwrap(), 10_000);
    assert_eq!(after.economic_observation().unwrap().pending_sol.economic_lamports().unwrap(), 10_007);
    assert_eq!(after.config(), before.config());
}
