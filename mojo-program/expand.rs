#![feature(prelude_import)]
#![allow(unexpected_cfgs)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};
use crate::instructions::MojoInstructions;
mod constants {
    ///The constant program ID.
    pub const ID: ::pinocchio_pubkey::reexport::Pubkey = ::pinocchio_pubkey::from_str(
        "mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev",
    );
    ///Returns `true` if given pubkey is the program ID.
    #[inline]
    pub fn check_id(id: &::pinocchio_pubkey::reexport::Pubkey) -> bool {
        ::pinocchio_pubkey::reexport::pubkey_eq(id, &ID)
    }
    ///Returns the program ID.
    #[inline]
    pub const fn id() -> ::pinocchio_pubkey::reexport::Pubkey {
        ID
    }
}
mod instructions {
    pub mod commit {
        use ephemeral_rollups_pinocchio::{
            consts::{MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
            instruction::commit_accounts,
        };
        use pinocchio::{
            account_info::AccountInfo, program_error::ProgramError, pubkey, ProgramResult,
        };
        use crate::state::GenIxHandler;
        pub fn process_commit_instruction(
            accounts: &[AccountInfo],
            instruction_data: &[u8],
        ) -> ProgramResult {
            let [creator, creator_account, magic_context, magic_program, _system_program,
            ] = accounts else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            if !creator.is_signer() {
                return Err(ProgramError::MissingRequiredSignature);
            }
            if magic_context.key() != &MAGIC_CONTEXT_ID {
                return Err(ProgramError::InvalidArgument);
            }
            if magic_program.key() != &MAGIC_PROGRAM_ID {
                return Err(ProgramError::InvalidArgument);
            }
            if creator_account.data_is_empty() {
                return Err(ProgramError::InvalidAccountData);
            }
            if instruction_data.len() < GenIxHandler::LEN {
                return Err(ProgramError::InvalidInstructionData);
            }
            let mojo_data = &instruction_data[0..GenIxHandler::LEN];
            let mojo_ser_data = bytemuck::try_pod_read_unaligned::<
                GenIxHandler,
            >(mojo_data)
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            let [seed1, seed2, seed3, seed4, seed5] = mojo_ser_data.get_seed_slices();
            let (derived_pda, _bump) = pubkey::find_program_address(
                &[seed1, seed2, seed3, seed4, seed5],
                &crate::ID,
            );
            if creator_account.key() != &derived_pda {
                return Err(ProgramError::InvalidSeeds);
            }
            commit_accounts(creator, &accounts[1..2], magic_context, magic_program)?;
            Ok(())
        }
    }
    pub mod create_account {
        use pinocchio::{
            account_info::AccountInfo, instruction::Signer, pubkey, seeds,
            sysvars::{rent::Rent, Sysvar},
            ProgramResult,
        };
        use pinocchio_system::instructions::CreateAccount;
        use crate::state::GenIxHandler;
        pub fn create_state_account(
            accounts: &[AccountInfo],
            data: &[u8],
        ) -> ProgramResult {
            let [creator, account_to_create, _system_program, _rent_sysvar @ ..] = accounts
            else {
                return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
            };
            let mojo_data = &data[0..GenIxHandler::LEN];
            let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);
            let [seed1, seed2, seed3, seed4, seed5] = mojo_ser_data.get_seed_slices();
            if !&creator.is_signer() {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Creator should be a signer"),
                    );
                }
            }
            if !&account_to_create.data_is_empty() {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Account should be empty"),
                    );
                }
            }
            let (account_pda, bump) = pubkey::find_program_address(
                &[seed1, seed2, seed3, seed4, seed5],
                &crate::ID,
            );
            let seed_bump = [bump];
            let seeds = [
                ::pinocchio::instruction::Seed::from(seed1),
                ::pinocchio::instruction::Seed::from(seed2),
                ::pinocchio::instruction::Seed::from(seed3),
                ::pinocchio::instruction::Seed::from(seed4),
                ::pinocchio::instruction::Seed::from(seed5),
                ::pinocchio::instruction::Seed::from(&seed_bump),
            ];
            let signer = Signer::from(&seeds);
            match (&&account_pda, &account_to_create.key()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("You provided the wrong user pda"),
                            ),
                        );
                    }
                }
            };
            CreateAccount {
                from: creator,
                lamports: Rent::get()?
                    .minimum_balance(usize::from_le_bytes(mojo_ser_data.size)),
                owner: &crate::ID,
                space: u64::from_le_bytes(mojo_ser_data.size),
                to: account_to_create,
            }
                .invoke_signed(&[signer])?;
            let mut some_fist_account = account_to_create.try_borrow_mut_data().unwrap();
            some_fist_account.copy_from_slice(&data[GenIxHandler::LEN..]);
            Ok(())
        }
    }
    pub use commit::*;
    pub use create_account::*;
    pub mod update_account {
        use pinocchio::{
            account_info::AccountInfo, instruction::Signer, pubkey::self, seeds,
            ProgramResult,
        };
        use pinocchio_log::log;
        use crate::state::GenIxHandler;
        pub fn update_delegated_account(
            accounts: &[AccountInfo],
            data: &[u8],
        ) -> ProgramResult {
            let [creator, account_to_update, _system_program, _rent_sysvar @ ..] = accounts
            else {
                return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
            };
            let mojo_data = &data[0..GenIxHandler::LEN];
            let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);
            let [seed1, seed2, seed3, seed4, seed5] = mojo_ser_data.get_seed_slices();
            if !&creator.is_signer() {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Creator should be a signer"),
                    );
                }
            }
            if !!(&account_to_update.data_is_empty()) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Account should be empty"),
                    );
                }
            }
            let (account_pda, bump) = pubkey::find_program_address(
                &[seed1, seed2, seed3, seed4, seed5],
                &crate::ID,
            );
            let seed_bump = [bump];
            let seeds = [
                ::pinocchio::instruction::Seed::from(seed1),
                ::pinocchio::instruction::Seed::from(seed2),
                ::pinocchio::instruction::Seed::from(seed3),
                ::pinocchio::instruction::Seed::from(seed4),
                ::pinocchio::instruction::Seed::from(seed5),
                ::pinocchio::instruction::Seed::from(&seed_bump),
            ];
            let signer = Signer::from(&seeds);
            match (&&account_pda, &account_to_update.key()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("You provided the wrong user pda"),
                            ),
                        );
                    }
                }
            };
            let current_owner: &[u8; 32] = unsafe { account_to_update.owner() };
            unsafe {
                {
                    let mut logger = pinocchio_log::logger::Logger::<200>::default();
                    logger.append("owner of the account is ");
                    logger.append(current_owner);
                    logger.log();
                };
            }
            let mut some_fist_account = account_to_update.try_borrow_mut_data().unwrap();
            some_fist_account.copy_from_slice(&data[GenIxHandler::LEN..]);
            Ok(())
        }
    }
    pub use update_account::*;
    pub mod delegate_account {
        use pinocchio::{
            ProgramResult, account_info::AccountInfo, instruction::{Seed, Signer},
            program_error::ProgramError, pubkey::find_program_address, seeds,
        };
        use ephemeral_rollups_pinocchio::{
            types::DelegateAccountArgs, utils::{close_pda_acc, cpi_delegate},
            consts::{DELEGATION_PROGRAM_ID, BUFFER},
        };
        use pinocchio_system::instructions::{CreateAccount, Assign};
        use crate::state::GenIxHandler;
        #[allow(clippy::cloned_ref_to_slice_refs)]
        pub fn process_delegate_account(
            accounts: &[AccountInfo],
            instruction_data: &[u8],
        ) -> ProgramResult {
            let [creator, creator_account, owner_program, buffer_account,
            delegation_record, delegation_metadata, system_program, _rest @ ..] = accounts
            else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            if instruction_data.len() < GenIxHandler::LEN {
                return Err(ProgramError::InvalidAccountData);
            }
            let mojo_data = &instruction_data[0..GenIxHandler::LEN];
            let mojo_ser_data = bytemuck::from_bytes::<GenIxHandler>(mojo_data);
            let _size = u64::from_le_bytes(mojo_ser_data.size) as usize;
            let seed_slice = mojo_ser_data.get_seed_slices();
            let (derived_pda, bump) = find_program_address(
                &seed_slice[0..5],
                &crate::ID,
            );
            if creator_account.key() != &derived_pda {
                return Err(ProgramError::InvalidSeeds);
            }
            let seed_bump = [bump];
            let seeds = [
                ::pinocchio::instruction::Seed::from(seed_slice[0]),
                ::pinocchio::instruction::Seed::from(seed_slice[1]),
                ::pinocchio::instruction::Seed::from(seed_slice[2]),
                ::pinocchio::instruction::Seed::from(seed_slice[3]),
                ::pinocchio::instruction::Seed::from(seed_slice[4]),
                ::pinocchio::instruction::Seed::from(&seed_bump),
            ];
            let signer_seeds = Signer::from(&seeds);
            let buffer_seeds: &[&[u8]] = &[BUFFER, creator_account.key().as_ref()];
            let (buffer_pda, buffer_bump) = find_program_address(
                buffer_seeds,
                &crate::ID,
            );
            let buffer_bump_slice = [buffer_bump];
            let buffer_seed_binding = [
                Seed::from(BUFFER),
                Seed::from(creator_account.key().as_ref()),
                Seed::from(&buffer_bump_slice),
            ];
            let buffer_signer_seeds = Signer::from(&buffer_seed_binding);
            let data_len = creator_account.data_len();
            CreateAccount {
                from: creator,
                to: buffer_account,
                lamports: 0,
                space: data_len as u64,
                owner: &crate::ID,
            }
                .invoke_signed(&[buffer_signer_seeds])?;
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
            let current_owner = unsafe { creator_account.owner() };
            if current_owner != &pinocchio_system::id() {
                unsafe { creator_account.assign(&pinocchio_system::id()) };
            }
            let current_owner = unsafe { creator_account.owner() };
            let delegation_program_pubkey = unsafe {
                &*(DELEGATION_PROGRAM_ID.as_ptr() as *const pinocchio::pubkey::Pubkey)
            };
            if current_owner != delegation_program_pubkey {
                Assign {
                    account: creator_account,
                    owner: delegation_program_pubkey,
                }
                    .invoke_signed(&[signer_seeds.clone()])?;
            }
            let delegate_config = DelegateAccountArgs {
                commit_frequency_ms: 30000,
                ..Default::default()
            };
            cpi_delegate(
                    creator,
                    creator_account,
                    owner_program,
                    buffer_account,
                    delegation_record,
                    delegation_metadata,
                    delegate_config,
                    signer_seeds,
                )
                .map_err(|_| ProgramError::InvalidAccountData)?;
            close_pda_acc(creator, buffer_account)?;
            Ok(())
        }
    }
    pub use delegate_account::*;
    pub mod undelegate_account {
        use pinocchio::{
            account_info::AccountInfo, program_error::ProgramError,
            pubkey::find_program_address, ProgramResult,
        };
        use crate::state::GenIxHandler;
        pub fn process_undelegate_account(
            accounts: &[AccountInfo],
            instruction_data: &[u8],
        ) -> ProgramResult {
            let [creator, mojo_account_pda, magic_context, magic_program, ..] = accounts
            else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            if !creator.is_signer() {
                return Err(ProgramError::MissingRequiredSignature);
            }
            let mojo_bytes = mojo_account_pda.try_borrow_data()?;
            if mojo_bytes.len() < GenIxHandler::LEN {
                return Err(ProgramError::InvalidAccountData);
            }
            let mojo_data: &GenIxHandler = bytemuck::try_from_bytes(
                    &mojo_bytes[..GenIxHandler::LEN],
                )
                .map_err(|_| ProgramError::InvalidAccountData)?;
            let size = u64::from_le_bytes(mojo_data.size) as usize;
            if size > 256 || size == 0 {
                return Err(ProgramError::InvalidArgument);
            }
            let seeds_slice = &mojo_data.seeds[..size];
            let (derived_pda, _) = find_program_address(&[seeds_slice], &crate::ID);
            if derived_pda != *mojo_account_pda.key() {
                return Err(ProgramError::InvalidSeeds);
            }
            let accounts_to_commit = [mojo_account_pda];
            ephemeral_rollups_pinocchio::instruction::commit_and_undelegate_accounts(
                    creator,
                    &accounts[1..2],
                    magic_context,
                    magic_program,
                )
                .map_err(|_| ProgramError::InvalidAccountData)?;
            Ok(())
        }
    }
    pub use undelegate_account::*;
    #[repr(u8)]
    pub enum MojoInstructions {
        Initialize,
        CreateAccount,
        DelegateAccount,
        Commit,
        UpdateDelegatedAccount,
        UndelegateAccount,
    }
    impl TryFrom<&u8> for MojoInstructions {
        type Error = pinocchio::program_error::ProgramError;
        fn try_from(value: &u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(MojoInstructions::Initialize),
                1 => Ok(MojoInstructions::CreateAccount),
                2 => Ok(MojoInstructions::DelegateAccount),
                3 => Ok(MojoInstructions::Commit),
                4 => Ok(MojoInstructions::UpdateDelegatedAccount),
                5 => Ok(MojoInstructions::UndelegateAccount),
                _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
            }
        }
    }
}
mod state {
    pub mod gen_ix_handler {
        use bytemuck::{Pod, Zeroable};
        #[repr(C)]
        pub struct GenIxHandler {
            pub seeds: [u8; 128],
            pub size: [u8; 8],
        }
        const _: () = {
            if !(::core::mem::size_of::<GenIxHandler>()
                == (::core::mem::size_of::<[u8; 128]>()
                    + ::core::mem::size_of::<[u8; 8]>()))
            {
                {
                    ::std::rt::begin_panic(
                        "derive(Pod) was applied to a type with padding",
                    );
                }
            }
        };
        const _: fn() = || {
            #[allow(clippy::missing_const_for_fn)]
            #[doc(hidden)]
            fn check() {
                fn assert_impl<T: ::bytemuck::Pod>() {}
                assert_impl::<[u8; 128]>();
            }
        };
        const _: fn() = || {
            #[allow(clippy::missing_const_for_fn)]
            #[doc(hidden)]
            fn check() {
                fn assert_impl<T: ::bytemuck::Pod>() {}
                assert_impl::<[u8; 8]>();
            }
        };
        unsafe impl ::bytemuck::Pod for GenIxHandler {}
        const _: fn() = || {
            #[allow(clippy::missing_const_for_fn)]
            #[doc(hidden)]
            fn check() {
                fn assert_impl<T: ::bytemuck::Zeroable>() {}
                assert_impl::<[u8; 128]>();
            }
        };
        const _: fn() = || {
            #[allow(clippy::missing_const_for_fn)]
            #[doc(hidden)]
            fn check() {
                fn assert_impl<T: ::bytemuck::Zeroable>() {}
                assert_impl::<[u8; 8]>();
            }
        };
        unsafe impl ::bytemuck::Zeroable for GenIxHandler {}
        #[automatically_derived]
        impl ::core::clone::Clone for GenIxHandler {
            #[inline]
            fn clone(&self) -> GenIxHandler {
                let _: ::core::clone::AssertParamIsClone<[u8; 128]>;
                let _: ::core::clone::AssertParamIsClone<[u8; 8]>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for GenIxHandler {}
        #[automatically_derived]
        impl ::core::fmt::Debug for GenIxHandler {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "GenIxHandler",
                    "seeds",
                    &self.seeds,
                    "size",
                    &&self.size,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for GenIxHandler {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for GenIxHandler {
            #[inline]
            fn eq(&self, other: &GenIxHandler) -> bool {
                self.seeds == other.seeds && self.size == other.size
            }
        }
        impl GenIxHandler {
            pub const LEN: usize = core::mem::size_of::<GenIxHandler>();
            pub fn to_bytes(&self) -> Vec<u8> {
                bytemuck::bytes_of(self).to_vec()
            }
            pub fn get_seed_slices(&self) -> [&[u8]; 5] {
                [
                    &self.seeds[0..8],
                    &self.seeds[8..16],
                    &self.seeds[16..48],
                    &self.seeds[48..80],
                    &self.seeds[80..112],
                ]
            }
            pub fn new(size: [u8; 8]) -> Self {
                Self { seeds: [0u8; 128], size }
            }
            pub fn fill_first(&mut self, first_bytes: &[u8; 8]) -> &mut Self {
                self.seeds[0..8].copy_from_slice(first_bytes);
                self
            }
            pub fn fill_second(&mut self, second_bytes: &[u8; 8]) -> &mut Self {
                self.seeds[8..16].copy_from_slice(second_bytes);
                self
            }
            pub fn fill_third(&mut self, third_bytes: &[u8; 32]) -> &mut Self {
                self.seeds[16..48].copy_from_slice(third_bytes);
                self
            }
            pub fn fill_fourth(&mut self, fourth_bytes: &[u8; 32]) -> &mut Self {
                self.seeds[48..80].copy_from_slice(fourth_bytes);
                self
            }
            pub fn fill_fifth(&mut self, fifth_bytes: &[u8; 32]) -> &mut Self {
                self.seeds[80..112].copy_from_slice(fifth_bytes);
                self
            }
        }
    }
    pub use gen_ix_handler::*;
}
mod tests {}
/// Program entrypoint.
#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    const UNINIT: core::mem::MaybeUninit<::pinocchio::account_info::AccountInfo> = core::mem::MaybeUninit::<
        ::pinocchio::account_info::AccountInfo,
    >::uninit();
    let mut accounts = [UNINIT; { ::pinocchio::MAX_TX_ACCOUNTS }];
    let (program_id, count, instruction_data) = ::pinocchio::entrypoint::deserialize::<
        { ::pinocchio::MAX_TX_ACCOUNTS },
    >(input, &mut accounts);
    match process_instruction(
        &program_id,
        core::slice::from_raw_parts(accounts.as_ptr() as _, count),
        &instruction_data,
    ) {
        Ok(()) => ::pinocchio::SUCCESS,
        Err(error) => error.into(),
    }
}
/// A default allocator for when the program is compiled on a target different than
/// `"solana"`.
///
/// This links the `std` library, which will set up a default global allocator.
mod __private_alloc {
    extern crate std as __std;
}
///The constant program ID.
pub const ID: ::pinocchio_pubkey::reexport::Pubkey = ::pinocchio_pubkey::from_str(
    "3jyHnrGq1z9YiGyx5QEUDR5hnZ7PYeYW5stFUq2skYZz",
);
///Returns `true` if given pubkey is the program ID.
#[inline]
pub fn check_id(id: &::pinocchio_pubkey::reexport::Pubkey) -> bool {
    ::pinocchio_pubkey::reexport::pubkey_eq(id, &ID)
}
///Returns the program ID.
#[inline]
pub const fn id() -> ::pinocchio_pubkey::reexport::Pubkey {
    ID
}
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match (&program_id, &&ID) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidInstructionData)?;
    match MojoInstructions::try_from(discriminator)? {
        MojoInstructions::CreateAccount => {
            instructions::create_state_account(accounts, data)?;
        }
        MojoInstructions::DelegateAccount => {
            instructions::process_delegate_account(accounts, data)?;
        }
        MojoInstructions::UndelegateAccount => {
            instructions::process_undelegate_account(accounts, data)?;
        }
        MojoInstructions::UpdateDelegatedAccount => {
            instructions::update_delegated_account(accounts, data)?;
        }
        MojoInstructions::Commit => {
            instructions::process_commit_instruction(accounts, data)?;
        }
        _ => return Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
