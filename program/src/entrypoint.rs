use solana_program::{
    entrypoint, 
    entrypoint::ProgramResult, 
    pubkey::Pubkey, 
    account_info::{AccountInfo}};

use crate::processor::Processor;

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process_instruction(program_id, accounts, instruction_data)
}