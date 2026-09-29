//! Host callbacks execute pinned SPL Token code and a modeled System transfer.
//! These account oracles do not establish SVM signatures or transaction rollback.
mod support;
#[path = "support/pending_custody.rs"]
pub mod pending_custody;
use support::kif_claim_custody;
use std::cell::Cell;
use anchor_lang::{prelude::{AccountInfo, Rent}, AnchorDeserialize,
    solana_program::{entrypoint::ProgramResult, instruction::{AccountMeta, Instruction},
        program_error::ProgramError, program_option::COption, program_pack::Pack, system_program}};
use spl_token::state::{Account as TokenAccount, AccountState, Mint, Multisig};
use piv1::{accounts::CONFIG_DISCRIMINATOR,
    contribution_execution::{process_instruction_with_host_callbacks as execute, ContributionEvent},
    errors::Piv1Error, events::{JitoSolContribution, SolContribution},
    instruction_boundary::{process_instruction, process_instruction_with_host_callbacks as claim_seam},
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::{deposit_sol::*, deposit_jitosol::*}, integrations::FeeFraction,
    state::{DistributionLifecycle, PivConfig}};
use support::{kif_claim_custody::{BackingAccount, envelope, key, PROGRAM},
    vault_custody_model::{World, PENDING, PENDING_TOKEN}};

