use crate::error::VestingError;

use solana_program::program_error::ProgramError;

use std::mem::size_of;
use std::convert::TryInto;

pub enum VestingInstruction {
    Lock {
        amount: u64,
        release_height: u64
    },
    Unlock
}

impl VestingInstruction {


    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use VestingError::InvalidInstruction;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let release_height = rest
                .get(8..16)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(InvalidInstruction)?;
                Self::Lock { amount, release_height }
            }
            1 => Self::Unlock,
            _ => return Err(InvalidInstruction.into())
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Lock { amount, release_height } => {
                buf.push(0);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&release_height.to_le_bytes());
            }
            Self::Unlock => buf.push(1)
        };
        buf
    }
}