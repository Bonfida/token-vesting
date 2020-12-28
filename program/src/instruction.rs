use crate::error::VestingError;

use solana_program::{program_error::ProgramError, msg};

use std::mem::size_of;
use std::convert::TryInto;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum VestingInstruction {
    Init {
        seeds: [u8; 32]
    },
    Lock {
        seeds: [u8; 32],
        amount: u64,
        release_height: u64
    },
    Unlock {
        seeds: [u8; 32]
    }
}

impl VestingInstruction {


    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use VestingError::InvalidInstruction;
        msg!("Received : {:?}", input);
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        msg!("Parsed tag : {:?}", tag);
        Ok(match tag {
            0 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                VestingInstruction::Init{ seeds }
            }
            1 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                let amount = rest
                    .get(32..40)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                msg!("Parsed amount");
                let release_height = rest
                .get(40..48)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(InvalidInstruction)?;
                msg!("Parsed release_height");
                Self::Lock { seeds , amount, release_height }
            }
            2 => {
                let seeds:[u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok()).unwrap();
                Self::Unlock { seeds }},
            _ => return Err(InvalidInstruction.into())
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Init { seeds} => {
                buf.push(0);
                buf.extend_from_slice(&seeds);
            }
            &Self::Lock { seeds, amount, release_height } => {
                buf.push(1);
                buf.extend_from_slice(&seeds);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&release_height.to_le_bytes());
            }
            &Self::Unlock { seeds} => {
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
        let check = VestingInstruction::Lock {
            seeds: [50u8;32],
            amount: 42,
            release_height: 250
        };
        let mut expected = Vec::from([1]);
        let seeds = [50u8;32];
        let data = [42, 0, 0, 0, 0, 0, 0, 0, 250, 0, 0, 0, 0, 0, 0, 0];
        expected.extend_from_slice(&seeds);
        expected.extend_from_slice(&data);
        let packed = check.pack();
        assert_eq!(expected, packed);
        let unpacked = VestingInstruction::unpack(&packed).unwrap();
        assert_eq!(check, unpacked);
    }

}