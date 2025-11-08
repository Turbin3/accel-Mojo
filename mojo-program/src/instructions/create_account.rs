use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    pubkey, seeds,
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
    let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);

    // checks
    // check that maker is a signer ✅
    assert!(&creator.is_signer(), "Creator should be a signer");
    // check that account_to_create is empty
    assert!(
        &account_to_create.data_is_empty(),
        "Account should be empty"
    );

    let seeds_data = &mojo_ser_data.seeds;
    // let seed_bump = [bump];
    let seeds = &[seeds_data, creator.key().as_ref()];

    let (derived_pda, bump) = pubkey::find_program_address(seeds, &crate::id());
    let bump_binding = [bump];
    // let seeds = seeds!(seed1, seed2, seed3, seed4, seed5, &seed_bump);
    let signer_seeds = seeds!(seeds_data, creator.key().as_ref(), &bump_binding);
    let signers: [Signer<'_, '_>; 1] = [Signer::from(&signer_seeds[..])];

    assert_eq!(
        &derived_pda,
        account_to_create.key(),
        "You provided the wrong user pda"
    );

    CreateAccount {
        from: creator,
        lamports: Rent::get()?.minimum_balance(usize::from_le_bytes(mojo_ser_data.size)),
        owner: &crate::ID,
        space: u64::from_le_bytes(mojo_ser_data.size),
        to: account_to_create,
    }
    .invoke_signed(&signers)?;

    let mut some_fist_account = account_to_create.try_borrow_mut_data().unwrap();

    // this will modify the account state
    some_fist_account.copy_from_slice(&data[GenIxHandler::LEN..]);
    Ok(())
}