const AMOUNT: u64 = 11;
#[derive(Clone, Debug, PartialEq)]
struct Fixture { accounts: Vec<BackingAccount>, rent: Rent, token: bool }
impl Fixture {
    fn new(token: bool, world: &World) -> Self {
        let f = pending_custody::Fixture::from_state(world.config.clone(), world.round.clone(),
            world.spendable(PENDING).unwrap(), world.tokens[PENDING_TOKEN]);
        let config = f.config(); let rent = f.rent; let mut accounts = Vec::from(f.accounts);
        accounts[if token { 3 } else { 2 }].writable = true;
        let backing = |key, owner, data: Vec<u8>, signer, writable, executable| BackingAccount {
            key, owner, lamports: rent.minimum_balance(data.len()), data, signer, writable, executable };
        if token {
            let source = TokenAccount { mint: config.jitosol_mint, owner: key(211), amount: 100,
                delegate: COption::None, state: AccountState::Initialized, is_native: COption::None,
                delegated_amount: 0, close_authority: COption::Some(key(212)) };
            // Ignored None payload bytes must survive real SPL packing.
            let mut data = vec![0xa7; TokenAccount::LEN]; TokenAccount::pack(source, &mut data).unwrap();
            accounts.push(backing(key(210), spl_token::ID, data, false, true, false));
            accounts.push(backing(key(211), key(213), vec![9; 29], true, false, false));
            let mint = Mint { mint_authority: COption::None, supply: 10_000_000,
                decimals: 9, is_initialized: true, freeze_authority: COption::None };
            let mut data = vec![0xb3; Mint::LEN]; Mint::pack(mint, &mut data).unwrap();
            accounts.push(backing(config.jitosol_mint, spl_token::ID, data, false, false, false));
            accounts.push(backing(spl_token::ID, key(214), vec![7; 123], false, false, true));
        } else {
            accounts.push(backing(key(210), system_program::ID, vec![], true, true, false));
            accounts[4].lamports += 100;
            accounts.push(backing(system_program::ID, key(214), vec![7; 123], false, false, true));
        }
        Self { accounts, rent, token }
    }
    fn config(&self) -> PivConfig { PivConfig::deserialize(&mut &self.accounts[0].data[8..]).unwrap() }
    fn edit_config(&mut self, edit: impl FnOnce(&mut PivConfig)) {
        let mut c = self.config(); edit(&mut c);
        self.accounts[0].data = envelope(&c, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    }
    fn edit_token(&mut self, index: usize, edit: impl FnOnce(&mut TokenAccount)) {
        let mut token = TokenAccount::unpack(&self.accounts[index].data).unwrap(); edit(&mut token);
        TokenAccount::pack(token, &mut self.accounts[index].data).unwrap();
    }
    fn data(&self, amount: u64) -> [u8; 17] {
        if self.token { DepositJitoSolRequest { amount_units: amount }.encode() }
        else { DepositSolRequest { amount_lamports: amount }.encode() }
    }
    fn with_infos<T>(&mut self, action: impl FnOnce(&[AccountInfo<'_>]) -> T) -> T {
        let infos: Vec<_> = self.accounts.iter_mut().map(BackingAccount::info).collect(); action(&infos)
    }
}
fn world() -> World { World::new(100_000, 20, 0, 7, 3, FeeFraction::ZERO, 100_000) }
fn err(value: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(value)) }

// Literal independent wire/account-order assertions precede modeled effects.
fn transfer(token: bool, amount: u64, ix: &Instruction, a: &[AccountInfo<'_>]) -> ProgramResult {
    if token {
        let mut data = vec![12]; data.extend_from_slice(&amount.to_le_bytes()); data.push(9);
        assert_eq!(ix.program_id, spl_token::ID); assert_eq!(ix.data, data); assert_eq!(a.len(), 5);
        assert_eq!(ix.accounts, vec![AccountMeta::new(*a[0].key, false),
            AccountMeta::new_readonly(*a[1].key, false), AccountMeta::new(*a[2].key, false),
            AccountMeta::new_readonly(*a[3].key, true)]);
        assert_eq!(*a[4].key, spl_token::ID);
        spl_token::processor::Processor::process(&spl_token::ID, &a[..4], &ix.data)
    } else {
        let mut data = vec![2, 0, 0, 0]; data.extend_from_slice(&amount.to_le_bytes());
        assert_eq!(ix.program_id, system_program::ID); assert_eq!(ix.data, data); assert_eq!(a.len(), 3);
        assert_eq!(ix.accounts, vec![AccountMeta::new(*a[0].key, true), AccountMeta::new(*a[1].key, false)]);
        assert_eq!(*a[2].key, system_program::ID);
        **a[0].try_borrow_mut_lamports()? -= amount; **a[1].try_borrow_mut_lamports()? += amount; Ok(())
    }
}
fn success(f: &mut Fixture, amount: u64) {
    let mut expected = f.clone(); let token = f.token; let old = f.config();
    let event = if token {
        let source = TokenAccount::unpack(&expected.accounts[4].data).unwrap();
        expected.accounts[4].data[64..72].copy_from_slice(&(source.amount - amount).to_le_bytes());
        if source.delegate == COption::Some(expected.accounts[5].key) {
            let allowance = source.delegated_amount - amount;
            expected.accounts[4].data[121..129].copy_from_slice(&allowance.to_le_bytes());
            if allowance == 0 { expected.accounts[4].data[72..76].fill(0); }
        }
        let destination = TokenAccount::unpack(&expected.accounts[3].data).unwrap().amount;
        expected.accounts[3].data[64..72].copy_from_slice(&(destination + amount).to_le_bytes());
        expected.edit_config(|c| c.accounted_pending_jitosol_units += amount);
        ContributionEvent::JitoSol(JitoSolContribution { config: f.accounts[0].key, source: f.accounts[4].key,
            owner: f.accounts[5].key, amount_units: amount, pending_units_after: old.accounted_pending_jitosol_units + amount })
    } else {
        expected.accounts[4].lamports -= amount; expected.accounts[2].lamports += amount;
        expected.edit_config(|c| c.accounted_pending_sol_lamports += amount);
        ContributionEvent::Sol(SolContribution { config: f.accounts[0].key, donor: f.accounts[4].key,
            amount_lamports: amount, pending_lamports_after: old.accounted_pending_sol_lamports + amount })
    };
    let data = f.data(amount); let rent = f.rent.clone(); let calls = Cell::new(0); let mut events = vec![];
    f.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(rent), |ix, infos| {
        calls.set(calls.get() + 1); transfer(token, amount, ix, infos)
    }, |event| events.push(event))).unwrap();
    assert_eq!(calls.get(), 1); assert_eq!(events, vec![event]); assert_eq!(*f, expected);
}
fn reject(mut f: Fixture, amount: u64, expected: ProgramError) {
    let before = f.clone(); let data = f.data(amount); let rent = f.rent.clone();
    assert_eq!(f.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(rent),
        |_, _| panic!("preflight rejection must not invoke"), |_| panic!("failed intake event"))), Err(expected));
    assert_eq!(f, before);
}

