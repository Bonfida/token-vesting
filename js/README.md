# Simple JS binding

## Quickstart

Run `yarn` in the `js` directory to install the node modules. Run `yarn dev` to get started and `yarn build` to build.

Contract address on Devnet

```
Hj9R6bEfrULLNrApMsKCEaHR9QJ2JgRtM381xgYcjFmQ
```

See on the [Solana Explorer](https://explorer.solana.com/address/Hj9R6bEfrULLNrApMsKCEaHR9QJ2JgRtM381xgYcjFmQ?cluster=devnet)

The code allows you to

- Create vesting instructions for any SPL token: `createCreateInstruction`
- Create unlock instructions: `createUnlockInstruction`
- Change the destination of the vested tokens: `createChangeDestinationInstruction`

(To import Solana accounts created with [Sollet](https://sollet.io) you can use `getAccountFromSeed`)

```
Seed 9043936629442508205162695100279588102353854608998701852963634059
Vesting contract account pubkey:  r2p2mLJvyrTzetxxsttQ54CS1m18zMgYqKSRzxP9WpE
contract ID:  90439366294425082051626951002795881023538546089987018529636340fe
✅ Successfully created vesting instructions
🚚 Transaction signature: 2uypTM3QcroR7uk6g9Y4eLdniCHqdQBDq4XyrFM7hCtTbb4rftkEHMM6vJ6tTYpihYubHt55xWD86vHB857bqXXb

Fetching contract  r2p2mLJvyrTzetxxsttQ54CS1m18zMgYqKSRzxP9WpE
✅ Successfully created unlocking instructions
🚚 Transaction signature: 2Vg3W1w8WBdRAWBEwFTn2BtMkKPD3Xor7SRvzC193UnsUnhmneUChPHe7vLF9Lfw9BKxWH5JbbJmnda4XztHMVHz

Fetching contract  r2p2mLJvyrTzetxxsttQ54CS1m18zMgYqKSRzxP9WpE
✅ Successfully changed destination
🚚 Transaction signature: 4tgPgCdM5ubaSKNLKD1WrfAJPZgRajxRSnmcPkHcN1TCeCRmq3cUCYVdCzsYwr63JRf4D2K1UZnt8rwu2pkGxeYe
```
