use pinocchio::{
    ProgramResult, account_info::AccountInfo, program_error::ProgramError
};
use ephemeral_rollups_pinocchio::types::DelegateConfig;
use crate::state::transaction_handler::TransactionHandler;


#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn process_delegate_account (
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 0xAbim: Here goes the accounts to be delegated. inthis order
    // 0. [] The creator acts as the payer || The seeds used to derive PDA
    // 1. [] the account pda (is_writable)
    // 2. [] the owner' program owning the delgated PDA
    // 3. [] the buffer account (used by the Deleg Program)
    // 4. [] the delegation record account
    // 5. [] the delegation metadata account
    // 6. [] System Program ... Stops Here!
    
    let args = DelegateArgs::try_from_bytes(instruction_data)?;
    let [
        creator,
        creator_account,
        owner_program,
        buffer_account,
        delegation_record,
        delegation_metadata,
        system_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // load the transaction handler generator here
    let mut data = creator_account.try_borrow_mut_data().expect("Invalidity");
    let mojo_data: &mut TransactionHandler = bytemuck::from_bytes_mut(&mut data[0..TransactionHandler::LEN]);


    let config = DelegateConfig {
        validator: args.validator(),
        ..DelegateConfig::default()
    };

    let seed_slice = mojo_data.get_seed_slices();
    let seeds = &[seed_slice[0], seed_slice[1], seed_slice[2], seed_slice[3], seed_slice[4]];

    // 0xAbim: re-implementation of delegation
    ephemeral_rollups_pinocchio::instruction::delegate_account(
        &[
            creator,
            creator_account,
            owner_program,
            buffer_account,
            delegation_record,
            delegation_metadata,
        ],
        seeds,
        args.bump(),
        config,
    )

}

pub struct DelegateArgs {
    bump: u8,
    validator: Option<[u8; 32]>
}

impl DelegateArgs {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<DelegateArgs, ProgramError> {
        if bytes.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let bump = bytes[0];
        let rest = &bytes[1..];
        let validator = if rest.is_empty() {
            None
        } else if rest.len() >= 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&rest[..32]);
            Some(arr)
        } else {
            return Err(ProgramError::InvalidInstructionData);
        };
        Ok(DelegateArgs { bump, validator})
    }

    #[inline]
    pub fn validator(&self) -> Option<[u8; 32]> {
        self.validator
    }

    #[inline]
    pub fn bump(&self) -> u8 {
        self.bump
    }
}