use solana_program::{
    entrypoint::ProgramResult, 
    pubkey::Pubkey, 
    account_info::{AccountInfo},
    msg,
    clock::Clock
};

use crate::instruction::VestingInstruction;

pub struct Processor {}

impl Processor {
    pub fn process_lock(accounts: &[AccountInfo], amount: u64, release_height: u64) -> ProgramResult {
        Ok(())
    }

    pub fn process_unlock(accounts: &[AccountInfo]) -> ProgramResult {
        Ok(())
    }

    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8]
    ) -> ProgramResult {
        let instruction = VestingInstruction::unpack(instruction_data)?;
        match instruction {
            VestingInstruction::Lock { amount, release_height} => {
                msg!("Instruction: Lock");
                Self::process_lock(accounts, amount, release_height)
            }
            VestingInstruction::Unlock => {
                msg!("Instruction: Unlock");
                Self::process_unlock(accounts)
            }

        }
    }
}