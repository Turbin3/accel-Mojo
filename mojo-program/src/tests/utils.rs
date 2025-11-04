extern crate alloc;
use alloc::vec::Vec;
use litesvm::{types::TransactionResult, LiteSVM};
use sha2::{Digest, Sha256};
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_transaction::Transaction;

pub mod helpers {
    use solana_signer::Signer;

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

    // Helper: Send the transaction
    pub fn send_singed_tx(
        svm: &mut LiteSVM,
        ix: Instruction,
        payer: &Keypair,
    ) -> TransactionResult {
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[payer], message, recent_blockhash);

        svm.send_transaction(transaction)
    }
}

/**
 * This mod mocks what in our test what the sdk should implement
 */
pub mod sdk_calls {}
