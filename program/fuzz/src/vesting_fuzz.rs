use token_vesting::instruction;

use spl_token::error::TokenError;

use std::str::FromStr;
use spl_associated_token_account::{get_associated_token_address, create_associated_token_account};

use solana_program::{hash::Hash,
    pubkey::Pubkey,
    rent::Rent,
    sysvar,
    system_program
};
use futures::executor::block_on;
use honggfuzz::fuzz;
use solana_program_test::{BanksClient, ProgramTest, processor};
use solana_sdk::{
    signature::Signer,
    transaction::Transaction,
    account::Account,
    signature::Keypair,
    system_instruction
};
use arbitrary::{Arbitrary, Error};
use std::{collections::{HashMap, HashSet}, num};
use token_vesting::{error::VestingError, processor::Processor, instruction::{Schedule, VestingInstruction}};
use token_vesting::instruction::{init, unlock, change_destination, create};

struct TokenVestingEnv {
    system_program_id: Pubkey,
    token_program_id: Pubkey,
    sysvarclock_program_id: Pubkey,
    vesting_program_id: Pubkey
}

#[derive(Debug, Arbitrary, Clone)]
struct FuzzInstruction {
    vesting_account_key: AccountId,           
    vesting_token_account_key: AccountId,
    source_token_account_owner_key: AccountId,
    source_token_account_key: AccountId,
    destination_token_owner_key: AccountId,
    destination_token_key: AccountId,
    new_destination_token_key: AccountId,
    mint_key: AccountId,
    schedules: Vec<Schedule>,
    payer_key: AccountId,
    vesting_program_account: AccountId,
    seeds:[u8; 32],
    number_of_schedules: u8, // TODO limit everywhere
    instruction: instruction::VestingInstruction,
    // This flag decides wether the instruction will be executed with inputs that should
    // not provoke any errors. (The accounts and contracts will be set up before if needed)
    correct_inputs: bool
}
/// Use u8 as an account id to simplify the address space and re-use accounts
/// more often.
type AccountId = u8; 


#[tokio::main]
async fn main() {

    // Set up the fixed test environment
    let token_vesting_testenv = TokenVestingEnv {
        system_program_id: system_program::id(),
        sysvarclock_program_id: sysvar::clock::id(),
        token_program_id: spl_token::id(),
        vesting_program_id: Pubkey::from_str("VestingbGKPFXCWuBvfkegQfZyiNwAJb9Ss623VQ5DA").unwrap(),
    };

    loop {
        // Initialize and start the test network
        let program_test = ProgramTest::new(
            "token_vesting",
            system_program::id(),
            processor!(Processor::process_instruction),
        );
        let (banks_client, banks_payer, _) = block_on(program_test.start());

        fuzz!(|fuzz_instructions: Vec<FuzzInstruction>| {
            block_on(run_fuzz_instructions(&token_vesting_testenv, &banks_client, fuzz_instructions, &banks_payer));
        });
    }
}


async fn run_fuzz_instructions(
    token_vesting_testenv: &TokenVestingEnv,
    banks_client: &BanksClient,
    fuzz_instructions: Vec<FuzzInstruction>,
    banks_payer: &Keypair
) {
    // keep track of all accounts
    // let mut vesting_account_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    // let mut vesting_token_account_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut source_token_account_owner_keys: HashMap<AccountId, Keypair> = HashMap::new();
    let mut source_token_account_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut destination_token_owner_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut destination_token_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut new_destination_token_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut mint_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut payer_keys: HashMap<AccountId, Keypair> = HashMap::new();
    

    for fuzz_instruction in fuzz_instructions {
        // vesting_account_keys
        //     .entry(fuzz_instruction.vesting_account_key)
        //     .or_insert_with(|| Pubkey::new_unique());
        // vesting_token_account_keys
        //     .entry(fuzz_instruction.vesting_token_account_key)
        //     .or_insert_with(|| Pubkey::new_unique());
    
        // Add accounts         
        source_token_account_owner_keys
            .entry(fuzz_instruction.source_token_account_owner_key)
            .or_insert_with(|| Keypair::new());
        source_token_account_keys
            .entry(fuzz_instruction.source_token_account_key)
            .or_insert_with(|| Pubkey::new_unique());
        destination_token_owner_keys
            .entry(fuzz_instruction.destination_token_owner_key)
            .or_insert_with(|| Pubkey::new_unique());
        destination_token_keys
            .entry(fuzz_instruction.destination_token_key)
            .or_insert_with(|| Pubkey::new_unique());
        new_destination_token_keys
            .entry(fuzz_instruction.new_destination_token_key)
            .or_insert_with(|| Pubkey::new_unique());
        mint_keys
            .entry(fuzz_instruction.mint_key)
            .or_insert_with(|| Pubkey::new_unique());
        payer_keys
            .entry(fuzz_instruction.payer_key)
            .or_insert_with(|| Keypair::new());

        // Update the blockhash
        let recent_blockhash = 
            banks_client.to_owned()
            .get_recent_blockhash()
            .await.unwrap();

        run_fuzz_instruction(
            &token_vesting_testenv,
            banks_client.to_owned(),
            recent_blockhash,
            &fuzz_instruction,
            &banks_payer,
            mint_keys.get(&fuzz_instruction.mint_key).unwrap(),
            source_token_account_owner_keys.get(
                &fuzz_instruction.source_token_account_owner_key
            ).unwrap(),
            source_token_account_keys.get(
                &fuzz_instruction.source_token_account_key
            ).unwrap(),
            destination_token_owner_keys.get(
                &fuzz_instruction.destination_token_owner_key
            ).unwrap(),
            destination_token_keys.get(
                &fuzz_instruction.destination_token_key
            ).unwrap(),
            new_destination_token_keys.get(
                &fuzz_instruction.new_destination_token_key
            ).unwrap(),
            payer_keys.get(&fuzz_instruction.payer_key).unwrap()
        ).await;
    }
}


