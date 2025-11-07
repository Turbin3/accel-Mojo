use ephemeral_rollups_pinocchio::{
    consts::{MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    instruction::commit_accounts,
};

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey, ProgramResult};
use pinocchio_log::log;

use crate::state::GenIxHandler;

pub fn process_commit_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // need to discuss , how to handle magic context and magic program
    let [creator, creator_account, magic_context, magic_program] = accounts else {
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

    // parse GenIxHandler from instruction data
    if instruction_data.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mojo_data = &instruction_data[0..GenIxHandler::LEN];
    let mojo_ser_data = bytemuck::try_pod_read_unaligned::<GenIxHandler>(mojo_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let [seed1, seed2, seed3, seed4, seed5] = mojo_ser_data.get_seed_slices();

    // Derive PDA using all five seeds and verify it matches creator_account
    let (derived_pda, _bump) =
        pubkey::find_program_address(&[seed1, seed2, seed3, seed4, seed5], &crate::ID);

    if creator_account.key() != &derived_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // log!("{}", &accounts[1..2]);

    {
        let mut some_fist_account = creator_account.try_borrow_mut_data().unwrap();

        log!("current data is {}", some_fist_account.as_ref());
    }

    // comitting the updates
    commit_accounts(
        creator,
        &accounts[1..2], // expects creator_account as &[AccountInfo]
        magic_context,
        magic_program,
    )?;

    Ok(())
}
