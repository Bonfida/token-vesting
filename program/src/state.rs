use solana_program::{pubkey::Pubkey, program_error::ProgramError};

use std::convert::TryInto;

pub const STATE_SIZE:usize = 72;

pub struct VestingState {
    // A destination token address
    pub destination_address : Pubkey,
    pub mint_address : Pubkey,
    pub release_height: u64,
}

impl VestingState {
    pub fn pack_into(&self, target: &mut [u8;STATE_SIZE]){
        let destination_address_bytes = self.destination_address.to_bytes();
        let mint_address_bytes = self.mint_address.to_bytes();
        let release_height_bytes = self.release_height.to_le_bytes();
        for i in 0..32 {
            target[i] = destination_address_bytes[i];
        }

        for i in 32..64 {
            target[i] = mint_address_bytes[i-32];
        }

        for i in 64..72 {
            target[i] = release_height_bytes[i-64];
        }
    }

    pub fn pack(&self) -> [u8;STATE_SIZE]{
        let mut packed = [0u8;STATE_SIZE];
        self.pack_into(&mut packed);
        packed
    }

    pub fn unpack(input: &[u8])-> Result<Self, ProgramError>{
        let destination_address = Pubkey::new(&input[..32]);
        let mint_address = Pubkey::new(&input[32..64]);
        let release_height = u64::from_le_bytes(input[64..].try_into().unwrap());
        Ok(Self {destination_address, mint_address, release_height})
    }
}