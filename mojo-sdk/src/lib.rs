//! # Mojo SDK
//!
//! A Rust SDK for building Solana Games on-chain.
//!
//! ## Example
//!
//! ```no_run
//! use mojo_sdk::{SdkClient, MojoState, MojoSDKError};
//! // use solana_sdk::signer::keypair::read_keypair_file;
//! // use solana_sdk::pubkey;
//!
//! fn main() -> Result<(), MojoSDKError> {
//!   // const PROGRAM_ID: solana_sdk::pubkey::Pubkey = pubkey!("58sfdJaiSM7Ccr6nHNXXmwbfT6e9s8Zkee6zdRSH8CeS");
//!   // const RPC_URL: &str = "https://api.devnet.solana.com";
//!
//!   // let client = SdkClient::new(RPC_URL, PROGRAM_ID);
//!   // let creator = read_keypair_file("dev_wallet.json").expect("Failed to read keys");
//!
//!   // Create a world
//!   // let world = client.create_world(&creator, "my_world", initial_state)?;
//!   Ok(())
//! }
//! ```
//!

// Declare Modules
mod client;
mod errors;
mod sdk;
mod tests;
mod types;
mod utils;

// Re-export mods
pub use client::SdkClient;
pub use errors::*;
pub use sdk::*;
pub use types::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
