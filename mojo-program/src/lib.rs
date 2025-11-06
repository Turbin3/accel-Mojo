#![allow(unexpected_cfgs)]
use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

use crate::instructions::MojoInstructions;

mod constants;
mod instructions;
mod state;
mod tests;

entrypoint!(process_instruction);
pinocchio_pubkey::declare_id!("3jyHnrGq1z9YiGyx5QEUDR5hnZ7PYeYW5stFUq2skYZz");


pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    match MojoInstructions::try_from(discriminator)? {
        MojoInstructions::CreateAccount => {
            instructions::create_state_account(accounts, data)?;
        }
        MojoInstructions::DelegateAccount => {
            instructions::process_delegate_account(accounts, data)?;
        }
        MojoInstructions::UndelegateAccount => {
            instructions::process_undelegate_account(accounts, data)?;
        }
        MojoInstructions::UpdateDelegatedAccount => {
            instructions::update_delegated_account(accounts, data)?;
        }
        MojoInstructions::Commit => {
            instructions::process_commit_instruction(accounts, data)?;
        }
        _ => return Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
