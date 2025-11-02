use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct GenIxHandler {
    pub seeds: [u8; 8],
    pub size: [u8; 8],
}

impl GenIxHandler {
    pub const LEN: usize = core::mem::size_of::<GenIxHandler>();
}
