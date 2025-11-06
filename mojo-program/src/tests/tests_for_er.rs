use solana_client::rpc_client::RpcClient;
use solana_program::{
    hash::{hash, Hash},
    pubkey::Pubkey,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};

use crate::state::GenIxHandler;
use bytemuck::{Pod, Zeroable};
pub struct TestEnv {
    pub client: RpcClient,
    pub payer: Keypair,
}

const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);
const RPC_URL: &str = "http://localhost:8899";

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
mod tests {
    use std::str::FromStr;

    use super::*;
    use pinocchio::program_error::ProgramError;
    use solana_sdk::{
        account::{Account, WritableAccount},
        pubkey::Pubkey,
        signature::Signer,
        transaction::Transaction,
    };

    #[tokio::test]
    async fn test_create_state_account() {
        let rpc_client = RpcClient::new(RPC_URL);

        let payer = Keypair::new();

        let program = Pubkey::new_from_array(crate::ID);

        let account_to_create = Pubkey::find_program_address(
            &[
                &[0u8; 8],
                b"fundrais",
                payer.pubkey().as_ref(),
                &[0u8; 32],
                &[0u8; 32],
            ],
            &PROGRAM_ID,
        );

        let system_program = solana_sdk_ids::system_program::ID;
        let creator_account = payer.pubkey();

        let my_state_data = MyPosition { x: 24, y: 12 };

        // all of these would be handled on the sdk
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        // Seeds start as all zeros, just fill what you need
        let fundraiser_slice = b"fundrais"; // 8 bytes exactly
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator_account.pubkey().as_ref().try_into().unwrap());

        // const MAX_LEN: usize = 128;

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new(prereq_pda, false),
            AccountMeta::new(mint.pubkey(), true),
            AccountMeta::new(collection, false),
            AccountMeta::new_readonly(authority_prereq_pda, false),
            AccountMeta::new_readonly(mpl_core_program, false),
            AccountMeta::new_readonly(system_program, false),
        ];

        let blockchash = rpc_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");
        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some((&signer.pubkey())),
            &[&signer, &mint],
            blockchash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
}
