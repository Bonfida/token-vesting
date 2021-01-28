use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::PrintProgramError,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::{clock::Clock, Sysvar},
};

use num_traits::FromPrimitive;
use spl_token::{instruction::transfer, state::Account};

use crate::{
    error::VestingError,
    instruction::{Schedule, VestingInstruction, SCHEDULE_SIZE},
    state::{pack_schedules_into_slice, unpack_schedules, VestingSchedule, VestingScheduleHeader},
};

pub struct Processor {}

impl Processor {
    pub fn process_init(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seeds: [u8; 32],
        schedules: u32
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let system_program_account = next_account_info(accounts_iter)?;
        let rent_sysvar_account = next_account_info(accounts_iter)?;
        let payer = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;

        let rent = Rent::from_account_info(rent_sysvar_account)?;

        // Find the non reversible public key for the vesting contract via the seed
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
        if vesting_account_key != *vesting_account.key {
            msg!("Provided vesting account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        let state_size = (schedules as usize) * VestingSchedule::LEN + VestingScheduleHeader::LEN;

        let init_vesting_account = create_account(
            &payer.key,
            &vesting_account_key,
            rent.minimum_balance(state_size),
            state_size as u64,
            &program_id,
        );

        invoke_signed(
            &init_vesting_account,
            &[
                system_program_account.clone(),
                payer.clone(),
                vesting_account.clone(),
            ],
            &[&[&seeds]],
        )?;
        Ok(())
    }

    pub fn process_create(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seeds: [u8; 32],
        mint_address: &Pubkey,
        destination_token_address: &Pubkey,
        schedules: Vec<Schedule>,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let spl_token_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let source_token_account_owner = next_account_info(accounts_iter)?;
        let source_token_account = next_account_info(accounts_iter)?;

        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Provided vesting account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        if !source_token_account_owner.is_signer {
            msg!("Source token account owner should be a signer.");
            return Err(ProgramError::InvalidArgument);
        }

        if *vesting_account.owner != *program_id {
            msg!("Program should own vesting account");
            return Err(ProgramError::InvalidArgument);
        }

        // Verifying that no SVC was already created with this seed
        let is_initialized =
            vesting_account.try_borrow_data()?[VestingScheduleHeader::LEN - 1] == 1;

        if is_initialized {
            msg!("Cannot overwrite an existing vesting contract.");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        let state_header = VestingScheduleHeader {
            destination_address: *destination_token_address,
            mint_address: *mint_address,
            is_initialized: true,
        };

        let mut data = vesting_account.data.borrow_mut();
        if data.len() != VestingScheduleHeader::LEN + schedules.len() * VestingSchedule::LEN {
            return Err(ProgramError::InvalidAccountData)
        }
        state_header.pack_into_slice(&mut data);

        let mut offset = VestingScheduleHeader::LEN;
        let mut total_amount: u64 = 0;

        for s in schedules.iter() {
            let state_schedule = VestingSchedule {
                release_time: s.release_time,
                amount: s.amount,
            };
            state_schedule.pack_into_slice(&mut data[offset..]);
            let delta = total_amount.checked_add(s.amount);
            match delta {
                Some(n) => total_amount = n,
                None => return Err(ProgramError::InvalidInstructionData), // Total amount overflows u64
            }
            offset += SCHEDULE_SIZE;
        }
        
        if Account::unpack(&source_token_account.data.borrow())?.amount < total_amount {
            msg!("The source token account has insufficient funds.");
            return Err(ProgramError::InsufficientFunds)
        };

        let transfer_tokens_to_vesting_account = transfer(
            spl_token_account.key,
            source_token_account.key,
            vesting_token_account.key,
            source_token_account_owner.key,
            &[],
            total_amount,
        )?;

        invoke(
            &transfer_tokens_to_vesting_account,
            &[
                source_token_account.clone(),
                vesting_token_account.clone(),
                spl_token_account.clone(),
                source_token_account_owner.clone(),
            ],
        )?;
        Ok(())
    }

    pub fn process_unlock(
        program_id: &Pubkey,
        _accounts: &[AccountInfo],
        seeds: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut _accounts.iter();

        let spl_token_account = next_account_info(accounts_iter)?;
        let clock_sysvar_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let destination_token_account = next_account_info(accounts_iter)?;

        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument);
        }

        let packed_state = &vesting_account.data;
        let header_state =
            VestingScheduleHeader::unpack(&packed_state.borrow()[..VestingScheduleHeader::LEN])?;

        if header_state.destination_address != *destination_token_account.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        // Unlock the schedules that have reached maturity
        let clock = Clock::from_account_info(&clock_sysvar_account)?;
        let mut total_amount_to_transfer = 0;
        let mut schedules = unpack_schedules(&packed_state.borrow()[VestingScheduleHeader::LEN..])?;

        for s in schedules.iter_mut() {
            if clock.unix_timestamp as u64 >= s.release_time {
                total_amount_to_transfer += s.amount;
                s.amount = 0;
            }
        }
        if total_amount_to_transfer == 0 {
            msg!("Vesting contract has not yet reached release time");
            return Err(ProgramError::InvalidArgument);
        }

        let transfer_tokens_from_vesting_account = transfer(
            &spl_token_account.key,
            &vesting_token_account.key,
            destination_token_account.key,
            &vesting_account_key,
            &[],
            total_amount_to_transfer,
        )?;

        invoke_signed(
            &transfer_tokens_from_vesting_account,
            &[
                spl_token_account.clone(),
                vesting_token_account.clone(),
                destination_token_account.clone(),
                vesting_account.clone(),
            ],
            &[&[&seeds]],
        )?;

        // Reset released amounts to 0. This makes the simple unlock safe with complex scheduling contracts
        pack_schedules_into_slice(
            schedules,
            &mut packed_state.borrow_mut()[VestingScheduleHeader::LEN..],
        );

        Ok(())
    }

    pub fn process_change_destination(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seeds: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let vesting_account = next_account_info(accounts_iter)?;
        let destination_token_account = next_account_info(accounts_iter)?;
        let destination_token_account_owner = next_account_info(accounts_iter)?;
        let new_destination_token_account = next_account_info(accounts_iter)?;

        if vesting_account.data.borrow().len() < VestingScheduleHeader::LEN {
            return Err(ProgramError::InvalidAccountData)
        }
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        let state = VestingScheduleHeader::unpack(
            &vesting_account.data.borrow()[..VestingScheduleHeader::LEN],
        )?;

        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument);
        }

