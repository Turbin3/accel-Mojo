use super::utils::helpers::*;

use crate::encode_packed;

#[cfg(test)]
mod test_create_account {
    use super::*;

    use bytemuck::{Pod, Zeroable};
    use litesvm::LiteSVM;
    use std::{io::Error, string};

    use pinocchio::{
        msg,
        sysvars::rent::{Rent, RENT_ID},
    };
    use pinocchio_log::log;
    use sha2::{Digest, Sha256};
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    // use crate::instructions::MojoInstructions::CreateAccount;

    const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);

    fn program_id() -> Pubkey {
        PROGRAM_ID
    }

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

    fn setup() -> (LiteSVM, ReusableState) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let bytes = include_bytes!("../../target/deploy/mojo_program.so");
        svm.add_program(program_id(), bytes);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let combined = encode_packed!(b"fundraiser", payer.pubkey().as_ref());
        let account_to_create = Pubkey::find_program_address(
            &[&compute_hash(&combined), payer.pubkey().as_ref()],
            &PROGRAM_ID,
        );

        let pda = String::from(account_to_create.0.to_string());
        log!("{}", &*pda);

        let system_program = solana_sdk_ids::system_program::ID;

        let reusable_state = ReusableState {
            system_program,
            account_to_create,
            creator: payer,
            account_to_create2: None,
            creator_2: None,
        };
        (svm, reusable_state)
    }

    pub struct ReusableState {
        pub system_program: Pubkey,
        pub account_to_create: (Pubkey, u8),
        pub creator: Keypair,
        pub creator_2: Option<Keypair>,
        pub account_to_create2: Option<(Pubkey, u8)>,
    }

    #[test]
    pub fn create_account() -> Result<(), Error> {
        let (mut svm, mut state) = setup();

        let creator = state.creator;
        let account_to_create = state.account_to_create;
        let system_program = state.system_program;

        let my_state_data = MyPosition { x: 24, y: 12 };
        // TODO error here

        // Input from our Rust SDK
        let combined = encode_packed!(b"fundraiser", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_state_data.length().to_le_bytes(),
        };
        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let create_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(account_to_create.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Pubkey::new_from_array(RENT_ID), false),
            ],
            data: create_ix_data,
        };

        let tx = send_singed_tx(&mut svm, create_ix, &creator)
            .map_err(|e| format!("[create_account] Instrution create_account Failed: {:?}", e))
            .unwrap();
        // msg!("tx logs: {:#?}", tx.logs);
        log!("\nAdmin Claim transaction sucessful");
        log!("CUs Consumed: {}", tx.compute_units_consumed);
        Ok(())
    }
}
