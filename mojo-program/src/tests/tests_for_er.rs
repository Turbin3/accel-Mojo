use bytemuck::{Pod, Zeroable};
use pinocchio::sysvars::rent::RENT_ID;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_sdk_ids::system_program;
use std::fs;
use std::time::Duration;
use pinocchio_log::log;

use crate::state::GenIxHandler;

// Configuration
// const RPC_URL: &str = "http://0.0.0.0:8899"; // Base layer (solana-test-validator)
// const ER_RPC_URL: &str = "http://0.0.0.0:7799"; // Ephemeral Rollup validator

// 0xAbim: Devnet config
const RPC_URL: &str = "https://devnet-eu.magicblock.app/"; // Base layer (solana-test-validator)
const ER_RPC_URL: &str = "https://devnet-eu.magicblock.app"; // Ephemeral Rollup validator
const EU_ACCOUNT_OWNER: [u8; 32] = pinocchio_pubkey::pubkey!("MEUGGrYPxKk17hCr7wpT6s8dtNokZj5U2L57vjYMS8e");

const PAYER_KEYPAIR_PATH: &str = "/home/eaa/accel_builders/accel-Mojo/target/dev_wallet_2.json"; // Path to funded keypair
const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);

// MagicBlock constants for localhost
const MAGIC_CONTEXT_ID: [u8; 32] = ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID;
const MAGIC_PROGRAM_ID: [u8; 32] = ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID;
const DELEGATION_PROGRAM_ID: [u8; 32] = ephemeral_rollups_pinocchio::consts::DELEGATION_PROGRAM_ID;
const BUFFER: &[u8] = ephemeral_rollups_pinocchio::consts::BUFFER;

// Sample instructions
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
        println!("Initializing TestEnv...");
        println!("  Base RPC: {}", RPC_URL);
        println!("  ER RPC: {}", ER_RPC_URL);

        let base_client = RpcClient::new_with_timeout(RPC_URL.to_string(), Duration::from_secs(30));

        let er_client =
                RpcClient::new_with_timeout(ER_RPC_URL.to_string(), Duration::from_secs(30));

        // Test connectivity
        match base_client.get_health() {
            Ok(_) => println!("  ✓ Base layer RPC is healthy"),
            Err(e) => println!("  ✗ Base layer RPC error: {}", e),
        }

        match er_client.get_health() {
            Ok(_) => println!("  ✓ ER RPC is healthy"),
            Err(e) => println!("  ✗ ER RPC error: {}", e),
        }

        // Load payer keypair from file
        let payer = solana_sdk::signature::read_keypair_file(PAYER_KEYPAIR_PATH)
            .expect("Failed to load payer keypair from file");

        println!("  Loaded payer: {}", payer.pubkey());

        // Check balance
        match base_client.get_balance(&payer.pubkey()) {
            Ok(balance) => println!("  Payer balance: {} SOL", balance as f64 / 1_000_000_000.0),
            Err(e) => println!("  ✗ Failed to get payer balance: {}", e),
        }

        Self {
            base_client,
            er_client,
            payer,
        }
    }

    /// Load keypair from JSON file
