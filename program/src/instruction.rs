use crate::error::VestingError;

use solana_program::{program_error::ProgramError, pubkey::Pubkey};

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
    ///   0. `[]` The vesting account.
    ///   1. `[writable]` The vesting spl-token account
    ///   2. `[signer]` The source spl-token account owner.
    ///   3. `[writable]` The source spl-token account
    ///   4. `[]` The destination spl-token account
    Create {
        seeds: [u8; 32],
        amount: u64,
        release_height: u64,
        mint_address: Pubkey
    },

    // /// Unlocks a simple vesting contract (SVC) - can only be invoked by the program itself
    // /// TODO only program ?
    // /// Accounts expected by this instruction:
    // ///
    // ///   * Single owner
    // ///   0. `[]` The vesting account.
    // ///   1. `[writable]` The vesting spl-token account.
    // ///   2. `[writable]` The destination spl-token account.
    Unlock {
        seeds: [u8; 32]
    },

    // /// Change the destination account of a given simple vesting contract (SVC)
    // /// - can only be invoked by the present destination address of the contract.
    // ///
    // /// Accounts expected by this instruction:
    // ///
    // ///   * Single owner
    // ///   0. `[]` The vesting account.
    // ///   1. `[signer]` The destination spl-token account owner.
    // ///   2. `[]` The new destination spl-token account.
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
                buf.push(2);
                buf.extend_from_slice(&seeds);
            }
        };
        buf
    }
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