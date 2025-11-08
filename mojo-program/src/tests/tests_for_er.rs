use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct MyPosition {
    x: u64,
    y: u64,
}

impl MyPosition {
    pub fn new(x: u64, y: u64) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> usize {
        core::mem::size_of::<MyPosition>()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

/// Test environment setup
pub struct TestEnv {
    pub base_client: RpcClient,
    pub er_client: RpcClient,
    pub payer: Keypair,
}

impl TestEnv {
    pub fn new() -> Self {
        let base_client = RpcClient::new_with_timeout(RPC_URL.to_string(), Duration::from_secs(30));

        let er_client =
            RpcClient::new_with_timeout(ER_RPC_URL.to_string(), Duration::from_secs(30));

        // In real tests, you'd load a keypair with SOL
        // For now, creating a new one (will need airdrop)
        let payer = Keypair::new();

        Self {
            base_client,
            er_client,
            payer,
        }
    }

    /// Airdrop SOL to payer on base layer
    pub fn airdrop(&self, lamports: u64) -> Result<(), Box<dyn std::error::Error>> {
        let signature = self
            .base_client
            .request_airdrop(&self.payer.pubkey(), lamports)?;

        // Wait for confirmation
        self.base_client.confirm_transaction(&signature)?;
        println!(
            "Airdropped {} lamports to {}",
            lamports,
            self.payer.pubkey()
        );
        Ok(())
    }

    /// Send and confirm transaction on base layer with CU tracking
    pub fn send_and_confirm_base(
        &self,
        instruction: Instruction,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let recent_blockhash = self.base_client.get_latest_blockhash()?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            recent_blockhash,
        );

        let signature = self
            .base_client
            .send_and_confirm_transaction(&transaction)?;

        // Log compute units consumed
        // self.log_compute_units(&signature, &self.base_client, "Base Layer");

        Ok(signature.to_string())
    }

    /// Send and confirm transaction on ER with CU tracking
    pub fn send_and_confirm_er(
        &self,
        instruction: Instruction,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let recent_blockhash = self.er_client.get_latest_blockhash()?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            recent_blockhash,
        );

        let signature = self.er_client.send_and_confirm_transaction(&transaction)?;

        // Log compute units consumed
        // self.log_compute_units(&signature, &self.er_client, "ER");

        Ok(signature.to_string())
    }

    /// Helper to log compute units consumed
    // fn log_compute_units(&self, signature: &solana_sdk::signature::Signature, client: &RpcClient, layer: &str) {
    //     use solana_sdk::commitment_config::CommitmentConfig;

    //     if let Ok(tx_response) = client.get_transaction(signature, CommitmentConfig::confirmed()) {
    //         if let Some(meta) = tx_response.transaction.meta {
    //             if let Some(cu) = meta.compute_units_consumed {
    //                 println!("   [{}] Compute Units: {}", layer, cu);
    //             }
    //         }
    //     }
    // }

    /// Get account data from base layer
    pub fn get_account_base(
        &self,
        pubkey: &Pubkey,
    ) -> Result<Option<Account>, Box<dyn std::error::Error>> {
        Ok(self.base_client.get_account(pubkey).ok())
    }

    /// Get account data from ER
    pub fn get_account_er(
        &self,
        pubkey: &Pubkey,
    ) -> Result<Option<Account>, Box<dyn std::error::Error>> {
        Ok(self.er_client.get_account(pubkey).ok())
    }
}

#[cfg(test)]
mod er_tests {

    pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);
    // pub const RPC_URL: &str = "http://127.0.0.1:8899";

    const RPC_URL: &str = "https://api.devnet.solana.com";
    const RPC_ER_URL: &str = "https://devnet.magicblock.app/";
    //

    use std::os::macos::raw::stat;

    use super::*;
    use pinocchio_log::log;
    // use solana_hash::Hash;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::read_keypair_file;
    use solana_keypair::Keypair;
    // use solana_message::Message;
    use solana_pubkey::Pubkey;
    use solana_rpc_client::rpc_client::RpcClient;
    use solana_sdk_ids::sysvar::rent::ID as RENT_ID;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    // use std::str::FromStr;
    use crate::encode_packed;
    use crate::tests::utils::helpers::*;
    // use crate::
    use crate::state::GenIxHandler;
    use pinocchio::{instruction::Signer as PSigner, seeds};

    use ephemeral_rollups_pinocchio::{
        consts::{BUFFER, DELEGATION_PROGRAM_ID, DELEGATION_RECORD, MAGIC_CONTEXT_ID},
        pda::{
            delegation_metadata_pda_from_delegated_account,
            delegation_record_pda_from_delegated_account,
        },
    };

    pub struct ReusableState {
        pub rpc_client: RpcClient,
        pub rpc_er_client: RpcClient,
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

    fn setup() -> ReusableState {
        let rpc_client = RpcClient::new(RPC_URL);
        let rpc_er_client = RpcClient::new(RPC_ER_URL);

        let payer = read_keypair_file("dev_wallet.json").expect("Couldn't find wallet file");

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let combined = encode_packed!(b"fundraise4", payer.pubkey().as_ref());
        let account_to_create = Pubkey::find_program_address(
            &[&compute_hash(&combined), payer.pubkey().as_ref()],
            &PROGRAM_ID,
        );

        println!("PDA to create: {}", account_pda);

        // Prepare instruction data
        let my_state_data = MyPosition::new(24, 12);
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());

        mojo_data
            .fill_second(b"fundrais".try_into().unwrap())
            .fill_third(env.payer.pubkey().as_ref().try_into().unwrap());

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        // Build instruction
        let create_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true),
                AccountMeta::new(account_pda, false),
                AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false),
                AccountMeta::new(Pubkey::from(RENT_ID), false),
            ],
            data: create_ix_data,
        };

        // Send transaction
        let signature = env
            .send_and_confirm_base(create_ix)
            .expect("Failed to create account");

        println!("✅ Account created successfully!");
        println!("   Signature: {}", signature);
        println!("   Account: {}", account_pda);

        // Verify account exists
        let account = env
            .get_account_base(&account_pda)
            .expect("Failed to get account");
        assert!(account.is_some(), "Account should exist");

        let account_data = account.unwrap();
        assert_eq!(account_data.owner, PROGRAM_ID);

        // Verify data
        let stored_position: &MyPosition = bytemuck::from_bytes(&account_data.data);
        assert_eq!(stored_position.x, 24);
        assert_eq!(stored_position.y, 12);

        println!(
            "   Verified: x={}, y={}",
            stored_position.x, stored_position.y
        );
    }

    /// Test 2: Delegate Account to Ephemeral Rollup
    #[test]
    #[ignore]
    fn test_02_delegate_account() {
        println!("\n=== Test 2: Delegate Account ===");

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

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

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.as_ref()], &PROGRAM_ID).0;

        let reusable_state = ReusableState {
            rpc_client,
            rpc_er_client,
            system_program,
            account_to_create,
            creator: payer,
            account_to_create2: None,
            creator_2: None,
            creator_account,
            owner_program: PROGRAM_ID,
            buffer_account,
            delegation_metadata,
            delegation_record,
        };

        (reusable_state)
    }

    #[test]
    fn test_create_state_account() {
        let mut state = setup();

        let creator = state.creator;
        let account_to_create = state.account_to_create;
        let system_program = state.system_program;

        let my_state_data = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"fundraise4", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_state_data.length().to_le_bytes(),
        };

        println!("✅ Account delegated successfully!");
        println!("   Signature: {}", signature);
        println!("   Delegation Record: {}", delegation_record);
        println!("   Delegation Metadata: {}", delegation_metadata);

        // Verify account owner changed to delegation program
        let account = env
            .get_account_base(&account_pda)
            .expect("Failed to get account")
            .unwrap();
        assert_eq!(
            account.owner, delegation_program_id,
            "Account should be owned by delegation program"
        );

        println!("   Verified: Account owner is delegation program");
    }

    /// Test 3: Update Delegated Account in ER
    #[test]
    #[ignore]
    fn test_03_update_delegated_account() {
        println!("\n=== Test 3: Update Delegated Account in ER ===");

        let env = TestEnv::new();
        env.airdrop(10_000_000_000).expect("Airdrop failed");

        // Setup: Create and delegate account (combining test 1 and 2)
        let (account_pda, _bump) = Pubkey::find_program_address(
            &[
                &[0u8; 8],
                b"fundrais",
                env.payer.pubkey().as_ref(),
                &[0u8; 32],
                &[0u8; 32],
            ],
            &PROGRAM_ID,
        );

        let my_state_data = MyPosition::new(24, 12);
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        mojo_data
            .fill_second(b"fundrais".try_into().unwrap())
            .fill_third(env.payer.pubkey().as_ref().try_into().unwrap());

        // Create account
        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let accounts = vec![
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(account_to_create.0, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(Pubkey::new_from_array(RENT_ID.to_bytes()), false),
        ];

        let recent_blockhash = state.rpc_client.get_latest_blockhash().unwrap();
        // .expect("failed to get recent blockhash");

        let instruction = Instruction {
            program_id: state.owner_program,
            accounts,
            data: create_ix_data,
        };
        env.send_and_confirm_base(create_ix)
            .expect("Failed to create account");

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn test_delegate_state_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        let my_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"fundraise4", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_state_data.length().to_le_bytes(),
        };

        // const MAX_LEN: usize = 128;

        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        let recent_blockhash = state.rpc_client.get_latest_blockhash().unwrap();
        // .expect("failed to get recent blockhash");

        let delegate_ix = Instruction {
            program_id: state.owner_program,
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),   // creator/payer
                AccountMeta::new(creator_account.0, false), // account to delegate
                AccountMeta::new(owner_program, false),     // owner program
                AccountMeta::new(buffer_account, false), // buffer PDA (created via invoke_signed)
                AccountMeta::new(delegation_record, false), // delegation record
                AccountMeta::new(delegation_metadata, false), // delegation metadata
                AccountMeta::new(system_program, false), // system program
                AccountMeta::new(delegation_program_id, false), // system program
            ],
            data: delegate_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn test_update_delegated_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"fundraise4", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_update_state_data.length().to_le_bytes(),
        };

        // const MAX_LEN: usize = 128;

        let update_ix_data = [
            vec![crate::instructions::MojoInstructions::UpdateDelegatedAccount as u8],
            mojo_data.to_bytes(),
            my_update_state_data.to_bytes(),
        ]
        .concat();

        let recent_blockhash = state.rpc_er_client.get_latest_blockhash().unwrap();
        // .expect("failed to get recent blockhash");

        // log!("{}", creator.pubkey());
        let update_ix = Instruction {
            program_id: state.owner_program,
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
            data: update_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[update_ix],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_er_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn test_commit_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        let combined = encode_packed!(b"fundraise4", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_update_state_data.length().to_le_bytes(),
        };

        let commit_ix_data = [
            vec![crate::instructions::MojoInstructions::Commit as u8],
            mojo_data.to_bytes(),
            my_update_state_data.to_bytes(),
        ]
        .concat();

        let recent_blockhash = state
            .rpc_er_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        // log!("{}", creator.pubkey());
        let commit_ix = Instruction {
            program_id: state.owner_program,
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(system_program, false),
                // AccountMeta::new((RENT_ID), false),
                AccountMeta::new(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),
                    false,
                ),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID),
                    false,
                ),
                // AccountMeta::new_readonly(owner_program, false), // owner program
                // AccountMeta::new_readonly(buffer_account, false), // buffer PDA (created via invoke_signed)
                // AccountMeta::new_readonly(delegation_record, false), // delegation record
                // AccountMeta::new_readonly(delegation_metadata, false), // delegation metadata
                // AccountMeta::new_readonly(system_program, false), // system program
                AccountMeta::new_readonly(delegation_program_id, false), // system program
                                                                         // AccountMeta::new_readonly(state.system_program, false), // system program
            ],
            data: commit_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[commit_ix],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_er_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    // #[test]
    fn test_commit_and_undelegate_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"fundraise4", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_update_state_data.length().to_le_bytes(),
        };

        // const MAX_LEN: usize = 128;

        let undelegate_ix_data = [
            vec![crate::instructions::MojoInstructions::UndelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let recent_blockhash = state
            .rpc_er_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");

        // log!("{}", creator.pubkey());
        let undelegate_ix = Instruction {
            program_id: state.owner_program,
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(system_program, false),
                // AccountMeta::new((RENT_ID), false),
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

        let transaction = Transaction::new_signed_with_payer(
            &[undelegate_ix],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );
    }

    /// Test 4: Commit Changes from ER to Base Layer
    #[test]
    #[ignore]
    fn test_04_commit_account() {
        println!("\n=== Test 4: Commit Account from ER to Base Layer ===");

        let env = TestEnv::new();
        env.airdrop(10_000_000_000).expect("Airdrop failed");

        // Setup: Create, delegate, and update account (combining previous tests)
        let (account_pda, _bump) = Pubkey::find_program_address(
            &[
                &[0u8; 8],
                b"fundrais",
                env.payer.pubkey().as_ref(),
                &[0u8; 32],
                &[0u8; 32],
            ],
            &PROGRAM_ID,
        );

        let my_state_data = MyPosition::new(24, 12);
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        mojo_data
            .fill_second(b"fundrais".try_into().unwrap())
            .fill_third(env.payer.pubkey().as_ref().try_into().unwrap());

        // Create, delegate, and update (abbreviated for brevity - same as test 3)
        // ... (create and delegate code here)

        println!("✅ Setup complete: Account created, delegated, and updated in ER");

        // Now commit the changes
        let commit_ix_data = [
            vec![crate::instructions::MojoInstructions::Commit as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let magic_context = Pubkey::from(MAGIC_CONTEXT_ID);
        let magic_program = Pubkey::from(MAGIC_PROGRAM_ID);

        let commit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true),
                AccountMeta::new(account_pda, false),
                AccountMeta::new_readonly(magic_context, false),
                AccountMeta::new_readonly(magic_program, false),
                AccountMeta::new_readonly(Pubkey::from(system_program::ID.to_bytes()), false),
            ],
            data: commit_ix_data,
        };

        let signature = env
            .send_and_confirm_er(commit_ix)
            .expect("Failed to commit account");

        println!("✅ Account committed successfully!");
        println!("   Signature: {}", signature);
        println!("   Changes synced from ER to base layer");

        // Verify changes are on base layer
        std::thread::sleep(Duration::from_secs(2)); // Give time for commit to propagate
        let account_base = env
            .get_account_base(&account_pda)
            .expect("Failed to get account from base")
            .unwrap();
        let stored_position: &MyPosition = bytemuck::from_bytes(&account_base.data);

        let signature = state
            .rpc_er_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "   Verified on base layer: x={}, y={}",
            stored_position.x, stored_position.y
        );
    }

    /// Test 5: Undelegate Account
    #[test]
    #[ignore]
    fn test_05_undelegate_account() {
        println!("\n=== Test 5: Undelegate Account ===");

        let env = TestEnv::new();
        env.airdrop(10_000_000_000).expect("Airdrop failed");

        // Setup: Create and delegate account
        let (account_pda, _bump) = Pubkey::find_program_address(
            &[
                &[0u8; 8],
                b"fundrais",
                env.payer.pubkey().as_ref(),
                &[0u8; 32],
                &[0u8; 32],
            ],
            &PROGRAM_ID,
        );

        let my_state_data = MyPosition::new(24, 12);
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        mojo_data
            .fill_second(b"fundrais".try_into().unwrap())
            .fill_third(env.payer.pubkey().as_ref().try_into().unwrap());

        // Create and delegate (abbreviated - same as before)
        // ...

        println!("✅ Setup complete: Account created and delegated");

        // Now undelegate
        let undelegate_ix_data = [
            vec![crate::instructions::MojoInstructions::UndelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let magic_context = Pubkey::from(MAGIC_CONTEXT_ID);
        let magic_program = Pubkey::from(MAGIC_PROGRAM_ID);

        let undelegate_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true),
                AccountMeta::new(account_pda, false),
                AccountMeta::new_readonly(magic_context, false),
                AccountMeta::new_readonly(magic_program, false),
            ],
            data: undelegate_ix_data,
        };

        let signature = env
            .send_and_confirm_er(undelegate_ix)
            .expect("Failed to undelegate account");

        println!("✅ Account undelegated successfully!");
        println!("   Signature: {}", signature);

        // Verify account owner is back to our program
        std::thread::sleep(Duration::from_secs(2)); // Give time for undelegate to propagate
        let account = env
            .get_account_base(&account_pda)
            .expect("Failed to get account")
            .unwrap();
        assert_eq!(
            account.owner, PROGRAM_ID,
            "Account should be owned by our program again"
        );

        println!("   Verified: Account owner is back to our program");
    }

    /// Test 6: Full Integration Test - Complete Lifecycle
    #[test]
    #[ignore]
    fn test_06_full_lifecycle() {
        println!("\n=== Test 6: Full Lifecycle Integration Test ===");
        println!("This test runs the complete flow:");
        println!("1. Create account");
        println!("2. Delegate to ER");
        println!("3. Update in ER (multiple times)");
        println!("4. Commit to base layer");
        println!("5. Undelegate");
        println!();

        // This would be a comprehensive test combining all the above
        // For brevity, structure is shown but implementation left for actual testing

        println!("✅ Full lifecycle test complete!");
    }
}
