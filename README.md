# Token vesting contract

## Goal

- Simple vesting contract (SVC) that allows you to deposit X SPL tokens that are unlocked to a specified public key at a certain block height/ slot.
- Unlocking works by pushing a permissionless crank on the contract that moves the tokens to the pre-specified address
- Token Address should be derived from https://spl.solana.com/associated-token-account
- 'Vesting Schedule contract' - A contract containing an array of the SVC's that can be used to develop arbitrary- vesting schedules. (?)
- Tooling to easily setup vesting schedule contracts
- Recipient address should be modifiable by the owner of the current recipient key
- Implementation should be a rust spl compatible program, plus client side javascript bindings that include a CLI- interface. Rust program should be unit tested and fuzzed.

## Structure

- `cli` : CLI tool to interact with on-chain token vesting contract
- `js` : JavaScript binding to interact with on-chain token vesting contract
- `program` : The BPF compatible token vesting on-chain program/smart contract

## TODO: internal

- We are currently using one-word seeds, which amounts to a 256 bit contract identifier, which is sufficient. We should refactor our functions to take in an array of bytes instead of
 an array of array of bytes. Printing the seed should be done in base 58 like a public key to remain compact. We don't need a list of words since this identifier will not be private and is unencrypted on the blockchain.

## TODO: Open issues for the following bugs

- `Instruction::new` serializes vector data wrongly (adds vector length as prefix to byte array)
- `solana-program-test/lib.rs` - `invoke_signed` - Order of accounts matters. Writability is either not checked or falsely assumed for system program.
