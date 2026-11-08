//! One authenticated current route_v2 profile: Manifest, JitoSOL base -> WSOL
//! quote, exact input, one hop. API instructions are never executed or decoded
//! as authority. ABI/provenance: D108 retained current JUP6-owned Anchor IDL.
use anchor_lang::{prelude::{AccountInfo, Pubkey, Rent}, solana_program::{
    instruction::{AccountMeta, Instruction}, program_error::ProgramError,
    program_option::COption, program_pack::Pack, system_program, bpf_loader_upgradeable}};
use spl_token::state::{Account, AccountState, Mint};
use crate::{errors::Piv1Error, instruction_errors::piv1_error_code};

pub const JUPITER: Pubkey = Pubkey::from_str_const("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
pub const MANIFEST: Pubkey = Pubkey::from_str_const("MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms");
pub const WSOL: Pubkey = Pubkey::from_str_const("So11111111111111111111111111111111111111112");
const JUPITER_DATA: Pubkey = Pubkey::from_str_const("4Ec7ZxZS6Sbdg5UGSLHbAnM7GQHp2eFd4KYWRexAipQT");
const MANIFEST_DATA: Pubkey = Pubkey::from_str_const("G92J4ZFggKZ9LbsQ2zBpajRvRto7ed7uRuJJqWsX4BhJ");
pub const ROUTE_ACCOUNTS: usize = 12;
pub const SWAP_AUTHORITY: &[u8] = b"swap-authority";
pub const SWAP_INPUT: &[u8] = b"swap-jito";
pub const SWAP_OUTPUT: &[u8] = b"swap-wsol";

fn bad() -> ProgramError { ProgramError::Custom(piv1_error_code(Piv1Error::InvalidSwapInput)) }
pub fn token(a: &AccountInfo<'_>, mint: &Pubkey, authority: &Pubkey, native: bool) -> Result<Account,ProgramError> {
    if a.owner != &spl_token::ID || a.executable || a.try_borrow_data()?.len() != Account::LEN { return Err(bad()); }
    let value=Account::unpack(&a.try_borrow_data()?)?;
    if value.mint != *mint || value.owner != *authority || value.state != AccountState::Initialized
        || value.delegate != COption::None || value.delegated_amount != 0 || value.close_authority != COption::None
        || value.is_native.is_some() != native { return Err(bad()); }
    if let COption::Some(floor)=value.is_native {
        if a.lamports().checked_sub(floor).is_none_or(|available| available < value.amount) { return Err(bad()); }
    }
    Ok(value)
}
fn external_program(a: &AccountInfo<'_>, key: Pubkey, data_key: Pubkey) -> Result<(),ProgramError> {
    let bytes=a.try_borrow_data()?;
    if *a.key!=key || !a.executable || a.owner!=&bpf_loader_upgradeable::ID || a.is_writable
        || bytes.len()!=36 || bytes[..4]!=2_u32.to_le_bytes() || bytes[4..]!=data_key.to_bytes() { return Err(bad()); }
    Ok(())
}
fn key_at(data: &[u8], offset: usize) -> Result<Pubkey,ProgramError> {
    Ok(Pubkey::new_from_array(data.get(offset..offset+32).ok_or_else(bad)?.try_into().map_err(|_|bad())?))
}
fn dynamic_header(a:&AccountInfo<'_>,header:usize,offset:usize,discriminator:u64,rent:&Rent)->Result<(),ProgramError>{
    let d=a.try_borrow_data()?;
    if a.owner!=&MANIFEST || a.executable || !a.is_writable || d.len()<header || d.len()>262_144
        || d[..8]!=discriminator.to_le_bytes() || a.lamports()<rent.minimum_balance(d.len()) { return Err(bad()); }
    let allocated=u32::from_le_bytes(d[offset..offset+4].try_into().map_err(|_|bad())?) as usize;
    if header.checked_add(allocated)!=Some(d.len()) { return Err(bad()); }
    Ok(())
}

/// Suffix: authority,input,output,WSOLmint,Jupiter,eventAuthority,Manifest,
/// market,baseVault,quoteVault,quoteGlobal,quoteGlobalVault. No extra accounts.
pub fn authenticate(a:&[AccountInfo<'_>],b:usize,jito:Pubkey,rent:&Rent)->Result<(),ProgramError>{
    if a.len()!=b+ROUTE_ACCOUNTS { return Err(bad()); }
    external_program(&a[b+4],JUPITER,JUPITER_DATA)?;
    external_program(&a[b+6],MANIFEST,MANIFEST_DATA)?;
    let event=Pubkey::find_program_address(&[b"__event_authority"],&JUPITER).0;
    if a[b+5].key!=&event || a[b+5].executable || a[b+5].is_writable { return Err(bad()); }
    if a[b+3].key!=&WSOL || a[b+3].owner!=&spl_token::ID || a[b+3].executable || a[b+3].is_writable
        || a[b+3].try_borrow_data()?.len()!=Mint::LEN || a[b+3].lamports()<rent.minimum_balance(Mint::LEN) { return Err(bad()); }
    let mint=Mint::unpack(&a[b+3].try_borrow_data()?)?;
    if mint.decimals!=9 || mint.mint_authority!=COption::None || mint.freeze_authority!=COption::None || mint.supply!=0 { return Err(bad()); }
    dynamic_header(&a[b+7],256,152,4859840929024028656,rent)?;
    let market=a[b+7].try_borrow_data()?;
    if market[8]!=0 || market[9]!=9 || market[10]!=9 || key_at(&market,16)?!=jito || key_at(&market,48)?!=WSOL { return Err(bad()); }
    for (i,mint,offset,bump_offset) in [(b+8,jito,80,11),(b+9,WSOL,112,12)] {
        let (vault,bump)=Pubkey::find_program_address(&[b"vault",a[b+7].key.as_ref(),mint.as_ref()],&MANIFEST);
        if *a[i].key!=vault || key_at(&market,offset)?!=vault || market[bump_offset]!=bump
            || !a[i].is_writable || a[i].lamports()<rent.minimum_balance(Account::LEN) { return Err(bad()); }
        token(&a[i],&mint,&vault,mint==WSOL)?;
    }
    dynamic_header(&a[b+10],96,88,10787423733276977665,rent)?;
    let global=Pubkey::find_program_address(&[b"global",WSOL.as_ref()],&MANIFEST).0;
    let vault=Pubkey::find_program_address(&[b"global-vault",WSOL.as_ref()],&MANIFEST).0;
    let d=a[b+10].try_borrow_data()?;
    if a[b+10].key!=&global || a[b+11].key!=&vault || key_at(&d,8)?!=WSOL || key_at(&d,40)?!=vault
        || !a[b+11].is_writable || a[b+11].lamports()<rent.minimum_balance(Account::LEN) { return Err(bad()); }
    token(&a[b+11],&WSOL,&vault,true)?;
    Ok(())
}

pub fn instruction(a:&[AccountInfo<'_>],b:usize,input:u64,minimum:u64)->Instruction {
    let mut data=vec![187,100,250,204,49,196,175,20];
    data.extend(input.to_le_bytes()); data.extend(minimum.to_le_bytes());
    data.extend([0;6]); // slippage, platform fee, positive-slippage fee: all zero
    data.extend(1_u32.to_le_bytes());data.extend([116,1]);data.extend(10_000_u16.to_le_bytes());data.extend([0,1]);
    // Duplicate authority inherits signer AND writable privileges through union.
    // It is an isolated zero-native PDA, never PivAuthority or OperationalSOL.
    let slots=[b,b+1,b+2,10,b+3,12,12,b+4,b+5,b+4,b+6,b,b,b+7,11,b+1,b+2,b+8,b+9,12,10,12,b+3,b+10,b+11];
    let accounts=slots.iter().enumerate().map(|(i,index)|AccountMeta { pubkey:*a[*index].key,
        is_signer:i==0, is_writable:matches!(i,1|2|11|13|15|16|17|18|23|24) }).collect();
    Instruction { program_id:JUPITER, accounts,data }
}

/// Build a canonical unused System PDA. Prefunds are normalized separately,
/// before token allocation; they never become WSOL output or technical rent.
pub fn unused(a:&AccountInfo<'_>,key:Pubkey)->Result<(),ProgramError>{
    if *a.key!=key || a.owner!=&system_program::ID || a.executable || a.is_signer
        || !a.is_writable || !a.try_borrow_data()?.is_empty() { return Err(bad()); }
    Ok(())
}
