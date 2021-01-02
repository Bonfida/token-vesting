use spl_associated_token_account::get_associated_token_address;
use token_vesting::instruction::{VestingInstruction, create, unlock};
use clap::{
    crate_description, crate_name, crate_version, value_t, App, AppSettings, Arg, SubCommand
};
use solana_client::{
    rpc_client::RpcClient,
};
use solana_clap_utils::{input_parsers::{keypair_of, lamports_of_sol, pubkey_of, value_of}, input_validators::{is_amount, is_keypair, is_pubkey, is_url, is_parsable}};
use solana_sdk::{signature::Signer, signature::Keypair, transaction::Transaction};
use solana_program::{instruction::{AccountMeta, Instruction}, msg, pubkey::Pubkey, system_program};
use std::convert::TryInto;

// Lock the vesting contract
fn command_create_svc(
    rpc_client: RpcClient,
    program_id: Pubkey,
    mut vesting_seed: [u8;32],
    source_token_owner: Keypair,
    destination_token_pubkey: Pubkey,
    mint_address: Pubkey,
    vesting_amount: u64
) {
    // Find the non reversible public key for the vesting contract via the seed    
    let (vesting_pubkey, bump) = Pubkey::find_program_address(&[&vesting_seed[..31]], &program_id);
    vesting_seed[31] = bump;
    msg!("Vesting account pubkey: {:?}", vesting_pubkey);

    let source_token_pubkey = get_associated_token_address(
        &source_token_owner.pubkey(), 
        &mint_address
    );

    let decimals = rpc_client
        .get_token_account(&source_token_pubkey)
        .unwrap()
        .unwrap()
        .token_amount.decimals;

    let create_instruction = create(
        &program_id,
        &vesting_pubkey,
        &source_token_pubkey,
        &destination_token_pubkey,
        &mint_address,
        vesting_amount,
        0,
        vesting_seed
    ).unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[create_instruction],
        Some(&source_pubkey.pubkey()),
    );

    let recent_blockhash = rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&source_pubkey], recent_blockhash);

    rpc_client.send_transaction(&transaction).unwrap();
}

fn command_unlock_svc(
    rpc_client: RpcClient,
    program_id: Pubkey,
    mut vesting_seed: [u8;32],
    source_pubkey: Keypair,
    destination_pubkey: Pubkey,
    mint_address: Pubkey,
    vesting_amount: u64
) {
    // Find the non reversible public key for the vesting contract via the seed    
    let (vesting_pubkey, bump) = Pubkey::find_program_address(&[&vesting_seed[..31]], &program_id);
    vesting_seed[31] = bump;
    msg!("Vesting account pubkey: {:?}", vesting_pubkey);

    let unlock_instruction = unlock(
        &program_id,
        &vesting_pubkey,
        &destination_pubkey,
        &mint_address,
        vesting_seed,
    ).unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[unlock_instruction],
        Some(&source_pubkey.pubkey()),
    );

    let recent_blockhash = rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&source_pubkey], recent_blockhash);

    rpc_client.send_transaction(&transaction).unwrap();
}

