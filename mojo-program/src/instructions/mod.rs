pub mod commit;
pub mod create_account;
pub use commit::*;
pub use create_account::*;

pub mod update_account;
pub use update_account::*;

pub mod delegate_account;
pub use delegate_account::*;

pub mod commit;
pub use commit::*;

pub mod undelegate_account;
pub use undelegate_account::*;

#[repr(u8)]
pub enum MojoInstructions {
    // Initialize,
    CreateAccount,
    DelegateAccount,
    Commit,
    UpdateDelegatedAccount,
    UndelegateAccount,
}

impl TryFrom<&u8> for MojoInstructions {
    type Error = pinocchio::program_error::ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            // 0 => Ok(MojoInstructions::Initialize),
            0 => Ok(MojoInstructions::CreateAccount),
            1 => Ok(MojoInstructions::DelegateAccount),
            2 => Ok(MojoInstructions::Commit),
            3 => Ok(MojoInstructions::UpdateDelegatedAccount),
            4 => Ok(MojoInstructions::UndelegateAccount),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}
