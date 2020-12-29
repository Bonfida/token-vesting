use solana_program::{pubkey::Pubkey, program_error::ProgramError};

use std::convert::TryInto;

pub struct VestingState {
    pub destination_address : Pubkey,
    pub release_height: u64
}

impl VestingState {
    pub fn pack_into(&self, target: &mut [u8;40]){
        let destination_address_bytes = self.destination_address.to_bytes();
        let release_height_bytes = self.release_height.to_le_bytes();
        for i in 0..32 {
            target[i] = destination_address_bytes[i];
        }
        for i in 32..40 {
            target[i] = release_height_bytes[i-32];
        }
    }

    pub fn pack(&self) -> [u8;40]{
        let mut packed = [0u8;40];
        self.pack_into(&mut packed);
        packed
    }

    pub fn unpack(input: &[u8])-> Result<Self, ProgramError>{
        let destination_address = Pubkey::new(&input[..32]);
        let release_height = u64::from_le_bytes(input[32..].try_into().unwrap());
        Ok(Self {destination_address, release_height})
    }
}