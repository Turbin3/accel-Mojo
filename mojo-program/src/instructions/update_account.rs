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
    let [creator, account_to_update, _system_program, _rent_sysvar @ ..] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let mojo_data = &data[0..GenIxHandler::LEN];
    let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);

    let [seed1, seed2, seed3, seed4, seed5] = mojo_ser_data.get_seed_slices();

    // checks
    // check that maker is a signer ✅
    assert!(&creator.is_signer(), "Creator should be a signer");
    // check that account_to_update is empty
    assert!(
        !(&account_to_update.data_is_empty()),
        "Account should be empty"
    );
    // check that owner of account_to_update is this program

    // NOTE Always use all 5 seeds
    let (account_pda, bump) =
        pubkey::find_program_address(&[seed1, seed2, seed3, seed4, seed5], &crate::ID);

    let seed_bump = [bump];
    let seeds = seeds!(seed1, seed2, seed3, seed4, seed5, &seed_bump);
    let signer = Signer::from(&seeds);

    assert_eq!(
        &account_pda,
        account_to_update.key(),
        "You provided the wrong user pda"
    );

    let current_owner: &[u8; 32] = unsafe { account_to_update.owner() };

    unsafe {
        log!("owner of the account is {}", current_owner);
    }

    // // CreateAccount {
    // //     from: creator,
    // //     lamports: Rent::get()?.minimum_balance(usize::from_le_bytes(mojo_ser_data.size)),
    // //     owner: &crate::ID,
    // //     space: u64::from_le_bytes(mojo_ser_data.size),
    // //     to: &*account_to_update,
    // // }
    // // .invoke_signed(&[signer])?;

    let mut some_fist_account = account_to_update.try_borrow_mut_data().unwrap();

    // // this will modify the account state
    some_fist_account.copy_from_slice(&data[GenIxHandler::LEN..]);
    Ok(())
}
