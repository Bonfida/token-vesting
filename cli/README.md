## Testing Keys for the devnet

KEYS: 

token_mint: 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC

source_owner: ~/.config/solana/id.json
source_token: EWrBFuSdmMC3wQKvWaCUTLbDhQT3Mpmw2CVViK4P5Xk2

destination_owner: ~/.config/solana/id_dest.json
dest_token: An2CVh3tm13Ld1EKKfiFE6udNFjmZNTSAzM4QzAdqCVZ

CMDS (don't forget the url):

solana-keygen new --outfile ~/.config/solana/id.json
solana-keygen new --outfile ~/.config/solana/id_dest.json
solana airdrop 10 --url https://devnet.solana.com ~/.config/solana/id.json
solana deploy ../program/target/deploy/token_vesting.so --url https://devnet.solana.com

spl-token create-token
spl-token create-account MINT --url https://devnet.solana.com KEYPAIR
spl-token mint 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC 100 --url https://devnet.solana.com KEYPAIR

RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli --url https://devnet.solana.com --program_id 3Sr7jiAPKopcdjFrktZpQsViU3Ur6CJ9hyE7qcihkrcQ --mint_address 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC --seed 11111123451234512345123451234512 create-svc --amount 2 --destination_token_address An2CVh3tm13Ld1EKKfiFE6udNFjmZNTSAzM4QzAdqCVZ --source ~/.config/solana/id.json --source_token_address EWrBFuSdmMC3wQKvWaCUTLbDhQT3Mpmw2CVViK4P5Xk2

RUST_BACKTRACE=1 ./target/debug/vesting-contract-cli --url https://devnet.solana.com --program_id 3Sr7jiAPKopcdjFrktZpQsViU3Ur6CJ9hyE7qcihkrcQ --mint_address 3wmMWPDkSdKd697arrGWYJ1q4QL1jwGxnANUyXqSV9vC --seed 11111123451234512345123451234512 unlock-svc --destination An2CVh3tm13Ld1EKKfiFE6udNFjmZNTSAzM4QzAdqCVZ --payer ~/.config/solana/id.json

LINKS:

https://spl.solana.com/token