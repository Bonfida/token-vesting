use solana_program::{
    account_info::{AccountInfo, next_account_info},
    decode_error::DecodeError,
    program_error::ProgramError,
    entrypoint::ProgramResult,
    system_instruction::allocate,
    program::invoke_signed,
    msg, 
    program_error::{PrintProgramError},
    pubkey::Pubkey,
    sysvar::{Sysvar, clock::Clock}
};
use num_traits::FromPrimitive;

use crate::{instruction::VestingInstruction, error::VestingError};
use arrayref::array_mut_ref;

use std::{cell::RefCell, convert::TryInto, rc::Rc};

pub struct Processor {}

pub const SIZE: usize = 256;

impl Processor {
    pub fn process_lock(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64, release_height: u64) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let self_account = next_account_info(accounts_iter)?;
        let mut program_account = next_account_info(accounts_iter)?;
        let source_account = next_account_info(accounts_iter)?;
        let destination_account = next_account_info(accounts_iter)?;

        let program_account_key = Pubkey::create_program_address(&[&[42, 42]], program_id)?;
        if program_account_key != *program_account.key {
            return Err(ProgramError::InvalidArgument)
        }

        let mut buf = Vec::with_capacity(SIZE);
        buf.extend_from_slice(&destination_account.key.to_bytes());
        buf.extend_from_slice(&release_height.to_le_bytes());
        program_account.data.replace(array_mut_ref![buf]);
        // let mut array: [u8;SIZE] = buf.clone().try_into().unwrap();


        **program_account.try_borrow_mut_lamports()? += amount;
        // let new_program_account = AccountInfo::new(
        //     program_account.key,
        //     program_account.is_signer,
        //     program_account.is_writable,
        //     *program_account.clone().try_borrow_mut_lamports()?,
        //     array,
        //     program_account.owner,
        //     program_account.executable,
        //     program_account.rent_epoch
        // );
        // let new_program_account = edit_account_data(program_account.clone(), array);
        **source_account.try_borrow_mut_lamports()? -= amount;
        // **destination_account.try_borrow_mut_lamports()? += amount;
        invoke_signed(
            &allocate(&program_account_key, SIZE as u64),
            &[
                self_account.clone(),
                program_account.clone()
            ],
            &[&[&[42, 42]]]
        );
        Ok(())
    }

    pub fn process_unlock(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut _accounts.iter();
        let programm_account = next_account_info(accounts_iter)?;
        let receiver_account = next_account_info(accounts_iter)?;

        // Good structure for handling accounts (repack after):
        // let mut source_account = Account::unpack(&source_account_info.data.borrow())?;

        let clock = &Clock::from_account_info(programm_account)?; // Is this the right clock??

        if clock.slot > 3 { //TODO get the slot_height from the Vesting Schedule (master) contract
            **receiver_account.try_borrow_mut_lamports()? += **programm_account.try_borrow_mut_lamports()?;
            **programm_account.try_borrow_mut_lamports()? = 0;
        }
        Ok(())
    }

    pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = VestingInstruction::unpack(instruction_data)?;
        msg!("Instruction unpacked");
        match instruction {
            VestingInstruction::Lock { amount, release_height} => {
                msg!("Instruction: Lock");
                Self::process_lock(program_id, accounts, amount, release_height)
            }
            VestingInstruction::Unlock => {
                msg!("Instruction: Unlock");
                Self::process_unlock(program_id, accounts)
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

fn edit_account_data<'a>(base_account: AccountInfo<'a>, mut data: [u8; 256]) -> AccountInfo<'a> {
    AccountInfo {
        data: Rc::new(RefCell::new(&mut data)),
        ..base_account
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