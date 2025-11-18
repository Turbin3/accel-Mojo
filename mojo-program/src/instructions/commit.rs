use ephemeral_rollups_pinocchio::utils::create_schedule_commit_ix;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

pub fn process_commit_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // need to discuss , how to handle magic context and magic program
    let [creator, creator_account, magic_context, magic_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // 0xAbim: Used the commit scheduler UTIL to create the commit context
    create_schedule_commit_ix(
        creator,
        &[creator_account.clone()],
        magic_context,
        magic_program,
        false,
    )?;

    Ok(())
}
