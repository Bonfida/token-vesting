use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{signature::Signer, transaction::Transaction, account::Account};
use token_vesting::processor::Processor;
use token_vesting::instruction::VestingInstruction;
use std::str::FromStr;

#[tokio::test]
async fn test_token_vesting() {

    // TODO create key pair for signing
    let program_id = Pubkey::new_unique();
    let source_pubkey = Pubkey::new_unique();
    let destination_pubkey = Pubkey::new_unique();

    let mut program_test = ProgramTest::new(
        "token_vesting",
        program_id,
        processor!(Processor::process_instruction),
    );
    
    // program_test.add_program("token_vesting", program_id, None);

    program_test.add_account(
        source_pubkey,
        Account {
            lamports: 5,
            owner: program_id, // Can only withdraw lamports from accounts owned by the program
            ..Account::default()
        },
    );
    program_test.add_account(
        destination_pubkey,
        Account {
            lamports: 5,
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test
    .start()
    .await;

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new(
            program_id,
            &VestingInstruction::Lock{
                amount: 5,
                release_height: 0
            }.pack(),
            vec![
                AccountMeta::new(source_pubkey, false),
                AccountMeta::new(destination_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();
    // let mut transaction = Transaction::new_with_payer(
    //     &[Instruction::new(
    //         program_id,
    //         &(),
    //         vec![AccountMeta::new(sysvar::clock::id(), false)],
    //     )],
    //     Some(&payer.pubkey()),
    // );
    // transaction.sign(&[&payer], recent_blockhash);
    // banks_client.process_transaction(transaction).await.unwrap();
}