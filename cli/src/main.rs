use chrono::{DateTime, Duration};
use clap::{
    crate_description, crate_name, crate_version, value_t, App, AppSettings, Arg, SubCommand,
};
use solana_clap_utils::{
    input_parsers::{keypair_of, pubkey_of, value_of, values_of},
    input_validators::{is_amount, is_keypair, is_parsable, is_pubkey, is_slot, is_url},
};
use solana_client::rpc_client::RpcClient;
use solana_program::{msg, program_pack::Pack, pubkey::Pubkey, system_program, sysvar};
use solana_sdk::{
    self, commitment_config::CommitmentConfig, signature::Keypair, signature::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token;
use std::convert::TryInto;
use token_vesting::{
    instruction::{change_destination, create, init, unlock, Schedule},
    state::{unpack_schedules, VestingScheduleHeader},
};

// Lock the vesting contract
fn command_create_svc(
    rpc_client: RpcClient,
    program_id: Pubkey,
    payer: Keypair,
    source_token_owner: Keypair,
    possible_source_token_pubkey: Option<Pubkey>,
    destination_token_pubkey: Pubkey,
    mint_address: Pubkey,
    schedules: Vec<Schedule>,
    confirm: bool,
) {
    // If no source token account was given, use the associated source account
    let source_token_pubkey = match possible_source_token_pubkey {
        None => get_associated_token_address(&source_token_owner.pubkey(), &mint_address),
        _ => possible_source_token_pubkey.unwrap(),
    };

    // Find a valid seed for the vesting program account key to be non reversible and unused
    let mut not_found = true;
    let mut vesting_seed: [u8; 32] = [0; 32];
    let mut vesting_pubkey = Pubkey::new_unique();
    while not_found {
        vesting_seed = Pubkey::new_unique().to_bytes();
        let program_id_bump = Pubkey::find_program_address(&[&vesting_seed[..31]], &program_id);
        vesting_pubkey = program_id_bump.0;
        vesting_seed[31] = program_id_bump.1;
        not_found = match rpc_client.get_account(&vesting_pubkey) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    let vesting_token_pubkey = get_associated_token_address(&vesting_pubkey, &mint_address);

    let instructions = [
        init(
            &system_program::id(),
            &sysvar::rent::id(),
            &program_id,
            &payer.pubkey(),
            &vesting_pubkey,
            vesting_seed,
            schedules.len() as u32,
        )
        .unwrap(),
        create_associated_token_account(
            &source_token_owner.pubkey(),
            &vesting_pubkey,
            &mint_address,
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
            schedules,
            vesting_seed,
        )
        .unwrap(),
    ];

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));

    let recent_blockhash = rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&payer], recent_blockhash);

    msg!(
        "\nThe seed of the contract is: {:?}",
        Pubkey::new_from_array(vesting_seed)
    );
    msg!("Please write it down as it is needed to interact with the contract!");

    msg!("The vesting account pubkey: {:?}", vesting_pubkey,);

    if confirm {
        rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &transaction,
                CommitmentConfig::finalized(),
            )
            .unwrap();
    } else {
        rpc_client.send_transaction(&transaction).unwrap();
    }
}

fn command_unlock_svc(
    rpc_client: RpcClient,
    program_id: Pubkey,
    vesting_seed: [u8; 32],
    payer: Keypair,
) {
    // Find the non reversible public key for the vesting contract via the seed
    let (vesting_pubkey, _) = Pubkey::find_program_address(&[&vesting_seed[..31]], &program_id);

    let packed_state = rpc_client.get_account_data(&vesting_pubkey).unwrap();
    let header_state =
        VestingScheduleHeader::unpack(&packed_state[..VestingScheduleHeader::LEN]).unwrap();
    let destination_token_pubkey = header_state.destination_address;

    let vesting_token_pubkey =
        get_associated_token_address(&vesting_pubkey, &header_state.mint_address);

    let unlock_instruction = unlock(
        &program_id,
        &spl_token::id(),
        &sysvar::clock::id(),
        &vesting_pubkey,
        &vesting_token_pubkey,
        &destination_token_pubkey,
        vesting_seed,
    )
    .unwrap();

    let mut transaction = Transaction::new_with_payer(&[unlock_instruction], Some(&payer.pubkey()));

    let recent_blockhash = rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&payer], recent_blockhash);

    rpc_client.send_transaction(&transaction).unwrap();
}

