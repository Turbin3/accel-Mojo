use pinocchio::{
    account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey,
    pubkey::find_program_address, seeds, ProgramResult,
};
use pinocchio_log::log;

use crate::state::GenIxHandler;

pub fn process_undelegate_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    /// Undelegation callback invoked by the delegation program.
    ///
    /// Expected accounts (in order used below):
    /// 0. []         Payer (original authority for the delegated PDA)
    /// 1. [writable] Delegated PDA account to be restored (Creator Account PDA)
    /// 2. []         Owner program (this program ID)
    /// 3. [signer]   Undelegate buffer PDA (holds the snapshot of the delegated account)
    /// 4. []         System program
    let [creator_account,buffer_account, creator, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // 0xAbim: Validate creator is a signer
    if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Read GenIxHandler from instruction data
    if instruction_data.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    let mojo_data = &instruction_data[0..GenIxHandler::LEN];
    let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);

    let seeds_data = &mojo_ser_data.seeds;
    // let seed_bump = [bump];
    let seeds = &[seeds_data, creator.key().as_ref()];

    let (derived_pda, bump) = pubkey::find_program_address(seeds, &crate::id());

    if creator_account.key() != &derived_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // assert_eq!(
    //     &derived_pda,
    //     creator_account.key(),
    //     "You provided the wrong user pda"
    // );

    ephemeral_rollups_pinocchio::instruction::undelegate(
        creator_account,
        &crate::ID, // Some pretty issues here.
        buffer_account,
        creator,
        &instruction_data[1..]
    )
    .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}


// use pinocchio::{
//     account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey,
//     pubkey::find_program_address, seeds, ProgramResult,
// };
// use pinocchio_log::log;

// use crate::state::GenIxHandler;

// pub fn process_undelegate_account(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
//     log!("i was here");
//     let [creator, mojo_account_pda, magic_context, magic_program] = accounts else {
//         return Err(ProgramError::NotEnoughAccountKeys);
//     };

//     // 0xAbim: Validate creator is a signer
//     if !creator.is_signer() {
//         return Err(ProgramError::MissingRequiredSignature);
//     }
//     // check that account_to_create is empty
//     assert!(
//         !&mojo_account_pda.data_is_empty(),
//         "Account should be empty"
//     );

//     let mojo_data = &data[0..GenIxHandler::LEN];
//     let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);

//     let seeds_data = &mojo_ser_data.seeds;
//     // let seed_bump = [bump];
//     let seeds = &[seeds_data, creator.key().as_ref()];

//     let (derived_pda, bump) = pubkey::find_program_address(seeds, &crate::id());

//     assert_eq!(
//         &derived_pda,
//         mojo_account_pda.key(),
//         "You provided the wrong user pda"
//     );

//     ephemeral_rollups_pinocchio::instruction::commit_and_undelegate_accounts(
//         creator,
//         &accounts[1..2], // Some pretty issues here.
//         magic_context,
//         magic_program,
//     )
//     .map_err(|_| ProgramError::InvalidAccountData)?;

//     Ok(())
// }