        if state.destination_address != *destination_token_account.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument);
        }

        if !destination_token_account_owner.is_signer {
            msg!("Destination token account owner should be a signer.");
            return Err(ProgramError::InvalidArgument);
        }

        let destination_token_account = Account::unpack(&destination_token_account.data.borrow())?;

        if destination_token_account.owner != *destination_token_account_owner.key {
            msg!("The current destination token account isn't owned by the provided owner");
            return Err(ProgramError::InvalidArgument);
        }

        let mut new_state = state;
        new_state.destination_address = *new_destination_token_account.key;
        new_state
            .pack_into_slice(&mut vesting_account.data.borrow_mut()[..VestingScheduleHeader::LEN]);

        Ok(())
    }

    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = VestingInstruction::unpack(instruction_data)?;
        msg!("Instruction unpacked");
        match instruction {
            VestingInstruction::Init {
                seeds,
                number_of_schedules,
            } => {
                msg!("Instruction: Init");
                Self::process_init(program_id, accounts, seeds, number_of_schedules)
            }
            VestingInstruction::Unlock { seeds } => {
                msg!("Instruction: Unlock");
                Self::process_unlock(program_id, accounts, seeds)
            }
            VestingInstruction::ChangeDestination { seeds } => {
                msg!("Instruction: Change Destination");
                Self::process_change_destination(program_id, accounts, seeds)
            }
            VestingInstruction::Create {
                seeds,
                mint_address,
                destination_token_address,
                schedules,
            } => {
                msg!("Instruction: Create Schedule");
                Self::process_create(
                    program_id,
                    accounts,
                    seeds,
                    &mint_address,
                    &destination_token_address,
                    schedules,
                )
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
            VestingError::InvalidInstruction => msg!("Error: Invalid instruction!"),
        }
    }
}
