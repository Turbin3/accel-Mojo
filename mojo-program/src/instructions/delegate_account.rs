use crate::state::GenIxHandler;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
    sysvars::rent::Rent, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

// 0xAbim: TODO - Version mismatch between pinocchio 0.9.2 and ephemeral-rollups-pinocchio (uses 0.8.4)
// Need to either downgrade pinocchio or wait for ephemeral-rollups-pinocchio update
// use ephemeral_rollups_pinocchio::types::DelegateConfig;

pub fn process_delegate_account(
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

    let [creator, mojo_account_pda, owner_program, buffer_account, delegation_record, delegation_metadata, system_program, _rest @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // 0xAbim: Size validation
    let mojo_bytes = mojo_account_pda.try_borrow_data()?;
    if mojo_bytes.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    let mojo_ser_data: &GenIxHandler = bytemuck::try_from_bytes(&mojo_bytes[..GenIxHandler::LEN])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // 0xAbim: Security check to ensure size is consistent
    let size = u64::from_le_bytes(mojo_ser_data.size) as usize;
    if size > 256 || size == 0 {
        return Err(ProgramError::InvalidArgument);
    }

    // 0xAbim: Extract the seeds from account data
    let seeds_slice = &mojo_ser_data.seeds[..size];

    // 0xAbim: MOVED - Verify PDA derivation after seeds are extracted
    let (derived_pda, bump) = find_program_address(&[seeds_slice], &crate::ID);
    if mojo_account_pda.key() != &derived_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // 0xAbim: Creating the buffer account for delegation (not a PDA, use invoke)
    // FIXED: Use pinocchio Sysvar trait for Rent instead of get()
    let rent_sysvar = Rent::from_account_info(system_program)?;
    CreateAccount {
        from: creator,
        to: buffer_account,
        lamports: rent_sysvar.minimum_balance(mojo_account_pda.data_len()),
        space: mojo_account_pda.data_len() as u64,
        owner: &crate::ID,
    }
    .invoke()?; // FIXED: Changed from invoke_signed since BUFFER is not a signer

    // 0xAbim: TODO - MagicBlock delegation currently stubbed due to pinocchio version mismatch
    // Once ephemeral-rollups-pinocchio updates to pinocchio 0.9.x, uncomment this:
    /*
    let pda_seeds: &[&[u8]] = &[seeds_slice];
    let delegation_accounts: &[&AccountInfo] = &[
        creator, mojo_account_pda, owner_program,
        buffer_account, delegation_record, delegation_metadata,
    ];
    let delegate_config = DelegateConfig {
        commit_frequency_ms: 30000,
        ..Default::default()
    };
    ephemeral_rollups_pinocchio::instruction::delegate::delegate_account(
        delegation_accounts, pda_seeds, bump, delegate_config,
    ).map_err(|_| ProgramError::Custom(0))?;
    */

    // 0xAbim: Placeholder - Log delegation intent
    pinocchio::msg!("Delegation requested for PDA at commit_frequency: 30000ms");

    // 0xAbim: REMOVED close_pda_acc - account must stay open during delegation

    Ok(())
}

// 0xAbim: Faced some design decisions here.
//
//
