// #![cfg(all(target_arch = "bpf", not(feature = "no-entrypoint")))]

use solana_program::{account_info::{AccountInfo}, entrypoint, entrypoint::ProgramResult, msg, program_error::PrintProgramError, pubkey::Pubkey};

use crate::{error::VestingError, processor::Processor};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<VestingError>();
        return Err(error);
    }
    Ok(())
}

// solana_program::declare_id!("VestingbGKPFXCWuBvfkegQfZyiNwAJb9Ss623VQ5DA");
