//! World struct represents the crux of the engine

use crate::{
    errors::MojoSDKError, instruction_builder::UpdateDelegatedAccountBuilder, state::MojoState,
    types::derive_pda, utils::helpers as utils,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::sync::Arc;

/// Represents Mojo World which is seen as a container of states of the game
#[derive(Clone)]
pub struct World {
    /// The PDA of the game world
    pub world_pda: Pubkey,
    /// RPC client
    client: Arc<RpcClient>,
    /// The Mojo program ID
    program_id: Pubkey,
}

impl World {
    /// Create a new World instance
    /// TODO: Later we shall abstract program_id away from user Dev
    pub(crate) fn new(world_pda: Pubkey, client: Arc<RpcClient>, program_id: Pubkey) -> Self {
        Self {
            world_pda,
            client,
            program_id,
        }
    }

    /// Write state to a delegated account (PDA)
    ///
    /// # Arguments
    /// * `seed` - The seed string used to derive the PDA (e.g., "angry_bird")
    /// * `owner` - The keypair that owns/created this state (must sign the transaction)
    /// * `state` - The state data to write to the account
    ///
    /// # Example
    /// ```no_run
    /// # use mojo_sdk::{World, MojoState};
    /// # use solana_sdk::signature::Keypair;
    /// # fn example(world: World, owner: Keypair, state: impl MojoState) -> mojo_sdk::Result<()> {
    /// world.write_state("my_state", &owner, state)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_state<T: MojoState>(
        &self,
        seed: &Vec<u8>,
        owner: &Keypair,
        state: T,
    ) -> Result<Signature, MojoSDKError> {
        // Serialize the state data
        let state_data = state.serialize()?;

        // Compute seed to hash bytes
        let seed_bytes = utils::compute_hash(seed);

        // Derive the PDA
        let (account_pda, _bump) =
            derive_pda(&[&seed_bytes, owner.pubkey().as_ref()], &self.program_id);

        // Build the instruction
        let instruction = UpdateDelegatedAccountBuilder::new(
            self.program_id,
            owner.pubkey(),
            account_pda,
            seed,
            state_data,
        )
        .build()?;

        // Create and send the transaction
        // TODO add this to utils
        let recent_blockhash = self
            .client
            .get_latest_blockhash()
            .map_err(|e| MojoSDKError::SolanaClient(e))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&owner.pubkey()),
            &[owner],
            recent_blockhash,
        );

        // Send and confirm
        let signature = self
            .client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| MojoSDKError::TransactionFailed(e.to_string()))?;

        // log::info!("✅ State updated successfully. Signature: {}", signature);

        Ok(signature)
    }
}
