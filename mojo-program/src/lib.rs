#![cfg_attr(not(test), no_std)]
use pinocchio::{
    account_info::AccountInfo, entrypoint, nostd_panic_handler, pubkey::Pubkey, ProgramResult,
};

use crate::instructions::MojoInstructions;

// For tests
#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;

// Use the no_std panic handler.
#[cfg(target_os = "solana")]
nostd_panic_handler!();

mod instructions;
mod state;

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
        // MojoInstructions::CreateAccount => {}
        // MojoInstructions::DelegagteAccount => {}
        // MojoInstructions::Commit => {}
        // MojoInstructions::UpdateDelegatedAccount => (),
        // MojoInstructions::UnDelegateAccount => (),
        _ => return Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
