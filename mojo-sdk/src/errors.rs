//! Error types for the Mojo SDK

use thiserror::Error;

/// Errors that can occur when using the Mojo SDK
#[derive(Error, Debug)]
pub enum MojoSDKError {
    #[error("Solana client error: ")]
    SolanaClient(),

    #[error("Solana SDK error: {0}")]
    SolanaSdk(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Invalid PDA derivation: {0}")]
    InvalidPda(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Invalid seed: {0}")]
    InvalidSeed(String),

    #[error("Invalid state data: {0}")]
    InvalidStateData(String),

    #[error("Account owned by different program: {0}")]
    InvalidAccountOwner(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
