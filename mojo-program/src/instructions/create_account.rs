use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    seeds,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::state::GenIxHandler;

pub fn create_state_account(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // get all accounts
    let [creator, account_to_create, _system_program, _rent_sysvar @ ..] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let mojo_data = &data[0..GenIxHandler::LEN];
    let mojo_ser_data = bytemuck::try_pod_read_unaligned::<GenIxHandler>(mojo_data).unwrap();

    // checks
    // check that maker is a signer ✅
    assert!(&creator.is_signer(), "Creator should be a signer");
    // check that account_to_create is empty
    assert!(
        &account_to_create.data_is_empty(),
        "Account should be empty"
    );
    // check that owner of account_to_create is this program
    assert_eq!(
        account_to_create.owner(),
        &crate::ID,
        "Illegal Account Owner"
    );

    // create_account
    let seeds = seeds!(&mojo_ser_data.seeds);
    let signer = Signer::from(&seeds);

    CreateAccount {
        from: creator,
        lamports: Rent::get()?.minimum_balance(usize::from_le_bytes(mojo_ser_data.size)),
        owner: &crate::ID,
        space: u64::from_le_bytes(mojo_ser_data.size),
        to: &*account_to_create,
    }
    .invoke_signed(&[signer])?;

    let mut some_fist_account = account_to_create.try_borrow_mut_data().unwrap();

    // this will modify the account state
    some_fist_account.copy_from_slice(&data[GenIxHandler::LEN..]);
    Ok(())
}
