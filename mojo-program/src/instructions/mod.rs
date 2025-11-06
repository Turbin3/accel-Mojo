pub mod create_account;
pub use create_account::*;

pub mod update_account;
pub use update_account::*;

pub mod delegate_account;
pub use delegate_account::*;

pub mod undelegate_account;
pub use undelegate_account::*;

#[repr(u8)]
pub enum MojoInstructions {
    Initialize,
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
            0 => Ok(MojoInstructions::Initialize),
            1 => Ok(MojoInstructions::CreateAccount),
            2 => Ok(MojoInstructions::DelegateAccount),
            3 => Ok(MojoInstructions::Commit),
            4 => Ok(MojoInstructions::UpdateDelegatedAccount),
            5 => Ok(MojoInstructions::UndelegateAccount),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}
