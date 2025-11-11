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
use std::time::Duration;

use crate::state::GenIxHandler;

// Configuration - matches TypeScript test
const BASE_RPC_URL: &str = "https://devnet.magicblock.app/";
const ER_RPC_URL: &str = "https://devnet.magicblock.app";

const PAYER_KEYPAIR_PATH: &str = "/home/eaa/accel_builders/accel-Mojo/target/dev_wallet_2.json";
const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);

// MagicBlock constants
const MAGIC_CONTEXT_ID: [u8; 32] = ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID;
const MAGIC_PROGRAM_ID: [u8; 32] = ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID;
const DELEGATION_PROGRAM_ID: [u8; 32] = ephemeral_rollups_pinocchio::consts::DELEGATION_PROGRAM_ID;
const BUFFER: &[u8] = ephemeral_rollups_pinocchio::consts::BUFFER;

// MagicBlock validator public key
const VALIDATOR_PUBKEY: &str = "MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57";

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

/// Test environment - matches TypeScript provider setup
pub struct TestEnv {
    pub base_client: RpcClient,
    pub er_client: RpcClient,
    pub payer: Keypair,
}

impl TestEnv {
    pub fn new() -> Self {
        println!("\n=== Test Environment Setup ===");
        println!("Base Layer Connection: {}", BASE_RPC_URL);
        println!("Ephemeral Rollup Connection: {}", ER_RPC_URL);

        let base_client = RpcClient::new_with_commitment(
            BASE_RPC_URL.to_string(),
            CommitmentConfig::confirmed(),
        );

        let er_client = RpcClient::new_with_commitment(
            ER_RPC_URL.to_string(),
            CommitmentConfig::confirmed(),
        );

        // Load payer keypair
        let payer = solana_sdk::signature::read_keypair_file(PAYER_KEYPAIR_PATH)
            .expect("Failed to load payer keypair");

        println!("Current SOL Public Key: {}", payer.pubkey());

        // Check balance
        match base_client.get_balance(&payer.pubkey()) {
            Ok(balance) => println!(
                "Current balance is {} SOL\n",
                balance as f64 / 1_000_000_000.0
            ),
            Err(e) => println!("Failed to get balance: {}", e),
        }

        Self {
            base_client,
            er_client,
            payer,
        }
    }

    /// Send and confirm transaction on base layer
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