fn command_change_destination(
    rpc_client: RpcClient,
    program_id: Pubkey,
    destination_token_account_owner: Keypair,
    opt_new_destination_account: Option<Pubkey>,
    opt_new_destination_token_account: Option<Pubkey>,
    vesting_seed: [u8; 32],
    payer: Keypair,
) {
    // Find the non reversible public key for the vesting contract via the seed
    let (vesting_pubkey, _) = Pubkey::find_program_address(&[&vesting_seed[..31]], &program_id);

    let packed_state = rpc_client.get_account_data(&vesting_pubkey).unwrap();
    let state_header =
        VestingScheduleHeader::unpack(&packed_state[..VestingScheduleHeader::LEN]).unwrap();
    let destination_token_pubkey = state_header.destination_address;

    let new_destination_token_account = match opt_new_destination_token_account {
        None => get_associated_token_address(
            &opt_new_destination_account.unwrap(),
            &state_header.mint_address,
        ),
        Some(new_destination_token_account) => new_destination_token_account,
    };

    let unlock_instruction = change_destination(
        &program_id,
        &vesting_pubkey,
        &destination_token_account_owner.pubkey(),
        &destination_token_pubkey,
        &new_destination_token_account,
        vesting_seed,
    )
    .unwrap();

    let mut transaction = Transaction::new_with_payer(&[unlock_instruction], Some(&payer.pubkey()));

    let recent_blockhash = rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign(
        &[&payer, &destination_token_account_owner],
        recent_blockhash,
    );

    rpc_client.send_transaction(&transaction).unwrap();
}

