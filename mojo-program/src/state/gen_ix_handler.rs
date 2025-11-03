use bytemuck::{Pod, Zeroable};
use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct GenIxHandler {
    pub seeds: [u8; 8],
    pub size: [u8; 8],
}

impl GenIxHandler {
    pub const LEN: usize = core::mem::size_of::<GenIxHandler>();
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

#[repr(C)]
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct DelegateAccountArgs {
    pub commit_frequency_ms: u32,
    pub seeds: Vec<Vec<u8>>,
    pub validator: Option<Pubkey>,
}

impl Default for DelegateAccountArgs {
    fn default() -> Self {
        DelegateAccountArgs {
            commit_frequency_ms: u32::MAX,
            seeds: vec![],
            validator: None,
        }
    }
}