use token_vesting::instruction;
use spl_token::instruction::{initialize_mint, mint_to};

use std::{convert::TryInto, str::FromStr};
use spl_associated_token_account::{get_associated_token_address, create_associated_token_account};

use solana_program::{hash::Hash, instruction::Instruction, pubkey::Pubkey, rent::Rent, system_program, sysvar};
use futures::executor::block_on;
use honggfuzz::fuzz;
use solana_program_test::{BanksClient, ProgramTest, processor};
use solana_sdk::{signature::Keypair, signature::Signer, system_instruction, transaction::Transaction, transport::TransportError};
use arbitrary::Arbitrary;
use std::collections::HashMap;
use token_vesting::{instruction::{Schedule, VestingInstruction}, processor::Processor};
use token_vesting::instruction::{init, unlock, change_destination, create};
use solana_sdk::{account::Account, instruction::InstructionError, transaction::TransactionError};
struct TokenVestingEnv {
    system_program_id: Pubkey,
    token_program_id: Pubkey,
    sysvarclock_program_id: Pubkey,
    rent_program_id: Pubkey,
    vesting_program_id: Pubkey,
    mint_authority: Keypair
}

#[derive(Debug, Arbitrary, Clone)]
struct FuzzInstruction {
    vesting_account_key: AccountId,
    vesting_token_account_key: AccountId,
    source_token_account_owner_key: AccountId,
    source_token_account_key: AccountId,
    source_token_amount: u64,
    destination_token_owner_key: AccountId,
    destination_token_key: AccountId,
    new_destination_token_key: AccountId,
    mint_key: AccountId,
    schedules: Vec<Schedule>,
    payer_key: AccountId,
    vesting_program_account: AccountId,
    seeds:[u8; 32],
    number_of_schedules: u8,
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
        rent_program_id: sysvar::rent::id(),
        token_program_id: spl_token::id(),
        vesting_program_id: Pubkey::from_str("VestingbGKPFXCWuBvfkegQfZyiNwAJb9Ss623VQ5DA").unwrap(),
        mint_authority: Keypair::new()
    };

    loop {
        // Initialize and start the test network
        let mut program_test = ProgramTest::new(
            "token_vesting",
            token_vesting_testenv.vesting_program_id,
            processor!(Processor::process_instruction),
        );

        program_test.add_account(token_vesting_testenv.mint_authority.pubkey(), Account {
            lamports: u32::MAX as u64,
            ..Account::default()
        });
        let (mut banks_client, banks_payer, recent_blockhash) = program_test.start().await;

        fuzz!(|fuzz_instructions: Vec<FuzzInstruction>| {
            block_on(run_fuzz_instructions(&token_vesting_testenv, &mut banks_client, fuzz_instructions, &banks_payer, recent_blockhash));
        });
    }
}


