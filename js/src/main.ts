import { Account, PublicKey, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, u64 } from '@solana/spl-token';
import { createInitInstruction, Schedule } from './instructions';


const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey = new PublicKey(
    'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
  );
  
async function findAssociatedTokenAddress(
    walletAddress: PublicKey,
    tokenMintAddress: PublicKey
    ): Promise<PublicKey> {
    return (await PublicKey.findProgramAddress(
        [
            walletAddress.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            tokenMintAddress.toBuffer(),
        ],
        SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    ))[0];
}


async function create(
    programId: PublicKey,
    vestingSeed: Array<Buffer | Uint8Array>,
    payer: Account,
    source_token_owner: Account,
    possible_source_token_pubkey: PublicKey | null,
    destination_token_pubkey: PublicKey,
    mint_address: PublicKey,
    schedules: Array<Schedule>
    ){

    // If no source token account was given, use the associated source account
    if (possible_source_token_pubkey == null) {
        possible_source_token_pubkey = await findAssociatedTokenAddress(
            source_token_owner.publicKey, mint_address);
    }

    const numberOfSchedules = new u64(schedules.length);

    // Find the non reversible public key for the vesting contract via the seed    
    let vestingPubkey = await PublicKey.createProgramAddress(vestingSeed, programId);
    console.log("Vesting token account pubkey: ", vestingPubkey);

    let instruction = [
        createInitInstruction(
            SystemProgram.programId,
            programId,
            payer.publicKey,
            vestingPubkey,
            vestingSeed,
            schedules.length
        )

    ]

}