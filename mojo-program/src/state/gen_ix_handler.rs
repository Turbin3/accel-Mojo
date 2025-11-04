use bytemuck::{Pod, Zeroable};

extern crate alloc;
pub use alloc::vec::Vec;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct GenIxHandler {
    // TODO: Unneeded
    // pub seeds_size: [u8; 8],

    // seeds are represented as a sha256
    pub seeds: [u8; 32],
    pub size: [u8; 8],
}

impl GenIxHandler {
    pub const LEN: usize = core::mem::size_of::<GenIxHandler>();
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}
