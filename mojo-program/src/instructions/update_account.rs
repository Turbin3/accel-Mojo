use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    pubkey::{self},
    seeds,
    ProgramResult,
};

use pinocchio_log::log;

use crate::state::GenIxHandler;

pub fn update_delegated_account(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // get all accounts
    let [creator, account_to_update, rest @ ..] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let mojo_data = &data[0..GenIxHandler::LEN];
    let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);

    // checks
    // check that maker is a signer ✅
    assert!(&creator.is_signer(), "Creator should be a signer");
    // check that account_to_update is empty
    assert!(
        !(&account_to_update.data_is_empty()),
        "Account should be empty"
    );
    // check that owner of account_to_update is this program
    let seeds_data = &mojo_ser_data.seeds;
    // let seed_bump = [bump];
    let seeds = &[seeds_data, creator.key().as_ref()];

    let (derived_pda, bump) = pubkey::find_program_address(seeds, &crate::id());
    let bump_binding: [u8; 1] = [bump];
    assert_eq!(
        &derived_pda,
        account_to_update.key(),
        "You provided the wrong user pda"
    );

    let mut some_fist_account = account_to_update.try_borrow_mut_data().unwrap();

    log!("current data is {}", some_fist_account.as_ref());
    // this will modify the account state
    some_fist_account.copy_from_slice(&data[GenIxHandler::LEN..]);
    Ok(())
}
