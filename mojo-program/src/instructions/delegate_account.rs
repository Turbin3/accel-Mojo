use crate::state::GenIxHandler;
use ephemeral_rollups_pinocchio::pda::delegation_metadata_pda_from_delegated_account;
use ephemeral_rollups_pinocchio::{
    consts::{BUFFER, DELEGATION_PROGRAM_ID},
    instruction::delegate_account,
    types::{DelegateAccountArgs, DelegateConfig},
    utils::{close_pda_acc, cpi_delegate, make_seed_buf},
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey,
    pubkey::find_program_address,
    seeds,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use pinocchio_log::log;
use pinocchio_system::instructions::{Assign, CreateAccount};

#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn process_delegate_account(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 0xAbim: Here goes the accounts to be delegated.
    // 0. [] The creator acts as the payer
    // 1. [] the account pda (is_writable)
    // 2. [] the owner' program
    // 3. [] the buffer account
    // 4. [] the delegation record
    // 5. [] the delegation metadata
    // 6. [] System Program + ...Other essential accounts...

    let [creator, creator_account, owner_program, buffer_account, delegation_record, delegation_metadata, rest @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

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

    let config = DelegateConfig {
        commit_frequency_ms: 30000, // 30 seconds
        ..Default::default()
    };

    let delegate_accounts = [
        creator,
        creator_account,
        owner_program,
        buffer_account,
        delegation_record,
        delegation_metadata,
    ];

    delegate_account(&delegate_accounts, seeds, bump, config)?;

    Ok(())
}
