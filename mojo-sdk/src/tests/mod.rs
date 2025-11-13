mod utils;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct MyPosition {
    x: u64,
    y: u64,
}

impl MyPosition {
    pub const LEN: usize = core::mem::size_of::<MyPosition>();

    pub fn length(&self) -> usize {
        core::mem::size_of::<MyPosition>()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

#[cfg(test)]
mod er_tests {

    // pub const RPC_URL: &str = "http://127.0.0.1:8899";

    const RPC_URL: &str = "https://api.devnet.solana.com";
    const RPC_ER_URL: &str = "https://devnet-eu.magicblock.app/";
    //

    // use std::os::macos::raw::stat;

    use super::*;

    fn setup() {}

    #[test]
    fn test_create_world() {
        // let mut state = setup();
    }
}
