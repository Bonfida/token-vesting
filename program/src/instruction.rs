use crate::error::VestingError;

use solana_program::{program_error::ProgramError, msg};

use std::mem::size_of;
use std::convert::TryInto;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
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
        msg!("Received : {:?}", input);
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        msg!("Parsed tag : {:?}", tag);
        Ok(match tag {
            0 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                msg!("Parsed amount");
                let release_height = rest
                .get(8..16)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(InvalidInstruction)?;
                msg!("Parsed release_height");
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instruction_packing(){
        let check = VestingInstruction::Lock {
            amount: 42,
            release_height: 250
        };
        let expected = Vec::from([0u8, 42, 0, 0, 0, 0, 0, 0, 0, 250, 0, 0, 0, 0, 0, 0, 0]);
        let packed = check.pack();
        assert_eq!(expected, packed);
        let unpacked = VestingInstruction::unpack(&packed).unwrap();
        assert_eq!(check, unpacked);
    }

}