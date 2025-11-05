pub mod create_account;
pub use create_account::*;

pub mod helpers;
pub use helpers::*;

#[repr(u8)]
pub enum MojoInstructions {
    Initialize,
    CreateAccount,
    DelegagteAccount,
    Commit,
    UpdateDelegatedAccount,
    UnDelegateAccount,
}

impl TryFrom<&u8> for MojoInstructions {
    type Error = pinocchio::program_error::ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MojoInstructions::Initialize),
            1 => Ok(MojoInstructions::CreateAccount),
            2 => Ok(MojoInstructions::DelegagteAccount),
            3 => Ok(MojoInstructions::Commit),
            4 => Ok(MojoInstructions::UpdateDelegatedAccount),
            5 => Ok(MojoInstructions::UnDelegateAccount),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}
