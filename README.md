This code has been audited by Kudelski ✅

- Audit report: [Bonfida Token Vesting Report](/audit/Bonfida_SecurityAssessment_Vesting_Final050521.pdf)

# Token vesting contract

## Program ID

- mainnet: `CChTq6PthWU82YZkbveA3WDf7s97BWhBK4Vx9bmsT743`
- devnet: `DLxB9dSQtA4WJ49hWFhxqiQkD9v6m67Yfk9voxpxrBs4`

## UI

[Bonfida Token Vesting UI](https://vesting.bonfida.com)

The UI **only** works for vesting accounts using the mainnet deployment `CChTq6PthWU82YZkbveA3WDf7s97BWhBK4Vx9bmsT743`

## Goal

- Simple vesting contract (SVC) that allows you to deposit X SPL tokens that are unlocked to a specified public key at a certain block height/ slot.
- Unlocking works by pushing a permissionless crank on the contract that moves the tokens to the pre-specified address
- Token Address should be derived from https://spl.solana.com/associated-token-account
- 'Vesting Schedule contract' - A contract containing an array of the SVC's that can be used to develop arbitrary- vesting schedules.
- Tooling to easily setup vesting schedule contracts
- Recipient address should be modifiable by the owner of the current recipient key
- Implementation should be a rust spl compatible program, plus client side javascript bindings that include a CLI- interface. Rust program should be unit tested and fuzzed.

## Structure

- `cli` : CLI tool to interact with on-chain token vesting contract
- `js` : JavaScript binding to interact with on-chain token vesting contract
- `program` : The BPF compatible token vesting on-chain program/smart contract

![diagram](assets/structure.png)