fn main() {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )        
        .arg(
            Arg::with_name("rpc_url")
                .long("url")
                .value_name("URL")
                .validator(is_url)
                .takes_value(true)
                .help(
                    "Specify the url of the rpc client (solana network).",
                ),
        )
        .arg(
            Arg::with_name("program_id")
                .long("program_id")
                .value_name("ADDRESS")
                .validator(is_pubkey)
                .takes_value(true)
                .help(
                    "Specify the adress (publickey) of the program. \
                     This may be a keypair file, the ASK keyword. \
                     Defaults to the client keypair.",
                ),
        )
        .subcommand(SubCommand::with_name("create-svc").about("Create a new simple vesting contract")        
            .arg(
                Arg::with_name("seed")
                    .long("seed")
                    .value_name("ADDRESS")
                    // .validator(is_hash)
                    .takes_value(true)
                    .help(
                        "Specify the seed for the vesting contract. \
                        This may be a keypair file, the ASK keyword. \
                        Defaults to the client keypair.",
                    ),
            )        
            .arg(
                Arg::with_name("source")
                    .long("source")
                    .value_name("KEYPAIR")
                    .validator(is_keypair)
                    .takes_value(true)
                    .help(
                        "Specify the source account owner. \
                        This may be a keypair file, the ASK keyword. \
                        Defaults to the client keypair.",
                    ),
            )
            .arg(
                Arg::with_name("destination")
                    .long("destination")
                    .value_name("ADDRESS")
                    .validator(is_pubkey)
                    .takes_value(true)
                    .help(
                        "Specify the destination account address. \
                        This may be a keypair file, the ASK keyword. \
                        Defaults to the client keypair.",
                    ),
            )        
            .arg(
                Arg::with_name("amount")
                    .long("amount")
                    .value_name("AMOUNT")
                    .validator(is_amount)
                    .takes_value(true)
                    .help(
                        "Amount in SOL to transfer via the vesting \
                        contract.",
                    ),
            )
        )
        .subcommand(SubCommand::with_name("unlock-svc").about("Unlock a simple vesting contract")        
        .arg(
            Arg::with_name("seed")
                .long("seed")
                .value_name("ADDRESS")
                // .validator(is_hash)
                .takes_value(true)
                .help(
                    "Specify the seed for the vesting contract. \
                    This may be a keypair file, the ASK keyword. \
                    Defaults to the client keypair.",
                ),
        )        
        .arg(
            Arg::with_name("source")
                .long("source")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .help(
                    "Specify the source account owner. \
                    This may be a keypair file, the ASK keyword. \
                    Defaults to the client keypair.",
                ),
        )
        .arg(
            Arg::with_name("destination")
                .long("destination")
                .value_name("ADDRESS")
                .validator(is_pubkey)
                .takes_value(true)
                .help(
                    "Specify the destination account address. \
                    This may be a keypair file, the ASK keyword. \
                    Defaults to the client keypair.",
                ),
        )        
        .arg(
            Arg::with_name("amount")
                .long("amount")
                .value_name("AMOUNT")
                .validator(is_amount)
                .takes_value(true)
                .help(
                    "Amount in SOL to transfer via the vesting \
                    contract.",
                ),
        )
    )
        .get_matches();

    let rpc_url = value_t!(matches, "rpc_url", String)
    .unwrap();
    msg!("RPC URL: {:?}", &rpc_url);
    let rpc_client = RpcClient::new(rpc_url);

    let program_id = pubkey_of(&matches, "program_id").unwrap();
    
    // solana_logger::setup_with_default("solana=info");
    
    let _ = match matches.subcommand() {
        ("create-svc", Some(arg_matches)) => {
            let vesting_seed = (*String::as_bytes(&value_of(arg_matches, &"seed").unwrap())).try_into().unwrap();
            let source_pubkey = keypair_of(arg_matches, "source").unwrap();
            let destination_pubkey = pubkey_of(arg_matches, "destination").unwrap();
            let vesting_amount = lamports_of_sol(arg_matches, "amount").unwrap();
            msg!("Program ID: {:?}", &program_id);
            msg!("Vesting Seed: {:?}", &vesting_seed);
            msg!("Source Pubkey: {:?}", &source_pubkey);
            msg!("Destination Pubkey: {:?}", &destination_pubkey);
            msg!("Vesting Amount: {:?}", &vesting_amount);
            command_create_svc(
                rpc_client,
                program_id,
                vesting_seed,
                source_pubkey,
                destination_pubkey,
                vesting_amount
            )
        }
        ("unlock-svc", Some(arg_matches)) => {
            let vesting_seed = (*String::as_bytes(&value_of(arg_matches, &"seed").unwrap())).try_into().unwrap();
            let source_pubkey = keypair_of(arg_matches, "source").unwrap();
            let destination_pubkey = pubkey_of(arg_matches, "destination").unwrap();
            let vesting_amount = lamports_of_sol(arg_matches, "amount").unwrap();
            msg!("Program ID: {:?}", &program_id);
            msg!("Vesting Seed: {:?}", &vesting_seed);
            msg!("Source Pubkey: {:?}", &source_pubkey);
            msg!("Destination Pubkey: {:?}", &destination_pubkey);
            msg!("Vesting Amount: {:?}", &vesting_amount);
            command_unlock_svc(
                rpc_client,
                program_id,
                vesting_seed,
                source_pubkey,
                destination_pubkey,
                vesting_amount
            )
        }
        _ => unreachable!(),
    };

}