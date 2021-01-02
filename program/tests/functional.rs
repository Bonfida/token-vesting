#![cfg(feature = "test-bpf")]
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    sysvar};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    signature::Signer,
    transaction::Transaction,
    account::Account,
    signature::Keypair,
    system_instruction
};
use token_vesting::entrypoint::process_instruction;
use token_vesting::instruction::VestingInstruction;
use token_vesting::state::STATE_SIZE;
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::{initialize_mint, initialize_account};


#[tokio::test]
async fn test_token_vesting() {

    // Create program and test environment
    let program_id = Pubkey::new_unique();
    let mint_address_keypair = Keypair::new();
    let mut seeds = [42u8; 32];
    let (vesting_account_key, bump) = Pubkey::find_program_address(&[&seeds[..31]], &program_id);
    seeds[31] = bump;
    let vesting_token_account_key = Pubkey::new_unique();
    let source_token_account_owner_keypair = Keypair::new();
    let source_token_account_key = get_associated_token_address(
        &source_token_account_owner_keypair.pubkey(),
        &mint_address_keypair.pubkey()
    );
    let destination_token_account_owner_keypair = Keypair::new();
    let destination_token_account_key = Pubkey::new_unique();
    let new_destination_token_account_key = Pubkey::new_unique();

    let mut program_test = ProgramTest::new(
        "token_vesting",
        program_id,
        processor!(process_instruction),
    );
        

    // Add accounts 
    program_test.add_account(
        vesting_account_key,
        Account {
            lamports: Rent::default().minimum_balance(STATE_SIZE),
            data: [0u8;STATE_SIZE].to_vec(),
            owner: program_id,
            ..Account::default()
        });
        
    program_test.add_account(
        source_token_account_owner_keypair.pubkey(),
        Account {
            lamports: 5,
            ..Account::default()
        },
    );


    // Create vesting instructions
    let create_instruction_data = VestingInstruction::Create{
        seeds: seeds.clone(),
        amount: 2,
        release_height: 0,
        mint_address: mint_address_keypair.pubkey()
    }.pack();
    let _change_destination_instruction_data = VestingInstruction::ChangeDestination{
        seeds: seeds.clone()
    }.pack();
    let unlock_instruction_data = VestingInstruction::Unlock{
        seeds: seeds.clone()
    }.pack();


    // Create associated accountmetas
    let create_accounts = vec![
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(mint_address_keypair.pubkey(), false),
        AccountMeta::new(vesting_account_key, false),
        AccountMeta::new(vesting_token_account_key, false),
        AccountMeta::new(source_token_account_owner_keypair.pubkey(), false),
        AccountMeta::new(source_token_account_key, false),
        AccountMeta::new(destination_token_account_key, false),
    ];

    let _change_destination_accounts = vec![
        AccountMeta::new(vesting_account_key, false),
        AccountMeta::new(destination_token_account_owner_keypair.pubkey(), true),
        AccountMeta::new(new_destination_token_account_key, false),
    ];

    let unlock_accounts = vec![
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(vesting_account_key, false),
        AccountMeta::new(vesting_token_account_key, false),
        AccountMeta::new(destination_token_account_key, false),
    ];


    // Start the test network
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;


    // Package the instructions
    let instructions = [
        // Create Mint account. This instruction must be followed by initialize (see spl_token::Instruction)
        system_instruction::create_account(&payer.pubkey(), &mint_address_keypair.pubkey(), 1000, 0, &spl_token::id()),

        // Initialize mint account
        initialize_mint(&spl_token::id(), &mint_address_keypair.pubkey(), &program_id, None, 1).unwrap(),

        // Create token accounts
        initialize_account(
            &spl_token::id(),
            &vesting_token_account_key,
            &mint_address_keypair.pubkey(),
            &vesting_account_key
        ).unwrap(),
        initialize_account(&spl_token::id(),
            &source_token_account_key,
            &mint_address_keypair.pubkey(),
            &source_token_account_owner_keypair.pubkey()
        ).unwrap(),
        initialize_account(&spl_token::id(),
            &destination_token_account_key,
            &mint_address_keypair.pubkey(),
            &destination_token_account_owner_keypair.pubkey()
        ).unwrap(),

        // Create, change destination and unlock vesting contract
        Instruction { program_id, accounts: create_accounts, data: create_instruction_data },
        // Instruction { program_id, accounts: change_destination, data: change_destination_instruction_data },
        Instruction { program_id, accounts: unlock_accounts, data: unlock_instruction_data }
    ];

    
    // Process transaction on test network
    let mut transaction = Transaction::new_with_payer(
        &instructions,
        Some(&payer.pubkey()),
    );
    transaction.partial_sign(&[&payer, &mint_address_keypair], recent_blockhash);
    
    banks_client.process_transaction(transaction).await.unwrap();
}