//! Main SDK client for interacting with the Mojo program

use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey, pubkey::Pubkey};
use std::sync::Arc;

const PROGRAM_ID: Pubkey = pubkey!("58sfdJaiSM7Ccr6nHNXXmwbfT6e9s8Zkee6zdRSH8CeS");

/// Client Wrapper to interact with the Mojo Solana Program
pub struct SdkClient {
    client: Arc<RpcClient>,
    program_id: Pubkey,
}

pub enum RpcType {
    Main,
    Dev,
    ERMain,
    ERDev,
}

impl RpcType {
    pub fn url(&self) -> &str {
        match self {
            RpcType::Main => "https://api.mainnet-beta.solana.com",
            RpcType::Dev => "https://api.devnet.solana.com",
            RpcType::ERMain => "",
            RpcType::ERDev => "",
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
    /// use mojo_sdk::{SdkClient, RpcType};
    /// use solana_sdk::pubkey;
    ///
    /// let client = SdkClient::new(RpcType::DEV);
    /// ```
    pub fn new(rpc_type: RpcType) -> Self {
        let client = Arc::new(RpcClient::new(rpc_type.url()));
        let program_id = PROGRAM_ID;
        Self { client, program_id }
    }

    // /// Create a new world
    // ///
    // /// # Arguments
    // /// * `creator` - The keypair creating the world
    // /// * `world_name` - Name/seed for the world
    // /// * `initial_state` - Initial state data
    // pub fn create_world<T: MojoState>(
    //     &self,
    //     creator: &Keypair,
    //     world_name: &str,
    //     initial_state: T,
    // ) -> Result<World, MojoSDKError> {
    //     // This would call the CreateAccount instruction in Solana Program
    //     todo!("create_world implementation")
    // }
    //
    // /// Get a reference to the RPC client
    // pub fn client(&self) -> &RpcClient {
    //     &self.client
    // }
    //
    // /// Get the program ID
    // pub fn program_id(&self) -> &Pubkey {
    //     &self.program_id
    // }
}
