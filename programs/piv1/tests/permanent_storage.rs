mod support;
#[path = "support/pending_custody.rs"]
mod pending_custody;
#[path = "support/economic_custody.rs"]
mod economic_custody;

use piv1::{
    accounts::authenticate_fixed_accounts,
    errors::Piv1Error,
    integrations::FeeFraction,
    state::{config::StorageVault, reconciliation::economic_custody_obligations},
};
use economic_custody::{
    roles, Fixture, CONFIG, ESCROW_SOL, FIXTURE_PROGRAM, KIF_SOL, OPERATIONAL_SOL, PENDING_JITO,
    PENDING_SOL, PRINCIPAL_JITO, PRINCIPAL_SOL, ROUND,
};
use support::{kif_claim_custody, vault_custody_model::World};

fn fixture(initial_rate: u64) -> Fixture {
    let world = World::new(300, 5, 11, 7, 2, FeeFraction::ZERO, 100_000);
    let mut fixture = Fixture::from_world(&world);
    let prior_rent = fixture.rent.clone();
    fixture.rent.exemption_threshold = 1.0;
    fixture.rent.lamports_per_byte_year = initial_rate;
    for account in &mut fixture.accounts[..=PENDING_JITO] {
        let size = account.data.len();
        account.lamports -= prior_rent.minimum_balance(size);
        account.lamports += fixture.rent.minimum_balance(size);
    }
    let deposit = fixture.rent.minimum_balance(0);
    fixture.edit_config(|config| config.record_initial_storage_reserve(deposit));
    fixture
}

#[test]
fn covered_repricing_never_changes_financial_custody() {
    for initial in [1322, 2575, 5080, 6960] {
        let mut fixture = fixture(initial);
        let baseline = fixture.clone();
        let expected_operation = baseline.accounts[OPERATIONAL_SOL].lamports - baseline.rent.minimum_balance(0);
        for rate in [initial, 696, 1, initial - 1, initial, 696] {
            fixture.rent.lamports_per_byte_year = rate;
            let rent = fixture.rent.clone();
            fixture.with_infos(|accounts| {
                let verified = authenticate_fixed_accounts(&FIXTURE_PROGRAM, &rent, roles(accounts)).unwrap();
                let obligations = economic_custody_obligations(verified.config(), verified.distribution()).unwrap();
                let observed = verified.distribution_preparation_observation().unwrap();
                assert_eq!(observed.amounts().unwrap(), obligations);
                assert_eq!(verified.operational_sol().economic_lamports().unwrap(), expected_operation);
                for vault in [StorageVault::PendingSol, StorageVault::PrincipalSol, StorageVault::OperationalSol,
                    StorageVault::DistributionEscrow, StorageVault::KifSol] {
                    assert_eq!(verified.config().storage_reserve(vault), Ok(baseline.rent.minimum_balance(0)));
                }
            });
            assert_eq!(fixture.accounts, baseline.accounts);
        }
    }
}

#[test]
fn outside_credit_stays_excess_and_cannot_cover_a_different_vault() {
    let mut fixture = fixture(5080);
    fixture.rent.lamports_per_byte_year = 2575;
    fixture.accounts[KIF_SOL].lamports += 73;
    let rent = fixture.rent.clone();
    fixture.with_infos(|accounts| {
        let verified = authenticate_fixed_accounts(&FIXTURE_PROGRAM, &rent, roles(accounts)).unwrap();
        let expected = economic_custody_obligations(verified.config(), verified.distribution()).unwrap();
        let amounts = verified.distribution_preparation_observation().unwrap().amounts().unwrap();
        assert_eq!(amounts.kif_sol_lamports - expected.kif_sol_lamports, 73);
        assert_eq!(amounts.pending_sol_lamports, expected.pending_sol_lamports);
    });
    fixture.accounts[PENDING_SOL].lamports -= 1;
    let baseline = fixture.clone();
    fixture.with_infos(|accounts| {
        let verified = authenticate_fixed_accounts(&FIXTURE_PROGRAM, &rent, roles(accounts)).unwrap();
        assert_eq!(verified.distribution_preparation_observation(), Err(Piv1Error::EconomicCustodyDeficit));
    });
    assert_eq!(fixture, baseline);
}

#[test]
fn legacy_or_unfunded_increase_refuses_even_with_enough_raw_lamports() {
    let mut legacy = fixture(5080);
    legacy.edit_config(|config| { config.version = 1; config.migration_reserve.fill(0); });
    let rent = legacy.rent.clone();
    legacy.with_infos(|accounts| assert_eq!(authenticate_fixed_accounts(&FIXTURE_PROGRAM, &rent, roles(accounts)),
        Err(Piv1Error::InvalidVersion)));

    let mut raised = fixture(5080);
    raised.rent.lamports_per_byte_year = 6960;
    let rent = raised.rent.clone();
    for index in [CONFIG, ROUND, PRINCIPAL_JITO, PENDING_JITO] {
        let account = &mut raised.accounts[index];
        account.lamports = rent.minimum_balance(account.data.len());
    }
    for index in [PENDING_SOL, PRINCIPAL_SOL, OPERATIONAL_SOL, ESCROW_SOL, KIF_SOL] {
        raised.accounts[index].lamports += 2_000_000;
    }
    let before = raised.clone();
    raised.with_infos(|accounts| assert_eq!(authenticate_fixed_accounts(&FIXTURE_PROGRAM, &rent, roles(accounts)),
        Err(Piv1Error::AccountRentDeficit)));
    assert_eq!(raised, before);
}
