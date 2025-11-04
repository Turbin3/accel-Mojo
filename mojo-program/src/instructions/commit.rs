use ephemeral_rollups_pinocchio::{
    consts::{MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    instruction::commit_accounts,
};

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey, ProgramResult};

use crate::state::GenIxHandler;

pub fn process_commit_instruction(
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // need to discuss , how to handle magic context and magic program
    let [creator, creator_account, magic_context, magic_program, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // checking if creator is the signer
    if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // validating magic_context has correct key
    if magic_context.key() != &MAGIC_CONTEXT_ID {
        return Err(ProgramError::InvalidArgument);
    }

    // validating magic_program has correct key
    if magic_program.key() != &MAGIC_PROGRAM_ID {
        return Err(ProgramError::InvalidArgument);
    }

    // checking that creator_account pda should not be empty
    if creator_account.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    // reading and validating GenIxHandler from creator_account
    let mojo_bytes = creator_account.try_borrow_data()?;
    if mojo_bytes.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    let mojo_ser_data: &GenIxHandler = bytemuck::try_from_bytes(&mojo_bytes[..GenIxHandler::LEN])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Extract seeds_size properly (u64 from little-endian bytes)
    let seeds_size = u64::from_le_bytes(mojo_ser_data.seeds_size) as usize;
    if seeds_size > 96 || seeds_size == 0 {
        return Err(ProgramError::InvalidArgument);
    }

    // Split seeds into two parts
    // First 10 bytes is the first seed term, then 10..seeds_size is the second seed term
    if seeds_size < 10 {
        return Err(ProgramError::InvalidArgument);
    }

    let seeds_first_slice = &mojo_ser_data.seeds[0..10];
    let seeds_second_slice = &mojo_ser_data.seeds[10..seeds_size];

    // Derive PDA to verify it matches creator_account
    let seeds = &[seeds_first_slice, seeds_second_slice];
    let (derived_pda, _bump) = pubkey::find_program_address(seeds, &crate::ID);

    if creator_account.key() != &derived_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // comitting the updates
    commit_accounts(
        creator,
        // [creator_account],
        &accounts[1..2], // expects creator_account as &[AccountInfo]
        magic_context,
        magic_program,
    )?;

    Ok(())
}
