use ephemeral_rollups_pinocchio::{
    instruction::commit_and_undelegate_accounts, utils::create_schedule_commit_ix,
};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn process_undelegate_account(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    /// Undelegation callback invoked by the delegation program.
    /// Expected accounts (in order used below):
    /// 0. []         Payer (original authority for the delegated PDA)
    /// 1. [writable] Delegated PDA account to be restored (Creator Account PDA)
    /// 2. []         Owner program (this program ID)
    /// 3. [signer]   Undelegate buffer PDA (holds the snapshot of the delegated account)
    /// 4. []         System program
    // need to discuss , how to handle magic context and magic program
    let [creator, creator_account, magic_context, magic_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // 0xAbim: Used the commit scheduler UTIL to create the commit and undelegate context

    create_schedule_commit_ix(
        creator,
        &[creator_account.clone()],
        magic_context,
        magic_program,
        true,
    )?;
    Ok(())
}
