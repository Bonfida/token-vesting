use solana_program::{msg, program_error::ProgramError, program_pack::{IsInitialized, Pack, Sealed}, pubkey::Pubkey};

use std::convert::TryInto;

pub const STATE_SIZE:usize = 81;

pub const HEADER_SIZE:usize = 65;

pub const SCHEDULE_SIZE:usize = 16;

pub const TOTAL_SIZE:usize = STATE_SIZE;

pub struct VestingParameters {
    // A destination token address
    pub destination_address : Pubkey,
    pub mint_address : Pubkey,
    pub release_height: u64,
    pub amount: u64,
    pub is_initialized: bool,
}
pub struct VestingSchedule {
    pub release_height: u64,
    pub amount: u64
}

pub struct VestingScheduleHeader {
    pub destination_address : Pubkey,
    pub mint_address : Pubkey,
    pub is_initialized: bool,
}

impl Sealed for VestingScheduleHeader {}

impl Pack for VestingScheduleHeader {
    const LEN: usize = 65;
    
    fn pack_into_slice(&self, target: &mut [u8]){
        let destination_address_bytes = self.destination_address.to_bytes();
        let mint_address_bytes = self.mint_address.to_bytes();
        for i in 0..32 {
            target[i] = destination_address_bytes[i];
        }

        for i in 32..64 {
            target[i] = mint_address_bytes[i-32];
        }

        target[64] = self.is_initialized as u8;
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let destination_address = Pubkey::new(&src[..32]);
        let mint_address = Pubkey::new(&src[32..64]);
        let is_initialized = src[64] == 1;
        Ok(Self {destination_address, mint_address, is_initialized})
    }
}

impl Sealed for VestingSchedule {}

impl Pack for VestingSchedule {
    const LEN: usize = 16;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let release_height_bytes = self.release_height.to_le_bytes();
        let amount_bytes = self.amount.to_le_bytes();
        for i in 0..8 {
            dst[i] = release_height_bytes[i];
        }

        for i in 8..16 {
            dst[i] = amount_bytes[i-8];
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        msg!("Unpacking schedule");
        let release_height = u64::from_le_bytes(src[0..8].try_into().unwrap());
        let amount = u64::from_le_bytes(src[8..16].try_into().unwrap());
        msg!("Unpacked schedule");
        Ok(Self {release_height, amount})
    }
}

impl Sealed for VestingParameters {}

impl Pack for VestingParameters {
    const LEN: usize = 81;
    fn pack_into_slice(&self, target: &mut [u8]){
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

        target[64] = self.is_initialized as u8;

        for i in 65..73 {
            target[i] = release_height_bytes[i-65];
        }

        for i in 73..81 {
            target[i] = amount_bytes[i-73];
        }
    }

    fn unpack_from_slice(input: &[u8])-> Result<Self, ProgramError>{
        let destination_address = Pubkey::new(&input[..32]);
        let mint_address = Pubkey::new(&input[32..64]);
        let is_initialized = input[64] == 1;
        let release_height = u64::from_le_bytes(input[65..73].try_into().unwrap());
        let amount = u64::from_le_bytes(input[73..81].try_into().unwrap());
        Ok(Self {destination_address, mint_address, release_height, amount, is_initialized})
    }
}

impl IsInitialized for VestingParameters {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for VestingScheduleHeader {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

pub fn unpack_schedules(input: &[u8]) -> Result<Vec<VestingSchedule>, ProgramError> {
    let number_of_schedules = input.len()/SCHEDULE_SIZE;
    msg!("Number of schedules {:?}", number_of_schedules);
    let mut output:Vec<VestingSchedule> = Vec::with_capacity(number_of_schedules);
    let mut offset = 0;
    for i in 0..number_of_schedules {
        msg!("Preparing to unpack schedule {:?}", i);
        output.push(VestingSchedule::unpack_from_slice(&input[offset..offset+SCHEDULE_SIZE])?);
        msg!("Unpacked schedule {:?}", i);

        offset += SCHEDULE_SIZE;
    }
    Ok(output)
}

pub fn pack_schedules_into_slice(schedules: Vec<VestingSchedule>, target: &mut [u8]){
    let mut offset = 0;
    for s in schedules.iter(){
        s.pack_into_slice(&mut target[offset..]);
        offset += SCHEDULE_SIZE;
    }
}

#[cfg(test)]
mod tests {
    use super::{VestingParameters, STATE_SIZE};
    use solana_program::{pubkey::Pubkey, program_pack::Pack};

    #[test]
    fn test_state_packing(){
        let state = VestingParameters{
            destination_address: Pubkey::new_unique(),
            mint_address: Pubkey::new_unique(),
            release_height: 30767976,
            amount: 969,
            is_initialized: true
        };
        let mut state_array = [0u8;VestingParameters::LEN];
        state.pack_into_slice(&mut state_array);
        let packed = Vec::from(state_array);
        let mut expected = Vec::with_capacity(STATE_SIZE);
        expected.extend_from_slice(&state.destination_address.to_bytes());
        expected.extend_from_slice(&state.mint_address.to_bytes());
        expected.extend_from_slice(&[state.is_initialized as u8]);
        expected.extend_from_slice(&state.release_height.to_le_bytes());
        expected.extend_from_slice(&state.amount.to_le_bytes());

        assert_eq!(expected, packed);
        assert_eq!(packed.len(), STATE_SIZE);
    }
}