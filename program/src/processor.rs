use solana_program::{
    entrypoint::ProgramResult, 
    pubkey::Pubkey, 
    account_info::{AccountInfo, next_account_info},
    msg,
    program_error::{PrintProgramError},
    decode_error::DecodeError
};
use num_traits::FromPrimitive;

use crate::{instruction::VestingInstruction, error::VestingError};

pub struct Processor {}

impl Processor {
    pub fn process_lock(accounts: &[AccountInfo], amount: u64, _release_height: u64) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let source_account = next_account_info(accounts_iter)?;
        let destination_account = next_account_info(accounts_iter)?;

        **source_account.try_borrow_mut_lamports()? -= amount;
        **destination_account.try_borrow_mut_lamports()? += amount;

        Ok(())
    }

    pub fn process_unlock(_accounts: &[AccountInfo]) -> ProgramResult {
        Ok(())
    }

    pub fn process_instruction(_program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = VestingInstruction::unpack(instruction_data)?;
        msg!("Instruction unpacked");
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

impl PrintProgramError for VestingError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            VestingError::InvalidInstruction => msg!("Error: Invalid instruction!")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor(){

        let source_account = Pubkey::new_unique();
        let mut source_lamports = 42u64;
        let mut destination_lamports = 10u64;
        let destination_account = Pubkey::new_unique();
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();


        let accounts = vec![
            AccountInfo::new(
                &source_account,
                true,
                true,
                &mut source_lamports,
                &mut [],
                &owner,
                false,
                7000
            ),
            AccountInfo::new(
                &destination_account,
                true,
                true,
                &mut destination_lamports,
                &mut [],
                &owner,
                false,
                7000
            )
        ];
        Processor::process_instruction(
            &program_id,
            &accounts,
            &VestingInstruction::Lock {amount: 5, release_height: 0}.pack()
        ).unwrap();
        assert_eq!(source_lamports, 37);
        assert_eq!(destination_lamports, 15);
    }
}