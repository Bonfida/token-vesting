use solana_program::{
    entrypoint, 
    entrypoint::ProgramResult, 
    pubkey::Pubkey, 
    account_info::{AccountInfo},
    program_error::PrintProgramError
};

use crate::{error::VestingError, processor::Processor};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<VestingError>();
        return Err(error);
    }
    Ok(())
}