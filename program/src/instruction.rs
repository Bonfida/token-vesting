use crate::error::VestingError;

use solana_program::{instruction::{AccountMeta, Instruction}, msg, program_error::ProgramError, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;

use std::mem::size_of;
use std::convert::TryInto;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Schedule {
    pub release_height: u64,
    pub amount: u64
}

pub const SCHEDULE_SIZE:usize = 16;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum VestingInstruction {
    /// Initializes an empty program account for the token_vesting program
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The system program account
    ///   1. `[writable]` The source account (fee payer)
    Init {
        seeds: [u8; 32]
    },
    /// Creates a new simple vesting contract (SVC)
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The spl-token program account
    ///   1. `[writable]` The vesting account
    ///   2. `[writable]` The vesting spl-token account
    ///   3. `[signer]` The source spl-token account owner
    ///   4. `[writable]` The source spl-token account
    Create {
        seeds: [u8; 32],
        amount: u64,
        release_height: u64,
        mint_address: Pubkey,
        destination_token_address: Pubkey
    },
    /// Creates a new vesting schedule contract
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The spl-token program account
    ///   1. `[writable]` The vesting account
    ///   2. `[writable]` The vesting spl-token account
    ///   3. `[signer]` The source spl-token account owner
    ///   4. `[writable]` The source spl-token account
    CreateSchedule {
        seeds: [u8; 32],
        mint_address: Pubkey,
        destination_token_address: Pubkey,
        schedules: Vec<Schedule>
    },
    /// Unlocks a simple vesting contract (SVC) - can only be invoked by the program itself
    /// TODO only program ?
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The spl-token program account
    ///   1. `[]` The clock sysvar account
    ///   1. `[writable]` The vesting account
    ///   2. `[writable]` The vesting spl-token account
    ///   3. `[writable]` The destination spl-token account
    Unlock {
        seeds: [u8; 32]
    },

    /// Change the destination account of a given simple vesting contract (SVC)
    /// - can only be invoked by the present destination address of the contract.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The vesting account
    ///   1. `[]` The current destination token account
    ///   2. `[signer]` The destination spl-token account owner
    ///   3. `[]` The new destination spl-token account
    ChangeDestination {
        seeds: [u8; 32]
    }
}

impl VestingInstruction {


    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use VestingError::InvalidInstruction;
        // msg!("Received : {:?}", input);
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        // msg!("Parsed tag : {:?}", tag);
        Ok(match tag {
            0 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                Self::Init { seeds }},
            1 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                let amount = rest
                    .get(32..40)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                // msg!("Parsed amount");
                let release_height = rest
                .get(40..48)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(InvalidInstruction)?;

                let mint_address = rest
                .get(48..80)
                .and_then(|slice| slice.try_into().ok())
                .map(Pubkey::new)
                .ok_or(InvalidInstruction)?;
                // msg!("Parsed release_height");

                let destination_token_address = rest
                .get(80..112)
                .and_then(|slice| slice.try_into().ok())
                .map(Pubkey::new)
                .ok_or(InvalidInstruction)?;
                Self::Create { seeds , amount, release_height, mint_address , destination_token_address}

            },
            2 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                let mint_address = rest
                    .get(32..64)
                    .and_then(|slice| slice.try_into().ok())
                    .map(Pubkey::new)
                    .ok_or(InvalidInstruction)?;
                let destination_token_address = rest
                    .get(64..96)
                    .and_then(|slice| slice.try_into().ok())
                    .map(Pubkey::new)
                    .ok_or(InvalidInstruction)?;
                let number_of_schedules = rest[96..].len()/SCHEDULE_SIZE;
                let mut schedules:Vec<Schedule> = Vec::with_capacity(number_of_schedules);
                let mut offset = 96;
                for i in 0..number_of_schedules {
                    let release_height = rest
                        .get(offset..offset+8)
                        .and_then(|slice| slice.try_into().ok())
                        .map(u64::from_le_bytes)
                        .ok_or(InvalidInstruction)?;
                    let amount = rest
                            .get(offset+8..offset+16)
                            .and_then(|slice| slice.try_into().ok())
                            .map(u64::from_le_bytes)
                            .ok_or(InvalidInstruction)?;
                    offset += SCHEDULE_SIZE;
                    schedules.push(Schedule { release_height, amount })
                }
                Self::CreateSchedule{ seeds, mint_address, destination_token_address, schedules }
            },
            3 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                Self::Unlock { seeds }},
            4 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                Self::ChangeDestination { seeds }},
            _ => {
                msg!("Unsupported tag");
                return Err(InvalidInstruction.into())
            }
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Init {seeds} => {
                buf.push(0);
                buf.extend_from_slice(&seeds);
            }
            &Self::Create {seeds, amount, release_height , mint_address, destination_token_address} => {
                buf.push(1);
                buf.extend_from_slice(&seeds);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&release_height.to_le_bytes());
                buf.extend_from_slice(&mint_address.to_bytes());
                buf.extend_from_slice(&destination_token_address.to_bytes());
            }
            Self::CreateSchedule{seeds, mint_address, destination_token_address, schedules} => {
                buf.push(2);
                buf.extend_from_slice(seeds);
                buf.extend_from_slice(&mint_address.to_bytes());
                buf.extend_from_slice(&destination_token_address.to_bytes());
                for s in schedules.iter(){
                    buf.extend_from_slice(&s.release_height.to_le_bytes());
                    buf.extend_from_slice(&s.amount.to_le_bytes());
                }
            }
            &Self::Unlock {seeds} => {
                buf.push(3);
                buf.extend_from_slice(&seeds);
            }
            &Self::ChangeDestination {seeds} => {
                buf.push(4);
                buf.extend_from_slice(&seeds);
            }
        };
        buf
    }
}

