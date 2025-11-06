use pinocchio::{
    ProgramResult, account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::find_program_address, seeds
};
use ephemeral_rollups_pinocchio::{
    types::DelegateAccountArgs, utils::{close_pda_acc, cpi_delegate}, consts::{DELEGATION_PROGRAM_ID, BUFFER}
};
use pinocchio_system::instructions::{CreateAccount, Assign};
use crate::state::GenIxHandler;

#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn process_delegate_account (
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
    
    let [
        creator,
        creator_account,
        owner_program,
        buffer_account,
        delegation_record,
        delegation_metadata,
        system_program,
        _rest @..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };


    // Read GenIxHandler from instruction data
    if instruction_data.len() < GenIxHandler::LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    let mojo_data = &instruction_data[0..GenIxHandler::LEN];
    let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);

    // 0xAbim: Security check to ensure size is consistent (removed strict check for testing)
    let _size = u64::from_le_bytes(mojo_ser_data.size) as usize;

    // 0xAbim: Extract the seeds from instruction data
    let seed_slice = mojo_ser_data.get_seed_slices();

    // 0xAbim: Verify PDA derivation after seeds are extracted
    let (derived_pda, bump) = find_program_address(&seed_slice[0..5], &crate::ID);
    if creator_account.key() != &derived_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // 0xAbim: Create signer seeds with bump - required for CPI signing
    let seed_bump = [bump];
    let seeds = seeds!(seed_slice[0], seed_slice[1], seed_slice[2], seed_slice[3], seed_slice[4], &seed_bump);
    let signer_seeds = Signer::from(&seeds);

    // 0xAbim: Derive buffer PDA from [BUFFER, creator_account.key()] with OUR program ID
    let buffer_seeds: &[&[u8]] = &[BUFFER, creator_account.key().as_ref()];
    let (buffer_pda, buffer_bump) = find_program_address(buffer_seeds, &crate::ID);

    // 0xAbim: Creating the buffer PDA for delegation
    let buffer_bump_slice = [buffer_bump];
    let buffer_seed_binding = [
        Seed::from(BUFFER),
        Seed::from(creator_account.key().as_ref()),
        Seed::from(&buffer_bump_slice),
    ];
    let buffer_signer_seeds = Signer::from(&buffer_seed_binding);

    let data_len = creator_account.data_len();
    CreateAccount{
        from: creator,
        to: buffer_account,
        lamports: 0,  // Set to 0 as in reference implementation
        space: data_len as u64,
        owner: &crate::ID,  // Buffer owned by our program
    }.invoke_signed(&[buffer_signer_seeds])?;

    // 0xAbim: Copy PDA data to buffer, then zero out PDA data
    {
        let pda_data = creator_account.try_borrow_data()?;
        let mut buffer_data = buffer_account.try_borrow_mut_data()?;
        buffer_data.copy_from_slice(&pda_data);
    }
    {
        let mut pda_mut_data = creator_account.try_borrow_mut_data()?;
        for byte in pda_mut_data.iter_mut().take(data_len) {
            *byte = 0;
        }
    }

    // 0xAbim: Assign PDA to delegation program before delegating
    let current_owner = unsafe { creator_account.owner() };
    if current_owner != &pinocchio_system::id() {
        unsafe { creator_account.assign(&pinocchio_system::id()) };
    }

    let current_owner = unsafe { creator_account.owner() };
    let delegation_program_pubkey = unsafe { &*(DELEGATION_PROGRAM_ID.as_ptr() as *const pinocchio::pubkey::Pubkey) };
    if current_owner != delegation_program_pubkey {
        Assign {
            account: creator_account,
            owner: delegation_program_pubkey,
        }
        .invoke_signed(&[signer_seeds.clone()])?;
    }

    // 0xAbim: Prepare delegation config for MagicBlock
    let delegate_config = DelegateAccountArgs {
        commit_frequency_ms: 30000,  // 30 seconds
        ..Default::default()
    };

    // 0xAbim: delegate account ix using CPI (Cross Program Invocation)
    cpi_delegate(
        creator,
        creator_account,
        owner_program,
        buffer_account,
        delegation_record,
        delegation_metadata,
        delegate_config,
        signer_seeds
    ).map_err(|_| ProgramError::InvalidAccountData)?;

    close_pda_acc(creator, buffer_account)?;
    Ok(())
}

// 0xAbim: N.B:.
// The persistent errors (might be from setting up the localMagicblock instnace. I am not so sure of that yet.)
// Tried my possible best to avoid verbose code. this was my philosophy from the get go
// Will tweak the creator_account name to suit the other folders subsequently.