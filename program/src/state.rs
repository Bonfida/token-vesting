use solana_program::{pubkey::Pubkey, program_error::ProgramError};

use std::convert::TryInto;

pub const STATE_SIZE:usize = 81;

pub struct VestingParameters {
    // A destination token address
    pub destination_address : Pubkey,
    pub mint_address : Pubkey,
    pub release_height: u64,
    pub amount: u64,
    pub is_initialized: bool
}

impl VestingParameters {
    pub fn pack_into(&self, target: &mut [u8;STATE_SIZE]){
        let destination_address_bytes = self.destination_address.to_bytes();
        let mint_address_bytes = self.mint_address.to_bytes();
        let release_height_bytes = self.release_height.to_le_bytes();
        let amount_bytes = self.amount.to_le_bytes();
        for i in 0..32 {
            target[i] = destination_address_bytes[i];
        }

        for i in 32..64 {
            target[i] = mint_address_bytes[i-32];
        }

        for i in 64..72 {
            target[i] = release_height_bytes[i-64];
        }

        for i in 72..64 {
            target[i] = amount_bytes[i-64];
        }
        target[72] = self.is_initialized as u8;
    }

    pub fn pack(&self) -> [u8;STATE_SIZE]{
        let mut packed = [0u8;STATE_SIZE];
        self.pack_into(&mut packed);
        packed
    }

    pub fn unpack(input: &[u8])-> Result<Self, ProgramError>{
        let destination_address = Pubkey::new(&input[..32]);
        let mint_address = Pubkey::new(&input[32..64]);
        let release_height = u64::from_le_bytes(input[64..72].try_into().unwrap());
        let amount = u64::from_le_bytes(input[72..80].try_into().unwrap());
        let is_initialized = input[72] == 1;
        Ok(Self {destination_address, mint_address, release_height, amount, is_initialized})
    }
}

#[cfg(test)]
mod tests {
    use super::{VestingParameters, STATE_SIZE};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_state_packing(){
        let state = VestingParameters{
            destination_address: Pubkey::new_unique(),
            mint_address: Pubkey::new_unique(),
            release_height: 30767976,
            amount: 500,
            is_initialized: true
        };
        let packed = Vec::from(state.pack());
        let mut expected = Vec::with_capacity(STATE_SIZE);
        expected.extend_from_slice(&state.destination_address.to_bytes());
        expected.extend_from_slice(&state.mint_address.to_bytes());
        expected.extend_from_slice(&state.release_height.to_le_bytes());
        expected.extend_from_slice(&state.amount.to_le_bytes());
        expected.extend_from_slice(&[state.is_initialized as u8]);

        assert_eq!(expected, packed);
        assert_eq!(packed.len(), STATE_SIZE);
    }
}