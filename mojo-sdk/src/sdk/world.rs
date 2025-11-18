//! World struct represents the crux of the engine

use crate::{
    errors::MojoSDKError, instruction_builder::UpdateDelegatedAccountBuilder, state::MojoState,
    types::derive_pda, utils::helpers as utils, GenIxHandler, MojoInstructionDiscriminator,
    SdkClient,
};

use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::RpcError;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_program::id as system_program_id;
use solana_sysvar::rent::ID as rent_id;
use solana_transaction::Transaction;

/// Represents Mojo World which is seen as a container of states of the game
pub struct World {
    /// The PDA of the game world
    pub world_pda: Pubkey,
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

        let creator_pubkey = creator.pubkey();
        let world_seed_input = Self::world_seed_input(world_name, &creator_pubkey);

        // Compute seed to hash bytes
        let seed_bytes = utils::compute_hash(&world_seed_input);
        // Derive the PDA
        let (world_pda, _bump) =
            derive_pda(&[&seed_bytes, creator_pubkey.as_ref()], &client.program_id);

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
                AccountMeta::new(creator_pubkey, true),
                AccountMeta::new(world_pda, false),
                AccountMeta::new(system_program_id(), false),
                AccountMeta::new(rent_id, false),
            ],
            data: instruction_data,
        };

        Self::submit_instructions(client, creator, vec![ix])?;

        Ok(World {
            world_pda,
            world_seed_hash: seed_bytes,
        })
    }

    pub fn write_state<T: MojoState>(
        &self,
        client: &SdkClient,
        state_name: &str,
        owner: &Keypair,
        state: T,
    ) -> Result<(), MojoSDKError> {
        // Serialize the state data
        let state_data = state.serialize()?;

        let owner_pubkey = owner.pubkey();
        let (account_pda, state_seed_input, _seed_hash) =
            self.derive_state_pda(state_name, &owner_pubkey, client);

        let account_status = Self::delegated_account_status(client, &account_pda)?;

        match account_status {
            DelegatedAccountStatus::Exists => {
                let update_ix = Self::build_update_state_instruction(
                    client.program_id,
                    owner_pubkey,
                    account_pda,
                    &state_seed_input,
                    &state_data,
                )?;
                Self::submit_instructions(client, owner, vec![update_ix])
            }
            DelegatedAccountStatus::Missing => {
                let create_ix = Self::build_create_state_instruction(
                    client.program_id,
                    owner_pubkey,
                    account_pda,
                    &state_seed_input,
                    &state_data,
                );
                Self::submit_instructions(client, owner, vec![create_ix])
            }
        }
    }

    /// Read the current state stored in a delegated account
    pub fn read_delegated_state<T: MojoState>(
        &self,
        client: &SdkClient,
        state_name: &str,
        owner: &Pubkey,
    ) -> Result<T, MojoSDKError> {
        let (account_pda, _seed_input, _seed_hash) =
            self.derive_state_pda(state_name, owner, client);
        let account_data = Self::fetch_owned_account_data(client, &account_pda)?;
        T::deserialize(&account_data)
    }

    /// Read the data stored in the world PDA itself
    pub fn read_world_state<T: MojoState>(&self, client: &SdkClient) -> Result<T, MojoSDKError> {
        let account_data = Self::fetch_owned_account_data(client, &self.world_pda)?;
        T::deserialize(&account_data)
    }

    fn world_seed_input(world_name: &str, creator: &Pubkey) -> Vec<u8> {
        crate::encode_packed!(b"world", world_name.as_bytes(), creator.as_ref())
    }

    fn derive_state_pda(
        &self,
        state_name: &str,
        owner: &Pubkey,
        client: &SdkClient,
    ) -> (Pubkey, Vec<u8>, [u8; 32]) {
        let seed_input = crate::encode_packed!(
            b"state",
            self.world_seed_hash.as_ref(),
            state_name.as_bytes(),
            owner.as_ref()
        );
        let seed_hash = utils::compute_hash(&seed_input);
        let (pda, _bump) = derive_pda(&[&seed_hash, owner.as_ref()], &client.program_id);
        (pda, seed_input, seed_hash)
    }

    fn fetch_owned_account_data(
        client: &SdkClient,
        account: &Pubkey,
    ) -> Result<Vec<u8>, MojoSDKError> {
        let acc = client
            .client
            .get_account(account)
            .map_err(|e| MojoSDKError::AccountNotFound(format!("{}: {}", account, e)))?;

        if acc.owner != client.program_id {
            return Err(MojoSDKError::InvalidAccountOwner(format!(
                "expected {}, got {}",
                client.program_id, acc.owner
            )));
        }

        Ok(acc.data)
    }

    fn delegated_account_status(
        client: &SdkClient,
        account: &Pubkey,
    ) -> Result<DelegatedAccountStatus, MojoSDKError> {
        match client.client.get_account(account) {
            Ok(acc) => {
                if acc.owner != client.program_id {
                    return Err(MojoSDKError::InvalidAccountOwner(format!(
                        "expected {}, got {}",
                        client.program_id, acc.owner
                    )));
                }
                Ok(DelegatedAccountStatus::Exists)
            }
            Err(err) => {
                if Self::is_account_missing(&err) {
                    Ok(DelegatedAccountStatus::Missing)
                } else {
                    Err(MojoSDKError::SolanaSdk(err.to_string()))
                }
            }
        }
    }

    fn is_account_missing(err: &ClientError) -> bool {
        match err.kind() {
            ClientErrorKind::RpcError(RpcError::ForUser(message)) => {
                message.contains("AccountNotFound") || message.contains("could not find account")
            }
            _ => false,
        }
    }

    fn build_create_state_instruction(
        program_id: Pubkey,
        owner: Pubkey,
        account_pda: Pubkey,
        seed_input: &Vec<u8>,
        state_data: &[u8],
    ) -> Instruction {
        let mojo_data = GenIxHandler::new(seed_input, state_data.len());

        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(owner, true),
                AccountMeta::new(account_pda, false),
                AccountMeta::new(system_program_id(), false),
                AccountMeta::new(rent_id, false),
            ],
            data: [
                vec![MojoInstructionDiscriminator::CreateAccount as u8],
                bytemuck::bytes_of(&mojo_data).to_vec(),
                state_data.to_vec(),
            ]
            .concat(),
        }
    }

    fn build_update_state_instruction(
        program_id: Pubkey,
        owner: Pubkey,
        account_pda: Pubkey,
        seed_input: &Vec<u8>,
        state_data: &[u8],
    ) -> Result<Instruction, MojoSDKError> {
        UpdateDelegatedAccountBuilder::new(
            program_id,
            owner,
            account_pda,
            seed_input,
            state_data.to_vec(),
        )
        .build()
    }

    fn submit_instructions(
        client: &SdkClient,
        signer: &Keypair,
        instructions: Vec<Instruction>,
    ) -> Result<(), MojoSDKError> {
        let message = Message::new(&instructions, Some(&signer.pubkey()));
        let recent_blockhash = client
            .client
            .get_latest_blockhash()
            .map_err(|_e| MojoSDKError::SolanaClient())?;
        let transaction = Transaction::new(&[signer], message, recent_blockhash);

        client
            .client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| MojoSDKError::TransactionFailed(e.to_string()))?;
        Ok(())
    }
}

enum DelegatedAccountStatus {
    Exists,
    Missing,
}
