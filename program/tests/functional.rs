#![cfg(feature = "test-bpf")]
use std::str::FromStr;

use solana_program::{hash::Hash,
    pubkey::Pubkey,
    rent::Rent,
    sysvar,
    system_program
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    signature::Signer,
    transaction::Transaction,
    account::Account,
    signature::Keypair,
    system_instruction
};
use token_vesting::{entrypoint::process_instruction, instruction::Schedule};
use token_vesting::instruction::{init, unlock, change_destination, create};
use spl_token::{self, instruction::{initialize_mint, initialize_account, mint_to}};

#[tokio::test]
async fn test_token_vesting() {

    // Create program and test environment
    let program_id = Pubkey::from_str("VestingbGKPFXCWuBvfkegQfZyiNwAJb9Ss623VQ5DA").unwrap();
    let mint_authority = Keypair::new();
    let mint = Keypair::new();

    let source_account = Keypair::new();
    let source_token_account = Keypair::new();

    let destination_account = Keypair::new();
    let destination_token_account = Keypair::new();

    let new_destination_account = Keypair::new();
    let new_destination_token_account = Keypair::new();

    let mut seeds = [42u8; 32];
    let (vesting_account_key, bump) = Pubkey::find_program_address(&[&seeds[..31]], &program_id);
    seeds[31] = bump;
    let vesting_token_account = Keypair::new();
    
    let mut program_test = ProgramTest::new(
        "token_vesting",
        program_id,
        processor!(process_instruction),
    );

    // Add accounts         
    program_test.add_account(
        source_account.pubkey(),
        Account {
            lamports: 5000000,
            ..Account::default()
        },
    );

    // Start and process transactions on the test network
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Initialize the vesting program account
    let init_instruction = [init(
        &system_program::id(),
        &program_id,
        &payer.pubkey(),
        &vesting_account_key,
        seeds,
        1
    ).unwrap()
    ];
    let mut init_transaction = Transaction::new_with_payer(
        &init_instruction,
        Some(&payer.pubkey()),
    );
    init_transaction.partial_sign(
        &[&payer],
        recent_blockhash
    );
    banks_client.process_transaction(init_transaction).await.unwrap();


    // Initialize the token accounts
    banks_client.process_transaction(mint_init_transaction(
        &payer,
        &mint,
        &mint_authority,
        recent_blockhash
    )).await.unwrap();

    banks_client.process_transaction(
        create_token_account(&payer, &mint, recent_blockhash, &source_token_account, &source_account.pubkey())
    ).await.unwrap();
    banks_client.process_transaction(
        create_token_account(&payer, &mint, recent_blockhash, &vesting_token_account, &vesting_account_key)
    ).await.unwrap();
    banks_client.process_transaction(
        create_token_account(&payer, &mint, recent_blockhash, &destination_token_account, &destination_account.pubkey())
    ).await.unwrap();
    banks_client.process_transaction(
        create_token_account(&payer, &mint, recent_blockhash, &new_destination_token_account, &new_destination_account.pubkey())
    ).await.unwrap();


    // Create and process the vesting transactions
    let setup_instructions = [
        mint_to(
            &spl_token::id(), 
            &mint.pubkey(), 
            &source_token_account.pubkey(), 
            &mint_authority.pubkey(), 
            &[], 
            100
        ).unwrap()
    ];

    let schedules = vec![
        Schedule {amount: 20, release_height: 0},
        Schedule {amount: 20, release_height: 2},
        Schedule {amount: 20, release_height: 5}
    ];

    let test_instructions = [
        create(
            &program_id,
            &spl_token::id(),
            &vesting_account_key,
            &vesting_token_account.pubkey(),
            &source_account.pubkey(),
            &source_token_account.pubkey(),
            &destination_token_account.pubkey(),
            &mint.pubkey(),
            schedules,
            seeds.clone()
        ).unwrap(),
        unlock(
            &program_id,
            &spl_token::id(),
            &sysvar::clock::id(),
            &vesting_account_key,
            &vesting_token_account.pubkey(),
            &destination_token_account.pubkey(),
            seeds.clone()
        ).unwrap()
    ];

    let change_destination_instructions = [
        change_destination(
            &program_id,
            &vesting_account_key,
            &destination_account.pubkey(),
            &destination_token_account.pubkey(),
            &new_destination_token_account.pubkey(),
            seeds.clone()
        ).unwrap()
    ];
    
    // Process transaction on test network
    let mut setup_transaction = Transaction::new_with_payer(
        &setup_instructions,
        Some(&payer.pubkey()),
    );
    setup_transaction.partial_sign(
        &[
            &payer,
            &mint_authority
            ], 
        recent_blockhash
    );
    
    banks_client.process_transaction(setup_transaction).await.unwrap();

    // Process transaction on test network
    let mut test_transaction = Transaction::new_with_payer(
        &test_instructions,
        Some(&payer.pubkey()),
    );
    test_transaction.partial_sign(
        &[
            &payer,
            &source_account
            ], 
        recent_blockhash
    );
    
    banks_client.process_transaction(test_transaction).await.unwrap();
    
    let mut change_destination_transaction = Transaction::new_with_payer(
        &change_destination_instructions, 
        Some(&payer.pubkey())
    );

    change_destination_transaction.partial_sign(
        &[
            &payer,
            &destination_account
        ], 
        recent_blockhash
    );

    banks_client.process_transaction(change_destination_transaction).await.unwrap();
    
}

fn mint_init_transaction(
    payer: &Keypair, 
    mint:&Keypair, 
    mint_authority: &Keypair, 
    recent_blockhash: Hash) -> Transaction{
    let instructions = [
        system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            Rent::default().minimum_balance(82),
            82,
            &spl_token::id()
    
        ),
        initialize_mint(
            &spl_token::id(), 
            &mint.pubkey(), 
            &mint_authority.pubkey(),
            None, 
            0
        ).unwrap(),
    ];
    let mut transaction = Transaction::new_with_payer(
        &instructions,
        Some(&payer.pubkey()),
    );
    transaction.partial_sign(
        &[
            payer,
            mint
            ], 
        recent_blockhash
    );
    transaction
}

fn create_token_account(
    payer: &Keypair, 
    mint:&Keypair, 
    recent_blockhash: Hash,
    token_account:&Keypair,
    token_account_owner: &Pubkey
) -> Transaction {
    let instructions = [
        system_instruction::create_account(
            &payer.pubkey(),
            &token_account.pubkey(),
            Rent::default().minimum_balance(165),
            165,
            &spl_token::id()
        ),
        initialize_account(
            &spl_token::id(), 
            &token_account.pubkey(), 
            &mint.pubkey(), 
            token_account_owner
        ).unwrap()
   ];
   let mut transaction = Transaction::new_with_payer(
    &instructions,
    Some(&payer.pubkey()),
    );
    transaction.partial_sign(
        &[
            payer,
            token_account
            ], 
        recent_blockhash
    );
    transaction
}