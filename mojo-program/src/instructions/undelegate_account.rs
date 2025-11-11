use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::find_program_address, ProgramResult,
};

use crate::state::transaction_handler::TransactionHandler;

/// 0xAbim: Expected accounts in the order below
/// 0. [signer] the payer
/// 1. [writable] Our creator account PDA.
/// 2. [writable] Magic context account (needed by Deleg)
/// 3. [] Magic Program (Delegation program ID)

pub fn process_undelegate_account(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [
        creator,
        creator_account,
        magic_context,
        magic_program, .. 
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

     if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // load the transaction handler generator here
    let mut data = creator_account.try_borrow_mut_data().expect("Invalidity");
    let mojo_data: &mut TransactionHandler = bytemuck::from_bytes_mut(&mut data[0..TransactionHandler::LEN]);

    let seed_slice = mojo_data.get_seed_slices();
    let seeds: &[&[u8]] = &seed_slice;
    // let seeds = &[seed_slice[0], seed_slice[1], seed_slice[2], seed_slice[3], seed_slice[4]];

    let (derived_pda, _) = find_program_address(
        seeds, 
        &crate::ID
    );

    if derived_pda != *creator_account.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    // Commit an Undelegate with ease
    ephemeral_rollups_pinocchio::instruction::commit_and_undelegate_accounts(
        creator, 
        &[creator_account.clone()], 
        magic_context, 
        magic_program
    )

}

   