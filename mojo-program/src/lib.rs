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
        // 0xAbim: Initialize and CreateAccount both use the same handler
        MojoInstructions::Initialize | MojoInstructions::CreateAccount => {
            instructions::create_state_account(accounts, data)?;
        }
        MojoInstructions::DelegateAccount => {
            instructions::process_delegate_account(accounts, instruction_data)?;
        }
        MojoInstructions::UndelegateAccount => {
            instructions::process_undelegate_account(accounts, instruction_data)?;
        }
        // 0xAbim: TODO - Implement these instructions
        MojoInstructions::Commit | MojoInstructions::UpdateDelegatedAccount => {
            return Err(pinocchio::program_error::ProgramError::InvalidInstructionData);
        }
    }
    Ok(())
}
