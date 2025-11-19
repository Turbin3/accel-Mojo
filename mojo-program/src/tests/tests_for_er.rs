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

    pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);
    // pub const RPC_URL: &str = "http://127.0.0.1:8899";

    const RPC_URL: &str = "https://api.devnet.solana.com";
    const RPC_ER_URL: &str = "https://devnet-eu.magicblock.app/";
    //

    // use std::os::macos::raw::stat;

    use super::*;
    use ephemeral_rollups_pinocchio::consts::COMMIT_RECORD;
    use ephemeral_rollups_pinocchio::consts::MAGIC_PROGRAM_ID;
    use ephemeral_rollups_pinocchio::pda::commit_state_pda_from_delegated_account;
    use ephemeral_rollups_pinocchio::pda::fees_vault_pda;
    use ephemeral_rollups_pinocchio::pda::undelegate_buffer_pda_from_delegated_account;
    use ephemeral_rollups_pinocchio::pda::validator_fees_vault_pda_from_validator;
    use pinocchio::program_error::ProgramError;
    use pinocchio_log::log;
    // use solana_hash::Hash;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::read_keypair_file;
    use solana_keypair::Keypair;
    // use solana_message::Message;
    use solana_pubkey::pubkey;
    use solana_pubkey::Pubkey;
    use solana_rpc_client::rpc_client::RpcClient;
    use solana_sdk_ids::sysvar::rent::ID as RENT_ID;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    // use std::str::FromStr;
    use crate::encode_packed;
    use crate::tests::utils::helpers::*;
    // use crate::
    use crate::state::GenIxHandler;
    use pinocchio::{instruction::Signer as PSigner, seeds};

    use ephemeral_rollups_pinocchio::{
        consts::{BUFFER, DELEGATION_PROGRAM_ID, DELEGATION_RECORD, MAGIC_CONTEXT_ID},
        pda::{
            delegation_metadata_pda_from_delegated_account,
            delegation_record_pda_from_delegated_account,
        },
    };

    pub const EU_VALIDATOR: Pubkey = pubkey!("MEUGGrYPxKk17hCr7wpT6s8dtNokZj5U2L57vjYMS8e");
    pub struct ReusableState {
        pub rpc_client: RpcClient,
        pub rpc_er_client: RpcClient,
        pub system_program: Pubkey,
        pub account_to_create: (Pubkey, u8),
        pub creator: Keypair,
        pub program_payer: Keypair,
        pub account_to_create2: Option<(Pubkey, u8)>,
        pub creator_account: Pubkey,
        pub owner_program: Pubkey,
        pub buffer_account: Pubkey,
        pub delegation_record: Pubkey,
        pub delegation_metadata: Pubkey,
        pub commit_state_account: Pubkey,
        pub commit_state_record: Pubkey,
        pub undelegate_buffer: Pubkey,
        pub fees_vault: Pubkey,
        pub validator_fees_vault: Pubkey,
    }

    fn setup() -> ReusableState {
        let rpc_client = RpcClient::new(RPC_URL);
        let rpc_er_client = RpcClient::new(RPC_ER_URL);

        let payer = read_keypair_file("dev_wallet.json").expect("Couldn't find wallet file");
        let program_payer = read_keypair_file(
            "/home/eaa/accel_builders/mojo-2/mojo-program/target/deploy/mojo_program-keypair.json",
        )
        .expect("Couldn't find wallet file");

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let combined = encode_packed!(b"bredo", payer.pubkey().as_ref());
        let account_to_create = Pubkey::find_program_address(
            &[&compute_hash(&combined), payer.pubkey().as_ref()],
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

        let undelegate_buffer = Pubkey::find_program_address(
            &[b"undelegate-buffer", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

        let fees_vault = Pubkey::find_program_address(&[b"fees-vault"], &delegation_program_id).0;

        let validator_fees_vault = Pubkey::find_program_address(
            &[b"v-fees-vault", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

        // Derive delegation_metadata PDA: ["delegation-metadata", account_pubkey]
        let delegation_metadata = Pubkey::find_program_address(
            &[b"delegation-metadata", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

        let commit_state_account = Pubkey::find_program_address(
            &[b"commit-state-account", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

        let commit_state_record = Pubkey::find_program_address(
            &[b"commit-state-record", account_to_create.0.as_ref()],
            &delegation_program_id,
        )
        .0;

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
            program_payer: program_payer,
            creator_account,
            owner_program: PROGRAM_ID,
            buffer_account,
            delegation_metadata,
            delegation_record,
            commit_state_account,
            commit_state_record,
            undelegate_buffer,
            fees_vault,
            validator_fees_vault,
        };

        reusable_state
    }

    #[test]
    fn test_create_state_account() {
        let mut state = setup();

        let creator = state.creator;
        let program_payer = state.program_payer;
        let account_to_create = state.account_to_create;
        let system_program = state.system_program;

        let my_state_data = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"bredo", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_state_data.length().to_le_bytes(),
        };

        let create_ix_data = [
            vec![crate::instructions::MojoInstructions::CreateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(), // this was the source of my problems. I added it back and the code worked out.
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
            Some(&creator.pubkey()),
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

    #[test]
    fn test_delegate_state_account() {
        let mut state = setup();
        // let mut state_two = create_ingridients();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;
        let delegate_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

        // Derive the buffer PDA using [BUFFER, creator_account] with our PROGRAM_ID
        let buffer_account =
            Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        let my_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"bredo", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_state_data.length().to_le_bytes(),
        };

        // let mojo_data = state_two.data

        // const MAX_LEN: usize = 128;

        let delegate_ix_data = [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes(),
            my_state_data.to_bytes(),
        ]
        .concat();

        // let delegation_program_id = Pubkey::new_from_array(DELEGATION_PROGRAM_ID);

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
                AccountMeta::new(delegate_id, false),    // system program
                AccountMeta::new(EU_VALIDATOR, false),   // a different Validator for speed
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

    #[test]
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

        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"bredo", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_update_state_data.length().to_le_bytes(),
        };

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
            Some(&creator.pubkey()),
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

        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"bredo", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_update_state_data.length().to_le_bytes(),
        };

        let commit_ix_data = [
            vec![crate::instructions::MojoInstructions::Commit as u8],
            // mojo_data.to_bytes(),
            // my_update_state_data.to_bytes(),
            vec![0, 0, 0],
        ]
        .concat();

        let recent_blockhash = state
            .rpc_er_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");

        let commit_ix = Instruction {
            program_id: owner_program,
            accounts: vec![
                // 0xAbim: used definite ordering for the commit tests.
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(Pubkey::new_from_array(MAGIC_CONTEXT_ID), false),
                AccountMeta::new(creator_account.0, false),
                AccountMeta::new_readonly(Pubkey::new_from_array(MAGIC_PROGRAM_ID), false),
            ],
            // data: vec![1, 0, 0, 0],  // ALLOW_UNDELEGATION_DATA
            data: commit_ix_data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[commit_ix],
            Some(&creator.pubkey()),
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
    // 0xAbim: The test works, but something feels fundamentally wrong here.
    // I tried using all  triks in the playbook, including tweaking the bytes
    // But still couldn't demystify it.
    // Will love us to do some housekeeping on the MagicBlock Repo. There's some deep level shii there. Also some hidden codes that will be quite cool if the docs could just talk about them a bit.
    fn test_commit_and_undelegate_account() {
        let mut state = setup();

        let creator = state.creator;
        let creator_account = state.account_to_create;
        let owner_program = state.owner_program;

        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };

        let combined = encode_packed!(b"bredo", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_update_state_data.length().to_le_bytes(),
        };

        let commit_and_undelegate_ix_data = [
            vec![crate::instructions::MojoInstructions::UndelegateAccount as u8],
            // mojo_data.to_bytes(),
            // my_update_state_data.to_bytes(),
            // owner_program.as_array().to_vec(),
            // creator_account.0.as_array().to_vec(),
            // vec![2, 0, 0, 0, 0, 0, 0]
        ]
        .concat();

        let recent_blockhash = state
            .rpc_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");

        let commit_and_undelegate_ix = Instruction {
            program_id: owner_program,
            // program_id: Pubkey::new_from_array(DELEGATION_PROGRAM_ID),
            accounts: vec![
                // 0xAbim: used definite ordering for the commit tests.
                AccountMeta::new(creator.pubkey(), true),
                // AccountMeta::new(pubkey!("Ew1j4p6jU82qmLFLJe2SVp5ZKoNokMP6J1Bf5LaZ6GyE"), false),
                AccountMeta::new(creator_account.0, false),
                AccountMeta::new(Pubkey::new_from_array(MAGIC_CONTEXT_ID), false),
                // AccountMeta::new(creator_account.0, false),
                AccountMeta::new_readonly(Pubkey::new_from_array(MAGIC_PROGRAM_ID), false),
            ],
            // data: vec![2, 0, 0, 0],  // ALLOW_UNDELEGATION_DATA
            data: commit_and_undelegate_ix_data,
            // data:owner_program.as_array().to_vec()
        };

        let transaction = Transaction::new_signed_with_payer(
            &[commit_and_undelegate_ix],
            Some(&creator.pubkey()),
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

    // #[test]
    fn test_undelegate_account() {
        let mut state = setup();

        let creator = state.creator;
        log!("this passed");
        let creator_account = state.account_to_create;
        let my_update_state_data: MyPosition = MyPosition { x: 24, y: 12 };
        let owner_program = state.owner_program;
        let delegation_record = state.delegation_record;
        let delegation_metadata = state.delegation_metadata;
        let system_program = state.system_program;
        let undelegate_buffer = state.undelegate_buffer;
        let fees_vault = state.fees_vault;
        let validator_fees_vault = state.validator_fees_vault;
        log!("this passed 2");

        // let undelegate_buffer_account =
        //     Pubkey::find_program_address(&[BUFFER, creator_account.0.as_ref()], &PROGRAM_ID).0;

        let commit_state = state.commit_state_account;
        let commit_record = state.commit_state_record;

        let combined = encode_packed!(b"bredo", creator.pubkey().as_ref());
        let digest = compute_hash(&combined);

        let mojo_data = crate::state::GenIxHandler {
            seeds: digest,
            size: my_update_state_data.length().to_le_bytes(),
        };
        log!("this passed 3");

        // const MAX_LEN: usize = 128;

        log!("this passed  here");

        log!("this passed 4 ");

        // let undelegate_ix_data = [
        //     vec![crate::instructions::MojoInstructions::UndelegateAccount as u8],
        //     mojo_data.to_bytes(),
        //     my_update_state_data.to_bytes(),
        // ].concat();

        // The delegation program's commit_and_undelegate discriminator
        // let undelegate_ix_data = vec![2, 0, 0, 0];

        let undelegate_ix_data = [
            vec![crate::instructions::MojoInstructions::UndelegateAccount as u8],
            // mojo_data.to_bytes(),
            // my_update_state_data.to_bytes(),
            // vec![0, 0, 0]
        ]
        .concat();
        log!("this passed nooooo");

        // let normal_fee = pubkey!("7JrkjmZPprHwtuvtuGTXp9hwfGYFAQLnLeFM52kqAgXg");
        // let validator_fee = pubkey!("7EUuuQDZRKar7dUESDynEEKkXity3TeqGdq1TNqefaAn");
        let recent_blockhash = state
            .rpc_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");

        // log!("{}", creator.pubkey());
        let undelegate_ix = Instruction {
            program_id: owner_program,
            // program_id: Pubkey::new_from_array(DELEGATION_PROGRAM_ID),
            // program_id: Pubkey::new_from_array(MAGIC_PROGRAM_ID),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                // AccountMeta::new(creator.pubkey(), false), // delegated account 1
                AccountMeta::new(creator_account.0, false), // delegated account 1
                AccountMeta::new(Pubkey::new_from_array(DELEGATION_PROGRAM_ID), false), // magic context
                // AccountMeta::new_readonly(owner_program, false),
                AccountMeta::new(undelegate_buffer, false),
                // AccountMeta::new(creator.pubkey(), true),
                // AccountMeta::new(EU_VALIDATOR, true),
                // AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(commit_state, false),
                AccountMeta::new(commit_record, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new(EU_VALIDATOR, false),
                AccountMeta::new(fees_vault, false),
                AccountMeta::new(validator_fees_vault, false),
                AccountMeta::new(system_program, false),
                // AccountMeta::new_readonly(Pubkey::new_from_array(ephemeral_rollups_pinocchio::consts::MAGIC_CONTEXT_ID),false,),
                // AccountMeta::new_readonly(Pubkey::new_from_array(MAGIC_PROGRAM_ID), false),
            ],
            // data: vec![2, 0, 0, 0], // ALLOW_UNDELEGATION_DATA
            data: undelegate_ix_data, // ALLOW_UNDELEGATION_DATA
        };

        let transaction = Transaction::new_signed_with_payer(
            &[undelegate_ix],
            Some(&creator.pubkey()),
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
}
