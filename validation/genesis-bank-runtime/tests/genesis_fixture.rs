//! Current-SDK literal port of the Task 2.30/2.32 synthetic fixture and oracle.
//! No PIV1/Anchor/SPL host implementation or previously successful output runs.

use {
    solana_account::{AccountSharedData, WritableAccount},
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    solana_sdk_ids::{bpf_loader_upgradeable, system_program, sysvar},
};

pub const PROGRAM: Pubkey = Pubkey::new_from_array([217; 32]);
pub const SQUADS: Pubkey = Pubkey::from_str_const("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");
pub const TOKEN: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const JITO_PROGRAM: Pubkey = Pubkey::from_str_const("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");
pub const JITO_POOL: Pubkey = Pubkey::from_str_const("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb");
pub const JITO_MINT: Pubkey = Pubkey::from_str_const("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
pub const STAKE: Pubkey = Pubkey::from_str_const("Stake11111111111111111111111111111111111111");
pub const SIZES: [usize; 16] = [1014, 891, 210, 84, 84, 84, 84, 84, 84, 0, 0, 0, 0, 0, 165, 165];
pub const OUTER_DATA: [u8; 8] = [194, 8, 161, 87, 153, 164, 25, 171];

pub fn key(byte: u8) -> Pubkey { Pubkey::new_from_array([byte; 32]) }
pub fn floor(slot: usize) -> u64 { (128 + SIZES[slot] as u64) * 6_960 }
pub fn authority() -> Pubkey { Pubkey::find_program_address(&[b"authority"], &PROGRAM).0 }
pub fn program_data(program: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[program.as_ref()], &bpf_loader_upgradeable::id()).0
}
fn push_key(bytes: &mut Vec<u8>, key: Pubkey) { bytes.extend(key.to_bytes()); }
fn u32v(bytes: &mut Vec<u8>, value: u32) { bytes.extend(value.to_le_bytes()); }
fn u64v(bytes: &mut Vec<u8>, value: u64) { bytes.extend(value.to_le_bytes()); }
fn fee(bytes: &mut Vec<u8>, numerator: u64) { u64v(bytes, 100 + numerator); u64v(bytes, numerator); }

#[derive(Clone)]
pub struct Role {
    pub key: Pubkey,
    pub owner: Pubkey,
    pub data: Vec<u8>,
    pub executable: bool,
    pub writable: bool,
    pub signer: bool,
    pub lamports: u64,
}
impl Role {
    fn new(key: Pubkey, owner: Pubkey, data: Vec<u8>, executable: bool, writable: bool) -> Self {
        // Unlike Mollusk metadata fixtures, Bank needs positive balances for
        // non-target accounts. Targets are replaced by exact prefund values.
        let lamports = (128 + data.len() as u64) * 6_960;
        Self { key, owner, data, executable, writable, signer: false, lamports }
    }
    pub fn account(&self) -> AccountSharedData {
        let mut result = AccountSharedData::new(self.lamports, self.data.len(), &self.owner);
        result.set_data_from_slice(&self.data);
        result.set_executable(self.executable);
        result.set_rent_epoch(u64::MAX);
        result
    }
}

pub struct World {
    pub roles: Vec<Role>,
    pub inner_data: Vec<u8>,
    pub outer: Instruction,
    pub payer: usize,
    pub paused: bool,
    pub shared: bool,
    pub recipients: [Pubkey; 2],
    pub vault: Pubkey,
}

impl World {
    pub fn new(shared: bool, prefunded: bool) -> Self { Self::with_format(shared, prefunded, false) }

    pub fn production(shared: bool, prefunded: bool) -> Self { Self::with_format(shared, prefunded, true) }

