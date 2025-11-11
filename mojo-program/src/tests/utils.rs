// use solana_pubkey::Pubkey;

// pub struct Pdas {
//     pub creator_account: Pubkey,
//     pub bump_creator_account: u8,
// }

// // pub fn derive_pdas


// // Sample instructions
// #[repr(C)]
// #[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
// pub struct MyPosition {
//     x: u64,
//     y: u64,
// }

// impl MyPosition {
//     pub fn new(x: u64, y: u64) -> Self {
//         Self { x, y }
//     }

//     pub fn length(&self) -> usize {
//         core::mem::size_of::<MyPosition>()
//     }

//     pub fn to_bytes(&self) -> Vec<u8> {
//         bytemuck::bytes_of(self).to_vec()
//     }
// }

