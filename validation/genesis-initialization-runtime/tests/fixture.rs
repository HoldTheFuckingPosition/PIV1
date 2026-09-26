//! Independent expected state/operations; no initializer or model factory runs on host.
#[path = "../../genesis-preflight-runtime/tests/fixture.rs"]
mod previous;
pub use previous::support;
use anchor_lang::prelude::Pubkey;
use previous::World as Previous;
use support::{key, BackingAccount, Fixture, PROGRAM};
use piv1::instructions::initialize::GenesisModelParameters;

pub const SIZES: [usize; 16] = [1014,891,210,84,84,84,84,84,84,0,0,0,0,0,165,165];
pub fn floor(slot: usize) -> u64 { (128 + SIZES[slot] as u64) * 3480 * 2 }
pub fn system() -> Pubkey { anchor_lang::solana_program::system_program::ID }
pub fn authority() -> Pubkey { Pubkey::find_program_address(&[b"authority"], &PROGRAM).0 }
pub struct World { pub f: Fixture, pub parameters: GenesisModelParameters, pub payer: usize }
impl World {
    pub fn new(shared: bool, prefunded: bool, paused: bool) -> Self {
        let Previous { mut f, mut parameters } = Previous::new(shared);
        let payer = if shared {29} else {30};
        let recipients = f.accounts.split_off(payer);
        f.accounts.push(BackingAccount { key:key(222), owner:system(), data:vec![], executable:false,
            writable:true, signer:true, lamports:1_000_000_000 });
        // Runtime replaces these two executable metadata fixtures with its
        // real System builtin and the exact loaded canonical Token artifact.
        for address in [system(),spl_token::ID] {
            f.accounts.push(BackingAccount { key:address, owner:key(223), data:vec![], executable:true,
                writable:false, signer:false, lamports:1 });
        }
        f.accounts.extend(recipients);
        if prefunded {
            for slot in 0..16 {
                f.accounts[7+slot].lamports = match slot {
                    // Include an empty PendingSol and above-floor Token targets:
                    // later normalization must not subsidize its original rent.
                    9 => 0,
                    14 => floor(slot)+55,
                    15 => floor(slot)+89,
                    _ if slot % 3 == 0 => floor(slot)+17,
                    _ if slot % 3 == 1 => floor(slot)-1,
                    _ => floor(slot),
                };
            }
        }
        parameters.initially_paused=paused;
        f.inner_data=parameters.encode().unwrap().to_vec();f.rebuild_message();
        assert_eq!(f.accounts.len(),if shared {34}else{35});
        Self { f, parameters, payer }
    }
    fn derive(&self, slot: usize) -> (Pubkey,u8) {
        if (3..9).contains(&slot) {
            Pubkey::find_program_address(&[b"guardian-reward",key(96-(slot-3) as u8).as_ref(),&0_u64.to_le_bytes(),&[(slot-3) as u8]],&PROGRAM)
        } else {
            let seed:&[u8]=match slot {0=>b"config",1=>b"distribution",2=>b"guardian-registry",
                9=>b"pending-sol",10=>b"principal-sol",11=>b"operational-sol",12=>b"distribution-escrow",
                13=>b"kif-sol",14=>b"principal-jito-vault",15=>b"pending-jito-vault",_=>unreachable!()};
            Pubkey::find_program_address(&[seed],&PROGRAM)
        }
    }
    pub fn token_bytes(&self) -> Vec<u8> {
        let mut bytes=vec![0;165];bytes[..32].copy_from_slice(self.parameters.protocol.jitosol_mint.as_ref());
        bytes[32..64].copy_from_slice(authority().as_ref());bytes[108]=1;bytes
    }
    pub fn state_bytes(&self) -> Vec<Vec<u8>> {
        // Literal Borsh wire recipe independently derived from the accepted layout.
        // This deliberately does not use StateEnvelope, ApprovedGenesisModel,
        // production initialization helpers or their proposed-state factories.
        let bump=|slot|{let (address,bump)=self.derive(slot);assert_eq!(address,self.f.accounts[7+slot].key);bump};
        let address=|slot|self.derive(slot).0;
        let p=self.parameters.protocol;
        let mut config=vec![98,115,11,164,170,207,163,20,1,1,u8::from(self.parameters.initially_paused)];
        config.extend([bump(0),Pubkey::find_program_address(&[b"authority"],&PROGRAM).1,bump(1),bump(14),bump(15),bump(9),bump(10),bump(11),bump(12),bump(13),bump(2)]);
        for key in [p.stake_pool_program,p.stake_pool,p.validator_list,p.reserve_stake,p.jitosol_mint,
            spl_token::ID,Pubkey::from_str_const("Stake11111111111111111111111111111111111111"),system(),p.manager_fee_account,p.referrer_token_account,
            authority(),address(1),address(14),address(15),address(9),address(10),address(11),address(12),address(13),
            self.parameters.htfp_recipient,self.parameters.team_owner_recipient,address(2)] {config.extend(key.to_bytes());}
        for number in [10_000_u16,5900,1950,1950,200,1,1] {config.extend(number.to_le_bytes());}
        for seconds in [864_000_i64,86_400] {config.extend(seconds.to_le_bytes());}
        config.extend([0,0]); // Both optional timestamps are None.
        config.extend([0;19*8]); // Sequence plus eighteen principal/yield/KIF/audit ledgers.
        config.extend(self.parameters.kif_anchor_timestamp.to_le_bytes());
        config.extend(2_592_000_i64.to_le_bytes());config.extend([0;8+64]);
        assert_eq!(config.len(),998);config.resize(1014,0);
        let mut distribution=vec![0;891];distribution[..11].copy_from_slice(&[104,51,125,187,226,55,209,99,1,bump(1),1]);
        let mut registry=vec![72,14,254,2,76,233,97,92,1,bump(2)];registry.extend([0;8]);
        for slot in 0..6 {registry.extend(key(96-slot).to_bytes());}assert_eq!(registry.len(),210);
        let mut result=vec![config,distribution,registry];
        for slot in 0..6 {
            let mut reward=vec![169,109,89,17,75,171,105,39,1,bump(3+slot),slot as u8];
            reward.extend([0;8]);reward.extend(key(96-slot as u8).to_bytes());
            reward.extend([0;1+24]);assert_eq!(reward.len(),76);reward.resize(84,0);result.push(reward);
        }
        result
    }
    pub fn target_owner(&self, slot: usize) -> Pubkey {
        if slot<9 {PROGRAM} else if slot<14 {system()} else {spl_token::ID}
    }
    pub fn shortfall(&self) -> u64 { (0..16).map(|slot|floor(slot).saturating_sub(self.f.accounts[7+slot].lamports)).sum() }
    pub fn sweep(&self) -> u64 { (14..16).map(|slot|self.f.accounts[7+slot].lamports.saturating_sub(floor(slot))).sum() }
    pub fn target_balance(&self, slot: usize) -> u64 {
        if slot>=14 {floor(slot)} else {self.f.accounts[7+slot].lamports.max(floor(slot))+if slot==9 {self.sweep()} else {0}}
    }
}
#[derive(Debug,PartialEq)]
pub struct ExpectedCall { pub program:Pubkey, pub data:Vec<u8>, pub accounts:Vec<(Pubkey,bool,bool)> }
pub fn expected_calls(w:&World)->Vec<ExpectedCall> {
    let mut calls=vec![];let payer=w.f.accounts[w.payer].key;
    let mut push=|tag:u32,payload:Vec<u8>,accounts|{let mut data=tag.to_le_bytes().to_vec();data.extend(payload);calls.push(ExpectedCall {program:system(),data,accounts});};
    for (slot,size) in SIZES.into_iter().enumerate() {
        let target=w.f.accounts[7+slot].key;let amount=floor(slot).saturating_sub(w.f.accounts[7+slot].lamports);
        if amount>0 {push(2,amount.to_le_bytes().to_vec(),vec![(payer,true,true),(target,false,true)]);}
        if slot>=14 {
            let excess=w.f.accounts[7+slot].lamports.saturating_sub(floor(slot));
            if excess>0 {push(2,excess.to_le_bytes().to_vec(),vec![(target,true,true),(w.f.accounts[16].key,false,true)]);}
        }
        if size>0 {
            push(8,(size as u64).to_le_bytes().to_vec(),vec![(target,true,true)]);
            push(1,w.target_owner(slot).to_bytes().to_vec(),vec![(target,true,true)]);
        }
    }
    for slot in [14,15] {let mut data=vec![18];data.extend(authority().to_bytes());calls.push(ExpectedCall {
        program:spl_token::ID,data,accounts:vec![(w.f.accounts[7+slot].key,false,true),(w.parameters.protocol.jitosol_mint,false,false)]});}
    calls
}
