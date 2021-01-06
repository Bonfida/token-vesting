## Testing Keys for the devnet

KEYS: 

token_mint: 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC

source_owner: ~/.config/solana/id.json
source_token: EWrBFuSdmMC3wQKvWaCUTLbDhQT3Mpmw2CVViK4P5Xk2

destination_owner: ~/.config/solana/id_dest.json
dest_token: An2CVh3tm13Ld1EKKfiFE6udNFjmZNTSAzM4QzAdqCVZ

new_destination_owner: ~/.config/solana/id_new_dest.json
dest_token: B1cCkRiCe6j3gLjR3oWQvqCGvuM9eLp3FY7CQQYGJr1C

CMDS (don't forget the url):

solana-keygen new --outfile ~/.config/solana/id.json
solana-keygen new --outfile ~/.config/solana/id_dest.json
solana-keygen new --outfile ~/.config/solana/id_new_dest.json
solana airdrop 10 --url https://devnet.solana.com ~/.config/solana/id.json
solana deploy ../program/target/deploy/token_vesting.so --url https://devnet.solana.com

spl-token create-token
spl-token create-account MINT --url https://devnet.solana.com KEYPAIR
spl-token mint 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC 100 --url https://devnet.solana.com KEYPAIR


echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id HpghgSYBipxHKMVRkuLv3YeVpx7bDphsK7VQVjeNQNw3                           \
--seed 11111111111114512345123451234512                                             \
create                                                                              \
--mint_address 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC                         \
--source_owner ~/.config/solana/id.json                                             \
--source_token_address EWrBFuSdmMC3wQKvWaCUTLbDhQT3Mpmw2CVViK4P5Xk2                 \
--destination_token_address An2CVh3tm13Ld1EKKfiFE6udNFjmZNTSAzM4QzAdqCVZ            \
--amounts 2,!                                                                       \
--release-heights 1,!                                                               \
--payer ~/.config/solana/id.json" | bash                                    


echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id HpghgSYBipxHKMVRkuLv3YeVpx7bDphsK7VQVjeNQNw3                           \
--seed 11111111111114512345123451234512                                             \
info" | bash                                                                    


echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id HpghgSYBipxHKMVRkuLv3YeVpx7bDphsK7VQVjeNQNw3                           \
--seed 11111111111114512345123451234512                                             \
change-destination                                                                  \
--current_destination_owner ~/.config/solana/id_dest.json                           \
--new_destination_address B1cCkRiCe6j3gLjR3oWQvqCGvuM9eLp3FY7CQQYGJr1C              \
--payer ~/.config/solana/id.json" | bash                           


echo "RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli                          \
--url https://devnet.solana.com                                                     \
--program_id HpghgSYBipxHKMVRkuLv3YeVpx7bDphsK7VQVjeNQNw3                           \
--seed 11111111111114512345123451234512                                             \
unlock                                                                              \
--payer ~/.config/solana/id.json" | bash



LINKS:

https://spl.solana.com/token