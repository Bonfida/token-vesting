use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Token vesting program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum VestingError {
    // Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,

    // #[error("Invalid Instruction")]
    // InvalidInstruction,
}


impl From<VestingError> for ProgramError {
    fn from(e: VestingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}


impl<T> DecodeError<T> for VestingError {
    fn type_of() -> &'static str {
        "VestingError"
    }
}