use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    msg
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{signature::Signer, transaction::Transaction, account::Account};
use token_vesting::entrypoint::process_instruction;
use token_vesting::instruction::VestingInstruction;

#[tokio::test]
async fn test_token_vesting() {
    // TODO create key pair for signing
    let program_id = Pubkey::new_unique();
    let source_pubkey = Pubkey::new_unique();
    let destination_pubkey = Pubkey::new_unique();
    let transaction_pubkey = Pubkey::create_program_address(&[&[42, 42]], &program_id).unwrap();

    let mut program_test = ProgramTest::new(
        "token_vesting",
        program_id,
        processor!(process_instruction),
    );
    
    // program_test.add_program("token_vesting", program_id, None);

    let source_account = Account {
        lamports: 5,
        owner: program_id, // Can only withdraw lamports from accounts owned by the program
        ..Account::default()
    };

    // msg!("Account : {:?}", &source_account);

    program_test.add_account(
        source_pubkey,
        source_account,
    );
    program_test.add_account(
        destination_pubkey,
        Account {
            lamports: 5,
            ..Account::default()
        },
    );

    
    let instruction_data = VestingInstruction::Lock{
        amount: 5,
        release_height: 0
    }.pack();
    
    msg!("Packed instruction data: {:?}", instruction_data);

    let accounts = vec![
        AccountMeta::new(program_id, false),
        AccountMeta::new(transaction_pubkey, false),
        AccountMeta::new(source_pubkey, false),
        AccountMeta::new(destination_pubkey, false),
    ];

    let instruction = Instruction { program_id: program_id, accounts: accounts, data: instruction_data };

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();
}