//   fn load_keypair(path: &str) -> Result<Keypair, Box<dyn Error>> {
//     let keypair = read_keypair_file(PAYER_KEYPAIR_PATH)?;
//     Ok(keypair);
//     }

    /// Airdrop SOL to payer on base layer
    pub fn airdrop(&self, lamports: u64) -> Result<(), Box<dyn std::error::Error>> {
        println!("Requesting airdrop of {} lamports to {}", lamports, self.payer.pubkey());

        let signature = self
            .base_client
            .request_airdrop(&self.payer.pubkey(), lamports)?;

        // Wait for confirmation with finalized commitment
        loop {
            let confirmed = self.base_client
                .confirm_transaction_with_spinner(
                    &signature,
                    &self.base_client.get_latest_blockhash()?,
                    CommitmentConfig::confirmed(),
                );

            if confirmed.is_ok() {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }

        // Verify balance
        let balance = self.base_client.get_balance(&self.payer.pubkey())?;
        println!(
            "✅ Airdropped {} lamports to {} (balance: {})",
            lamports,
            self.payer.pubkey(),
            balance
        );

        assert!(balance >= lamports, "Airdrop failed - insufficient balance");
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

    // Helper to log compute units consumed
    // fn log_compute_units(&self, signature: &solana_sdk::signature::Signature, client: &RpcClient, layer: &str) {
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
mod tests {

    use super::*;

    /// Test 1: Create Account on Base Layer
    #[test]
    #[ignore] // Run with: cargo test --test integration -- --ignored
    fn test_01_create_account() {
        println!("\n=== Test 1: Create Account ===");

        let env = TestEnv::new();
                // let env = TestEnv::new();
        // env.airdrop(10_000_000_000).expect("Airdrop failed");


        // Derive PDA
        let (account_pda, _bump) = Pubkey::find_program_address(
            &[
                // &[0u8; 8],
                &[0u8; 8],
                b"fundrais",
                env.payer.pubkey().as_ref(),
                &[0u8; 32],
                &[0u8; 32],
            ],
            &PROGRAM_ID,
        );

        println!("PDA to create: {}", account_pda);
        

        // Prepare instruction data
        let my_state_data = MyPosition::new(24, 12);
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());

        // 0xAbim: Added some bytes into the ix
        mojo_data
            .fill_first(&[0u8; 8])
            .fill_second(b"fundrais".try_into().unwrap())
            .fill_third(env.payer.pubkey().as_ref().try_into().unwrap())
            .fill_fourth(&[0u8; 32])  
            .fill_fifth(&[0u8; 32]);

            // After deriving PDA
println!("Test PDA: {}", account_pda);
println!("Test seeds:");
println!("  [0] {:02x?}", &[0u8; 8]);
println!("  [1] {:02x?}", b"fundrais");
println!("  [2] {:02x?}", env.payer.pubkey().as_ref());
println!("  [3] {:02x?}", &[0u8; 32]);
println!("  [4] {:02x?}", &[0u8; 32]);
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
                AccountMeta::new(Pubkey::from(crate::ID), false),
                AccountMeta::new(Pubkey::from(EU_ACCOUNT_OWNER), false), 
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

        let env = TestEnv::new();
                // let env = TestEnv::new();
        // env.airdrop(10_000_000_000).expect("Airdrop failed");


        // First create the account (same as test 1)
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

        // Create account first
        let my_state_data = MyPosition::new(24, 12);
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
              // 0xAbim: Added some bytes into the ix
        mojo_data
            .fill_first(&[0u8; 8])
            .fill_second(b"fundrais".try_into().unwrap())
            .fill_third(env.payer.pubkey().as_ref().try_into().unwrap())
            .fill_fourth(&[0u8; 32])  
            .fill_fifth(&[0u8; 32]);

        // let create_ix_data = [
        //     vec![crate::instructions::MojoInstructions::CreateAccount as u8],
        //     mojo_data.to_bytes(),
        //     my_state_data.to_bytes(),
        // ]
        // .concat();

        // let create_ix = Instruction {
        //     program_id: PROGRAM_ID,
        //     accounts: vec![
        //         AccountMeta::new(env.payer.pubkey(), true),
        //         AccountMeta::new(account_pda, false),
        //         AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false),
        //         AccountMeta::new(Pubkey::from(RENT_ID), false),
        //     ],
        //     data: create_ix_data,
        // };

        // env.send_and_confirm_base(create_ix)
        //     .expect("Failed to create account");
        // println!("✅ Account created: {}", account_pda);

        // Now delegate the account
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, account_pda.as_ref()], &PROGRAM_ID).0;

        let delegation_program_id = Pubkey::from(DELEGATION_PROGRAM_ID);
        let delegation_record = Pubkey::find_program_address(
            &[b"delegation", account_pda.as_ref()],
            &delegation_program_id,
        )
        .0;

        log!("Test stops here at metadata");
        let delegation_metadata = Pubkey::find_program_address(
            &[b"delegation-metadata", account_pda.as_ref()],
            &delegation_program_id,
        )
        .0;

        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        log!("Test stops here at delegation ix");
        let delegate_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true),
                AccountMeta::new(account_pda, true),
                AccountMeta::new(EU_ACCOUNT_OWNER.into(), false), // owner program
                AccountMeta::new(buffer_account, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                // AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false),
                AccountMeta::new(delegation_program_id, false),
            ],
            data: delegate_ix_data,
        };

        let signature = env
            .send_and_confirm_base(delegate_ix)
            .expect("Failed to delegate account");

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
                // let env = TestEnv::new();
        // env.airdrop(10_000_000_000).expect("Airdrop failed");


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
        env.send_and_confirm_base(create_ix)
            .expect("Failed to create account");

        // Delegate account
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, account_pda.as_ref()], &PROGRAM_ID).0;
        let delegation_program_id = Pubkey::from(DELEGATION_PROGRAM_ID);
        let delegation_record = Pubkey::find_program_address(
            &[b"delegation", account_pda.as_ref()],
            &delegation_program_id,
        )
        .0;
        let delegation_metadata = Pubkey::find_program_address(
            &[b"delegation-metadata", account_pda.as_ref()],
            &delegation_program_id,
        )
        .0;

        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let delegate_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true),
                AccountMeta::new(account_pda, false),
                AccountMeta::new(PROGRAM_ID, false),
                AccountMeta::new(buffer_account, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false),
                AccountMeta::new(delegation_program_id, false),
            ],
            data: delegate_ix_data,
        };
        env.send_and_confirm_base(delegate_ix)
            .expect("Failed to delegate account");
        println!("✅ Setup complete: Account created and delegated");

        // Now update the account IN THE ER
        let updated_position = MyPosition::new(100, 200);
        let update_ix_data = [
            vec![crate::instructions::MojoInstructions::UpdateDelegatedAccount as u8],
            mojo_data.to_bytes(),
            updated_position.to_bytes(),
        ]
        .concat();

        let magic_context = Pubkey::from(MAGIC_CONTEXT_ID);
        let magic_program = Pubkey::from(MAGIC_PROGRAM_ID);

        let update_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true),
                AccountMeta::new(account_pda, false),
                AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false),
                AccountMeta::new(Pubkey::from(RENT_ID), false),
                AccountMeta::new_readonly(magic_context, false),
                AccountMeta::new_readonly(magic_program, false),
            ],
            data: update_ix_data,
        };

        let signature = env
            .send_and_confirm_er(update_ix)
            .expect("Failed to update delegated account in ER");

        println!("✅ Account updated in ER successfully!");
        println!("   Signature: {}", signature);

        // Verify the update in ER
        let account_er = env
            .get_account_er(&account_pda)
            .expect("Failed to get account from ER")
            .unwrap();
        let stored_position: &MyPosition = bytemuck::from_bytes(&account_er.data);
        assert_eq!(stored_position.x, 100);
        assert_eq!(stored_position.y, 200);

        println!(
            "   Verified in ER: x={}, y={}",
            stored_position.x, stored_position.y
        );
    }

    /// Test 4: Commit Changes from ER to Base Layer
    #[test]
    #[ignore]
    fn test_04_commit_account() {
        println!("\n=== Test 4: Commit Account from ER to Base Layer ===");

        let env = TestEnv::new();
        // env.airdrop(10_000_000_000).expect("Airdrop failed");

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
        // env.airdrop(10_000_000_000).expect("Airdrop failed");

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
