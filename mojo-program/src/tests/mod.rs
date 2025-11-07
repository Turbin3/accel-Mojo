mod tests_for_er;

// #[cfg(test)]
// mod tests {

//     use bytemuck::{Pod, Zeroable};
//     use ephemeral_rollups_pinocchio::{
//         consts::{BUFFER, DELEGATION_PROGRAM_ID, DELEGATION_RECORD, MAGIC_CONTEXT_ID},
//         pda::{
//             delegation_metadata_pda_from_delegated_account,
//             delegation_record_pda_from_delegated_account,
//         },
//     };
//     use litesvm::LiteSVM;
//     use std::{io::Error, string};

//     use pinocchio::{
//         msg,
//         pubkey::find_program_address,
//         sysvars::rent::{Rent, RENT_ID},
//     };
//     use pinocchio_log::log;
//     use solana_instruction::{AccountMeta, Instruction};
//     use solana_keypair::Keypair;
//     use solana_message::Message;
//     use solana_native_token::LAMPORTS_PER_SOL;
//     use solana_pubkey::Pubkey;
//     use solana_signer::Signer;
//     use solana_transaction::Transaction;

//     use crate::{instructions::delegate_account, state::GenIxHandler};

//     // use crate::instructions::MojoInstructions::CreateAccount;

//     const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);
//     const LOCAL_ER: &str = "mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev";
//     // const magic_context: [u8; 32] = MAGIC_CONTEXT_ID;
//     // Pubkey;

//     fn program_id() -> Pubkey {
//         PROGRAM_ID
//     }

//     #[repr(C)]
//     #[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
//     pub struct MyPosition {
//         x: u64,
//         y: u64,
//     }

//     impl MyPosition {
//         pub const LEN: usize = core::mem::size_of::<MyPosition>();

//         pub fn length(&self) -> usize {
//             core::mem::size_of::<MyPosition>()
//         }

//         pub fn to_bytes(&self) -> Vec<u8> {
//             bytemuck::bytes_of(self).to_vec()
//         }
//     }

//     fn setup() -> (LiteSVM, ReusableState) {
//         let mut svm = LiteSVM::new();
//         let payer = Keypair::new();

//         svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
//             .expect("Airdrop failed");

//         let bytes = include_bytes!("../../target/deploy/mojo_program.so");
//         svm.add_program(program_id(), bytes);

//         // Derive the PDA for the escrow account using the maker's public key and a seed value
//         // NOTE: Using empty first seed, "fundrais" (8 bytes) as second, and pubkey as third
//         // to match the GenIxHandler seed layout
//         let account_to_create = Pubkey::find_program_address(
//             &[
//                 &[0u8; 8],
//                 b"fundrais",
//                 payer.pubkey().as_ref(),
//                 &[0u8; 32],
//                 &[0u8; 32],
//             ],
//             &PROGRAM_ID,
//         );

//         // Generate a new keypair for buffer account
//         let buffer_keypair = Keypair::new();

//         let pda = String::from(account_to_create.0.to_string());
//         log!("{}", &*pda);

//         let system_program = solana_sdk_ids::system_program::ID;
//         let creator_account = payer.pubkey();

//         // Derive delegation PDAs from the account we'll delegate
//         // NOTE: We can't use ephemeral_rollups_pinocchio PDA functions in tests because
//         // pinocchio's find_program_address only works on-chain. We need to derive manually.
//         let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

//         svm.add_program_from_file(delegation_program_id, "delegate.so")
//             .unwrap();

//         // Derive delegation_record PDA: ["delegation", account_pubkey]
//         let delegation_record = Pubkey::find_program_address(
//             &[b"delegation", account_to_create.0.as_ref()],
//             &delegation_program_id,
//         )
//         .0;

//         // Derive delegation_metadata PDA: ["delegation-metadata", account_pubkey]
//         let delegation_metadata = Pubkey::find_program_address(
//             &[b"delegation-metadata", account_to_create.0.as_ref()],
//             &delegation_program_id,
//         )
//         .0;

//         let reusable_state = ReusableState {
//             system_program,
//             account_to_create,
//             creator: payer,
//             account_to_create2: None,
//             creator_2: None,
//             creator_account,
//             owner_program: PROGRAM_ID,
//             buffer_account: buffer_keypair.pubkey(),
//             delegation_metadata,
//             delegation_record,
//         };
//         (svm, reusable_state)
//     }

//     pub struct ReusableState {
//         pub system_program: Pubkey,
//         pub account_to_create: (Pubkey, u8),
//         pub creator: Keypair,
//         pub creator_2: Option<Keypair>,
//         pub account_to_create2: Option<(Pubkey, u8)>,
//         pub creator_account: Pubkey,
//         pub owner_program: Pubkey,
//         pub buffer_account: Pubkey,
//         pub delegation_record: Pubkey,
//         pub delegation_metadata: Pubkey,
//     }

