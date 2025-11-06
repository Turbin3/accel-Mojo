use ephemeral_rollups_pinocchio::{
    consts::{BUFFER, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    instruction::{commit_and_undelegate, undelegate},
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    seeds, ProgramResult,
};

use crate::state::GenIxHandler;

pub fn process_undelegate_account(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [creator, mojo_account_pda, magic_context, magic_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // 0xAbim: Validate creator is a signer
    if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 0xAbim: Load account data using bytemuck
    let mojo_bytes = mojo_account_pda.try_borrow_data()?;
    if mojo_bytes.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    // 0xAbim: Validate the data exists
    let mojo_data: &GenIxHandler = bytemuck::try_from_bytes(&mojo_bytes[..GenIxHandler::LEN])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // 0xAbim: Extract seeds from account data
    let size = u64::from_le_bytes(mojo_data.size) as usize;
    if size > 256 || size == 0 {
        return Err(ProgramError::InvalidArgument);
    }
    let seeds_slice = &mojo_data.seeds[..size];

    // 0xAbim: Verify PDA derivation with extracted seeds
    let (derived_pda, _) = find_program_address(&[seeds_slice], &crate::ID);

    if derived_pda != *mojo_account_pda.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    // 0xAbim: Pass actual account references in correct format
    // Function expects: creator, accounts: &[AccountInfo], magic_context, magic_program
    // Still has issues with the acounts passed in to the function
    let accounts_to_commit = [mojo_account_pda];

    ephemeral_rollups_pinocchio::instruction::commit_and_undelegate_accounts(
        creator,
        &accounts[1..2], // Some pretty issues here.
        magic_context,
        magic_program,
    )
    .map_err(|_| ProgramError::InvalidAccountData)?;

    // undelegate()
    Ok(())
}
