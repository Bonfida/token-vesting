use spl_associated_token_account::{get_associated_token_address, create_associated_token_account};
use token_vesting::instruction::{VestingInstruction, create, unlock};
use clap::{
    crate_description, crate_name, crate_version, value_t, App, AppSettings, Arg, SubCommand
};
use solana_client::{
    rpc_client::RpcClient,
};
use solana_clap_utils::{input_parsers::{keypair_of, lamports_of_sol, pubkey_of, value_of}, input_validators::{is_amount, is_keypair, is_pubkey, is_url, is_parsable}};
use solana_sdk::{self, system_instruction, signature::Signer, signature::{Keypair, keypair_from_seed}, transaction::Transaction};
use solana_program::{instruction::{AccountMeta, Instruction}, msg, pubkey::Pubkey, system_program, rent::Rent, sysvar};
use spl_token;
use std::convert::TryInto;

// Lock the vesting contract
fn command_create_svc(
    rpc_client: RpcClient,
    program_id: Pubkey,
    vesting_seed: [u8;32],
    source_token_owner: Keypair,
    source_token_pubkey: Pubkey, // TODO make it Option, if None, use associated
    destination_token_pubkey: Pubkey,
    mint_address: Pubkey,
    vesting_amount: u64
) {
    // Find the non reversible public key for the vesting contract via the seed    
    // let (vesting_pubkey, bump) = Pubkey::find_program_address(&[&vesting_seed[..31]], &program_id);
    // vesting_seed[31] = bump;
    let vesting_keypair = keypair_from_seed(&vesting_seed).unwrap();
    let vesting_pubkey = vesting_keypair.pubkey();
    msg!("Vesting account pubkey: {:?}", vesting_pubkey);

    let vesting_token_pubkey = get_associated_token_address(
        &vesting_pubkey, 
        &mint_address
    );

    // let decimals = rpc_client
    //     .get_token_account(&source_token_pubkey)
    //     .unwrap()
    //     .unwrap()
    //     .token_amount.decimals;

    let instructions = [
        // Create the vesting account
        // This is the fundamental problem of program account creation. 
        // How to create the account without a private key as the 
        // create_with_seed address generation does not match the find_programm_adress ?
        // (Maybe create and assign, loose the private kez and keep the seeds?)
        // Leaving this for later.
        system_instruction::create_account(
            &source_token_owner.pubkey(),
            &vesting_pubkey,
            // &source_token_owner.pubkey(),
            // std::str::from_utf8(&vesting_seed).unwrap(),
            Rent::default().minimum_balance(165),
            165,
            &program_id
        ),
        // Create and initiliaze the vesting token account
        create_associated_token_account(
            &source_token_owner.pubkey(),
            &vesting_pubkey,
            &mint_address
        ),
        create(
            &program_id,
            &spl_token::id(),
            &vesting_pubkey,
            &vesting_token_pubkey,
            &source_token_owner.pubkey(),
            &source_token_pubkey,
            &destination_token_pubkey,
            &mint_address,
            vesting_amount,
            0,
            vesting_seed
        ).unwrap()
   ];

    let mut transaction = Transaction::new_with_payer(
        &instructions,
        Some(&source_token_owner.pubkey()),
    );

    let recent_blockhash = rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&source_token_owner, &vesting_keypair], recent_blockhash);

    rpc_client.send_transaction(&transaction).unwrap();
}

fn command_unlock_svc(
    rpc_client: RpcClient,
    program_id: Pubkey,
    vesting_seed: [u8;32],
    source_pubkey: Keypair,
    destination_token_pubkey: Pubkey,
    mint_address: Pubkey,
    vesting_amount: u64
) {
    // Find the non reversible public key for the vesting contract via the seed    
    // let (vesting_pubkey, bump) = Pubkey::find_program_address(&[&vesting_seed[..31]], &program_id);
    // vesting_seed[31] = bump;
    let vesting_keypair = keypair_from_seed(&vesting_seed).unwrap();
    let vesting_pubkey = vesting_keypair.pubkey();
    msg!("Vesting account pubkey: {:?}", vesting_pubkey);

    let vesting_token_pubkey = get_associated_token_address(
        &vesting_pubkey, 
        &mint_address
    );

    //TODO initialize associated destination token account if none found

    let unlock_instruction = unlock(
        &program_id,
        &spl_token::id(),
        &sysvar::clock::id(),
        &vesting_pubkey,
        &vesting_token_pubkey,
        &destination_token_pubkey,
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
                    "Specify the address (public key) of the program.",
                ),
        )
        .arg(
            Arg::with_name("mint_address")
                .long("mint_address")
                .value_name("ADDRESS")
                .validator(is_pubkey)
                .takes_value(true)
                .help(
                    "Specify the adress (publickey) of the mint for the token that should be used.",
                ),
        )
        .subcommand(SubCommand::with_name("create-svc").about("Create a new simple vesting contract")        
            .arg(
                Arg::with_name("seed")
                    .long("seed")
                    .value_name("ADDRESS")
                    // .validator(is_hash)  //TODO
                    .takes_value(true)
                    .help(
                        "Specify the seed for the vesting contract.",
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
                Arg::with_name("source_token_address")
                    .long("source_token_address")
                    .value_name("ADDRESS")
                    .validator(is_pubkey)
                    .takes_value(true)
                    .help(
                        "Specify the source token account address.",
                    ),
            )     
            .arg(
                Arg::with_name("destination")
                    .long("destination_token_address")
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
    let mint_address = pubkey_of(&matches, "mint_address").unwrap();
    
    // solana_logger::setup_with_default("solana=info");
    
    let _ = match matches.subcommand() {
        ("create-svc", Some(arg_matches)) => {
            let vesting_seed = (*String::as_bytes(&value_of(arg_matches, &"seed").unwrap())).try_into().unwrap();
            let source_keypair = keypair_of(arg_matches, "source").unwrap();
            let source_token_pubkey = pubkey_of(arg_matches, "source_token_address").unwrap();
            let destination_pubkey = pubkey_of(arg_matches, "destination").unwrap();
            let vesting_amount = lamports_of_sol(arg_matches, "amount").unwrap();
            msg!("Program ID: {:?}", &program_id);
            msg!("Vesting Seed: {:?}", &vesting_seed);
            msg!("Source Pubkey: {:?}", &source_keypair.pubkey());
            msg!("Destination Pubkey: {:?}", &destination_pubkey);
            msg!("Vesting Amount: {:?}", &vesting_amount);
            command_create_svc(
                rpc_client,
                program_id,
                vesting_seed,
                source_keypair,
                source_token_pubkey,
                destination_pubkey,
                mint_address,
                vesting_amount,

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
                mint_address,
                vesting_amount
            )
        }
        _ => unreachable!(),
    };

}