//     #[test]
//     pub fn create_account() -> Result<(), Error> {
//         let (mut svm, mut state) = setup();

//         let creator = state.creator;
//         let account_to_create = state.account_to_create;
//         let system_program = state.system_program;

//         let my_state_data = MyPosition { x: 24, y: 12 };

//         // all of these would be handled on the sdk
//         let account_size = my_state_data.length() as u64;
//         let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
//         // Seeds start as all zeros, just fill what you need
//         let fundraiser_slice = b"fundrais"; // 8 bytes exactly
//         mojo_data
//             .fill_second(fundraiser_slice.try_into().unwrap())
//             .fill_third(creator.pubkey().as_ref().try_into().unwrap());

//         // const MAX_LEN: usize = 128;

//         let create_ix_data = [
//             vec![crate::instructions::MojoInstructions::CreateAccount as u8],
//             mojo_data.to_bytes(),
//             my_state_data.to_bytes(),
//         ]
//         .concat();

//         let create_ix = Instruction {
//             program_id: program_id(),
//             accounts: vec![
//                 AccountMeta::new(creator.pubkey(), true),
//                 AccountMeta::new(account_to_create.0, false),
//                 AccountMeta::new(system_program, false),
//                 AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
//             ],
//             data: create_ix_data,
//         };

//         let message = Message::new(&[create_ix], Some(&creator.pubkey()));
//         let recent_blockhash = svm.latest_blockhash();

//         let transaction = Transaction::new(&[&creator], message, recent_blockhash);

//         // Send the transaction and capture the result
//         let tx = svm.send_transaction(transaction).unwrap();
//         // msg!("tx logs: {:#?}", tx.logs);
//         log!("\nAdmin Claim transaction sucessful");
//         log!("CUs Consumed: {}", tx.compute_units_consumed);
//         Ok(())
//     }

//     #[test]
//     pub fn delegate_account() -> Result<(), Error> {
//         let (mut svm, state) = setup();

//         let creator = state.creator;
//         let creator_account = state.account_to_create;
//         let owner_program = state.owner_program;
//         let delegation_record = state.delegation_record;
//         let delegation_metadata = state.delegation_metadata;
//         let system_program = state.system_program;

//         // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
//         let buffer_account =
//             Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

//         // First create the account with proper structure before delegating it
//         let my_state_data = MyPosition { x: 24, y: 12 };

//         // Note: GenIxHandler.size should be the account data size (MyPosition), not total instruction size
//         let account_size = my_state_data.length() as u64;
//         let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
//         let fundraiser_slice = b"fundrais";
//         mojo_data
//             .fill_second(fundraiser_slice.try_into().unwrap())
//             .fill_third(creator.pubkey().as_ref().try_into().unwrap());

//         let create_ix_data = [
//             vec![crate::instructions::MojoInstructions::CreateAccount as u8],
//             mojo_data.to_bytes(),
//             my_state_data.to_bytes(),
//         ]
//         .concat();

//         let create_ix = Instruction {
//             program_id: program_id(),
//             accounts: vec![
//                 AccountMeta::new(creator.pubkey(), true),
//                 AccountMeta::new(creator_account.0, false),
//                 AccountMeta::new(system_program, false),
//                 AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
//             ],
//             data: create_ix_data,
//         };

//         let message = Message::new(&[create_ix], Some(&creator.pubkey()));
//         let recent_blockhash = svm.latest_blockhash();
//         let transaction = Transaction::new(&[&creator], message, recent_blockhash);
//         svm.send_transaction(transaction).unwrap();

//         // Now delegate the account
//         // Need to pass GenIxHandler in instruction data for seed derivation
//         let delegate_ix_data = [
//             vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
//             mojo_data.to_bytes(),
//         ]
//         .concat();

//         let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

//         let delegate_ix = Instruction {
//             program_id: program_id(),
//             accounts: vec![
//                 AccountMeta::new(creator.pubkey(), true),   // creator/payer
//                 AccountMeta::new(creator_account.0, false), // account to delegate
//                 AccountMeta::new(owner_program, false),     // owner program
//                 AccountMeta::new(buffer_account, false), // buffer PDA (created via invoke_signed)
//                 AccountMeta::new(delegation_record, false), // delegation record
//                 AccountMeta::new(delegation_metadata, false), // delegation metadata
//                 AccountMeta::new(system_program, false), // system program
//                 AccountMeta::new(delegation_program_id, false), // system program
//             ],
//             data: delegate_ix_data,
//         };

//         let message = Message::new(&[delegate_ix], Some(&creator.pubkey()));
//         let recent_blockhash = svm.latest_blockhash();

//         // Only creator needs to sign
//         let transaction = Transaction::new(&[&creator], message, recent_blockhash);

