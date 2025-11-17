use ephemeral_rollups_pinocchio::{
    consts::{DELEGATION_METADATA, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    instruction::commit_accounts, pda::{commit_record_pda_from_delegated_account, commit_state_pda_from_delegated_account, delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account},
};

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey, ProgramResult};
use pinocchio_log::log;

use crate::state::GenIxHandler;

pub fn process_commit_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // need to discuss , how to handle magic context and magic program
    let [
        creator, creator_account, magic_context, magic_program
        ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };


    commit_accounts(
        creator,
        &[creator_account.clone()],
        magic_context,
        magic_program,
    )?;

    Ok(())
}