// Creates a `Init` instruction
pub fn init(
    system_program_id: &Pubkey,
    vesting_program_id: &Pubkey,
    source_token_account_owner_key: &Pubkey,
    vesting_program_account: &Pubkey,
    seeds:[u8; 32]
) -> Result<Instruction, ProgramError> {
    let data = VestingInstruction::Init{seeds}.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*system_program_id, false),
        AccountMeta::new(*source_token_account_owner_key, true),
        AccountMeta::new(*vesting_program_account, false)
    ];
    Ok(Instruction {
        program_id: *vesting_program_id,
        accounts,
        data
    })
}

// Creates a `Create` instruction
pub fn create(
    vesting_program_id: &Pubkey,
    token_program_id: &Pubkey,
    vesting_account_key: &Pubkey,
    vesting_token_account_key: &Pubkey,
    source_token_account_owner_key: &Pubkey,
    source_token_account_key: &Pubkey,
    destination_token_account_key: &Pubkey,
    mint_address: &Pubkey,
    amount:u64,
    release_height:u64,
    seeds:[u8; 32]
) -> Result<Instruction, ProgramError> {
    let data = VestingInstruction::Create { 
        amount, mint_address: *mint_address, release_height, seeds, destination_token_address: *destination_token_account_key 
    }.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new(*vesting_account_key, false),
        AccountMeta::new(*vesting_token_account_key, false),
        AccountMeta::new_readonly(*source_token_account_owner_key, true),
        AccountMeta::new(*source_token_account_key, false)
    ];
    Ok(Instruction {
        program_id: *vesting_program_id,
        accounts,
        data
    })

}

// Creates a `CreateSchedule` instruction
pub fn create_schedule(
    vesting_program_id: &Pubkey,
    token_program_id: &Pubkey,
    vesting_account_key: &Pubkey,
    vesting_token_account_key: &Pubkey,
    source_token_account_owner_key: &Pubkey,
    source_token_account_key: &Pubkey,
    destination_token_account_key: &Pubkey,
    mint_address: &Pubkey,
    schedules: Vec<Schedule>,
    seeds:[u8; 32]
) -> Result<Instruction, ProgramError> {
    let data = VestingInstruction::CreateSchedule { 
        mint_address: *mint_address, seeds, destination_token_address: *destination_token_account_key, schedules
    }.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new(*vesting_account_key, false),
        AccountMeta::new(*vesting_token_account_key, false),
        AccountMeta::new_readonly(*source_token_account_owner_key, true),
        AccountMeta::new(*source_token_account_key, false)
    ];
    Ok(Instruction {
        program_id: *vesting_program_id,
        accounts,
        data
    })

}

// Creates an `Unlock` instruction
pub fn unlock(
    vesting_program_id: &Pubkey,
    token_program_id: &Pubkey,
    clock_sysvar_id: &Pubkey,
    vesting_account_key: &Pubkey,
    vesting_token_account_key: &Pubkey,
    destination_token_account_key: &Pubkey,
    seeds: [u8;32]
) -> Result<Instruction, ProgramError> {
    let data = VestingInstruction::Unlock { seeds }.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(*clock_sysvar_id, false),
        AccountMeta::new(*vesting_account_key, false),
        AccountMeta::new(*vesting_token_account_key, false),
        AccountMeta::new(*destination_token_account_key, false)
    ];
    Ok(Instruction {
        program_id: *vesting_program_id,
        accounts,
        data
    })
}


pub fn change_destination(
    vesting_program_id: &Pubkey,
    vesting_account_key: &Pubkey,
    current_destination_token_account_owner: &Pubkey,
    current_destination_token_account: &Pubkey,
    target_destination_token_account: &Pubkey,
    seeds:[u8;32],
) -> Result<Instruction, ProgramError> {
    let data = VestingInstruction::ChangeDestination { seeds }.pack();
    let accounts = vec![
        AccountMeta::new(*vesting_account_key, false),
        AccountMeta::new_readonly(*current_destination_token_account, false),
        AccountMeta::new_readonly(*current_destination_token_account_owner, true),
        AccountMeta::new_readonly(*target_destination_token_account, false),
    ];
    Ok(Instruction {
        program_id: *vesting_program_id,
        accounts,
        data
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instruction_packing(){
        let mint_address = Pubkey::new_unique();
        let destination_token_address = Pubkey::new_unique();
        let check = VestingInstruction::Create {
            seeds: [50u8;32],
            amount: 42,
            release_height: 250,
            mint_address: mint_address.clone(),
            destination_token_address
        };
        let mut expected = Vec::from([1]);
        let seeds = [50u8;32];
        let data = [42, 0, 0, 0, 0, 0, 0, 0, 250, 0, 0, 0, 0, 0, 0, 0];
        expected.extend_from_slice(&seeds);
        expected.extend_from_slice(&data);
        expected.extend_from_slice(&mint_address.to_bytes());
        expected.extend_from_slice(&destination_token_address.to_bytes());
        let packed = check.pack();
        assert_eq!(expected, packed);
        let unpacked = VestingInstruction::unpack(&packed).unwrap();
        assert_eq!(check, unpacked);
    }

}