//         // Send the transaction and capture the result
//         let tx = svm.send_transaction(transaction).unwrap();
//         log!("\nDelegate Account transaction successful");
//         log!("CUs Consumed: {}", tx.compute_units_consumed);
//         Ok(())
//     }

//     #[test]
//     pub fn update_account() -> Result<(), Error> {
//         let (mut svm, state) = setup();

//         let creator = state.creator;
//         let creator_account = state.account_to_create;
//         let owner_program = state.owner_program;
//         let delegation_record = state.delegation_record;
//         let delegation_metadata = state.delegation_metadata;
//         let system_program = state.system_program;

//         // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
//         let buffer_account =
//             Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

//         // First create the account with proper structure before delegating it
//         let my_state_data = MyPosition { x: 24, y: 12 };

//         // Note: GenIxHandler.size should be the account data size (MyPosition), not total instruction size
//         let account_size = my_state_data.length() as u64;
//         let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
//         let fundraiser_slice = b"fundrais";
//         mojo_data
//             .fill_second(fundraiser_slice.try_into().unwrap())
//             .fill_third(creator.pubkey().as_ref().try_into().unwrap());

//         let create_ix_data = [
//             vec![crate::instructions::MojoInstructions::CreateAccount as u8],
//             mojo_data.to_bytes(),
//             my_state_data.to_bytes(),
//         ]
//         .concat();

//         let create_ix = Instruction {
//             program_id: program_id(),
//             accounts: vec![
//                 AccountMeta::new(creator.pubkey(), true),
//                 AccountMeta::new(creator_account.0, false),
//                 AccountMeta::new(system_program, false),
//                 AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
//             ],
//             data: create_ix_data,
//         };

//         log!("creator address {}", &creator.pubkey().to_bytes());

//         let message = Message::new(&[create_ix], Some(&creator.pubkey()));
//         let recent_blockhash = svm.latest_blockhash();
//         let transaction = Transaction::new(&[&creator], message, recent_blockhash);
//         svm.send_transaction(transaction).unwrap();

//         // Now delegate the account
//         // Need to pass GenIxHandler in instruction data for seed derivation
//         let delegate_ix_data = [
//             vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
//             mojo_data.to_bytes(),
//         ]
//         .concat();

//         let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

//         let delegate_ix = Instruction {
//             program_id: program_id(),
//             accounts: vec![
//                 AccountMeta::new(creator.pubkey(), true),   // creator/payer
//                 AccountMeta::new(creator_account.0, false), // account to delegate
//                 AccountMeta::new(owner_program, false),     // owner program
//                 AccountMeta::new(buffer_account, false), // buffer PDA (created via invoke_signed)
//                 AccountMeta::new(delegation_record, false), // delegation record
//                 AccountMeta::new(delegation_metadata, false), // delegation metadata
//                 AccountMeta::new(system_program, false), // system program
//                 AccountMeta::new(delegation_program_id, false), // system program
//             ],
//             data: delegate_ix_data,
//         };

//         let message = Message::new(&[delegate_ix], Some(&creator.pubkey()));
//         let recent_blockhash = svm.latest_blockhash();

//         // Only creator needs to sign
//         let transaction = Transaction::new(&[&creator], message, recent_blockhash);

//         // Send the transaction and capture the result
//         let tx2 = svm.send_transaction(transaction).unwrap();
//         log!("\nDelegate Account transaction successful");
//         log!("CUs Consumed: {}", tx2.compute_units_consumed);

//         // Now update the account

//         log!("delegate program {}", &delegation_program_id.to_bytes());

//         let my_update_state_data = MyPosition { x: 26, y: 14 };

//         // Need to pass GenIxHandler in instruction data for seed derivation
//         let update_ix_data = [
//             vec![crate::instructions::MojoInstructions::UpdateDelegatedAccount as u8],
//             mojo_data.to_bytes(),
//             my_update_state_data.to_bytes(),
//         ]
//         .concat();

//         let update_ix = Instruction {
//             program_id: program_id(),
//             accounts: vec![
//                 AccountMeta::new(creator.pubkey(), true),
//                 AccountMeta::new(creator_account.0, false),
//                 AccountMeta::new(system_program, false),
//                 AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
//                 AccountMeta::new_readonly(
//                     Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),
//                     false,
//                 ),
//                 AccountMeta::new_readonly(
//                     Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID),
//                     false,
//                 ),
//             ],
//             data: update_ix_data,
//         };

//         let message = Message::new(&[update_ix], Some(&creator.pubkey()));
//         let recent_blockhash = svm.latest_blockhash();

//         // Only creator needs to sign
//         let transaction = Transaction::new(&[&creator], message, recent_blockhash);

//         // Send the transaction and capture the result
//         let tx3 = svm.send_transaction(transaction).unwrap();
//         log!("\nUpdate Account transaction successful");
//         log!("CUs Consumed: {}", tx3.compute_units_consumed);
//         Ok(())
//     }
// }
