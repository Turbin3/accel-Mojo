use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_test::{ProgramTest, tokio};
use solana_signer::Signer;
use solana_pubkey::Pubkey;
use solana_program::{bpf_loader, rent::Rent};
use solana_transaction::Transaction;
// use ;


use crate::tests::tests_for_er::MyPosition;
use crate::{program, state::transaction_handler::TransactionHandler};
use crate::instructions::*;

pub const PROGRAM: Pubkey = Pubkey::new_from_array(ID);

#[tokio::test]
async fn delegate_account_success() {
    let mut test_one = ProgramTest::new("delegate_account", PROGRAM, None);
    test_one.prefer_bpf(true);

    // Setting up the delegation Program
    let data = read_file("tests/fixtures/delegate.so");
    test_one.add_account(
        ephemeral_rollups_pinocchio::ID.into(),
        Account {
            lamports: Rent::default().minimum_balance(data.len()).max(1),
            data,
            owner: bpf_loader::id(),
            executable: true,
            rent_epoch: 0
        },
    );

    let mut context = test_one.start_with_context().await;
    let payer = context.payer.pubkey();

    // Deriving the PDAs for our program and other accounts
    // ........ 
    let (creator_account, _bump) = Pubkey::find_program_address(
            &[
                &[0u8; 8],
                b"fundrais",
                env.payer.pubkey().as_ref(),
                &[0u8; 32],
                &[0u8; 32],
            ],
            &PROGRAM_ID,
        );

        // Create account first
    let creator_account_data = MyPosition::new(24, 12);
    let account_size = creator_account_data.length() as u64;
    let mut mojo_data = TransactionHandler::new(account_size.to_le_bytes());

    mojo_data
            .fill_first(&[0u8; 8])
            .fill_second(b"fundrais".try_into().unwrap())
            .fill_third(payer.pubkey().as_ref().try_into().unwrap())
            .fill_fourth(&[0u8; 32])  
            .fill_fifth(&[0u8; 32]);
    

    // Derive the required PDAs for the successful ix
    let (buffer_pda, _) = Pubkey::find_program_address(
        &[b"buffer", creator_account.as_ref()],
        &program::ID.into(),
    );
    let (delegation_record, _) = Pubkey::find_program_address(
        &["delegation", creator_account.as_ref()],
        &program::DELEGATION_PROGRAM_ID.into(),
    );
    let (delegation_metadata_pda, _) = Pubkey::find_program_address(
        &["delegation-metadata", pdas.creator_account.as_ref()],
        &program::DELEGATION_PROGRAM_ID.into(),
    );

    let delegate_ix = Instruction{
        program_id: PROGRAM,
        accounts: vec![
            AccountMeta::new_readonly(payer, true), 
            AccountMeta::new(creator_account, false), 
            AccountMeta::new_readonly(PROGRAM, false), 
            AccountMeta::new(buffer_pda, false), 
            AccountMeta::new(delegation_record_pda, false), 
            AccountMeta::new(delegation_metadata_pda, false), 
            AccountMeta::new_readonly(ephemeral_rollups_pinocchio::ID.into(), false), 
            AccountMeta::new_readonly(solana_system_interface::program::ID, false), 
        ],
        data: [
            vec![crate::instructions::MojoInstructions::DelegateAccount as u8],
            mojo_data.to_bytes()
            ].concat()
    };

    let tx = Transaction::new_signed_with_payer(
        &[delegate_ix],
        Some(&payer),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

}