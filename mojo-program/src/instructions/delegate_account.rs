use pinocchio::{
    ProgramResult, account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address, seeds, sysvars::rent::Rent,
};
use ephemeral_rollups_pinocchio::{
    consts::{BUFFER, DELEGATION_METADATA, DELEGATION_PROGRAM_ID, DELEGATION_RECORD, MAGIC_PROGRAM_ID}, 
    seeds::Seed, types::{DelegateAccountArgs, DelegateConfig}, utils::{close_pda_acc, cpi_delegate, make_seed_buf}
};
use ephemeral_rollups_pinocchio::pda::delegation_metadata_pda_from_delegated_account;
use pinocchio_system::instructions::CreateAccount;
use crate::state::GenIxHandler;

pub fn process_delegate_account (
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 0xAbim: Here goes the accounts to be delegated.
    // 0. [] The creator acts as the payer 
    // 1. [] the account pda (is_writable)
    // 2. [] the owner' program 
    // 3. [] the buffer account
    // 4. [] the delegation record
    // 5. [] the delegation metadata
    // 6. [] System Program + ...Other essential accounts...
    
    let [
        creator,
        creator_account,
        owner_program,
        buffer_account,
        delegation_record,
        delegation_metadata,
        system_program,
        _rest @..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };


    // 0xAbim: Size validation
    let mojo_bytes = creator_account.try_borrow_data()?;
    if mojo_bytes.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    // 0xAbim: Validation too. Just unsure what it validates😂😂😂
    let mojo_ser_data: &GenIxHandler = bytemuck::try_from_bytes(
        &mojo_bytes[..GenIxHandler::LEN]
    ).map_err(|_| ProgramError::InvalidAccountData)?;

    // 0xAbim: Security check to ensure size is consistent
    let size = u64::from_le_bytes(mojo_ser_data.size) as usize;
    if size > 256 || size == 0 {
        return Err(ProgramError::InvalidArgument);
    }

    // 0xAbim: Extract the seeds from account data
    let seeds_slice = &mojo_ser_data.seeds[..size];

    // 0xAbim: Verify PDA derivation after seeds are extracted
    let (derived_pda, bump) = find_program_address(&[seeds_slice], &crate::ID);
    if creator_account.key() != &derived_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // 0xAbim: Creating the buffer account for delegation (not a PDA, use invoke)
    let rent_sysvar = Rent::from_account_info(system_program)?;
    CreateAccount{
        from: creator,
        to: buffer_account,
        lamports: rent_sysvar.minimum_balance(creator_account.data_len()),
        space: creator_account.data_len() as u64,
        owner: &crate::ID,
    }.invoke()?; 

    // 0xAbim: Prepare delegation using MagicBlock 
    // delegate_account expects: &[&AccountInfo], seeds: &[&[u8]], bump: u8, config: DelegateConfig
    let pda_seeds: &[&[u8]] = &[seeds_slice];
    let delegation_accounts: &[&AccountInfo] = &[
        creator,
        creator_account,
        owner_program,
        buffer_account,
        delegation_record,
        delegation_metadata,
    ];

    let delegate_config = DelegateConfig {
        commit_frequency_ms: 30000,  // 30 seconds
        ..Default::default()
    };

    // 0xAbim: delegate account ix
    ephemeral_rollups_pinocchio::instruction::delegate::delegate_account(
        delegation_accounts, // spots account error here. TO-FIX
        pda_seeds,
        bump,
        delegate_config,
    ).map_err(|_| ProgramError::InvalidAccountData)?;

    // 0xAbim: REMOVED close_pda_acc - account must stay open during delegation

    Ok(())
}

// 0xAbim: Faced some design decisions here.
// 
// 