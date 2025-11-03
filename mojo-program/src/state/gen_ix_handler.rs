use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct GenIxHandler {
    pub seeds_size: [u8; 8],
    pub seeds: [u8; 96],
    pub size: [u8; 8],
}

impl GenIxHandler {
    pub const LEN: usize = core::mem::size_of::<GenIxHandler>();
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}