#[test]
fn successful_intake_preserves_full_idle_active_settled_recovery_state_and_old_surplus() {
    let idle = world(); let mut active = idle.clone(); active.open(1_000_000).unwrap();
    assert!(active.round.pending_sol_used_lamports > 0);
    let mut settled = active.clone(); settled.settle().unwrap();
    let mut recovery = World::new(4_000, 20, 0, 7, 3, FeeFraction::ZERO, 100_000);
    recovery.open(1_000_000).unwrap(); let leg = recovery.initiate(1).unwrap();
    recovery.pool.decrease_exchange_rate(recovery.pool.raw_snapshot().total_pool_lamports - 1).unwrap();
    recovery.advance_epoch().unwrap(); recovery.finalize(leg).unwrap();
    assert_eq!(recovery.round.lifecycle, DistributionLifecycle::RecoveryRequired);
    for w in [&idle, &active, &settled, &recovery] { for token in [false, true] {
        let mut f = Fixture::new(token, w);
        f.accounts[2].lamports += 17; f.accounts[3].lamports += 19;
        f.edit_token(3, |t| t.amount += 13);
        // Each successful call is a new donation; no artificial idempotency.
        success(&mut f, AMOUNT); success(&mut f, 3);
    } }
}

#[test]
fn owner_transfer_preserves_distinct_delegates_and_models_self_delegate_allowance() {
    for delegate in [None, Some(key(212)), Some(key(211))] { for allowance in [AMOUNT, AMOUNT + 7] {
        let mut f = Fixture::new(true, &world());
        f.edit_token(4, |t| { t.delegate = delegate.map_or(COption::None, COption::Some); t.delegated_amount = allowance; });
        success(&mut f, AMOUNT);
    } }
    let mut f = Fixture::new(true, &world()); f.edit_token(4, |t| {
        t.delegate = COption::Some(t.owner); t.delegated_amount = AMOUNT - 1;
    }); reject(f, AMOUNT, ProgramError::InsufficientFunds);
    let mut f = Fixture::new(true, &world()); f.accounts[5].owner = spl_token::ID;
    f.accounts[5].data = vec![0; Multisig::LEN]; reject(f, AMOUNT, err(Piv1Error::InvalidTokenCustody));
}

#[test]
fn paused_zero_and_either_old_deficit_reject_before_any_transfer() {
    for token in [false, true] {
        let mut f = Fixture::new(token, &world()); f.edit_config(|c| c.paused = true);
        reject(f, AMOUNT, err(Piv1Error::PausedOperation));
        reject(Fixture::new(token, &world()), 0, err(Piv1Error::ZeroContribution));
        let mut f = Fixture::new(token, &world()); f.accounts[2].lamports -= 1;
        reject(f, AMOUNT, err(Piv1Error::PendingCustodyDeficit));
        let mut f = Fixture::new(token, &world()); f.edit_token(3, |t| t.amount -= 1);
        reject(f, AMOUNT, err(Piv1Error::PendingCustodyDeficit));
    }
}

