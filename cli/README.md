# Testing

Create participants accounts:
```bash
solana-keygen new --outfile ~/.config/solana/id_owner.json --force
solana-keygen new --outfile ~/.config/solana/id_dest.json --force
solana-keygen new --outfile ~/.config/solana/id_new_dest.json --force
```

Owner would do all operations, so put some SOL to his account:
```bash
solana airdrop 2 --url https://api.devnet.solana.com ~/.config/solana/id_owner.json
```

Build program:
```bash
( cd ../program ; cargo build-bpf;  )
```

Deploy program and copy `PROGRAM_ID`.
```bash
solana deploy ../program/target/deploy/token_vesting.so --url https://api.devnet.solana.com --keypair  ~/.config/solana/id_owner.json
```

Create mint and get its public key(`MINT`):
```bash
spl-token create-token --url https://api.devnet.solana.com --fee-payer  ~/.config/solana/id_owner.json
```

Create source token account(`TOKEN_ACCOUNT_SOURCE`)
```bash
spl-token create-account $MINT --url https://api.devnet.solana.com --owner ~/.config/solana/id_owner.json --fee-payer  ~/.config/solana/id_owner.json
```

Mint test source token:
```bash
spl-token mint $MINT 100000 --url https://api.devnet.solana.com $TOKEN_ACCOUNT_SOURCE --fee-payer  ~/.config/solana/id_owner.json
```

Create vesting destination token account(`ACCOUNT_TOKEN_DEST`):
```bash
spl-token create-account $MINT --url https://api.devnet.solana.com --owner ~/.config/solana/id_dest.json --fee-payer  ~/.config/solana/id_owner.json
```

And new one(`ACCOUNT_TOKEN_NEW_DEST`):
```bash
spl-token create-account $MINT --url https://api.devnet.solana.com --owner ~/.config/solana/id_new_dest.json --fee-payer  ~/.config/solana/id_owner.json
```

Build CLI:

```bash
cargo build
```

Create vesting instance and store its SEED value
```bash
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli      \
--url https://api.devnet.solana.com                             \
--program_id $PROGRAM_ID                                        \
create                                                          \
--mint_address $MINT                                            \
--source_owner ~/.config/solana/id_owner.json                   \
--source_token_address $TOKEN_ACCOUNT_SOURCE                    \
--destination_token_address $ACCOUNT_TOKEN_DEST                 \
--amounts 2,1,3,!                                               \
--release-times 1,28504431,2850600000000000,!                   \
--payer ~/.config/solana/id_owner.json"                         \
--verbose | bash              
```

To use [Associated Token Account](https://spl.solana.com/associated-token-account) as destination use `--destination_address`(with public key of `id_dest`) instead of `--destination_token_address`.

Observe contract state:
```bash
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli      \
--url https://api.devnet.solana.com                             \
--program_id $PROGRAM_ID                                        \
info                                                            \
--seed $SEED " | bash                                          
```

Change owner:
```bash
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli      \
--url https://api.devnet.solana.com                             \
--program_id $PROGRAM_ID                                        \
change-destination                                              \
--seed $SEED                                                    \
--current_destination_owner ~/.config/solana/id_dest.json       \
--new_destination_token_address $ACCOUNT_TOKEN_NEW_DEST         \
--payer ~/.config/solana/id_owner.json" | bash                           
```

And unlock tokens according schedule:
```bash
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli      \
--url https://api.devnet.solana.com                             \
--program_id $PROGRAM_ID                                        \
unlock                                                          \
--seed $SEED                                                    \
--payer ~/.config/solana/id_owner.json" | bash
```

Create linear vesting:
```bash
echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli      \
--url https://api.devnet.solana.com                             \
--program_id $PROGRAM_ID                                        \
create                                                          \
--mint_address $MINT                                            \
--source_owner ~/.config/solana/id_owner.json                   \
--source_token_address $TOKEN_ACCOUNT_SOURCE                    \
--destination_token_address $ACCOUNT_TOKEN_DEST                 \
--amounts 42,!                                                  \
--release-frequency 'P1D'                                       \
--start-date-time '2022-01-06T20:11:18Z'                        \
--end-date-time '2022-01-12T20:11:18Z'                          \
--payer ~/.config/solana/id_owner.json"                         \
--verbose | bash 
```

## Links

https://spl.solana.com/token
