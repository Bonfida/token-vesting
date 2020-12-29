use solana_program::{instruction::{AccountMeta, Instruction}, msg, pubkey::Pubkey, rent::Rent, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{signature::Signer, transaction::Transaction, account::Account};
use token_vesting::entrypoint::process_instruction;
use token_vesting::instruction::VestingInstruction;

#[tokio::test]
async fn test_token_vesting() {
    // TODO create key pair for signing
    let program_id = Pubkey::new_unique();
    let destination_pubkey = Pubkey::new_unique();
    let mut seeds = [42u8; 32];

    let (transaction_pubkey, bump) = Pubkey::find_program_address(&[&seeds[..31]], &program_id);
    seeds[31] = bump;


    let mut program_test = ProgramTest::new(
        "token_vesting",
        program_id,
        processor!(process_instruction),
    );
    
    // program_test.add_program("token_vesting", program_id, None);

    program_test.add_account(
        destination_pubkey,
        Account {
            lamports: 5,
            ..Account::default()
        },
    );

    program_test.add_account(
        transaction_pubkey, 
        Account {
            lamports: Rent::default().minimum_balance(40),
            ..Account::default()
        });

    
    let init_instruction_data = VestingInstruction::Init{
        seeds: seeds.clone()
    }.pack();
    
    let lock_instruction_data = VestingInstruction::Lock{
        seeds: seeds.clone(),
        amount: 5,
        release_height: 0
    }.pack();
    
    msg!("Packed instruction data: {:?}", lock_instruction_data);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let lock_accounts = vec![
        AccountMeta::new(program_id, false),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new(transaction_pubkey, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(destination_pubkey, false),
    ];

    let init_accounts = vec![
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new(transaction_pubkey, false),
    ];

    let instructions = [
        Instruction { program_id: program_id, accounts: init_accounts, data: init_instruction_data },
        Instruction { program_id: program_id, accounts: lock_accounts, data: lock_instruction_data }
        ];

    
    let mut transaction = Transaction::new_with_payer(
        &instructions,
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();
}