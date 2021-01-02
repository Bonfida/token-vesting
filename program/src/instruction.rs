use crate::error::VestingError;

use solana_program::{instruction::{AccountMeta, Instruction}, msg, program_error::ProgramError, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;

use std::mem::size_of;
use std::convert::TryInto;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum VestingInstruction {
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
    ///   5. `[]` The destination spl-token account
    Create {
        seeds: [u8; 32],
        amount: u64,
        release_height: u64,
        mint_address: Pubkey
    },  
    /// Unlocks a simple vesting contract (SVC) - can only be invoked by the program itself
    /// TODO only program ?
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The spl-token program account
    ///   1. `[]` The clock sysvar account
    ///   1. `[]` The vesting account
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
            0 | 1 => {
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
                match tag {
                    0 => Self::Create { seeds , amount, release_height, mint_address },
                    // 1 => Self::CreatePrivate { seeds , amount, release_height, mint_address },
                    _ => unreachable!()
                }

            }
            2 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                Self::Unlock { seeds }},
            3 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                Self::ChangeDestination { seeds }},
            _ => return Err(InvalidInstruction.into())
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Create {seeds, amount, release_height , mint_address} => {
                buf.push(0);
                buf.extend_from_slice(&seeds);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&release_height.to_le_bytes());
                buf.extend_from_slice(&mint_address.to_bytes());
            }
            &Self::Unlock {seeds} => {
                buf.push(2);
                buf.extend_from_slice(&seeds);
            }
            &Self::ChangeDestination {seeds} => {
                buf.push(3);
                buf.extend_from_slice(&seeds);
            }
        };
        buf
    }
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
    let data = VestingInstruction::Create { amount, mint_address: *mint_address, release_height, seeds }.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new(*vesting_account_key, false),
        AccountMeta::new(*vesting_token_account_key, false),
        AccountMeta::new_readonly(*source_token_account_owner_key, true),
        AccountMeta::new(*source_token_account_key, false),
        AccountMeta::new_readonly(*destination_token_account_key, false),
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
        AccountMeta::new_readonly(*vesting_account_key, false),
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
        let check = VestingInstruction::Create {
            seeds: [50u8;32],
            amount: 42,
            release_height: 250,
            mint_address: mint_address.clone()
        };
        let mut expected = Vec::from([0]);
        let seeds = [50u8;32];
        let data = [42, 0, 0, 0, 0, 0, 0, 0, 250, 0, 0, 0, 0, 0, 0, 0];
        expected.extend_from_slice(&seeds);
        expected.extend_from_slice(&data);
        expected.extend_from_slice(&mint_address.to_bytes());
        let packed = check.pack();
        assert_eq!(expected, packed);
        let unpacked = VestingInstruction::unpack(&packed).unwrap();
        assert_eq!(check, unpacked);
    }

}