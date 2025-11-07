// Integration tests for Ephemeral Rollups with local MagicBlock validator
#[cfg(test)]
pub mod tests_for_er;

#[cfg(test)]
mod tests {

    use bytemuck::{Pod, Zeroable};
    use ephemeral_rollups_pinocchio::{
        consts::{BUFFER, DELEGATION_PROGRAM_ID, DELEGATION_RECORD, MAGIC_CONTEXT_ID},
        pda::{
            delegation_metadata_pda_from_delegated_account,
            delegation_record_pda_from_delegated_account,
        },
    };
    use litesvm::LiteSVM;
    use std::io::Error;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    use pinocchio::{
        msg,
        pubkey::find_program_address,
        sysvars::rent::{Rent, RENT_ID},
    };
    use pinocchio_log::log;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    use crate::{instructions::delegate_account, state::GenIxHandler};

    // use crate::instructions::MojoInstructions::CreateAccount;

    const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);
    const LOCAL_ER: &str = "mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev";
    // const magic_context: [u8; 32] = MAGIC_CONTEXT_ID;
    // Pubkey;

    // Global counter for tracking total compute units across all tests
    static TOTAL_CUS: AtomicU64 = AtomicU64::new(0);

    fn program_id() -> Pubkey {
        PROGRAM_ID
    }

    #[repr(C)]
    #[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
    pub struct MyPosition {
        x: u64,
        y: u64,
    }

    impl MyPosition {
        pub const LEN: usize = core::mem::size_of::<MyPosition>();

        pub fn length(&self) -> usize {
            core::mem::size_of::<MyPosition>()
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            bytemuck::bytes_of(self).to_vec()
        }
    }

    fn setup() -> (LiteSVM, ReusableState) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let bytes = include_bytes!("../../target/deploy/mojo_program.so");
        svm.add_program(program_id(), bytes);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        // NOTE: Using empty first seed, "fundrais" (8 bytes) as second, and pubkey as third
        // to match the GenIxHandler seed layout
        let account_to_create = Pubkey::find_program_address(
            &[
                &[0u8; 8],
                b"fundrais",
                payer.pubkey().as_ref(),
                &[0u8; 32],
                &[0u8; 32],
            ],
            &PROGRAM_ID,
        );

        // Generate a new keypair for buffer account
        let buffer_keypair = Keypair::new();

        let pda = String::from(account_to_create.0.to_string());
        log!("{}", &*pda);

        let system_program = solana_sdk_ids::system_program::ID;
        let creator_account = payer.pubkey();

        // Derive delegation PDAs from the account we'll delegate
        // NOTE: We can't use ephemeral_rollups_pinocchio PDA functions in tests because
        // pinocchio's find_program_address only works on-chain. We need to derive manually.
        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        svm.add_program_from_file(delegation_program_id, "delegate.so")
            .unwrap();

        // Derive delegation_record PDA: ["delegation", account_pubkey]
        let delegation_record = Pubkey::find_program_address(
            &[b"delegation", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

        // Derive delegation_metadata PDA: ["delegation-metadata", account_pubkey]
        let delegation_metadata = Pubkey::find_program_address(
            &[b"delegation-metadata", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

        let reusable_state = ReusableState {
            system_program,
            account_to_create,
            creator: payer,
            account_to_create2: None,
            creator_2: None,
            creator_account,
            owner_program: PROGRAM_ID,
            buffer_account: buffer_keypair.pubkey(),
            delegation_metadata,
            delegation_record,
        };
        (svm, reusable_state)
    }

    pub struct ReusableState {
        pub system_program: Pubkey,
        pub account_to_create: (Pubkey, u8),
        pub creator: Keypair,
        pub creator_2: Option<Keypair>,
        pub account_to_create2: Option<(Pubkey, u8)>,
        pub creator_account: Pubkey,
        pub owner_program: Pubkey,
        pub buffer_account: Pubkey,
        pub delegation_record: Pubkey,
        pub delegation_metadata: Pubkey,
    }

    #[test]
    pub fn create_account() -> Result<(), Error> {
        let test_start = Instant::now();
        let (mut svm, mut state) = setup();

        let creator = state.creator;
        let account_to_create = state.account_to_create;
        let system_program = state.system_program;

        let my_state_data = MyPosition { x: 24, y: 12 };

        // all of these would be handled on the sdk
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        // Seeds start as all zeros, just fill what you need
        let fundraiser_slice = b"fundrais"; // 8 bytes exactly
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        // const MAX_LEN: usize = 128;

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let create_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(account_to_create.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
            ],
            data: create_ix_data,
        };

        let message = Message::new(&[create_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&creator], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx_start = Instant::now();
        let tx = svm.send_transaction(transaction).unwrap();
        let tx_duration = tx_start.elapsed();
        let cus = tx.compute_units_consumed;

        // Track CUs
        TOTAL_CUS.fetch_add(cus, Ordering::SeqCst);

        let test_duration = test_start.elapsed();

        log!("\n=== create_account Test ===");
        log!("Transaction successful");
        log!("TX Time: {}ms", tx_duration.as_millis());
        log!("CUs Consumed: {}", cus);
        log!("Test Total Time: {}ms", test_duration.as_millis());
        log!("Total CUs (all tests): {}", TOTAL_CUS.load(Ordering::SeqCst));
        Ok(())
    }

    #[test]
    pub fn delegate_account() -> Result<(), Error> {
        let test_start = Instant::now();
        let (mut svm, state) = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        // First create the account with proper structure before delegating it
        let my_state_data = MyPosition { x: 24, y: 12 };

        // Note: GenIxHandler.size should be the account data size (MyPosition), not total instruction size
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        let fundraiser_slice = b"fundrais";
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let create_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
            ],
            data: create_ix_data,
        };

        let message = Message::new(&[create_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);
        let tx_create_start = Instant::now();
        let tx_create = svm.send_transaction(transaction).unwrap();
        let tx_create_duration = tx_create_start.elapsed();
        let cus_create = tx_create.compute_units_consumed;
        TOTAL_CUS.fetch_add(cus_create, Ordering::SeqCst);

        // Now delegate the account
        // Need to pass GenIxHandler in instruction data for seed derivation
        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        let delegate_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),   // 0. creator/payer
                AccountMeta::new(system_program, false),     // 1. system program (must be native system program id)
                AccountMeta::new(creator_account.0, false), // 2. creator account PDA (delegated account)
                AccountMeta::new(owner_program, false),     // 3. owner program
                AccountMeta::new(buffer_account, false),    // 4. buffer PDA (created via CPI)
                AccountMeta::new(delegation_record, false), // 5. delegation record PDA
                AccountMeta::new(delegation_metadata, false), // 6. delegation metadata PDA
                AccountMeta::new(delegation_program_id, false), // 7. (optional) remaining accounts
            ],
            data: delegate_ix_data,
        };

        let message = Message::new(&[delegate_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        // Only creator needs to sign
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx_delegate_start = Instant::now();
        let tx_delegate = svm.send_transaction(transaction).unwrap();
        let tx_delegate_duration = tx_delegate_start.elapsed();
        let cus_delegate = tx_delegate.compute_units_consumed;
        TOTAL_CUS.fetch_add(cus_delegate, Ordering::SeqCst);

        let test_duration = test_start.elapsed();
        let test_total_cus = cus_create + cus_delegate;

        log!("\n=== delegate_account Test ===");
        log!("Create TX Time: {}ms, CUs: {}", tx_create_duration.as_millis(), cus_create);
        log!("Delegate TX Time: {}ms, CUs: {}", tx_delegate_duration.as_millis(), cus_delegate);
        log!("Test Total Time: {}ms", test_duration.as_millis());
        log!("Test Total CUs: {}", test_total_cus);
        log!("Total CUs (all tests): {}", TOTAL_CUS.load(Ordering::SeqCst));
        Ok(())
    }

    #[test]
    pub fn update_account() -> Result<(), Error> {
        let test_start = Instant::now();
        let (mut svm, state) = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        // First create the account with proper structure before delegating it
        let my_state_data = MyPosition { x: 24, y: 12 };

        // Note: GenIxHandler.size should be the account data size (MyPosition), not total instruction size
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        let fundraiser_slice = b"fundrais";
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let create_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
            ],
            data: create_ix_data,
        };

        log!("creator address {}", &creator.pubkey().to_bytes());

        let message = Message::new(&[create_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);
        let tx_create_start = Instant::now();
        let tx_create = svm.send_transaction(transaction).unwrap();
        let tx_create_duration = tx_create_start.elapsed();
        let cus_create = tx_create.compute_units_consumed;
        TOTAL_CUS.fetch_add(cus_create, Ordering::SeqCst);

        // Now delegate the account
        // Need to pass GenIxHandler in instruction data for seed derivation
        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        let delegate_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),   // creator/payer
                AccountMeta::new(system_program, false),     // system program (must be native system program id)
                AccountMeta::new(creator_account.0, false), // creator account PDA (delegated account)
                AccountMeta::new(owner_program, false),     // owner program
                AccountMeta::new(buffer_account, false),    // buffer PDA (created via CPI)
                AccountMeta::new(delegation_record, false), // delegation record PDA
                AccountMeta::new(delegation_metadata, false), // delegation metadata PDA
                AccountMeta::new(delegation_program_id, false), // (optional) remaining accounts
            ],
            data: delegate_ix_data,
        };

        let message = Message::new(&[delegate_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        // Only creator needs to sign
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx_delegate_start = Instant::now();
        let tx_delegate = svm.send_transaction(transaction).unwrap();
        let tx_delegate_duration = tx_delegate_start.elapsed();
        let cus_delegate = tx_delegate.compute_units_consumed;
        TOTAL_CUS.fetch_add(cus_delegate, Ordering::SeqCst);

        // Now update the account

        log!("delegate program {}", &delegation_program_id.to_bytes());

        let my_update_state_data = MyPosition { x: 26, y: 14 };

        // Need to pass GenIxHandler in instruction data for seed derivation
        let update_ix_data = [
            vec![crate::instructions::MojoInstructions::UpdateDelegatedAccount as u8],
            mojo_data.to_bytes(),
            my_update_state_data.to_bytes(),
        ]
        .concat();

        let update_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),
                    false,
                ),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID),
                    false,
                ),
            ],
            data: update_ix_data,
        };

        let message = Message::new(&[update_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        // Only creator needs to sign
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);

        // Send the transaction and capture the result
        let res_update = svm.send_transaction(transaction);

        // In unit tests, ER magic accounts/program are not available, so update is expected to fail
        // because the account is owned by the delegation program after delegation
        assert!(
            res_update.is_err(),
            "Update delegated account should fail in unit tests without ER context"
        );

        let test_duration = test_start.elapsed();
        let test_total_cus = cus_create + cus_delegate;

        log!("\n=== update_account Test ===");
        log!("Create TX Time: {}ms, CUs: {}", tx_create_duration.as_millis(), cus_create);
        log!("Delegate TX Time: {}ms, CUs: {}", tx_delegate_duration.as_millis(), cus_delegate);
        log!("Test Total Time: {}ms", test_duration.as_millis());
        log!("Test Total CUs: {}", test_total_cus);
        log!("Total CUs (all tests): {}", TOTAL_CUS.load(Ordering::SeqCst));
        Ok(())
    }

    #[test]
    pub fn commit_account() -> Result<(), Error> {
        let test_start = Instant::now();
        let (mut svm, state) = setup();

        let creator = state.creator;
        let account_to_create = state.account_to_create;
        let system_program = state.system_program;

        // Prepare user state and seeds
        let my_state_data = MyPosition { x: 42, y: 24 };

        let mut mojo_data = GenIxHandler::new(my_state_data.length().to_le_bytes());
        let fundraiser_slice = b"fundrais"; // 8 bytes exactly
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        // 1) Create the PDA account so it's non-empty
        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let create_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(account_to_create.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
            ],
            data: create_ix_data,
        };

        let message = Message::new(&[create_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx_create = Transaction::new(&[&creator], message, recent_blockhash);
        let tx_create_start = Instant::now();
        let tx_create = svm.send_transaction(tx_create).unwrap();
        let tx_create_duration = tx_create_start.elapsed();
        let cus_create = tx_create.compute_units_consumed;
        TOTAL_CUS.fetch_add(cus_create, Ordering::SeqCst);

        // 2) Commit the PDA account through ER
        let commit_ix_data = [
            vec![crate::instructions::MojoInstructions::Commit as u8],
            mojo_data.to_bytes(), // GenIxHandler only
        ]
        .concat();

        let commit_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(account_to_create.0, false),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),
                    false,
                ),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID),
                    false,
                ),
                AccountMeta::new_readonly(system_program, false),
            ],
            data: commit_ix_data,
        };

        let message = Message::new(&[commit_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx_commit = Transaction::new(&[&creator], message, recent_blockhash);
        let res_commit = svm.send_transaction(tx_commit);
        // In unit tests, ER magic accounts/program are not available, so commit is expected to fail.
        assert!(
            res_commit.is_err(),
            "Commit should fail in unit tests without ER context"
        );

        let test_duration = test_start.elapsed();
        let test_total_cus = cus_create;

        log!("\n=== commit_account Test ===");
        log!("Create TX Time: {}ms, CUs: {}", tx_create_duration.as_millis(), cus_create);
        log!("Test Total Time: {}ms", test_duration.as_millis());
        log!("Test Total CUs: {}", test_total_cus);
        log!("Total CUs (all tests): {}", TOTAL_CUS.load(Ordering::SeqCst));
        Ok(())
    }

    #[test]
    pub fn undelegate_account() -> Result<(), Error> {
        let test_start = Instant::now();
        let (mut svm, state) = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        // First create the account with proper structure
        let my_state_data = MyPosition { x: 24, y: 12 };

        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        let fundraiser_slice = b"fundrais";
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let create_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
            ],
            data: create_ix_data,
        };

        let message = Message::new(&[create_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);
        let tx_create_start = Instant::now();
        let tx_create = svm.send_transaction(transaction).unwrap();
        let tx_create_duration = tx_create_start.elapsed();
        let cus_create = tx_create.compute_units_consumed;
        TOTAL_CUS.fetch_add(cus_create, Ordering::SeqCst);

        // Delegate the account
        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        let delegate_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(system_program, false),
                AccountMeta::new(creator_account.0, false),
                AccountMeta::new(owner_program, false),
                AccountMeta::new(buffer_account, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new_readonly(delegation_program_id, false),
            ],
            data: delegate_ix_data,
        };

        let message = Message::new(&[delegate_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);
        let tx_delegate_start = Instant::now();
        let tx_delegate = svm.send_transaction(transaction).unwrap();
        let tx_delegate_duration = tx_delegate_start.elapsed();
        let cus_delegate = tx_delegate.compute_units_consumed;
        TOTAL_CUS.fetch_add(cus_delegate, Ordering::SeqCst);

        // Now undelegate the account
        let undelegate_ix_data = [
            vec![crate::instructions::MojoInstructions::UndelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let undelegate_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),
                    false,
                ),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID),
                    false,
                ),
            ],
            data: undelegate_ix_data,
        };

        let message = Message::new(&[undelegate_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);
        let res_undelegate = svm.send_transaction(transaction);

        // In unit tests, ER magic accounts/program are not available, so undelegate is expected to fail.
        assert!(
            res_undelegate.is_err(),
            "Undelegate should fail in unit tests without ER context"
        );

        let test_duration = test_start.elapsed();
        let test_total_cus = cus_create + cus_delegate;

        log!("\n=== undelegate_account Test ===");
        log!("Create TX Time: {}ms, CUs: {}", tx_create_duration.as_millis(), cus_create);
        log!("Delegate TX Time: {}ms, CUs: {}", tx_delegate_duration.as_millis(), cus_delegate);
        log!("Test Total Time: {}ms", test_duration.as_millis());
        log!("Test Total CUs: {}", test_total_cus);
        log!("Total CUs (all tests): {}", TOTAL_CUS.load(Ordering::SeqCst));
        Ok(())
    }
}
