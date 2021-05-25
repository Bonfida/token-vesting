# Testing

Create participants accounts:
```
solana-keygen new --outfile ~/.config/solana/id_owner.json --force
solana-keygen new --outfile ~/.config/solana/id_dest.json --force
solana-keygen new --outfile ~/.config/solana/id_new_dest.json --force
```

Owner would do all operations, so put some SOL to his account:
```
solana airdrop 10 --url https://devnet.solana.com ~/.config/solana/id_owner.json
```

Deploy program and copy PROGRAM_ID.
```
solana deploy ../program/target/deploy/token_vesting.so --url https://devnet.solana.com --config  ~/.config/solana/id_owner.json
```

Create mint and get its public key(MINT):
```
spl-token create-token --url https://devnet.solana.com --config  ~/.config/solana/id_owner.json
```

Create source token(TOKEN_ACCOUNT)
```
spl-token create-account $MINT --url https://devnet.solana.com --owner ~/.config/solana/id_owner.json

```

Mint test source token:
```
spl-token mint $MINT 10000 --url https://devnet.solana.com $TOKEN_ACCOUNT --config  ~/.config/solana/id_owner.json
```

Create vesting destination token account(ACCOUNT_TOKEN_DEST):
```
spl-token create-account $MINT --url https://devnet.solana.com --owner ~/.config/solana/id_dest.json
```

And new one(ACCOUNT_TOKEN_NEW_DEST):
```
spl-token create-account $MINT --url https://devnet.solana.com --owner ~/.config/solana/id_new_dest.json
```

Build CLI:

```
cargo build
```

Create vesting instance and store its SEED value
```
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id $PROGRAM_ID                           \
create                                                                              \
--mint_address $MINT                         \
--source_owner ~/.config/solana/id_owner.json                                             \
--source_token_address $TOKEN_ACCOUNT                                             \
--destination_token_address $ACCOUNT_TOKEN_DEST                  \
--amounts 2,1,3,!                                                                   \
--release-times 1,28504431,2850600000000000,!                                             \
--payer ~/.config/solana/id_owner.json"                  \
--verbose | bash              
```

To use [Associated Token Account](https://spl.solana.com/associated-token-account) as destination use `--destination_address`(with public key of `id_dest`) instead of `--destination_token_address`.

Observe contract state:
```
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id $PROGRAM_ID                           \
info                                                                                \
--seed $SEED " | bash                                          
```

Change owner:
```
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id $PROGRAM_ID                           \
change-destination                                                                  \
--seed $SEED                                  \
--current_destination_owner ~/.config/solana/id_dest.json                           \
--new_destination_token_address $ACCOUNT_TOKEN_NEW_DEST        \
--payer ~/.config/solana/id_owner.json" | bash                           
```

And unlock tokens according schedule:
```
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id $PROGRAM_ID                           \
unlock                                                                              \
--seed $SEED                                  \
--payer ~/.config/solana/id_owner.json" | bash
```

## Links

https://spl.solana.com/token