async fn run_fuzz_instructions(
    token_vesting_testenv: &TokenVestingEnv,
    banks_client: &mut BanksClient,
    fuzz_instructions: Vec<FuzzInstruction>,
    correct_payer: &Keypair,
    recent_blockhash: Hash
) {
    // keep track of all accounts
    let mut vesting_account_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut vesting_token_account_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut source_token_account_owner_keys: HashMap<AccountId, Keypair> = HashMap::new();
    let mut destination_token_owner_keys: HashMap<AccountId, Keypair> = HashMap::new();
    let mut destination_token_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut new_destination_token_keys: HashMap<AccountId, Pubkey> = HashMap::new();
    let mut mint_keys: HashMap<AccountId, Keypair> = HashMap::new();
    let mut payer_keys: HashMap<AccountId, Keypair> = HashMap::new();

    let mut global_output_instructions = vec![];
    let mut global_signer_keys = vec![];

    for fuzz_instruction in fuzz_instructions {

        // Add accounts
        vesting_account_keys
            .entry(fuzz_instruction.vesting_account_key)
            .or_insert_with(|| Pubkey::new_unique());
        vesting_token_account_keys
            .entry(fuzz_instruction.vesting_token_account_key)
            .or_insert_with(|| Pubkey::new_unique());
        source_token_account_owner_keys
            .entry(fuzz_instruction.source_token_account_owner_key)
            .or_insert_with(|| Keypair::new());
        destination_token_owner_keys
            .entry(fuzz_instruction.destination_token_owner_key)
            .or_insert_with(|| Keypair::new());
        destination_token_keys
            .entry(fuzz_instruction.destination_token_key)
            .or_insert_with(|| Pubkey::new_unique());
        new_destination_token_keys
            .entry(fuzz_instruction.new_destination_token_key)
            .or_insert_with(|| Pubkey::new_unique());
        mint_keys
            .entry(fuzz_instruction.mint_key)
            .or_insert_with(|| Keypair::new());
        payer_keys
            .entry(fuzz_instruction.payer_key)
            .or_insert_with(|| Keypair::new());

        let (mut output_instructions, mut signer_keys) = run_fuzz_instruction(
            &token_vesting_testenv,
            &fuzz_instruction,
            &correct_payer,
            mint_keys.get(&fuzz_instruction.mint_key).unwrap(),
            vesting_account_keys.get(&fuzz_instruction.vesting_account_key).unwrap(),
            vesting_token_account_keys.get(&fuzz_instruction.vesting_token_account_key).unwrap(),
            source_token_account_owner_keys.get(
                &fuzz_instruction.source_token_account_owner_key
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
        );
        global_output_instructions.append(&mut output_instructions);
        global_signer_keys.append(&mut signer_keys);
    }
    // Process transaction on test network
    let mut transaction = Transaction::new_with_payer(
        &global_output_instructions,
        Some(&correct_payer.pubkey()),
    );
    let signers = [correct_payer].iter().map(|&v| v).chain(global_signer_keys.iter()).collect::<Vec<&Keypair>>();
    transaction.partial_sign(
        &signers,
        recent_blockhash
    );

    banks_client.process_transaction(transaction).await.unwrap_or_else(|e| {
        if let TransportError::TransactionError(te) = e {
            match te {
                TransactionError::InstructionError(_, ie) => {
                    match ie {
                        InstructionError::InvalidArgument
                        | InstructionError::InvalidInstructionData
                        | InstructionError::InvalidAccountData
                        | InstructionError::InsufficientFunds
                        | InstructionError::AccountAlreadyInitialized
                        | InstructionError::InvalidSeeds
                        | InstructionError::Custom(0) => {},
                        _ => {
                            print!("{:?}", ie);
                            Err(ie).unwrap()
                        }
                    }
                },
                TransactionError::SignatureFailure
                | TransactionError::InvalidAccountForFee
                | TransactionError::InsufficientFundsForFee => {},
                _ => {
                    print!("{:?}", te);
                    panic!()
                }
            }

        } else {
            print!("{:?}", e);
            panic!()
        }
    });
}


fn run_fuzz_instruction(
    token_vesting_testenv: &TokenVestingEnv,
    fuzz_instruction: &FuzzInstruction,
    correct_payer: &Keypair,
    mint_key: &Keypair,
    vesting_account_key: &Pubkey,
    vesting_token_account_key: &Pubkey,
    source_token_account_owner_key: &Keypair,
    destination_token_owner_key: &Keypair,
    destination_token_key: &Pubkey,
    new_destination_token_key: &Pubkey,
    payer_key: &Keypair
) -> (Vec<Instruction>, Vec<Keypair>) {

    // Execute the fuzzing in a more restrained way in order to go deeper into the program branches.
    // For each possible fuzz instruction we first instantiate the needed accounts for the instruction
    if fuzz_instruction.correct_inputs {

        let mut correct_seeds = fuzz_instruction.seeds;
        let (correct_vesting_account_key, bump) = Pubkey::find_program_address(
            &[&correct_seeds[..31]],
            &token_vesting_testenv.vesting_program_id
        );
        correct_seeds[31] = bump;
        let correct_vesting_token_key = get_associated_token_address(
            &correct_vesting_account_key,
            &mint_key.pubkey()
        );
        let correct_source_token_account_key = get_associated_token_address(
            &source_token_account_owner_key.pubkey(),
            &mint_key.pubkey()
        );

        match fuzz_instruction {
            FuzzInstruction {
                instruction: VestingInstruction::Init{ .. },
                ..
            } => {
                return (vec![init(
                    &token_vesting_testenv.system_program_id,
                    &token_vesting_testenv.rent_program_id,
                    &token_vesting_testenv.vesting_program_id,
                    &correct_payer.pubkey(),
                    &correct_vesting_account_key,
                    correct_seeds,
                    fuzz_instruction.number_of_schedules as u32
                ).unwrap()], vec![]);
            },

            FuzzInstruction {
                instruction: VestingInstruction::Create { .. },
                ..
            } => {
                let mut instructions_acc = vec![init(
                    &token_vesting_testenv.system_program_id,
                    &token_vesting_testenv.rent_program_id,
                    &token_vesting_testenv.vesting_program_id,
                    &correct_payer.pubkey(),
                    &correct_vesting_account_key,
                    correct_seeds,
                    fuzz_instruction.number_of_schedules as u32
                ).unwrap()];
                let mut create_instructions = create_fuzzinstruction(
                    token_vesting_testenv,
                    fuzz_instruction,
                    correct_payer,
                    &correct_source_token_account_key,
                    source_token_account_owner_key,
                    destination_token_key,
                    &destination_token_owner_key.pubkey(),
                    &correct_vesting_account_key,
                    &correct_vesting_token_key,
                    correct_seeds,
                    mint_key,
                    fuzz_instruction.source_token_amount
                );
                instructions_acc.append(&mut create_instructions);
                return (instructions_acc, vec![clone_keypair(mint_key),
                    clone_keypair(&token_vesting_testenv.mint_authority),
                    clone_keypair(source_token_account_owner_key)]);
            },

            FuzzInstruction {
                instruction: VestingInstruction::Unlock{ .. },
                ..
            } => {
                let mut instructions_acc = vec![init(
                    &token_vesting_testenv.system_program_id,
                    &token_vesting_testenv.rent_program_id,
                    &token_vesting_testenv.vesting_program_id,
                    &correct_payer.pubkey(),
                    &correct_vesting_account_key,
                    correct_seeds,
                    fuzz_instruction.number_of_schedules as u32
                ).unwrap()];
                let mut create_instructions = create_fuzzinstruction(
                    token_vesting_testenv,
                    fuzz_instruction,
                    correct_payer,
                    &correct_source_token_account_key,
                    source_token_account_owner_key,
                    destination_token_key,
                    &destination_token_owner_key.pubkey(),
                    &correct_vesting_account_key,
                    &correct_vesting_token_key,
                    correct_seeds,
                    mint_key,
                    fuzz_instruction.source_token_amount
                );
                instructions_acc.append(&mut create_instructions);

                let unlock_instruction = unlock(
                    &token_vesting_testenv.vesting_program_id,
                    &token_vesting_testenv.token_program_id,
                    &token_vesting_testenv.sysvarclock_program_id,
                    &correct_vesting_account_key,
                    &correct_vesting_token_key,
                    destination_token_key,
                    correct_seeds
                ).unwrap();
                instructions_acc.push(unlock_instruction);
            return (instructions_acc, vec![
                clone_keypair(mint_key),
                clone_keypair(&token_vesting_testenv.mint_authority),
                clone_keypair(source_token_account_owner_key),
                ]);
            },

            FuzzInstruction {
                instruction: VestingInstruction::ChangeDestination{ .. },
                ..
            } => {
                let mut instructions_acc = vec![init(
                    &token_vesting_testenv.system_program_id,
                    &token_vesting_testenv.rent_program_id,
                    &token_vesting_testenv.vesting_program_id,
                    &correct_payer.pubkey(),
                    &correct_vesting_account_key,
                    correct_seeds,
                    fuzz_instruction.number_of_schedules as u32
                ).unwrap()];
                let mut create_instructions = create_fuzzinstruction(
                    token_vesting_testenv,
                    fuzz_instruction,
                    correct_payer,
                    &correct_source_token_account_key,
                    source_token_account_owner_key,
                    destination_token_key,
                    &destination_token_owner_key.pubkey(),
                    &correct_vesting_account_key,
                    &correct_vesting_token_key,
                    correct_seeds,
                    mint_key,
                    fuzz_instruction.source_token_amount
                );
                instructions_acc.append(&mut create_instructions);

                let new_destination_instruction = create_associated_token_account(
                    &correct_payer.pubkey(),
                    &Pubkey::new_unique(), // Arbitrary
                    &mint_key.pubkey()
                );
                instructions_acc.push(new_destination_instruction);

                let change_instruction = change_destination(
                    &token_vesting_testenv.vesting_program_id,
                    &correct_vesting_account_key,
                    &destination_token_owner_key.pubkey(),
                    &destination_token_key,
                    new_destination_token_key,
                    correct_seeds
                ).unwrap();
                instructions_acc.push(change_instruction);
                return (instructions_acc, vec![
                    clone_keypair(mint_key),
                    clone_keypair(&token_vesting_testenv.mint_authority),
                    clone_keypair(source_token_account_owner_key),
                    clone_keypair(destination_token_owner_key),
                ]);
            }
        };

    // Execute a more random input fuzzing (these should give an error almost surely)
    } else {
        match fuzz_instruction {

            FuzzInstruction {
                instruction: VestingInstruction::Init{ .. },
                ..
            } => {
                return (vec![init(
                    &token_vesting_testenv.system_program_id,
                    &token_vesting_testenv.rent_program_id,
                    &token_vesting_testenv.vesting_program_id,
                    &payer_key.pubkey(),
                    vesting_account_key,
                    fuzz_instruction.seeds,
                    fuzz_instruction.number_of_schedules as u32
                ).unwrap()], vec![]);
            },

            FuzzInstruction {
                instruction: VestingInstruction::Create { .. },
                ..
            } => {
                let create_instructions = create(
                    &token_vesting_testenv.vesting_program_id,
                    &token_vesting_testenv.token_program_id,
                    vesting_account_key,
                    vesting_token_account_key,
                    &source_token_account_owner_key.pubkey(),
                    &destination_token_owner_key.pubkey(),
                    destination_token_key,
                    &mint_key.pubkey(),
                    fuzz_instruction.schedules.clone(),
                    fuzz_instruction.seeds
                ).unwrap();
                return (
                    vec![create_instructions],
                    vec![clone_keypair(source_token_account_owner_key)]
                );
            },

            FuzzInstruction {
                instruction: VestingInstruction::Unlock{ .. },
                ..
            } => {
                let unlock_instruction = unlock(
                    &token_vesting_testenv.vesting_program_id,
                    &token_vesting_testenv.token_program_id,
                    &token_vesting_testenv.sysvarclock_program_id,
                    vesting_account_key,
                    vesting_token_account_key,
                    destination_token_key,
                    fuzz_instruction.seeds,
                ).unwrap();
                return (
                    vec![unlock_instruction],
                    vec![]
                );
            },

            FuzzInstruction {
                instruction: VestingInstruction::ChangeDestination{ .. },
                ..
            } => {
                let change_instruction = change_destination(
                    &token_vesting_testenv.vesting_program_id,
                    vesting_account_key,
                    &destination_token_owner_key.pubkey(),
                    &destination_token_key,
                    new_destination_token_key,
                    fuzz_instruction.seeds,
                ).unwrap();
                return (
                    vec![change_instruction],
                    vec![clone_keypair(destination_token_owner_key)]
                );
            }
        };
    }

}


// A correct vesting create fuzz instruction
fn create_fuzzinstruction(
    token_vesting_testenv: &TokenVestingEnv,
    fuzz_instruction: &FuzzInstruction,
    payer: &Keypair,
    correct_source_token_account_key: &Pubkey,
    source_token_account_owner_key: &Keypair,
    destination_token_key: &Pubkey,
    destination_token_owner_key: &Pubkey,
    correct_vesting_account_key: &Pubkey,
    correct_vesting_token_key: &Pubkey,
    correct_seeds: [u8; 32],
    mint_key: &Keypair,
    source_amount: u64
) -> Vec<Instruction> {

    // Initialize the token mint account
    let mut instructions_acc = mint_init_instruction(
        &payer,
        &mint_key,
        &token_vesting_testenv.mint_authority
    );

    // Create the associated token accounts
    let source_instruction = create_associated_token_account(
        &payer.pubkey(),
        &source_token_account_owner_key.pubkey(),
        &mint_key.pubkey()
    );
    instructions_acc.push(source_instruction);

    let vesting_instruction = create_associated_token_account(
            &payer.pubkey(),
            &correct_vesting_account_key,
            &mint_key.pubkey()
    );
    instructions_acc.push(vesting_instruction);

    let destination_instruction = create_associated_token_account(
            &payer.pubkey(),
            &destination_token_owner_key,
            &mint_key.pubkey()
    );
    instructions_acc.push(destination_instruction);

    // Credit the source account
    let setup_instruction = mint_to(
        &spl_token::id(),
        &mint_key.pubkey(),
        &correct_source_token_account_key,
        &token_vesting_testenv.mint_authority.pubkey(),
        &[],
        source_amount
    ).unwrap();
    instructions_acc.push(setup_instruction);

    let used_number_of_schedules = fuzz_instruction.number_of_schedules.min(
        fuzz_instruction.schedules.len().try_into().unwrap()
    );
    // Initialize the vesting program account
    let create_instruction = create(
        &token_vesting_testenv.vesting_program_id,
        &token_vesting_testenv.token_program_id,
        &correct_vesting_account_key,
        &correct_vesting_token_key,
        &source_token_account_owner_key.pubkey(),
        &correct_source_token_account_key,
        &destination_token_key,
        &mint_key.pubkey(),
        fuzz_instruction.schedules.clone()[..used_number_of_schedules.into()].into(),
        correct_seeds,
    ).unwrap();
    instructions_acc.push(create_instruction);

    return instructions_acc;
}


// Helper functions
fn mint_init_instruction(
    payer: &Keypair,
    mint:&Keypair,
    mint_authority: &Keypair) -> Vec<Instruction> {
    let instructions = vec![
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
    return instructions;
}

fn clone_keypair(keypair: &Keypair) -> Keypair {
    return Keypair::from_bytes(&keypair.to_bytes().clone()).unwrap();
}
