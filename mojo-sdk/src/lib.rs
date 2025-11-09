//! # Mojo SDK
//!
//! A Rust SDK for building Solana Games on-chain.
//!
//! ## Example
//!
//! ```no_run
//! use mojo_sdk::{SdkClient, MojoState};
//! use solana_sdk::signer::keypair::read_keypair_file;
//! use solana_sdk::pubkey;
//!
//! fn main() -> anyhow::Result<()> {
//!   const PROGRAM_ID: solana_sdk::pubkey::Pubkey = pubkey!("58sfdJaiSM7Ccr6nHNXXmwbfT6e9s8Zkee6zdRSH8CeS");
//!   const RPC_URL: &str = "https://api.devnet.solana.com";
//!
//!   let client = SdkClient::new(RPC_URL, PROGRAM_ID);
//!   let creator = read_keypair_file("dev_wallet.json")?;
//!
//!   // Create a world
//!   let world = client.create_world(&creator, "my_world", initial_state)?;
//!   Ok(())
//! }
//! ```
//!

// Declare Modules
mod client;
mod errors;
mod sdk;
mod types;
mod utils;

// Re-export mods
pub use client::SdkClient;
pub use errors::*;
pub use sdk::*;
pub use types::*;

// Re-export Solana types for convenience
pub mod solana {
    pub use solana_client::rpc_client::RpcClient;
    pub use solana_sdk::{
        pubkey::Pubkey,
        signature::Keypair,
        signer::{keypair::read_keypair_file, Signer},
    };
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