fn command_info(
    rpc_client: RpcClient,
    rpc_url: String,
    program_id: Pubkey,
    vesting_seed: [u8; 32],
) {
    msg!("\n---------------VESTING--CONTRACT--INFO-----------------\n");
    msg!("RPC URL: {:?}", &rpc_url);
    msg!("Program ID: {:?}", &program_id);
    msg!("Vesting Seed: {:?}", Pubkey::new_from_array(vesting_seed));

    // Find the non reversible public key for the vesting contract via the seed
    let (vesting_pubkey, _) = Pubkey::find_program_address(&[&vesting_seed[..31]], &program_id);
    msg!("Vesting Account Pubkey: {:?}", &vesting_pubkey);

    let packed_state = rpc_client.get_account_data(&vesting_pubkey).unwrap();
    let state_header =
        VestingScheduleHeader::unpack(&packed_state[..VestingScheduleHeader::LEN]).unwrap();
    let vesting_token_pubkey =
        get_associated_token_address(&vesting_pubkey, &state_header.mint_address);
    msg!("Vesting Token Account Pubkey: {:?}", &vesting_token_pubkey);
    msg!("Initialized: {:?}", &state_header.is_initialized);
    msg!("Mint Address: {:?}", &state_header.mint_address);
    msg!(
        "Destination Token Address: {:?}",
        &state_header.destination_address
    );

    let schedules = unpack_schedules(&packed_state[VestingScheduleHeader::LEN..]).unwrap();

    for i in 0..schedules.len() {
        msg!("\nSCHEDULE {:?}", i);
        msg!("Release Height: {:?}", &schedules[i].release_time);
        msg!("Amount: {:?}", &schedules[i].amount);
    }
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
                .global(true)
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
        .subcommand(SubCommand::with_name("create").about("Create a new vesting contract with an optional release schedule")        
            .arg(
                Arg::with_name("mint_address")
                    .long("mint_address")
                    .value_name("ADDRESS")
                    .validator(is_pubkey)
                    .takes_value(true)
                    .help(
                        "Specify the address (publickey) of the mint for the token that should be used.",
                    ),
            )
            .arg(
                Arg::with_name("source_owner")
                    .long("source_owner")
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
                Arg::with_name("destination_address")
                    .long("destination_address")
                    .value_name("ADDRESS")
                    .validator(is_pubkey)
                    .takes_value(true)
                    .help(
                        "Specify the destination (non-token) account address. \
                        If specified, the vesting destination will be the associated \
                        token account for the mint of the contract."
                    ),
            )
            .arg(
                Arg::with_name("destination_token_address")
                    .long("destination_token_address")
                    .value_name("ADDRESS")
                    .validator(is_pubkey)
                    .takes_value(true)
                    .help(
                        "Specify the destination token account address. \
                        If specified, this address will be used as a destination, \
                        and overwrite the associated token account.",
                    ),
            )
            .arg(
                Arg::with_name("amounts")
                    .long("amounts")
                    .value_name("AMOUNT")
                    .validator(is_amount)
                    .takes_value(true)
                    .multiple(true)
                    .use_delimiter(true)
                    .value_terminator("!")
                    .allow_hyphen_values(true)
                    .help(
                        "Amounts of tokens to transfer via the vesting \
                        contract. Multiple inputs separated by a comma are
                        accepted for the creation of multiple schedules. The sequence of inputs \
                        needs to end with an exclamation mark ( e.g. 1,2,3,! )",
                    ),
            )
            // scheduled vesting
            .arg(
                Arg::with_name("release-times")
                    .long("release-times")
                    .conflicts_with("release-frequency")
                    .value_name("SLOT")
                    .validator(is_slot)
                    .takes_value(true)
                    .multiple(true)
                    .use_delimiter(true)
                    .value_terminator("!")
                    .allow_hyphen_values(true)
                    .help(
                        "Release times in unix timestamp to decide when the contract is \
                        unlockable. Multiple inputs separated by a comma are
                        accepted for the creation of multiple schedules. The sequence of inputs \
                        needs to end with an exclamation mark ( e.g. 1,2,3,! ).",
                    ),
            )
            // linear vesting
            .arg(
                Arg::with_name("release-frequency")
                    .long("release-frequency")
                    .value_name("RELEASE_FREQUENCY")
                    .takes_value(true)
                    .conflicts_with("release-times")                                        
                    .help(
                        "Frequency of release amount. \
                        You start on 1sth of Nov and end on 5th of Nov. \
                        With 1 day frequency it will vest from total amount 5 times \
                        splitted linearly.
                        Duration must be ISO8601 duration format. Example, P1D.
                        Internally all dates will be transformed into schedule.",
                    ),
            )
            .arg(
                Arg::with_name("start-date-time")
                    .long("start-date-time")
                    .value_name("START_DATE_TIME")
                    .takes_value(true)
                    .help(
                        "First time of release in linear vesting. \
                        Must be RFC 3339 and ISO 8601 sortable date time. \
                        Example, 2022-01-06T20:11:18Z",
                    ),
            )
            .arg(
                Arg::with_name("end-date-time")
                    .long("end-date-time")
                    .value_name("END_DATE_TIME")
                    .takes_value(true)
                    .help(
                        "Last time of release in linear vesting. \
                        If frequency will go over last release time, \
                        tokens will be released later than end date. 
                        Must be RFC 3339 and ISO 8601 sortable date time. \
                        Example, 2022-17-06T20:11:18Z",
                    ),
            )
            .arg(
                Arg::with_name("payer")
                    .long("payer")
                    .value_name("KEYPAIR")
                    .validator(is_keypair)
                    .takes_value(true)
                    .help(
                        "Specify the transaction fee payer account address. \
                        This may be a keypair file, the ASK keyword. \
                        Defaults to the client keypair.",
                    ),
            )
            .arg(
                Arg::with_name("confirm")
                    .long("confirm")
                    .value_name("CONFIRM")
                    .takes_value(true)
                    .default_value("true")
                    .help(
                        "Specify whether to wait transaction confirmation"
                    ),
            )
        )
        .subcommand(SubCommand::with_name("unlock").about("Unlock a vesting contract. This will only release \
        the schedules that have reached maturity.")
            .arg(
                Arg::with_name("seed")
                    .long("seed")
                    .value_name("SEED")
                    .validator(is_parsable::<String>)
                    .takes_value(true)
                    .help(
                        "Specify the seed for the vesting contract.",
                    ),
            )
            .arg(
                Arg::with_name("payer")
                    .long("payer")
                    .value_name("KEYPAIR")
                    .validator(is_keypair)
                    .takes_value(true)
                    .help(
                        "Specify the transaction fee payer account address. \
                        This may be a keypair file, the ASK keyword. \
                        Defaults to the client keypair.",
                    ),
            )
        )
        .subcommand(SubCommand::with_name("change-destination").about("Change the destination of a vesting contract")
            .arg(
                Arg::with_name("seed")
                    .long("seed")
                    .value_name("SEED")
                    .validator(is_parsable::<String>)
                    .takes_value(true)
                    .help(
                        "Specify the seed for the vesting contract.",
                    ),
            )
            .arg(
                Arg::with_name("current_destination_owner")
                    .long("current_destination_owner")
                    .value_name("KEYPAIR")
                    .validator(is_keypair)
                    .takes_value(true)
                    .help(
                        "Specify the current destination owner account keypair. \
                        This may be a keypair file, the ASK keyword. \
                        Defaults to the client keypair.",
                    ),
            )
            .arg(
                Arg::with_name("new_destination_address")
                    .long("new_destination_address")
                    .value_name("ADDRESS")
                    .validator(is_pubkey)
                    .takes_value(true)
                    .help(
                        "Specify the new destination (non-token) account address. \
                        If specified, the vesting destination will be the associated \
                        token account for the mint of the contract."
                    ),
            )
            .arg(
                Arg::with_name("new_destination_token_address")
                    .long("new_destination_token_address")
                    .value_name("ADDRESS")
                    .validator(is_pubkey)
                    .takes_value(true)
                    .help(
                        "Specify the new destination token account address. \
                        If specified, this address will be used as a destination, \
                        and overwrite the associated token account.",
                    ),
            )
            .arg(
                Arg::with_name("payer")
                    .long("payer")
                    .value_name("KEYPAIR")
                    .validator(is_keypair)
                    .takes_value(true)
                    .help(
                        "Specify the transaction fee payer account address. \
                        This may be a keypair file, the ASK keyword. \
                        Defaults to the client keypair.",
                    ),
            )
        )
        .subcommand(SubCommand::with_name("info").about("Print information about a vesting contract")
            .arg(
                Arg::with_name("seed")
                    .long("seed")
                    .value_name("SEED")
                    .validator(is_parsable::<String>)
                    .takes_value(true)
                    .help(
                        "Specify the seed for the vesting contract.",
                    ),
            )
        )
        .get_matches();

    let rpc_url = value_t!(matches, "rpc_url", String).unwrap();
    let rpc_client = RpcClient::new(rpc_url);
    let program_id = pubkey_of(&matches, "program_id").unwrap();

    let _ = match matches.subcommand() {
        ("create", Some(arg_matches)) => {
            let source_keypair = keypair_of(arg_matches, "source_owner").unwrap();
            let source_token_pubkey = pubkey_of(arg_matches, "source_token_address");
            let mint_address = pubkey_of(arg_matches, "mint_address").unwrap();
            let destination_pubkey = match pubkey_of(arg_matches, "destination_token_address") {
                None => get_associated_token_address(
                    &pubkey_of(arg_matches, "destination_address").unwrap(),
                    &mint_address,
                ),
                Some(destination_token_pubkey) => destination_token_pubkey,
            };
            let payer_keypair = keypair_of(arg_matches, "payer").unwrap();

            // Parsing schedules
            let mut schedule_amounts: Vec<u64> = values_of(arg_matches, "amounts").unwrap();
            let confirm: bool = value_of(arg_matches, "confirm").unwrap();
            let release_frequency: Option<String> = value_of(arg_matches, "release-frequency");

            let schedule_times = if release_frequency.is_some() {
                // best found in rust
                let release_frequency: iso8601_duration::Duration =
                    release_frequency.unwrap().parse().unwrap();
                let release_frequency: u64 = Duration::from_std(release_frequency.to_std())
                    .unwrap()
                    .num_seconds()
                    .try_into()
                    .unwrap();
                if schedule_amounts.len() > 1 {
                    panic!("Linear vesting must have one amount which will split into parts per period")
                }
                let start: u64 = DateTime::parse_from_rfc3339(
                    &value_of::<String>(arg_matches, "start-date-time").unwrap(),
                )
                .unwrap()
                .timestamp()
                .try_into()
                .unwrap();
                let end: u64 = DateTime::parse_from_rfc3339(
                    &value_of::<String>(arg_matches, "end-date-time").unwrap(),
                )
                .unwrap()
                .timestamp()
                .try_into()
                .unwrap();
                let total = schedule_amounts[0];
                let part = (((total as u128) * (release_frequency as u128))
                    / ((end - start) as u128))
                    .try_into()
                    .unwrap();
                schedule_amounts.clear();
                let mut linear_vesting = Vec::new();

                let q = total / part;
                let r = total % part;

                for n in 0..q {
                    linear_vesting.push(start + n * release_frequency);
                    schedule_amounts.push(part);
                }

                if r != 0 {
                    schedule_amounts[(q - 1) as usize] += r;
                }

                if linear_vesting.len() > 365 {
                    panic!("Total count of vesting periods is more than 365. Not sure if you want to do that.")
                }

                assert_eq!(schedule_amounts.iter().sum::<u64>(), total);

                linear_vesting
            } else {
                values_of(arg_matches, "release-times").unwrap()
            };

            if schedule_amounts.len() != schedule_times.len() {
                eprintln!("error: Number of amounts given is not equal to number of release heights given.");
                std::process::exit(1);
            }
            let mut schedules: Vec<Schedule> = Vec::with_capacity(schedule_amounts.len());
            for (&a, &h) in schedule_amounts.iter().zip(schedule_times.iter()) {
                schedules.push(Schedule {
                    release_time: h,
                    amount: a,
                });
            }

            command_create_svc(
                rpc_client,
                program_id,
                payer_keypair,
                source_keypair,
                source_token_pubkey,
                destination_pubkey,
                mint_address,
                schedules,
                confirm,
            )
        }
        ("unlock", Some(arg_matches)) => {
            // The seed is given in the format of a pubkey on the user side but it's handled as a [u8;32] in the program
            let vesting_seed = pubkey_of(arg_matches, "seed").unwrap().to_bytes();
            let payer_keypair = keypair_of(arg_matches, "payer").unwrap();
            command_unlock_svc(rpc_client, program_id, vesting_seed, payer_keypair)
        }
        ("change-destination", Some(arg_matches)) => {
            let vesting_seed = pubkey_of(arg_matches, "seed").unwrap().to_bytes();
            let destination_account_owner =
                keypair_of(arg_matches, "current_destination_owner").unwrap();
            let opt_new_destination_account = pubkey_of(arg_matches, "new_destination_address");
            let opt_new_destination_token_account =
                pubkey_of(arg_matches, "new_destination_token_address");
            let payer_keypair = keypair_of(arg_matches, "payer").unwrap();
            command_change_destination(
                rpc_client,
                program_id,
                destination_account_owner,
                opt_new_destination_account,
                opt_new_destination_token_account,
                vesting_seed,
                payer_keypair,
            )
        }
        ("info", Some(arg_matches)) => {
            let vesting_seed = pubkey_of(arg_matches, "seed").unwrap().to_bytes();
            let rpcurl = value_of(arg_matches, "rpc_url").unwrap();
            command_info(rpc_client, rpcurl, program_id, vesting_seed)
        }
        _ => unreachable!(),
    };
}
