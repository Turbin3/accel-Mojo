use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

use crate::instructions::MojoInstructions;

mod constants;
mod instructions;
mod state;
mod tests;

entrypoint!(process_instruction);
// pinocchio_pubkey::declare_id!("7iMdvW8A4Tw3yxjbXjpx4b8LTW13EQLB4eTmPyqRvxzM");
pinocchio_pubkey::declare_id!("3zt2gQuNsVRG8PAbZdYS2mgyzhUqG8sNwcaGJ1DYvECo");

use pinocchio_log::log;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // assert_eq!(program_id, &ID);
    if program_id != &ID {
        return Err(pinocchio::program_error::ProgramError::IncorrectProgramId);
    }

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    log!("discri {}", *discriminator);
    match MojoInstructions::try_from(discriminator)? {
        MojoInstructions::CreateAccount => {
            log!("didn't fail here create account");

            instructions::create_state_account(accounts, data)?;
        }
        MojoInstructions::Commit => {
            log!("didn't fail here commit");

            instructions::process_commit_instruction(accounts, data)?;
        }
        MojoInstructions::DelegateAccount => {
            instructions::process_delegate_account(accounts, data)?;
        }
        MojoInstructions::UpdateDelegatedAccount => {
            log!("didn't fail update");

            instructions::update_delegated_account(accounts, data)?;
        }
        MojoInstructions::UndelegateAccount => {
            instructions::process_undelegate_account(accounts, data)?;
        }
        _ => return Err(pinocchio::program_error::ProgramError::IncorrectAuthority),
    }
    Ok(())
}
