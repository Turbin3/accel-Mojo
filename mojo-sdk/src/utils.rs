extern crate alloc;
use alloc::vec::Vec;
use sha2::{Digest, Sha256};

pub mod helpers {

    use super::*;

    #[macro_export]
    macro_rules! encode_packed{
        ($($data:expr),* $(,)?) => {{
            let mut combined = Vec::new();
            $(
                combined.extend_from_slice($data.as_ref());
            )*
            combined
        }};
    }

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
