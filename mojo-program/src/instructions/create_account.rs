use pinocchio::{
    ProgramResult, account_info::AccountInfo, instruction::{Seed, Signer}, pubkey, seeds, sysvars::{Sysvar, rent::Rent}
};

use pinocchio_system::instructions::CreateAccount;

use crate::state::transaction_handler::TransactionHandler;

pub fn create_state_account(accounts: &[AccountInfo], data_xy: &[u8]) -> ProgramResult {
    // get all accounts
    let [creator, creator_account, _system_program, _rent_sysvar @ ..] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let data = &data_xy[0..]; // for the discriminator

    let mojo_data = &data[0..TransactionHandler::LEN];
    let mojo_ser_data = bytemuck::from_bytes::<TransactionHandler>(mojo_data);

    let slices = mojo_ser_data.get_seed_slices();
    // checks
    // check that maker is a signer ✅
    // assert!(&creator.is_signer(), "Creator should be a signer");
    if !creator.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }
    // check that creator_account is empty
    // assert!(        &creator_account.data_is_empty(), "Account should be empty");
    if !creator_account.data_is_empty() {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }
    // check that owner of creator_account is this program

    // NOTE Always use all 5 seeds
    let (account_pda, bump) =
        pubkey::find_program_address(&slices, &crate::ID);

    let seed_bump = [bump];
    let seeds = seeds!(
        Seed::from(slices[0]), 
        Seed::from(slices[1]), 
        Seed::from(slices[2]), 
        Seed::from(slices[3]), 
        Seed::from(slices[4]), 
        Seed::from(&seed_bump));
    let signer = Signer::from(&seeds);

    if account_pda != *creator_account.key() {
        return Err(pinocchio::program_error::ProgramError::InvalidSeeds);
    }


    let size = u64::from_le_bytes(mojo_ser_data.size) as usize; 
    CreateAccount {
        from: creator,
        lamports: Rent::get()?.minimum_balance(size),
        owner: &crate::ID,
        space: u64::from_le_bytes(mojo_ser_data.size),
        to: creator_account,
    }
    .invoke_signed(&[signer])?;


    // let mut some_fist_account = creator_account.try_borrow_mut_data().unwrap();
    let mut some_fist_account = creator_account
    .try_borrow_mut_data()
    .map_err(|_| pinocchio::program_error::ProgramError::AccountBorrowFailed)?;


    // this will modify the account state
    some_fist_account.copy_from_slice(&data[TransactionHandler::LEN..]);
    Ok(())
}