    fn with_format(shared: bool, prefunded: bool, native: bool) -> Self {
        let (multisig, multisig_bump) = Pubkey::find_program_address(
            &[b"multisig", b"multisig", key(80).as_ref()], &SQUADS);
        let (transaction, transaction_bump) = Pubkey::find_program_address(
            &[b"multisig", multisig.as_ref(), b"transaction", &19_u64.to_le_bytes()], &SQUADS);
        let (proposal, proposal_bump) = Pubkey::find_program_address(
            &[b"multisig", multisig.as_ref(), b"transaction", &19_u64.to_le_bytes(), b"proposal"], &SQUADS);
        let (vault, vault_bump) = Pubkey::find_program_address(
            &[b"multisig", multisig.as_ref(), b"vault", &[7]], &SQUADS);
        // Independent native witnesses intentionally differ from the old probe.
        let witnesses = if native { [31, 202] } else { [0, 255] };
        let recipients = witnesses.map(|index| Pubkey::find_program_address(
            &[b"multisig", multisig.as_ref(), b"vault", &[index]], &SQUADS).0);
        let protocol = [JITO_PROGRAM, JITO_POOL, key(201), key(202), JITO_MINT,
            key(203), key(if shared { 203 } else { 204 })];
        let mut inner_data = if native { b"PIV1IN01" } else { b"PIV1GM01" }.to_vec();
        inner_data.extend([1, 7, u8::from(prefunded)]);
        for address in protocol.into_iter().chain(recipients) { push_key(&mut inner_data, address); }
        u64v(&mut inner_data, 0); inner_data.extend([5, 4, 3, 2, 1, 0]);
        if native { inner_data.extend(witnesses); }
        assert_eq!(inner_data.len(), if native { 315 } else { 313 });

        let mut multisig_bytes = vec![224, 116, 121, 186, 68, 161, 79, 236];
        push_key(&mut multisig_bytes, key(80)); push_key(&mut multisig_bytes, Pubkey::default());
        multisig_bytes.extend(4_u16.to_le_bytes()); u32v(&mut multisig_bytes, 10);
        u64v(&mut multisig_bytes, 19); u64v(&mut multisig_bytes, 3);
        multisig_bytes.extend([0, multisig_bump]); u32v(&mut multisig_bytes, 6);
        for tag in 91..97 { push_key(&mut multisig_bytes, key(tag)); multisig_bytes.push(7); }
        multisig_bytes.resize(330, 0);
        let mut proposal_bytes = vec![26, 94, 189, 187, 116, 136, 53, 33];
        push_key(&mut proposal_bytes, multisig); u64v(&mut proposal_bytes, 19);
        proposal_bytes.push(3); proposal_bytes.extend(90_i64.to_le_bytes()); proposal_bytes.push(proposal_bump);
        u32v(&mut proposal_bytes, 4);
        for tag in 91..95 { push_key(&mut proposal_bytes, key(tag)); }
        u32v(&mut proposal_bytes, 0); u32v(&mut proposal_bytes, 0); proposal_bytes.resize(646, 0);
        let mut program_bytes = 2_u32.to_le_bytes().to_vec(); push_key(&mut program_bytes, program_data(PROGRAM));
        // Replaced with exact real ProgramData + ELF before Bank construction.
        let mut pd = 3_u32.to_le_bytes().to_vec(); u64v(&mut pd, 0); pd.push(1); push_key(&mut pd, vault);
        let mut roles = vec![
            Role::new(PROGRAM, bpf_loader_upgradeable::id(), program_bytes, true, false),
            Role::new(program_data(PROGRAM), bpf_loader_upgradeable::id(), pd, false, false),
            Role::new(multisig, SQUADS, multisig_bytes, false, false),
            Role::new(proposal, SQUADS, proposal_bytes, false, false),
            Role::new(transaction, SQUADS, vec![], false, false),
            Role::new(vault, system_program::id(), vec![], false, false),
            Role::new(sysvar::instructions::id(), sysvar::id(), vec![], false, false),
        ];
        roles[5].signer = true;
        for slot in 0..16 {
            let mut role = Role::new(Self::derive(slot).0, system_program::id(), vec![], false, true);
            role.lamports = if !prefunded { 0 } else { match slot {
                9 => 0, 14 => floor(slot) + 55, 15 => floor(slot) + 89,
                _ if slot % 3 == 0 => floor(slot) + 17,
                _ if slot % 3 == 1 => floor(slot) - 1,
                _ => floor(slot),
            }};
            roles.push(role);
        }
        let (withdraw, withdraw_bump) = Pubkey::find_program_address(
            &[JITO_POOL.as_ref(), b"withdraw"], &JITO_PROGRAM);
        let mut jito_program_bytes = 2_u32.to_le_bytes().to_vec();
        push_key(&mut jito_program_bytes, program_data(JITO_PROGRAM));
        let mut pool = vec![1];
        for tag in [1, 2, 3] { push_key(&mut pool, key(tag)); }
        pool.push(withdraw_bump);
        for address in [key(201), key(202), JITO_MINT, key(203), TOKEN] { push_key(&mut pool, address); }
        for value in [1011, 1012, 1013] { u64v(&mut pool, value); }
        pool.extend([0; 48]); // Default lockup: timestamp8/epoch8/custodian32.
        fee(&mut pool, 1); pool.extend([0, 0, 0]); // Future and both preferred options None.
        fee(&mut pool, 3); fee(&mut pool, 4); pool.extend([0, 21, 0]);
        fee(&mut pool, 6); pool.extend([23, 0]); fee(&mut pool, 7); pool.push(0);
        u64v(&mut pool, 1025); u64v(&mut pool, 1026); pool.resize(611, 0xA5);
        let mut list = vec![2]; u32v(&mut list, 1); u32v(&mut list, 0); list.resize(82, 0xA5);
        let mut reserve = 1_u32.to_le_bytes().to_vec(); u64v(&mut reserve, 123);
        push_key(&mut reserve, withdraw); push_key(&mut reserve, withdraw);
        reserve.extend([0; 48]); reserve.resize(200, 0xA5);
        let mut mint = vec![0; 82]; mint[..4].copy_from_slice(&1_u32.to_le_bytes());
        mint[4..36].copy_from_slice(withdraw.as_ref()); mint[36..44].copy_from_slice(&9999_u64.to_le_bytes());
        mint[44] = 9; mint[45] = 1;
        let mut receiver = vec![0; 165]; receiver[..32].copy_from_slice(JITO_MINT.as_ref());
        receiver[32..64].copy_from_slice(key(207).as_ref()); receiver[64..72].copy_from_slice(&99_u64.to_le_bytes());
        receiver[72..76].copy_from_slice(&1_u32.to_le_bytes()); receiver[76..108].copy_from_slice(key(208).as_ref());
        receiver[108] = 1; receiver[121..129].copy_from_slice(&1_u64.to_le_bytes());
        receiver[129..133].copy_from_slice(&1_u32.to_le_bytes()); receiver[133..].copy_from_slice(key(209).as_ref());
        roles.extend([
            Role::new(JITO_PROGRAM, bpf_loader_upgradeable::id(), jito_program_bytes, true, false),
            Role::new(JITO_POOL, JITO_PROGRAM, pool, false, false),
            Role::new(key(201), JITO_PROGRAM, list, false, false),
            Role::new(key(202), STAKE, reserve, false, false),
            Role::new(JITO_MINT, TOKEN, mint, false, false),
            Role::new(key(203), TOKEN, receiver.clone(), false, false),
        ]);
        if !shared { roles.push(Role::new(key(204), TOKEN, receiver, false, false)); }
        let payer = roles.len(); assert_eq!(payer, if shared { 29 } else { 30 });
        let mut rent_payer = Role::new(key(222), system_program::id(), vec![], false, true);
        rent_payer.signer = true; rent_payer.lamports = 1_000_000_000; roles.push(rent_payer);
        // Runtime genesis provides the actual builtin and hash-bound Token program.
        roles.push(Role::new(system_program::id(), Pubkey::default(), vec![], true, false));
        roles.push(Role::new(TOKEN, bpf_loader_upgradeable::id(), vec![], true, false));
        for recipient in recipients { roles.push(Role::new(recipient, system_program::id(), vec![], false, false)); }
        assert_eq!(roles.len(), if shared { 34 } else { 35 });

        let mut order: Vec<usize> = (0..roles.len()).collect();
        order.sort_by_key(|index| match (roles[*index].signer, roles[*index].writable) {
            (true, true) => 0, (true, false) => 1, (false, true) => 2, _ => 3,
        });
        let indices: Vec<u8> = (0..roles.len()).map(|index| order.iter().position(|p| *p == index).unwrap() as u8).collect();
        let mut tx = vec![168, 250, 162, 100, 81, 14, 162, 207];
        push_key(&mut tx, multisig); push_key(&mut tx, key(91)); u64v(&mut tx, 19);
        tx.extend([transaction_bump, 7, vault_bump]); u32v(&mut tx, 0);
        tx.extend([2, 1, 16]); u32v(&mut tx, roles.len() as u32);
        for index in &order { push_key(&mut tx, roles[*index].key); }
        u32v(&mut tx, 1); tx.push(indices[0]); u32v(&mut tx, roles.len() as u32);
        tx.extend(indices); u32v(&mut tx, inner_data.len() as u32); tx.extend(&inner_data); u32v(&mut tx, 0);
        roles[4].data = tx; roles[4].lamports = (128 + roles[4].data.len() as u64) * 6_960;
        let mut outer_accounts = vec![AccountMeta::new_readonly(multisig, false), AccountMeta::new(proposal, false),
            AccountMeta::new_readonly(transaction, false), AccountMeta::new_readonly(key(91), true)];
        outer_accounts.extend(order.into_iter().map(|index| AccountMeta {
            pubkey: roles[index].key, is_signer: roles[index].signer && index != 5,
            is_writable: roles[index].writable || index == 3,
        }));
        Self { roles, inner_data, outer: Instruction { program_id: SQUADS, accounts: outer_accounts,
            data: OUTER_DATA.to_vec() }, payer, paused: prefunded, shared, recipients, vault }
    }

