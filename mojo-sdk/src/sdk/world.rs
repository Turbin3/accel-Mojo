//! World struct represents the crux of the engine

use crate::{
    errors::MojoSDKError, instruction_builder::UpdateDelegatedAccountBuilder, state::MojoState,
    types::derive_pda, utils::helpers as utils, GenIxHandler, MojoInstructionDiscriminator,
    SdkClient,
};

use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use solana_signer::Signer;
use solana_system_program::id as system_program_id;
use solana_sysvar::rent::ID as rent_id;
use solana_transaction::Transaction;

/// Represents Mojo World which is seen as a container of states of the game
pub struct World {
    /// The PDA of the game world
    pub world_pda: Pubkey,
    /// The Keypair of the world's creator, needed to sign for state changes.
    pub creator_keypair: Keypair,
    pub world_seed_hash: [u8; 32],
}

impl World {
    /// Create a new World instance

    pub fn create_world<T: MojoState>(
        client: &SdkClient,
        creator: &Keypair,
        world_name: &str,
        initial_world_state: T,
    ) -> Result<World, MojoSDKError> {
        // Serialize the state data
        let state_data = initial_world_state.serialize()?;

        let mut combined_seeds = Vec::new();
        combined_seeds.extend_from_slice(world_name.as_bytes());
        combined_seeds.extend_from_slice(creator.pubkey().as_ref());

        // Compute seed to hash bytes
        let seed_bytes = utils::compute_hash(&combined_seeds);
        // Derive the PDA
        let (world_pda, _bump) = derive_pda(
            &[&seed_bytes, creator.pubkey().as_ref()],
            &client.program_id,
        );

        // 3. Prepare the instruction data
        let account_size = state_data.len() as u64;
        let mojo_data = GenIxHandler {
            seeds: seed_bytes,
            size: account_size.to_le_bytes(),
        };

        let instruction_data = [
            vec![MojoInstructionDiscriminator::CreateAccount as u8], // Discriminator
            bytemuck::bytes_of(&mojo_data).to_vec(),
            state_data,
        ]
        .concat();

        // 4. Build the instruction
        let ix = Instruction {
            program_id: client.program_id,
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(world_pda, false),
                AccountMeta::new(system_program_id(), false),
                AccountMeta::new(rent_id, false),
            ],
            data: instruction_data,
        };

        let message = Message::new(&[ix], Some(&creator.pubkey()));

        // Create and send the transaction
        let recent_blockhash = client
            .client
            .get_latest_blockhash()
            .map_err(|_e| MojoSDKError::SolanaClient())?;

        let transaction = Transaction::new(&[creator], message, recent_blockhash);

        // Send and confirm
        client
            .client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| MojoSDKError::TransactionFailed(e.to_string()))?;

        Ok(World {
            creator_keypair: Keypair::from_bytes(&creator.to_bytes()).unwrap(), // Simple clone
            world_pda,
            world_seed_hash: seed_bytes,
        })
    }

    /// Write state to a delegated account (PDA) on-chain
    ///
    /// # Arguments
    /// * `seed` - The seed string used to derive the PDA (e.g., "angry_bird")
    /// * `owner` - The keypair that owns/created this state (must sign the transaction)
    /// * `state` - The state data to write to the account
    ///
    /// # Example
    /// ```no_run
    /// // use mojo_sdk::{World, MojoState};
    /// // use solana_sdk::signature::Keypair;
    /// // fn example(world: World, owner: Keypair, state: impl MojoState) -> Result<(),
    /// // MojoSDKError> {
    /// // world.write_state(client, "my_state", &owner, state)?;
    /// // Ok(())
    /// // }
    /// ```
    pub fn write_state<T: MojoState>(
        &self,
        client: &SdkClient,
        state_name: &str,
        owner: &Keypair,
        state: T,
    ) -> Result<(), MojoSDKError> {
        // Serialize the state data
        let state_data = state.serialize()?;

        let mut combined_seeds = Vec::new();
        combined_seeds.extend_from_slice(b"state");
        combined_seeds.extend_from_slice(self.world_pda.as_ref());
        combined_seeds.extend_from_slice(state_name.as_bytes());
        combined_seeds.extend_from_slice(owner.pubkey().as_ref());

        // Compute seed to hash bytes
        let seed_bytes = utils::compute_hash(&combined_seeds);

        // Derive the PDA
        let (account_pda, _bump) =
            derive_pda(&[&seed_bytes, owner.pubkey().as_ref()], &client.program_id);

        // Build the instruction
        let instruction = UpdateDelegatedAccountBuilder::new(
            client.program_id,
            owner.pubkey(),
            account_pda,
            &combined_seeds,
            state_data,
        )
        .build()?;

        let message = Message::new(&[instruction], Some(&owner.pubkey()));

        // Create and send the transaction
        let recent_blockhash = client
            .client
            .get_latest_blockhash()
            .map_err(|_e| MojoSDKError::SolanaClient())?;

        let transaction = Transaction::new(&[owner], message, recent_blockhash);

        // Send and confirm
        client
            .client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| MojoSDKError::TransactionFailed(e.to_string()))?;

        // log::info!("✅ State updated successfully. Signature: {}", signature);

        Ok(())
    }
}
