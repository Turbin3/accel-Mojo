#[cfg(test)]
mod tests {

    use bytemuck::{Pod, Zeroable};
    use ephemeral_rollups_pinocchio::{consts::MAGIC_CONTEXT_ID, pda::delegation_record_pda_from_delegated_account};
    use litesvm::LiteSVM;
    use std::{io::Error, string};

    use pinocchio::{
        msg,
        sysvars::rent::{Rent, RENT_ID},
    };
    use pinocchio_log::log;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    use crate::instructions::delegate_account;

    // use crate::instructions::MojoInstructions::CreateAccount;

    const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);
    const LOCAL_ER: &str = "mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev";
    // const magic_context: [u8; 32] = MAGIC_CONTEXT_ID;
    // Pubkey;

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
        let account_to_create = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID,
        );

        let delegate_record = delegation_record_pda_from_delegated_account(delegate_accounts);


        let pda = String::from(account_to_create.0.to_string());
        log!("{}", &*pda);

        let system_program = solana_sdk_ids::system_program::ID;

        let reusable_state = ReusableState {
            system_program,
            account_to_create,
            creator: payer,
            account_to_create2: None,
            creator_2: None,
            mojo_account_pda: delegation,
            buffer_account,
            delegation_record,
            delegation_metadata
        };
        (svm, reusable_state)
    }

    pub struct ReusableState {
        pub system_program: Pubkey,
        pub account_to_create: (Pubkey, u8),
        pub creator: Keypair,
        pub creator_2: Option<Keypair>,
        pub account_to_create2: Option<(Pubkey, u8)>,
        pub mojo_account_pda: Pubkey,
        pub buffer_account: Pubkey,
        pub delegation_record: Pubkey,
        pub delegation_metadata: Pubkey
    }

    #[test]
    pub fn create_account() -> Result<(), Error> {
        let (mut svm, mut state) = setup();

        let creator = state.creator;
        let account_to_create = state.account_to_create;
        let system_program = state.system_program;

        let my_state_data = MyPosition { x: 24, y: 12 };
        // TODO error here

        let mut combined = [0u8; 96];
        combined[..10].copy_from_slice(b"fundraiser".as_ref());
        combined[10..42].copy_from_slice(creator.pubkey().as_ref());
        combined[42..].fill(0);

        log!("seed passed {}", &combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: combined,
            seeds_size: 42u64.to_le_bytes(),
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

        let message = Message::new(&[create_ix], Some(&creator.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&creator], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();
        // msg!("tx logs: {:#?}", tx.logs);
        log!("\nAdmin Claim transaction sucessful");
        log!("CUs Consumed: {}", tx.compute_units_consumed);
        Ok(())
    }

    #[test]

    pub fn delegate_account() -> Result<(), Error> {
        let 
    }
}


    // 0xAbim: Here goes the accounts to be delegated.
    // 0. [] The creator acts as the payer 
    // 1. [] the account pda (is_writable)
    // 2. [] the owner' program 
    // 3. [] the buffer account
    // 4. [] the delegation record
    // 5. [] the delegation metadata
    // 6. [] System Program + ...Other essential accounts...