use pinocchio::{
    ProgramResult, account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::{find_program_address}, seeds,
};
use ephemeral_rollups_pinocchio::{
    consts::{BUFFER, DELEGATION_METADATA, DELEGATION_PROGRAM_ID, DELEGATION_RECORD, MAGIC_PROGRAM_ID}, seeds::Seed, types::{DelegateAccountArgs, DelegateConfig}, utils::{close_pda_acc, cpi_delegate, make_seed_buf}
};

use crate::{state::GenIxHandler};

pub fn process_delegate_instruction (
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 0xAbim: Here goes the accounts to be delegated.
    // 0. [] The creator
    // 1. [] the account created (is_writable)
    // 2. [] the owner of the program 
    // 3. []
    // 4. []
    // 5. []
    // 6. []

    // let args = DelegateArgs::try_from_bytes(instruction_data)?;

    
    let [
        creator,
        mojo_account,
        owner,
        buffer_account,
        delegation_record,
        _delegation_program,
        _system_program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let delegate_args: DelegateAccountArgs = match DelegateAccountArgs::try_from(instruction_data) {
        Ok(args) => args,
        Err(_) => return Err(ProgramError::InvalidInstructionData)
    };

    let mojo_bytes = mojo_account.try_borrow_data()?;
    if mojo_bytes.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    let mojo_ser_data: &GenIxHandler = bytemuck::try_from_bytes(
        &mojo_bytes[..GenIxHandler::LEN]
    ).map_err(|_| ProgramError::InvalidAccountData)?;

    let size = u64::from_le_bytes(mojo_ser_data.size) as usize;
    if size > 96 || size == 0 {
        return Err(ProgramError::InvalidArgument);
    }
    let seeds_slice = &mojo_ser_data.seeds[..size];
    let seeds = seeds!(&mojo_ser_data.seeds[1..size]);
    // let seeds = slice::from_ref(&seeds_slice);

    let (derived_pda, bump) = find_program_address(&[seeds_slice], &crate::ID);
    if mojo_account.key() != &derived_pda {
        return Err(ProgramError::InvalidSeeds);
    }
    let config = DelegateConfig{
        validator: delegate_args.validator(),
        ..Default::default()
    };


    let delegate_signer_seeds = [
        Seed::from(seeds),
        Seed::from(&[bump])
    ];
    let signers = [Signer::from(&delegate_signer_seeds)];

    // 0xAbim: Delegate to the MB validator
    cpi_delegate(
        creator,
        mojo_account,
        MAGIC_PROGRAM_ID,
        BUFFER,
        DELEGATION_RECORD,
        DELEGATION_METADATA,
        delegate_args,
        &signers,
    )?;
    close_pda_acc(creator, mojo_account)?;

    Ok(()), ProgramError::AccountAlreadyInitialized;
}

