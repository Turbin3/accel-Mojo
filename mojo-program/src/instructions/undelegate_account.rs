use pinocchio::{
    account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey,
    pubkey::find_program_address, seeds, ProgramResult,
};
use pinocchio_log::log;

use crate::state::GenIxHandler;

pub fn process_undelegate_account(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    log!("i was here");
    let [creator, mojo_account_pda, magic_context, magic_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // 0xAbim: Validate creator is a signer
    if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // check that account_to_create is empty
    assert!(
        !&mojo_account_pda.data_is_empty(),
        "Account should be empty"
    );

    let mojo_data = &data[0..GenIxHandler::LEN];
    let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);

    let seeds_data = &mojo_ser_data.seeds;
    // let seed_bump = [bump];
    let seeds = &[seeds_data, creator.key().as_ref()];

    let (derived_pda, bump) = pubkey::find_program_address(seeds, &crate::id());

    assert_eq!(
        &derived_pda,
        mojo_account_pda.key(),
        "You provided the wrong user pda"
    );

    ephemeral_rollups_pinocchio::instruction::commit_and_undelegate_accounts(
        creator,
        &accounts[1..2], // Some pretty issues here.
        magic_context,
        magic_program,
    )
    .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