async fn run_fuzz_instruction(
    token_vesting_testenv: &TokenVestingEnv,
    banks_client: BanksClient,
    recent_blockhash: Hash,
    fuzz_instruction: &FuzzInstruction,
    banks_payer: &Keypair,
    mint_key: &Pubkey,   // TODO use the fuzzinstruction data
    source_token_account_owner_key: &Keypair,
    source_token_account_key: &Pubkey,
    destination_token_owner_key: &Pubkey,
    destination_token_key: &Pubkey,
    new_destination_token_key: &Pubkey,
    payer_key: &Keypair
) {

    // Execute the fuzzing in a more restrained way in order to go deeper into the program branches
    if fuzz_instruction.correct_inputs {

        let mut correct_seeds = fuzz_instruction.seeds;
        let (correct_vesting_account_key, bump) = Pubkey::find_program_address(
            &[&correct_seeds[..31]],
            &token_vesting_testenv.vesting_program_id
        );
        correct_seeds[31] = bump;
        let correct_vesting_token_key = get_associated_token_address(
            &correct_vesting_account_key,
            &mint_key
        );

        let result = match fuzz_instruction {

            FuzzInstruction {
                instruction: VestingInstruction::Init{ .. },
                ..
            } => {
                // Initialize the vesting program account
                let init_instruction = [init(
                    &token_vesting_testenv.system_program_id,
                    &token_vesting_testenv.vesting_program_id,
                    &banks_payer.pubkey(),
                    &correct_vesting_account_key,
                    correct_seeds,
                    fuzz_instruction.number_of_schedules as u64
                ).unwrap()
                ];
                let mut init_transaction = Transaction::new_with_payer(
                    &init_instruction,
                    Some(&banks_payer.pubkey()),
                );
                init_transaction.partial_sign(
                    &[banks_payer],
                    recent_blockhash
                );
                banks_client.to_owned().process_transaction(init_transaction).await.unwrap();
            },
            FuzzInstruction {
                instruction: VestingInstruction::Create {
                    mint_address,
                    destination_token_address,
                    schedules,
                    ..
                },
                ..
            } => {
                // Initialize the vesting program account
                let init_instruction = init(
                    &token_vesting_testenv.system_program_id,
                    &token_vesting_testenv.vesting_program_id,
                    &banks_payer.pubkey(),
                    &correct_vesting_account_key,
                    correct_seeds,
                    fuzz_instruction.number_of_schedules as u64
                ).unwrap();

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

                // Initialize the vesting program account 
                // TODO fuzz token amounts
                let create_instruction = create(
                    &token_vesting_testenv.vesting_program_id,
                    &token_vesting_testenv.token_program_id,
                    &correct_vesting_account_key,
                    &correct_vesting_token_key,
                    &source_token_account_owner_key.pubkey(),
                    source_token_account_key,
                    destination_token_key,
                    mint_address,
                    schedules.clone().to_vec(),
                    correct_seeds,
                ).unwrap();

                let mut transaction = Transaction::new_with_payer(
                    &[init_instruction, create_instruction],
                    Some(&banks_payer.pubkey()),
                );
                transaction.partial_sign(
                    &[banks_payer],
                    recent_blockhash
                );
                banks_client.to_owned().process_transaction(transaction).await.unwrap();
            },
            FuzzInstruction {
                instruction: VestingInstruction::Unlock{ .. },
                ..
            } => {
                // Initialize the vesting program account
                let init_instruction = [unlock(
                    &token_vesting_testenv.vesting_program_id,
                    &token_vesting_testenv.token_program_id,
                    &token_vesting_testenv.sysvarclock_program_id,
                    &correct_vesting_account_key,
                    &correct_vesting_token_key,
                    destination_token_key,
                    correct_seeds
                ).unwrap()
                ];
                let mut init_transaction = Transaction::new_with_payer(
                    &init_instruction,
                    Some(&banks_payer.pubkey()),
                );
                init_transaction.partial_sign(
                    &[banks_payer],
                    recent_blockhash
                );
                banks_client.to_owned().process_transaction(init_transaction).await.unwrap();
            },
            FuzzInstruction {
                instruction: VestingInstruction::ChangeDestination{ .. },
                ..
            } => {
                // Initialize the vesting program account
                let init_instruction = [change_destination(
                    &token_vesting_testenv.vesting_program_id,
                    &correct_vesting_account_key,
                    &destination_token_owner_key,
                    &destination_token_key,
                    new_destination_token_key,
                    correct_seeds
                ).unwrap()
                ];
                let mut init_transaction = Transaction::new_with_payer(
                    &init_instruction,
                    Some(&banks_payer.pubkey()),
                );
                init_transaction.partial_sign(
                    &[banks_payer],
                    recent_blockhash
                );
                banks_client.to_owned().process_transaction(init_transaction).await.unwrap();
            }
        };

    // Execute a random input fuzzing
    } else {
        let result = match fuzz_instruction {
            FuzzInstruction {
                instruction: VestingInstruction::Init{ seeds, number_of_schedules},
                ..
            } => {
                // Initialize the vesting program account
                let init_instruction = [init(
                    &token_vesting_testenv.system_program_id,
                    &token_vesting_testenv.vesting_program_id,
                    &payer_key.pubkey(),
                    &vesting_account_key,
                    *seeds,
                    *number_of_schedules
                ).unwrap()
                ];
                let mut init_transaction = Transaction::new_with_payer(
                    &init_instruction,
                    Some(&payer_key.pubkey()),
                );
                init_transaction.partial_sign(
                    &[payer_key],
                    recent_blockhash
                );
                banks_client.to_owned().process_transaction(init_transaction).await.unwrap();
            },
            FuzzInstruction {
                instruction: VestingInstruction::Create {
                    seeds,
                    mint_address,
                    destination_token_address,
                    schedules
                },
                ..
            } => {
                // Initialize the vesting program account
                let create_instruction = [create(
                    &token_vesting_testenv.vesting_program_id,
                    &token_vesting_testenv.token_program_id,
                    &vesting_account_key,
                    &vesting_token_key,
                    &source_token_account_owner_key.pubkey(),
                    source_token_account_key,
                    destination_token_key,
                    mint_address,
                    schedules.clone().to_vec(),
                    *seeds,
                ).unwrap()
                ];
                let mut init_transaction = Transaction::new_with_payer(
                    &create_instruction,
                    Some(&payer_key.pubkey()),
                );
                init_transaction.partial_sign(
                    &[payer_key],
                    recent_blockhash
                );
                banks_client.to_owned().process_transaction(init_transaction).await.unwrap();
            },
            FuzzInstruction {
                instruction: VestingInstruction::Unlock{ seeds },
                ..
            } => {
                // Initialize the vesting program account
                let init_instruction = [unlock(
                    &token_vesting_testenv.vesting_program_id,
                    &token_vesting_testenv.token_program_id,
                    &token_vesting_testenv.sysvarclock_program_id,
                    &vesting_account_key,
                    &vesting_token_key,
                    destination_token_key,
                    *seeds
                ).unwrap()
                ];
                let mut init_transaction = Transaction::new_with_payer(
                    &init_instruction,
                    Some(&payer_key.pubkey()),
                );
                init_transaction.partial_sign(
                    &[payer_key],
                    recent_blockhash
                );
                banks_client.to_owned().process_transaction(init_transaction).await.unwrap();
            },
            FuzzInstruction {
                instruction: VestingInstruction::ChangeDestination{ seeds },
                ..
            } => {
                // Initialize the vesting program account
                let init_instruction = [change_destination(
                    &token_vesting_testenv.vesting_program_id,
                    &vesting_account_key,
                    &destination_token_owner_key,
                    &destination_token_key,
                    new_destination_token_key,
                    *seeds
                ).unwrap()
                ];
                let mut init_transaction = Transaction::new_with_payer(
                    &init_instruction,
                    Some(&payer_key.pubkey()),
                );
                init_transaction.partial_sign(
                    &[payer_key],
                    recent_blockhash
                );
                banks_client.to_owned().process_transaction(init_transaction).await.unwrap();
            }
        };
    }
    // result
    //     .map_err(|e| {
    //         match e {

    //         }
    //         if !(e == VestingError::InvalidInstruction.into())
    //         {
    //             Err(e).unwrap()
    //         }
    //     })
    //     .ok();
}
