//! Main SDK client for interacting with the Mojo program
//!
use crate::{errors::MojoSDKError, state::MojoState, world::*};

use solana_keypair::Keypair;
use solana_pubkey::{pubkey, Pubkey};
use solana_rpc_client::rpc_client::RpcClient;

const PROGRAM_ID: Pubkey = pubkey!("57DTMgVYppP35GGWfcu9s2jtLo6afryGDEcrMYHoEhKn");

/// Client Wrapper to interact with the Mojo Solana Program
pub struct SdkClient {
    pub client: RpcClient,
    pub program_id: Pubkey,
}

pub enum RpcType {
    Main,
    Dev,
    ERMain,
    MBDev, // MagicBlock Devnet
    ERDev, // Ephemeral Rollup Devnet
    Surf,  // Localnet surfpool
}

impl RpcType {
    pub fn url(&self) -> &str {
        match self {
            RpcType::Main => "https://api.mainnet-beta.solana.com",
            RpcType::Dev => "https://api.devnet.solana.com",
            RpcType::ERMain => "",
            RpcType::MBDev => "https://devnet-rpc.magicblock.app",
            RpcType::ERDev => "https://devnet.magicblock.app",
            RpcType::Surf => "http://127.0.0.1:8899",
        }
    }
}

impl SdkClient {
    /// Create a new SDK client
    ///
    /// # Arguments
    /// * `rpc_url` - The Solana RPC endpoint URL
    /// * `program_id` - The Mojo program ID
    ///
    /// # Example
    /// ```
    /// // use mojo_sdk::{SdkClient, RpcType};
    /// // use solana_sdk::pubkey;
    ///
    /// // let client = SdkClient::new(RpcType::DEV);
    /// ```
    pub fn new(rpc_type: RpcType) -> Self {
        let client = RpcClient::new(rpc_type.url());
        let program_id = PROGRAM_ID;
        Self { client, program_id }
    }

    /// Create a new world
    ///
    /// # Arguments
    /// * `creator` - The keypair creating the world
    /// * `world_name` - Name/seed for the world
    /// * `initial_state` - Initial state data
    /// # Example
    /// ```
    /// // use mojo_sdk::{SdkClient, RpcType};
    /// // use solana_sdk::pubkey;
    ///
    /// // let client = SdkClient::new(RpcType::DEV);
    /// // let world = client.create_world(creator_keypair, "New World", Position{x:0, y:0});
    pub fn create_world<T: MojoState>(
        &self,
        creator: &Keypair,
        world_name: &str,
        initial_world_state: T,
    ) -> Result<World, MojoSDKError> {
        World::create_world(&self, creator, world_name, initial_world_state)
    }

    /// Write state of world
    ///
    /// # Arguments
    /// * `world` - The world object handle to be mutated on chain
    /// * `state_name` - Name/seed for the state to be changed
    /// * `owner` - The keypair of the world owner
    /// * `state` - State data to be written
    /// # Example
    /// ```
    /// // use mojo_sdk::{SdkClient, RpcType};
    /// // use solana_sdk::pubkey;
    ///
    /// // let client = SdkClient::new(RpcType::DEV);
    /// // let world = client.write_state(world, "my beast boxer", creator_keypair, Position{x:0, y:0});
    pub fn write_state<T: MojoState>(
        &self, // client
        world: &World,
        state_name: &str,
        owner: &Keypair,
        state: T,
    ) -> Result<(), MojoSDKError> {
        world.write_state(&self, state_name, owner, state)
    }

    /// Get a reference to the RPC client
    pub fn client(&self) -> &RpcClient {
        &self.client
    }

    /// Get the program ID
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }
}