        Ok(signature.to_string())
    }

    /// Send and confirm transaction on ER
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

        Ok(signature.to_string())
    }

    /// Wait for commitment to propagate from ER to base layer
    /// Mimics TypeScript GetCommitmentSignature function
    pub fn wait_for_commitment(
        &self,
        account_pubkey: &Pubkey,
        max_retries: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Waiting for commitment to propagate to base layer...");

        for attempt in 0..max_retries {
            std::thread::sleep(Duration::from_secs(2));

            // Check if account state matches on base layer
            if let Ok(base_account) = self.base_client.get_account(account_pubkey) {
                if let Ok(er_account) = self.er_client.get_account(account_pubkey) {
                    // Compare data to verify sync
                    if base_account.data == er_account.data {
                        println!("✅ Commitment confirmed after {} attempts", attempt + 1);
                        return Ok(());
                    }
                }
            }

            println!("Attempt {}/{}: Still waiting...", attempt + 1, max_retries);
        }

        Err("Commitment did not propagate within timeout".into())
    }

    /// Get account from base layer
    pub fn get_account_base(&self, pubkey: &Pubkey) -> Result<Account, Box<dyn std::error::Error>> {
        Ok(self.base_client.get_account(pubkey)?)
    }

    /// Get account from ER
    pub fn get_account_er(&self, pubkey: &Pubkey) -> Result<Account, Box<dyn std::error::Error>> {
        Ok(self.er_client.get_account(pubkey)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Complete integration test - mirrors TypeScript test structure
    #[test]
    #[ignore] // Run with: cargo test new_test -- --ignored
    fn test_complete_lifecycle() {
        println!("\n=== Complete Lifecycle Integration Test ===");
        println!("This test runs the complete flow:");
        println!("1. Initialize account on base layer");
        println!("2. Update state on base layer");
        println!("3. Delegate to Ephemeral Rollup");
        println!("4. Update and commit in ER");
        println!("5. Undelegate from ER");
        println!("6. Update on base layer");
        println!("7. Close account");
        println!();

        let env = TestEnv::new();

        // Derive user account PDA - matches TypeScript: findProgramAddressSync([Buffer.from("user"), publicKey])
        let (user_account, bump) = Pubkey::find_program_address(
            &[b"user", env.payer.pubkey().as_ref()],
            &PROGRAM_ID,
        );

        println!("User Account PDA: {}", user_account);
        println!("Bump: {}\n", bump);

        // ========================================
        // Test 1: Initialize Account
        // ========================================
        println!("--- Test 1: Initialize Account ---");

        let my_state_data = MyPosition::new(0, 0);
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());

        // Fill seeds: user + payer pubkey
        mojo_data
            .fill_second(b"user\0\0\0\0".try_into().unwrap())
            .fill_third(env.payer.pubkey().as_ref().try_into().unwrap());

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let create_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true), // payer (signer)
                AccountMeta::new(user_account, false),      // PDA account to create
                AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false), // system program
                AccountMeta::new_readonly(Pubkey::from(RENT_ID), false), // rent sysvar
            ],
            data: create_ix_data,
        };

        let signature = env
            .send_and_confirm_base(create_ix)
            .expect("Failed to initialize account");

        println!("✅ User Account initialized: {}", signature);

        // Verify account exists
        let account = env
            .get_account_base(&user_account)
            .expect("Failed to get account");
        let stored_position: &MyPosition = bytemuck::from_bytes(&account.data);
        println!("   Initial state: x={}, y={}\n", stored_position.x, stored_position.y);

        // ========================================
        // Test 2: Update State on Base Layer
        // ========================================
        println!("--- Test 2: Update State ---");

        let updated_position = MyPosition::new(42, 0);
        let update_ix_data = [
            vec![crate::instructions::MojoInstructions::UpdateDelegatedAccount as u8],
            mojo_data.to_bytes(),
            updated_position.to_bytes(),
        ]
        .concat();

        let update_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true), // payer (signer)
                AccountMeta::new(user_account, false),      // user account
                AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false), // system program
                AccountMeta::new_readonly(Pubkey::from(RENT_ID), false), // rent sysvar
                AccountMeta::new_readonly(Pubkey::from(MAGIC_CONTEXT_ID), false), // magic context
                AccountMeta::new_readonly(Pubkey::from(MAGIC_PROGRAM_ID), false), // magic program
            ],
            data: update_ix_data,
        };

        let signature = env
            .send_and_confirm_base(update_ix)
            .expect("Failed to update account");

        println!("✅ User Account State Updated: {}", signature);

        // Verify update
        let account = env.get_account_base(&user_account).expect("Failed to get account");
        let stored_position: &MyPosition = bytemuck::from_bytes(&account.data);
        println!("   Updated state: x={}, y={}\n", stored_position.x, stored_position.y);

        // ========================================
        // Test 3: Delegate to Ephemeral Rollup
        // ========================================
        println!("--- Test 3: Delegate to Ephemeral Rollup ---");

        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, user_account.as_ref()], &PROGRAM_ID).0;

        let delegation_program_id = Pubkey::from(DELEGATION_PROGRAM_ID);
        let delegation_record = Pubkey::find_program_address(
            &[b"delegation", user_account.as_ref()],
            &delegation_program_id,
        )
        .0;

        let delegation_metadata = Pubkey::find_program_address(
            &[b"delegation-metadata", user_account.as_ref()],
            &delegation_program_id,
        )
        .0;

        let validator_pubkey = Pubkey::try_from(VALIDATOR_PUBKEY).expect("Invalid validator pubkey");

        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let delegate_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true), // payer (signer)
                AccountMeta::new(user_account, false),      // user account
                AccountMeta::new_readonly(PROGRAM_ID, false), // owner program
                AccountMeta::new(buffer_account, false),    // buffer account
                AccountMeta::new(delegation_record, false), // delegation record
                AccountMeta::new(delegation_metadata, false), // delegation metadata
                AccountMeta::new_readonly(delegation_program_id, false), // delegation program
                AccountMeta::new_readonly(validator_pubkey, false), // validator (for devnet)
                AccountMeta::new_readonly(Pubkey::from(system_program::ID.to_bytes()), false), // system program
            ],
            data: delegate_ix_data,
        };

        let signature = env
            .send_and_confirm_base(delegate_ix)
            .expect("Failed to delegate account");

        println!("✅ User Account Delegated to Ephemeral Rollup: {}", signature);

        // Verify delegation
        let account = env.get_account_base(&user_account).expect("Failed to get account");
        assert_eq!(
            account.owner, delegation_program_id,
            "Account should be owned by delegation program"
        );
        println!("   Verified: Account owner is delegation program\n");

        // ========================================
        // Test 4: Update and Commit in ER
        // ========================================
        println!("--- Test 4: Update State and Commit to Base Layer ---");

        let updated_position = MyPosition::new(43, 0);
        let update_commit_ix_data = [
            vec![crate::instructions::MojoInstructions::Commit as u8],
            mojo_data.to_bytes(),
            updated_position.to_bytes(),
        ]
        .concat();

        let update_commit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true), // payer (signer)
                AccountMeta::new(user_account, false),      // user account
                AccountMeta::new_readonly(Pubkey::from(MAGIC_CONTEXT_ID), false), // magic context
                AccountMeta::new_readonly(Pubkey::from(MAGIC_PROGRAM_ID), false), // magic program
                AccountMeta::new_readonly(Pubkey::from(system_program::ID.to_bytes()), false), // system program
            ],
            data: update_commit_ix_data,
        };

        let signature = env
            .send_and_confirm_er(update_commit_ix)
            .expect("Failed to update and commit");

        println!("✅ User Account State Updated: {}", signature);

        // Wait for commitment to propagate
        env.wait_for_commitment(&user_account, 30)
            .expect("Commitment did not propagate");

        // Verify state on base layer
        let account = env.get_account_base(&user_account).expect("Failed to get account");
        let stored_position: &MyPosition = bytemuck::from_bytes(&account.data);
        println!("   Committed state: x={}, y={}\n", stored_position.x, stored_position.y);

        // ========================================
        // Test 5: Undelegate from ER
        // ========================================
        println!("--- Test 5: Commit and Undelegate from Ephemeral Rollup ---");

        let undelegate_ix_data = [
            vec![crate::instructions::MojoInstructions::UndelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let undelegate_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true), // payer (signer)
                AccountMeta::new(user_account, false),      // user account
                AccountMeta::new_readonly(Pubkey::from(MAGIC_CONTEXT_ID), false), // magic context
                AccountMeta::new_readonly(Pubkey::from(MAGIC_PROGRAM_ID), false), // magic program
            ],
            data: undelegate_ix_data,
        };

        let signature = env
            .send_and_confirm_er(undelegate_ix)
            .expect("Failed to undelegate");

        println!("✅ User Account Undelegated: {}", signature);

        // Wait for undelegation to propagate
        std::thread::sleep(Duration::from_secs(3));

        // Verify account owner is back to our program
        let account = env.get_account_base(&user_account).expect("Failed to get account");
        assert_eq!(
            account.owner, PROGRAM_ID,
            "Account should be owned by our program again"
        );
        println!("   Verified: Account owner is back to our program\n");

        // ========================================
        // Test 6: Update State on Base Layer (Post-Undelegation)
        // ========================================
        println!("--- Test 6: Update State (Post-Undelegation) ---");

        let updated_position = MyPosition::new(45, 0);
        let update_ix_data = [
            vec![crate::instructions::MojoInstructions::UpdateDelegatedAccount as u8],
            mojo_data.to_bytes(),
            updated_position.to_bytes(),
        ]
        .concat();

        let update_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true), // payer (signer)
                AccountMeta::new(user_account, false),      // user account
                AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false), // system program
                AccountMeta::new_readonly(Pubkey::from(RENT_ID), false), // rent sysvar
                AccountMeta::new_readonly(Pubkey::from(MAGIC_CONTEXT_ID), false), // magic context
                AccountMeta::new_readonly(Pubkey::from(MAGIC_PROGRAM_ID), false), // magic program
            ],
            data: update_ix_data,
        };

        let signature = env
            .send_and_confirm_base(update_ix)
            .expect("Failed to update account");

        println!("✅ User Account State Updated: {}", signature);

        // Verify update
        let account = env.get_account_base(&user_account).expect("Failed to get account");
        let stored_position: &MyPosition = bytemuck::from_bytes(&account.data);
        println!("   Final state: x={}, y={}\n", stored_position.x, stored_position.y);

        // ========================================
        // Test 7: Close Account
        // ========================================
        println!("--- Test 7: Close Account ---");

        let close_ix_data = vec![crate::instructions::MojoInstructions::UndelegateAccount as u8];

        let close_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(env.payer.pubkey(), true),            // payer (signer, receives lamports)
                AccountMeta::new(user_account, false),                 // account to close
                AccountMeta::new(Pubkey::from(system_program::ID.to_bytes()), false), // system program
            ],
            data: close_ix_data,
        };

        let signature = env
            .send_and_confirm_base(close_ix)
            .expect("Failed to close account");

        println!("✅ User Account Closed: {}", signature);

        // Verify account is closed
        match env.get_account_base(&user_account) {
            Err(_) => println!("   Verified: Account no longer exists\n"),
            Ok(_) => panic!("Account should be closed"),
        }

        println!("\n=== ✅ Complete Lifecycle Test Passed! ===\n");
    }
}
