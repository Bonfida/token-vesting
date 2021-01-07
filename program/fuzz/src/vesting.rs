use token_vesting::instruction;

use spl_token::error::TokenError;

use honggfuzz::fuzz;

use arbitrary::Arbitrary;
use std::{collections::{HashMap, HashSet}, num};

use instruction::{VestingInstruction, change_destination, create, unlock, init};

#[derive(Debug, Arbitrary, Clone)]
struct FuzzInstruction {
    system_program_id: AccountId,
    vesting_program_id: AccountId,
    token_program_id: AccountId,
    vesting_account_key: AccountId,
    vesting_token_account_key: AccountId,
    source_token_account_owner_key: AccountId,
    source_token_account_key: AccountId,
    destination_token_account_key: AccountId,
    mint_address: AccountId,
    schedules: Vec<Schedule>,   // TODO as u8
    payer_key: AccountId,
    vesting_program_account: AccountId,
    seeds:[u8; 32],
    number_of_schedules: u64,
    instruction: instruction::VestingInstruction
}

/// Use u8 as an account id to simplify the address space and re-use accounts
/// more often.
type AccountId = u8;

const INITIAL_SWAP_TOKEN_A_AMOUNT: u64 = 100_000_000_000;
const INITIAL_SWAP_TOKEN_B_AMOUNT: u64 = 300_000_000_000;

const INITIAL_USER_TOKEN_A_AMOUNT: u64 = 1_000_000_000;
const INITIAL_USER_TOKEN_B_AMOUNT: u64 = 3_000_000_000;

fn main() {
    loop {
        fuzz!(|fuzz_instructions: Vec<FuzzInstruction>| {
            run_fuzz_instructions(fuzz_instructions)
        });
    }
}

fn run_fuzz_instructions(fuzz_instructions: Vec<FuzzInstruction>) {
    let swap_curve = SwapCurve {
        curve_type: CurveType::ConstantProduct,
        calculator: Box::new(ConstantProductCurve {}),
    };
    let mut token_swap = NativeTokenSwap::new(
        fees,
        swap_curve,
        INITIAL_SWAP_TOKEN_A_AMOUNT,
        INITIAL_SWAP_TOKEN_B_AMOUNT,
    );

    // keep track of all accounts, including swap accounts
    let mut source_token_accounts: HashMap<AccountId, NativeAccountData> = HashMap::new();
    let mut destination_token_accounts: HashMap<AccountId, NativeAccountData> = HashMap::new();
    let mut vesting_accounts: HashMap<AccountId, NativeAccountData> = HashMap::new();
    let mut payer_accounts: HashMap<AccountId, NativeAccountData> = HashMap::new();

    let pool_tokens = [&token_swap.pool_token_account, &token_swap.pool_fee_account]
        .iter()
        .map(|&x| get_token_balance(x))
        .sum::<u64>() as u128;
    let initial_pool_token_amount =
        pool_tokens + pool_accounts.values().map(get_token_balance).sum::<u64>() as u128;
    let initial_swap_token_a_amount = get_token_balance(&token_swap.token_a_account) as u128;
    let initial_swap_token_b_amount = get_token_balance(&token_swap.token_b_account) as u128;

    // to ensure that we never create or remove base tokens
    let before_total_token_a =
        INITIAL_SWAP_TOKEN_A_AMOUNT + get_total_token_a_amount(&fuzz_instructions);
    let before_total_token_b =
        INITIAL_SWAP_TOKEN_B_AMOUNT + get_total_token_b_amount(&fuzz_instructions);

    for fuzz_instruction in fuzz_instructions {
        run_fuzz_instruction(
            fuzz_instruction,
            &mut token_swap,
            &mut token_a_accounts,
            &mut token_b_accounts,
            &mut pool_accounts,
        );
    }

    let pool_tokens = [&token_swap.pool_token_account, &token_swap.pool_fee_account]
        .iter()
        .map(|&x| get_token_balance(x))
        .sum::<u64>() as u128;
    let pool_token_amount =
        pool_tokens + pool_accounts.values().map(get_token_balance).sum::<u64>() as u128;
    let swap_token_a_amount = get_token_balance(&token_swap.token_a_account) as u128;
    let swap_token_b_amount = get_token_balance(&token_swap.token_b_account) as u128;

    let lost_a_value = initial_swap_token_a_amount * pool_token_amount
        > swap_token_a_amount * initial_pool_token_amount;
    let lost_b_value = initial_swap_token_b_amount * pool_token_amount
        > swap_token_b_amount * initial_pool_token_amount;
    assert!(!(lost_a_value && lost_b_value));

    // check total token a and b amounts
    let after_total_token_a = token_a_accounts
        .values()
        .map(get_token_balance)
        .sum::<u64>()
        + get_token_balance(&token_swap.token_a_account);
    assert_eq!(before_total_token_a, after_total_token_a);
    let after_total_token_b = token_b_accounts
        .values()
        .map(get_token_balance)
        .sum::<u64>()
        + get_token_balance(&token_swap.token_b_account);
    assert_eq!(before_total_token_b, after_total_token_b);
}

