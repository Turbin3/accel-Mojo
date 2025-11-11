#![no_std]
#![allow(unexpected_cfgs)]
use pinocchio::{account_info::AccountInfo, 
    entrypoint, 
    pubkey::Pubkey, 
    no_allocator, nostd_panic_handler,
    ProgramResult};

use crate::instructions::MojoInstructions;

mod constants;
mod instructions;
mod state;
mod tests;

no_allocator!();
// Use the no_std panic handler.
nostd_panic_handler!();

// entrypoint!(process_instruction);
pinocchio_pubkey::declare_id!("HGqcFg8D1wSMDeaoeUTh1uetHVgwr1Q5VHMyXrgyD3vL");

pub mod program {
    pinocchio_pubkey::declare_id!("HGqcFg8D1wSMDeaoeUTh1uetHVgwr1Q5VHMyXrgyD3vL");
    pub use ephemeral_rollups_pinocchio::consts::DELEGATION_PROGRAM_ID;
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);
    // let [discriminator, instruction_data @ ..] = instruction_data else {
    //     return Err(pinocchio::program_error::ProgramError::InvalidArgument);
    // };

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
