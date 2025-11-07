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
mod tests {

    pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);
    // pub const RPC_URL: &str = "http://127.0.0.1:8899";

    const RPC_URL: &str = "https://api.devnet.solana.com";
    const RPC_ER_URL: &str = "https://devnet.magicblock.app/";
    //

    use std::os::macos::raw::stat;

    use super::*;
    use pinocchio_log::log;
    // use solana_hash::Hash;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::read_keypair_file;
    use solana_keypair::Keypair;
    // use solana_message::Message;
    use solana_pubkey::Pubkey;
    use solana_rpc_client::rpc_client::RpcClient;
    use solana_sdk_ids::sysvar::rent::ID as RENT_ID;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    // use std::str::FromStr;
    use crate::state::GenIxHandler;

    use ephemeral_rollups_pinocchio::{
        consts::{BUFFER, DELEGATION_PROGRAM_ID, DELEGATION_RECORD, MAGIC_CONTEXT_ID},
        pda::{
            delegation_metadata_pda_from_delegated_account,
            delegation_record_pda_from_delegated_account,
        },
    };

    pub struct ReusableState {
        pub rpc_client: RpcClient,
        pub rpc_er_client: RpcClient,
        pub system_program: Pubkey,
        pub account_to_create: (Pubkey, u8),
        pub creator: Keypair,
        pub creator_2: Option<Keypair>,
        pub account_to_create2: Option<(Pubkey, u8)>,
        pub creator_account: Pubkey,
        pub owner_program: Pubkey,
        pub buffer_account: Pubkey,
        pub delegation_record: Pubkey,
        pub delegation_metadata: Pubkey,
    }

    fn setup() -> ReusableState {
        let rpc_client = RpcClient::new(RPC_URL);
        let rpc_er_client = RpcClient::new(RPC_ER_URL);

        let payer = read_keypair_file("dev_wallet.json").expect("Couldn't find wallet file");

        let program_id = Pubkey::new_from_array(crate::ID);

        let account_to_create = Pubkey::find_program_address(
            &[
                &[0u8; 8],
                b"fundra43",
                payer.pubkey().as_ref(),
                &[0u8; 32],
                &[0u8; 32],
            ],
            &PROGRAM_ID,
        );

        let system_program = solana_sdk_ids::system_program::ID;
        let creator_account = payer.pubkey();

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        // Derive delegation_record PDA: ["delegation", account_pubkey]
        let delegation_record = Pubkey::find_program_address(
            &[b"delegation", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

        // Derive delegation_metadata PDA: ["delegation-metadata", account_pubkey]
        let delegation_metadata = Pubkey::find_program_address(
            &[b"delegation-metadata", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

        // let buffer_keypair = Keypair::new();

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.as_ref()], &PROGRAM_ID).0;

        let reusable_state = ReusableState {
            rpc_client,
            rpc_er_client,
            system_program,
            account_to_create,
            creator: payer,
            account_to_create2: None,
            creator_2: None,
            creator_account,
            owner_program: PROGRAM_ID,
            buffer_account,
            delegation_metadata,
            delegation_record,
        };

        (reusable_state)
    }

    #[ignore = "reason"]
    fn test_create_state_account() {
        let mut state = setup();

        let creator = state.creator;
        let account_to_create = state.account_to_create;
        let system_program = state.system_program;

        let my_state_data = MyPosition { x: 24, y: 12 };

        // all of these would be handled on the sdk
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        // Seeds start as all zeros, just fill what you need
        let fundraiser_slice = b"fundra43"; // 8 bytes exactly
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        // const MAX_LEN: usize = 128;

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        let accounts = vec![
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(account_to_create.0, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(Pubkey::new_from_array(RENT_ID.to_bytes()), false),
        ];

        let recent_blockhash = state.rpc_client.get_latest_blockhash().unwrap();
        // .expect("failed to get recent blockhash");

        let instruction = Instruction {
            program_id: state.owner_program,
            accounts,
            data: create_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[ignore]
    fn test_delegate_state_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        let my_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        // all of these would be handled on the sdk
        let account_size = my_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        // Seeds start as all zeros, just fill what you need
        let fundraiser_slice = b"fundra43"; // 8 bytes exactly
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        // const MAX_LEN: usize = 128;

        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        let recent_blockhash = state.rpc_client.get_latest_blockhash().unwrap();
        // .expect("failed to get recent blockhash");

        let delegate_ix = Instruction {
            program_id: state.owner_program,
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),   // creator/payer
                AccountMeta::new(creator_account.0, false), // account to delegate
                AccountMeta::new(owner_program, false),     // owner program
                AccountMeta::new(buffer_account, false), // buffer PDA (created via invoke_signed)
                AccountMeta::new(delegation_record, false), // delegation record
                AccountMeta::new(delegation_metadata, false), // delegation metadata
                AccountMeta::new(system_program, false), // system program
                AccountMeta::new(delegation_program_id, false), // system program
            ],
            data: delegate_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[ignore = "reason"]
    fn test_update_delegated_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        let my_update_state_data: MyPosition = MyPosition { x: 26, y: 12 };

        // all of these would be handled on the sdk
        let account_size = my_update_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        // Seeds start as all zeros, just fill what you need
        let fundraiser_slice = b"fundra43"; // 8 bytes exactly
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        // const MAX_LEN: usize = 128;

        let update_ix_data = [
            vec![crate::instructions::MojoInstructions::UpdateDelegatedAccount as u8],
            mojo_data.to_bytes(),
            my_update_state_data.to_bytes(),
        ]
        .concat();

        let recent_blockhash = state.rpc_er_client.get_latest_blockhash().unwrap();
        // .expect("failed to get recent blockhash");

        // log!("{}", creator.pubkey());
        let update_ix = Instruction {
            program_id: state.owner_program,
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(system_program, false),
                // AccountMeta::new((RENT_ID), false),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),
                    false,
                ),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID),
                    false,
                ),
            ],
            data: update_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[update_ix],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_er_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn test_commit_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;

        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        // all of these would be handled on the sdk
        let account_size = my_update_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        // Seeds start as all zeros, just fill what you need
        let fundraiser_slice = b"fundra43"; // 8 bytes exactly
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        // const MAX_LEN: usize = 128;

        let commit_ix_data = [
            vec![crate::instructions::MojoInstructions::Commit as u8],
            mojo_data.to_bytes(),
            my_update_state_data.to_bytes(),
        ]
        .concat();

        let recent_blockhash = state
            .rpc_er_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");

        let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        // log!("{}", creator.pubkey());
        let commit_ix = Instruction {
            program_id: state.owner_program,
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(system_program, false),
                // AccountMeta::new((RENT_ID), false),
                AccountMeta::new(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),
                    false,
                ),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID),
                    false,
                ),
                // AccountMeta::new_readonly(owner_program, false), // owner program
                // AccountMeta::new_readonly(buffer_account, false), // buffer PDA (created via invoke_signed)
                // AccountMeta::new_readonly(delegation_record, false), // delegation record
                // AccountMeta::new_readonly(delegation_metadata, false), // delegation metadata
                // AccountMeta::new_readonly(system_program, false), // system program
                // AccountMeta::new_readonly(delegation_program_id, false), // system program
                // AccountMeta::new_readonly(state.system_program, false), // system program
            ],
            data: commit_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[commit_ix],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_er_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    // #[test]
    fn test_commit_and_undelegate_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        // all of these would be handled on the sdk
        let account_size = my_update_state_data.length() as u64;
        let mut mojo_data = GenIxHandler::new(account_size.to_le_bytes());
        // Seeds start as all zeros, just fill what you need
        let fundraiser_slice = b"fundra43"; // 8 bytes exactly
        mojo_data
            .fill_second(fundraiser_slice.try_into().unwrap())
            .fill_third(creator.pubkey().as_ref().try_into().unwrap());

        // const MAX_LEN: usize = 128;

        let undelegate_ix_data = [
            vec![crate::instructions::MojoInstructions::UndelegateAccount as u8],
            mojo_data.to_bytes(),
        ]
        .concat();

        let recent_blockhash = state
            .rpc_er_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");

        // log!("{}", creator.pubkey());
        let undelegate_ix = Instruction {
            program_id: state.owner_program,
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(creator_account.0, false),
                // AccountMeta::new(system_program, false),
                // AccountMeta::new((RENT_ID), false),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),
                    false,
                ),
                AccountMeta::new_readonly(
                    Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID),
                    false,
                ),
            ],
            data: undelegate_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[undelegate_ix],
            Some((&creator.pubkey())),
            &[&creator],
            recent_blockhash,
        );

        let signature = state
            .rpc_er_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
}