#[test]
fn strict_wire_counts_runtime_guards_and_context_failures_do_not_borrow_claim_authority() {
    for token in [false, true] {
        let mut f = Fixture::new(token, &world()); let before = f.clone(); let bytes = f.data(AMOUNT);
        assert_eq!(&bytes[..8], if token { b"PIV1DJ01" } else { b"PIV1DS01" });
        assert_eq!(bytes[8], 1); assert_eq!(&bytes[9..], &AMOUNT.to_le_bytes());
        assert_eq!(DepositSolRequest::decode(&DepositSolRequest { amount_lamports: u64::MAX }.encode()).unwrap().amount_lamports, u64::MAX);
        assert_eq!(DepositJitoSolRequest::decode(&DepositJitoSolRequest { amount_units: u64::MAX }.encode()).unwrap().amount_units, u64::MAX);
        for mutation in 0..4 {
            let mut data = bytes.to_vec(); match mutation { 0 => data[0] ^= 1, 1 => data[8] = 2,
                2 => { data.pop(); }, _ => data.push(0) }
            f.with_infos(|a| assert_eq!(execute(&PROGRAM, a, &data, || panic!("decode before context"),
                |_, _| panic!(), |_| panic!()), Err(ProgramError::InvalidInstructionData)));
        }
        f.with_infos(|a| {
            assert_eq!(execute(&PROGRAM, &a[..a.len()-1], &bytes, || panic!(), |_, _| panic!(), |_| panic!()), Err(ProgramError::NotEnoughAccountKeys));
            let mut extra = a.to_vec(); extra.push(a[0].clone());
            assert_eq!(execute(&PROGRAM, &extra, &bytes, || panic!(), |_, _| panic!(), |_| panic!()), Err(ProgramError::InvalidArgument));
            assert_eq!(process_instruction(&PROGRAM, a, &bytes), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
            assert_eq!(claim_seam(&PROGRAM, a, &bytes, || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
            assert_eq!(execute(&PROGRAM, a, &bytes, || Err(ProgramError::Custom(919)), |_, _| panic!(), |_| panic!()), Err(ProgramError::Custom(919)));
        });
        assert_eq!(f, before);
    }
}

#[test]
fn source_signer_internal_custody_and_program_authentication_fail_closed() {
    for token in [false, true] {
        let mut f = Fixture::new(token, &world()); f.accounts[if token { 5 } else { 4 }].signer = false;
        reject(f, AMOUNT, ProgramError::MissingRequiredSignature);
        for index in [0, 4, if token { 3 } else { 2 }] {
            let mut f = Fixture::new(token, &world()); f.accounts[index].writable = false;
            reject(f, AMOUNT, err(Piv1Error::AccountNotWritable));
        }
        let mut f = Fixture::new(token, &world()); f.accounts[4].owner = PROGRAM;
        reject(f, AMOUNT, err(Piv1Error::InvalidAccountOwner));
        for internal in 0..3 {
            let mut f = Fixture::new(token, &world()); let c = f.config();
            f.accounts[4].key = [c.principal_jito_vault, c.operational_sol_vault, c.piv_authority][internal];
            reject(f, AMOUNT, err(Piv1Error::InvalidAccountOwner));
        }
        for bad_key in [false, true] {
            let mut f = Fixture::new(token, &world()); let p = if token { 7 } else { 5 };
            if bad_key { f.accounts[p].key = key(230); } else { f.accounts[p].executable = false; }
            reject(f, AMOUNT, err(Piv1Error::InvalidProgramIdentity));
        }
    }
    let mut f = Fixture::new(false, &world()); f.accounts[4].data.push(1);
    reject(f, AMOUNT, err(Piv1Error::InvalidAccountData));
    let mut f = Fixture::new(false, &world()); f.accounts[4].owner = key(231);
    reject(f, AMOUNT, err(Piv1Error::InvalidAccountOwner));
    let mut f = Fixture::new(false, &world()); f.accounts[4].executable = true;
    reject(f, AMOUNT, err(Piv1Error::ExecutableAccount));
    let mut f = Fixture::new(true, &world()); f.accounts[5].key = f.config().piv_authority;
    reject(f, AMOUNT, err(Piv1Error::InvalidAccountOwner));
}

#[test]
fn source_mint_native_frozen_rent_and_checked_arithmetic_guards_precede_cpi() {
    for mutation in 0..4 {
        let mut f = Fixture::new(true, &world()); f.edit_token(4, |t| match mutation {
            0 => t.mint = key(231), 1 => t.owner = key(231),
            2 => t.state = AccountState::Frozen, _ => t.is_native = COption::Some(1) });
        reject(f, AMOUNT, err(Piv1Error::InvalidTokenCustody));
    }
    for index in [4, 6] {
        let mut f = Fixture::new(true, &world()); f.accounts[index].owner = key(232);
        reject(f, AMOUNT, err(Piv1Error::InvalidAccountOwner));
        let mut f = Fixture::new(true, &world()); f.accounts[index].data.push(0);
        reject(f, AMOUNT, err(Piv1Error::InvalidAccountSize));
        let mut f = Fixture::new(true, &world()); f.accounts[index].lamports -= 1;
        reject(f, AMOUNT, err(Piv1Error::AccountRentDeficit));
    }
    let mut f = Fixture::new(true, &world()); f.accounts[6].key = key(233);
    reject(f, AMOUNT, err(Piv1Error::InvalidTokenCustody));
    let mut f = Fixture::new(true, &world()); f.accounts[6].data[45] = 0;
    reject(f, AMOUNT, err(Piv1Error::InvalidTokenCustody));
    for token in [false, true] {
        let mut f = Fixture::new(token, &world());
        if token { f.edit_token(4, |t| t.amount = AMOUNT - 1); } else { f.accounts[4].lamports = AMOUNT - 1; }
        reject(f, AMOUNT, ProgramError::InsufficientFunds);
        let mut f = Fixture::new(token, &world());
        if token { f.edit_token(3, |t| t.amount = u64::MAX); } else { f.accounts[2].lamports = u64::MAX; }
        reject(f, AMOUNT, err(Piv1Error::ArithmeticOverflow));
        let mut f = Fixture::new(token, &world()); f.rent.exemption_threshold = f64::INFINITY;
        reject(f, AMOUNT, err(Piv1Error::InvalidRent));
    }
    let mut f = Fixture::new(false, &world()); let all = f.accounts[4].lamports;
    success(&mut f, all); assert_eq!(f.accounts[4].lamports, 0);
}

#[test]
fn key_storage_aliases_and_required_buffer_borrows_reject_before_effects() {
    for token in [false, true] { for kind in 0..3 {
        let mut f = Fixture::new(token, &world()); let before = f.clone(); let data = f.data(AMOUNT); let rent = f.rent.clone();
        f.with_infos(|a| {
            let mut a = a.to_vec(); match kind { 0 => a[4].key = a[2].key,
                1 => a[4].data = a[2].data.clone(), _ => a[4].lamports = a[2].lamports.clone() }
            assert_eq!(execute(&PROGRAM, &a, &data, || Ok(rent), |_, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountAlias)));
        }); assert_eq!(f, before);
    } }
    for token in [false, true] { for index in if token { vec![0, 4, 3, 6] } else { vec![0, 4, 2] } {
        for lamports in [false, true] {
            let mut f = Fixture::new(token, &world()); let before = f.clone(); let data = f.data(AMOUNT); let rent = f.rent.clone();
            f.with_infos(|a| {
                if lamports {
                    let _held = a[index].try_borrow_lamports().unwrap();
                    assert_eq!(execute(&PROGRAM, a, &data, || Ok(rent), |_, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountBorrowFailed)));
                } else {
                    let _held = a[index].try_borrow_data().unwrap();
                    assert_eq!(execute(&PROGRAM, a, &data, || Ok(rent), |_, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountBorrowFailed)));
                }
            }); assert_eq!(f, before);
        }
    } }
}

