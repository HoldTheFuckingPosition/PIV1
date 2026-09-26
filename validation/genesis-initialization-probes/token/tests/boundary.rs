use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::program_error::ProgramError};
use piv1_genesis_initialization_token::{process_instruction, validate_instruction, HOST_UNAVAILABLE};

#[test]
fn only_exact_canonical_initialize_account3_wire_profile_is_accepted() {
    let mut data = vec![18]; data.extend([217;32]);
    assert_eq!(validate_instruction(&spl_token::ID,2,&data),Ok(()));
    assert_eq!(validate_instruction(&Pubkey::new_from_array([218;32]),2,&data),Err(ProgramError::IncorrectProgramId));
    for count in [0,1,3,35,usize::MAX] {
        assert_eq!(validate_instruction(&spl_token::ID,count,&data),Err(ProgramError::InvalidArgument));
    }
    for length in 0..33 {
        assert_eq!(validate_instruction(&spl_token::ID,2,&data[..length]),Err(ProgramError::InvalidInstructionData));
    }
    for opcode in 0..=255 {
        if opcode == 18 {continue;}
        data[0]=opcode;
        assert_eq!(validate_instruction(&spl_token::ID,2,&data),Err(ProgramError::InvalidInstructionData));
    }
    data[0]=18;data.push(0);
    assert_eq!(validate_instruction(&spl_token::ID,2,&data),Err(ProgramError::InvalidInstructionData));
}

#[test]
fn ordinary_host_cannot_run_the_token_processor_or_mutate_accounts() {
    let target=Pubkey::new_from_array([217;32]);let mint=Pubkey::new_from_array([218;32]);
    let owner=spl_token::ID;let mut target_lamports=123;let mut mint_lamports=456;
    let mut target_bytes=[0;165];let mut mint_bytes=[0;82];
    {
        let accounts=[AccountInfo::new(&target,false,true,&mut target_lamports,&mut target_bytes,&owner,false,0),
            AccountInfo::new(&mint,false,false,&mut mint_lamports,&mut mint_bytes,&owner,false,0)];
        let mut valid=vec![18];valid.extend([219;32]);
        for data in [Vec::new(),valid] {
            assert_eq!(process_instruction(&spl_token::ID,&accounts,&data),Err(ProgramError::Custom(HOST_UNAVAILABLE)));
        }
    }
    assert_eq!((target_lamports,mint_lamports),(123,456));
    assert_eq!(target_bytes,[0;165]);assert_eq!(mint_bytes,[0;82]);
}
