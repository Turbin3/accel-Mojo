use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::find_program_address,
    ProgramResult,
};

use ephemeral_rollups_pinocchio::{
    instruction::delegate as er_delegate,
    types::DelegateConfig,
};

use pinocchio_system::ID as SYSTEM_PROGRAM_ID;

use crate::{constants, state::GenIxHandler};

#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn process_delegate_account(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Account order expected by the tests / ER program:
    // 0. [signer] creator / payer
    // 1. []       system program (must be the native system program id)
    // 2. [w]      creator account PDA (delegated account)
    // 3. [w]      owner program (usually our program id)
    // 4. [w]      buffer account PDA (created via CPI)
    // 5. [w]      delegation record PDA
    // 6. [w]      delegation metadata PDA
    // 7. []       (optional) remaining accounts
    let [
        creator,
        creator_account,
        owner_program,
        buffer_account,
        delegation_record,
        delegation_metadata,
        system_program,
        _rest @ ..
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if system_program.key() != &SYSTEM_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if instruction_data.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidInstructionData);
    }

    let handler_bytes = &instruction_data[..GenIxHandler::LEN];
    let handler =
        bytemuck::try_pod_read_unaligned::<GenIxHandler>(handler_bytes)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

    let seed_slices = handler.get_seed_slices();
    let (derived_pda, bump) =
        find_program_address(&seed_slices, &crate::ID);

    if creator_account.key() != &derived_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let delegate_accounts = [
        creator,
        creator_account,
        owner_program,
        buffer_account,
        delegation_record,
        delegation_metadata,
    ];

    let config = DelegateConfig {
        commit_frequency_ms: 30_000,
        validator: Some(constants::ID),
    };

    er_delegate::delegate_account(&delegate_accounts, &seed_slices, bump, config)
}