#[test]
fn cpi_errors_and_every_supplied_account_postcondition_fail_without_config_commit_or_event() {
    for token in [false, true] {
        let base = Fixture::new(token, &world());
        // Returned runtime/CPI error is preserved even after a staged transfer.
        let mut staged = base.clone(); let data = staged.data(AMOUNT); let rent = staged.rent.clone();
        assert_eq!(staged.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(rent), |ix, infos| {
            transfer(token, AMOUNT, ix, infos)?; Err(ProgramError::Custom(9123))
        }, |_| panic!())), Err(ProgramError::Custom(9123)));
        assert_ne!(staged, base); assert_eq!(staged.accounts[0], base.accounts[0]);
        // Host transaction model explicitly discards staged effects, then retries.
        let mut retry = base.clone(); success(&mut retry, AMOUNT);
        for index in 0..base.accounts.len() {
            let mut staged = base.clone(); let data = staged.data(AMOUNT); let rent = staged.rent.clone();
            assert_eq!(staged.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(rent), |ix, infos| {
                transfer(token, AMOUNT, ix, infos)?;
                **a[index].try_borrow_mut_lamports()? += 1; Ok(())
            }, |_| panic!())), Err(err(Piv1Error::ContributionObservationMismatch)));
            assert_eq!(staged.accounts[0].data, base.accounts[0].data);
        }
        for index in [0, 1, 3, 5] {
            let mut staged = base.clone(); let data = staged.data(AMOUNT); let rent = staged.rent.clone();
            assert_eq!(staged.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(rent), |ix, infos| {
                transfer(token, AMOUNT, ix, infos)?; a[index].try_borrow_mut_data()?[0] ^= 1; Ok(())
            }, |_| panic!())), Err(err(Piv1Error::ContributionObservationMismatch)));
        }
        let mut staged = base.clone(); let data = staged.data(AMOUNT); let rent = staged.rent.clone();
        assert_eq!(staged.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(rent), |_, _| Ok(()), |_| panic!())),
            Err(err(Piv1Error::ContributionObservationMismatch)));
        assert_eq!(staged, base);
    }
}

#[test]
fn opaque_authority_and_executable_bytes_are_preserved_without_wallet_policy() {
    let mut f = Fixture::new(true, &world()); f.accounts[5].data = vec![0xcc; 65_537];
    f.accounts[7].data = vec![0xdd; 131_073]; success(&mut f, AMOUNT);
    let mut f = Fixture::new(true, &world()); f.accounts[5].owner = spl_token::ID;
    f.accounts[5].data = vec![0; Multisig::LEN - 1]; success(&mut f, AMOUNT);
}
