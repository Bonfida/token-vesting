## Testing Keys for the devnet

KEYS: 

token_mint: 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC

source_owner: ~/.config/solana/id.json
Pubkey: FbqE3zeiu8ccBgt1xA6F5Yx8bq5T1D5j9eUcqFs4Dsvb
source_token: EWrBFuSdmMC3wQKvWaCUTLbDhQT3Mpmw2CVViK4P5Xk2

destination_owner: ~/.config/solana/id_dest.json
Pubkey: 8vBVs9hATt4C4DeMfheiqJ7kJhX9JQffDQ9bJW4dN7nX
dest_token: 9dFttH4GjGHqNxfpiRB5m59YdvH9ydHq9cJ6c6v2JR3p

new_destination_owner: ~/.config/solana/id_new_dest.json
Pubkey: 8bVoQtWUWqeNZcBSTNnWikxcqBzmyNVXVdU148DLDCYG
new_dest_token: CrCPEHiRz2bpC3kmtu3vdghhL62GFeRnUeck8RYNBQkh

CMDS (don't forget the url):

solana-keygen new --outfile ~/.config/solana/id.json
solana-keygen new --outfile ~/.config/solana/id_dest.json
solana-keygen new --outfile ~/.config/solana/id_new_dest.json
solana airdrop 10 --url https://devnet.solana.com ~/.config/solana/id.json
solana deploy ../program/target/deploy/token_vesting.so --url https://devnet.solana.com

spl-token create-token
spl-token create-account 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC --url https://devnet.solana.com --owner KEYPAIR
spl-token mint 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC 100 --url https://devnet.solana.com KEYPAIR


echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id Hj9R6bEfrULLNrApMsKCEaHR9QJ2JgRtM381xgYcjFmQ                           \
--seed 11111111111111111145123451234512                                             \
create                                                                              \
--mint_address 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC                         \
--source_owner ~/.config/solana/id.json                                             \
--destination_address 8vBVs9hATt4C4DeMfheiqJ7kJhX9JQffDQ9bJW4dN7nX                  \
--amounts 2,1,3,!                                                                   \
--release-heights 1,28504431,28506000,!                                             \
--payer ~/.config/solana/id.json" | bash               


echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id Hj9R6bEfrULLNrApMsKCEaHR9QJ2JgRtM381xgYcjFmQ                           \
--seed 11111111111111111145123451234512                                             \
info" | bash                                          


echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id Hj9R6bEfrULLNrApMsKCEaHR9QJ2JgRtM381xgYcjFmQ                           \
--seed 11111111111111111145123451234512                                             \
change-destination                                                                  \
--current_destination_owner ~/.config/solana/id_dest.json                           \
--new_destination_token_address CrCPEHiRz2bpC3kmtu3vdghhL62GFeRnUeck8RYNBQkh        \
--payer ~/.config/solana/id.json" | bash                           


echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id Hj9R6bEfrULLNrApMsKCEaHR9QJ2JgRtM381xgYcjFmQ                           \
--seed 11111111111111111145123451234512                                             \
unlock                                                                              \
--payer ~/.config/solana/id.json" | bash

// TODO config file parsing
// TODO Make Slot height relative?

LINKS:

https://spl.solana.com/token