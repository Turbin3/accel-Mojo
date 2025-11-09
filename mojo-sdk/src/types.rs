//! Common types that are used throughout the SDK to interact with the Solana Program

use bytemuck::{Pod, Zeroable};
use solana_sdk::pubkey::Pubkey;

use crate::utils::helpers as utils;

/// Instruction discriminators matching the Solana program
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MojoInstructionDiscriminator {
    CreateAccount = 0,
    DelegateAccount = 1,
    Commit = 2,
    UpdateDelegatedAccount = 3,
    UndelegateAccount = 4,
}

impl From<MojoInstructionDiscriminator> for u8 {
    fn from(discriminator: MojoInstructionDiscriminator) -> Self {
        discriminator as u8
    }
}

/// Generic instruction handler that matches the Solana program structure
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct GenIxHandler {
    pub seeds: [u8; 32], // represent hash of list of seeds
    pub size: [u8; 8],   // u64 as le bytes
}

impl GenIxHandler {
    pub const LEN: usize = std::mem::size_of::<Self>(); // 40 bytes

    /// Create a new GenIxHandler from seed and size
    pub fn new(seed: &Vec<u8>, size: usize) -> Self {
        let computed_hash_seed = utils::compute_hash(seed);

        Self {
            seeds: computed_hash_seed,
            size: (size as u64).to_le_bytes(),
        }
    }

    /// Get the seed bytes
    pub fn seed_bytes(&self) -> &[u8; 32] {
        &self.seeds
    }

    /// Get the size as usize
    pub fn size(&self) -> usize {
        u64::from_le_bytes(self.size) as usize
    }
}

/// Wrapper for derive PDA
pub fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, program_id)
}
