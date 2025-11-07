use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_system::instructions::CreateAccount;

/// Trait for validating account properties
pub trait AccountCheck {
    fn check(account: &AccountInfo) -> Result<(), ProgramError>;
}

/// Validates that an account is a signer
pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }
}

/// Validates that an account is owned by this program
pub struct ProgramAccount;

impl AccountCheck for ProgramAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }
}

/// Validates that an account is owned by the system program
pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.owner().ne(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }
}

/// Validates that an account is empty (has no data)
pub struct EmptyAccount;

impl AccountCheck for EmptyAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }
}

/// Validates that an account is not empty (has data)
pub struct NonEmptyAccount;

impl AccountCheck for NonEmptyAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.data_is_empty() {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

/// Validates that an account matches a specific pubkey
pub struct SpecificPubkeyAccount {
    pub expected_pubkey: Pubkey,
}

impl SpecificPubkeyAccount {
    pub fn check(account: &AccountInfo, expected_pubkey: &Pubkey) -> Result<(), ProgramError> {
        if account.key() != expected_pubkey {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(())
    }
}

pub struct PdaHelper;

impl PdaHelper {
    /// Verifies that a PDA matches the expected seeds (5 seeds pattern)
    pub fn verify_pda_5_seeds(
        account: &AccountInfo,
        seed1: &[u8],
        seed2: &[u8],
        seed3: &[u8],
        seed4: &[u8],
        seed5: &[u8],
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let seeds_array = [seed1, seed2, seed3, seed4, seed5];
        let (derived_pda, bump) = find_program_address(&seeds_array, program_id);

        if account.key() != &derived_pda {
            return Err(ProgramError::InvalidSeeds);
        }

        Ok(bump)
    }
}

/// Trait for initializing program-owned accounts
pub trait ProgramAccountInit {
    fn init<'a>(
        payer: &AccountInfo,
        account: &AccountInfo,
        seeds: &[Seed<'a>],
        space: usize,
    ) -> ProgramResult;
}

impl ProgramAccountInit for ProgramAccount {
    fn init<'a>(
        payer: &AccountInfo,
        account: &AccountInfo,
        seeds: &[Seed<'a>],
        space: usize,
    ) -> ProgramResult {
        let lamports = Rent::get()?.minimum_balance(space);

        let signer = [Signer::from(seeds)];

        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signer)?;

        Ok(())
    }
}

/// Trait for closing program-owned accounts
pub trait AccountClose {
    fn close(account: &AccountInfo, destination: &AccountInfo) -> ProgramResult;
}

impl AccountClose for ProgramAccount {
    fn close(account: &AccountInfo, destination: &AccountInfo) -> ProgramResult {
        {
            let mut data = account.try_borrow_mut_data()?;
            data[0] = 0xff;
        }

        *destination.try_borrow_mut_lamports()? += *account.try_borrow_lamports()?;
        account.resize(1)?;
        account.close()
    }
}