    pub fn derive(slot: usize) -> (Pubkey, u8) {
        if (3..9).contains(&slot) {
            Pubkey::find_program_address(&[b"guardian-reward", key(96 - (slot - 3) as u8).as_ref(),
                &0_u64.to_le_bytes(), &[(slot - 3) as u8]], &PROGRAM)
        } else {
            let seed: &[u8] = match slot { 0 => b"config", 1 => b"distribution", 2 => b"guardian-registry",
                9 => b"pending-sol", 10 => b"principal-sol", 11 => b"operational-sol", 12 => b"distribution-escrow",
                13 => b"kif-sol", 14 => b"principal-jito-vault", 15 => b"pending-jito-vault", _ => unreachable!() };
            Pubkey::find_program_address(&[seed], &PROGRAM)
        }
    }
    pub fn target_owner(slot: usize) -> Pubkey {
        if slot < 9 { PROGRAM } else if slot < 14 { system_program::id() } else { TOKEN }
    }
    pub fn shortfall(&self) -> u64 { (0..16).map(|slot| floor(slot).saturating_sub(self.roles[7 + slot].lamports)).sum() }
    pub fn sweep(&self) -> u64 { (14..16).map(|slot| self.roles[7 + slot].lamports.saturating_sub(floor(slot))).sum() }
    pub fn target_balance(&self, slot: usize) -> u64 {
        if slot >= 14 { floor(slot) } else {
            self.roles[7 + slot].lamports.max(floor(slot)) + if slot == 9 { self.sweep() } else { 0 }
        }
    }
    pub fn token_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; 165]; bytes[..32].copy_from_slice(JITO_MINT.as_ref());
        bytes[32..64].copy_from_slice(authority().as_ref()); bytes[108] = 1; bytes
    }
    pub fn state_bytes(&self) -> Vec<Vec<u8>> {
        let bump = |slot| Self::derive(slot).1;
        let address = |slot| Self::derive(slot).0;
        let mut config = vec![98, 115, 11, 164, 170, 207, 163, 20, 1, 1, u8::from(self.paused)];
        config.extend([bump(0), Pubkey::find_program_address(&[b"authority"], &PROGRAM).1, bump(1), bump(14),
            bump(15), bump(9), bump(10), bump(11), bump(12), bump(13), bump(2)]);
        for address in [JITO_PROGRAM, JITO_POOL, key(201), key(202), JITO_MINT, TOKEN, STAKE, system_program::id(),
            key(203), key(if self.shared { 203 } else { 204 }), authority(), address(1), address(14), address(15),
            address(9), address(10), address(11), address(12), address(13), self.recipients[0], self.recipients[1], address(2)] {
            push_key(&mut config, address);
        }
        for number in [10_000_u16, 5900, 1950, 1950, 200, 1, 1] { config.extend(number.to_le_bytes()); }
        for seconds in [864_000_i64, 86_400] { config.extend(seconds.to_le_bytes()); }
        config.extend([0, 0]); config.extend([0; 19 * 8]); config.extend(0_i64.to_le_bytes());
        config.extend(2_592_000_i64.to_le_bytes()); config.extend([0; 8 + 64]);
        assert_eq!(config.len(), 998); config.resize(1014, 0);
        let mut distribution = vec![0; 891];
        distribution[..11].copy_from_slice(&[104, 51, 125, 187, 226, 55, 209, 99, 1, bump(1), 1]);
        let mut registry = vec![72, 14, 254, 2, 76, 233, 97, 92, 1, bump(2)]; registry.extend([0; 8]);
        for slot in 0..6 { push_key(&mut registry, key(96 - slot)); }
        assert_eq!(registry.len(), 210);
        let mut states = vec![config, distribution, registry];
        for slot in 0..6 {
            let mut reward = vec![169, 109, 89, 17, 75, 171, 105, 39, 1, bump(3 + slot), slot as u8];
            reward.extend([0; 8]); push_key(&mut reward, key(96 - slot as u8)); reward.extend([0; 1 + 24]);
            assert_eq!(reward.len(), 76); reward.resize(84, 0); states.push(reward);
        }
        states
    }
    pub fn expected_cpis(&self) -> Vec<(Pubkey, Vec<u8>, Vec<Pubkey>, u8)> {
        let mut result = vec![(PROGRAM, self.inner_data.clone(), self.roles.iter().map(|r| r.key).collect(), 2)];
        let payer = self.roles[self.payer].key;
        let mut system = |tag: u32, payload: Vec<u8>, keys: Vec<Pubkey>| {
            let mut data = tag.to_le_bytes().to_vec(); data.extend(payload);
            result.push((system_program::id(), data, keys, 3));
        };
        for (slot, size) in SIZES.into_iter().enumerate() {
            let target = self.roles[7 + slot].key;
            let amount = floor(slot).saturating_sub(self.roles[7 + slot].lamports);
            if amount > 0 { system(2, amount.to_le_bytes().to_vec(), vec![payer, target]); }
            if slot >= 14 {
                let excess = self.roles[7 + slot].lamports.saturating_sub(floor(slot));
                if excess > 0 { system(2, excess.to_le_bytes().to_vec(), vec![target, self.roles[16].key]); }
            }
            if size > 0 {
                system(8, (size as u64).to_le_bytes().to_vec(), vec![target]);
                system(1, Self::target_owner(slot).to_bytes().to_vec(), vec![target]);
            }
        }
        for slot in [14, 15] {
            let mut data = vec![18]; data.extend(authority().to_bytes());
            result.push((TOKEN, data, vec![self.roles[7 + slot].key, JITO_MINT], 3));
        }
        result
    }
}