fn run_fuzz_instruction(
    fuzz_instruction: FuzzInstruction,
    source_token_accounts: &mut HashMap<AccountId, NativeAccountData>,
    destination_token_accounts: &mut HashMap<AccountId, NativeAccountData>,
    vesting_accounts: &mut HashMap<AccountId, NativeAccountData>,
    payer_accounts: &mut HashMap<AccountId, NativeAccountData>,
    pool_accounts: &mut HashMap<AccountId, NativeAccountData>,
) {
    let result = match fuzz_instruction {
        FuzzInstruction {
            system_program_id,
            vesting_program_id,
            payer_key,
            vesting_account_key,
            instruction: VestingInstruction::Init { seeds, number_of_schedules },
            ..
        } => {
            let mut payer_account = payer_accounts.get_mut(&payer_key).unwrap();
            let mut vesting_account = vesting_accounts.get_mut(&payer_key).unwrap();
            do_process_instruction(
                init(
                    &token_vesting_env.system_program_account.key,
                    &token_vesting_env.vesting_program_account.key,
                    &payer_account.key,
                    &vesting_account.key,
                    seeds,
                    number_of_schedules
                ).unwrap(),
                &[
                    token_vesting_env.system_program_account.as_account_info(),
                    payer_account.as_account_info(),
                    vesting_account.as_account_info(),
                ],
            );
        },        
        FuzzInstruction {
            system_program_id,
            vesting_program_id,
            payer_key,
            vesting_program_account,
            instruction: VestingInstruction::Create { seeds, mint_address, destination_token_address, schedules },
            ..
        } => {
            do_process_instruction(
                create(    
                    system_program_id,
                    vesting_program_id,
                    payer_key,
                    vesting_program_account,
                    seeds,
                    number_of_schedules
                ).unwrap(),
                &[
                    token_a_account.as_account_info(),
                    self.authority_account.as_account_info(),
                    self.user_account.as_account_info(),
                ],
            );
        },
        FuzzInstruction {
            system_program_id,
            vesting_program_id,
            payer_key,
            vesting_program_account,
            instruction: VestingInstruction::Unlock { seeds },
            ..
        } => {
            do_process_instruction(
                unlock(    
                    system_program_id,
                    vesting_program_id,
                    payer_key,
                    vesting_program_account,
                    seeds,
                    number_of_schedules
                ).unwrap(),
                &[
                    token_a_account.as_account_info(),
                    self.authority_account.as_account_info(),
                    self.user_account.as_account_info(),
                ],
            );
        },
        FuzzInstruction {
            system_program_id,
            vesting_program_id,
            payer_key,
            vesting_program_account,
            instruction: VestingInstruction::ChangeDestination { seeds },
            ..
        } => {
            do_process_instruction(
                change_destination (    
                    system_program_id,
                    vesting_program_id,
                    payer_key,
                    vesting_program_account,
                    seeds,
                    number_of_schedules
                ).unwrap(),
                &[
                    token_a_account.as_account_info(),
                    self.authority_account.as_account_info(),
                    self.user_account.as_account_info(),
                ],
            );
        },
    };
    result
        .map_err(|e| {
            if !(e == SwapError::CalculationFailure.into()
                || e == SwapError::ConversionFailure.into()
                || e == SwapError::FeeCalculationFailure.into()
                || e == SwapError::ExceededSlippage.into()
                || e == SwapError::ZeroTradingTokens.into()
                || e == TokenError::InsufficientFunds.into())
            {
                Err(e).unwrap()
            }
        })
        .ok();
}
