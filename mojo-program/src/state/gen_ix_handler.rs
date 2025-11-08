use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct GenIxHandler {
    pub seeds: [u8; 32], // 32 bytes for seeds hash
    pub size: [u8; 8],
}

impl GenIxHandler {
    pub const LEN: usize = core::mem::size_of::<GenIxHandler>();
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }

    pub fn get_seed_slices(&self) -> [&[u8]; 5] {
        [
            &self.seeds[0..8],    // string 1
            &self.seeds[8..16],   // string 2
            &self.seeds[16..48],  // pubkey 1
            &self.seeds[48..80],  // pubkey 2
            &self.seeds[80..112], // pubkey 3
        ]
    }

    // Create a new empty GenIxHandler
    pub fn new(size: [u8; 8]) -> Self {
        Self {
            seeds: [0u8; 32],
            size,
        }
    }

    pub fn fill_first(&mut self, first_bytes: &[u8; 8]) -> &mut Self {
        self.seeds[0..8].copy_from_slice(first_bytes);
        self
    }

    pub fn fill_second(&mut self, second_bytes: &[u8; 8]) -> &mut Self {
        self.seeds[8..16].copy_from_slice(second_bytes);
        self
    }

    pub fn fill_third(&mut self, third_bytes: &[u8; 32]) -> &mut Self {
        self.seeds[16..48].copy_from_slice(third_bytes);
        self
    }

    pub fn fill_fourth(&mut self, fourth_bytes: &[u8; 32]) -> &mut Self {
        self.seeds[48..80].copy_from_slice(fourth_bytes);
        self
    }

    pub fn fill_fifth(&mut self, fifth_bytes: &[u8; 32]) -> &mut Self {
        self.seeds[80..112].copy_from_slice(fifth_bytes);
        self
    }
    // next function to get all seeds and zero out the unassigned spaces
}
