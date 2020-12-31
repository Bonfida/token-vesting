#![cfg(feature = "test-bpf")]
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{signature::Signer, transaction::Transaction, account::Account, signature::Keypair};
use token_vesting::entrypoint::process_instruction;
use token_vesting::instruction::VestingInstruction;
use token_vesting::state::STATE_SIZE;
use spl_associated_token_account::get_associated_token_address;

#[tokio::test]
async fn test_token_vesting() {

    // Create program and test environment
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "token_vesting",
        program_id,
        processor!(process_instruction),
    );
        
    // Add accountslet 
    let mint_address = Pubkey::new_unique();
    program_test.add_account(
        mint_address,
        Account {
            lamports: 5,
            ..Account::default()
        }
    );

    let mut seeds = [42u8; 32];
    let (vesting_account_key, bump) = Pubkey::find_program_address(&[&seeds[..31]], &program_id);
    seeds[31] = bump;
    program_test.add_account(
        vesting_account_key,
        Account {
            lamports: Rent::default().minimum_balance(40),
            data: [0u8;STATE_SIZE].to_vec(),
            owner: program_id,
            ..Account::default()
        });
        
    let vesting_token_account_key = get_associated_token_address(&vesting_account_key, &mint_address);
    program_test.add_account(
        vesting_token_account_key,
        Account {
            lamports: 5,
            ..Account::default()
        },
    );

    let source_token_account_owner_keypair = Keypair::new();
    program_test.add_account(
        source_token_account_owner_keypair.pubkey(),
        Account {
            lamports: 5,
            ..Account::default()
        },
    );
    
    let source_token_account_key = get_associated_token_address(&source_token_account_owner_keypair.pubkey(), &mint_address);
    program_test.add_account(
        source_token_account_key,
        Account {
            lamports: 5,
            owner: source_token_account_owner_keypair.pubkey(),
            ..Account::default()
        },
    );

    let destination_token_account_key = Pubkey::new_unique();
    program_test.add_account(
        destination_token_account_key,
        Account {
            lamports: 5,
            ..Account::default()
        },
    );
    

    // Create instructions
    let create_instruction_data = VestingInstruction::Create{
        seeds: seeds.clone(),
        amount: 2,
        release_height: 0,
        mint_address: mint_address
    }.pack();
    // let unlock_instruction_data = VestingInstruction::Unlock{
    //     seeds: seeds.clone()
    // }.pack();
    

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let create_accounts = vec![
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(mint_address, false),
        AccountMeta::new(vesting_account_key, false),
        AccountMeta::new(vesting_token_account_key, false),
        AccountMeta::new(source_token_account_owner_keypair.pubkey(), true),
        AccountMeta::new(source_token_account_key, false),
        AccountMeta::new(destination_token_account_key, false),
    ];

    // let unlock_accounts = vec![
    //     AccountMeta::new(vesting_account_key, false),
    //     AccountMeta::new(vesting_token_account_key, false),
    //     AccountMeta::new(destination_token_account_key, false),
    // ];

    let instructions = [
        Instruction { program_id: program_id, accounts: create_accounts, data: create_instruction_data },
        // Instruction { program_id: program_id, accounts: unlock_accounts, data: unlock_instruction_data }
        ];

    
    let mut transaction = Transaction::new_with_payer(
        &instructions,
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &source_token_account_owner_keypair], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();
}

// fn intialize_token_account(owner: Pubkey, mint_address: Pubkey){
    
// }