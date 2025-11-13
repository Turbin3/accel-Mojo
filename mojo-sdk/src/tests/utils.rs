extern crate alloc;
use alloc::vec::Vec;

#[cfg(test)]
pub mod helpers {

    use sha2::{Digest, Sha256};

    use super::*;

    pub trait Pipe {
        fn pipe<F, R>(self, f: F) -> R
        where
            F: FnOnce(Self) -> R,
            Self: Sized,
        {
            f(self)
        }
    }

    impl<T> Pipe for T {}

    pub fn vec_to_array<const N: usize>(vec: Vec<u8>) -> [u8; N] {
        vec.try_into()
            .unwrap_or_else(|v: Vec<u8>| panic!("Expected length {}, got {}", N, v.len()))
    }

    pub fn compute_hash(data: &Vec<u8>) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

/**
 * This mod mocks what in our test what the sdk should implement
 */
pub mod sdk_calls {}
