import {
  Account,
  PublicKey,
  SystemProgram,
  Transaction,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { createInitInstruction, Schedule } from './instructions';
import {
  getDerivedSeed,
  getAccountFromSeed,
  connection,
  account,
  VESTING_PROGRAM_ID,
  tokenPubkey,
  destinationPubkey,
  mintAddress,
  schedule,
  signTransactionInstructions,
  walletSeed,
} from './utils';

async function findAssociatedTokenAddress(
  walletAddress: PublicKey,
  tokenMintAddress: PublicKey,
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        walletAddress.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        tokenMintAddress.toBuffer(),
      ],
      VESTING_PROGRAM_ID,
    )
  )[0];
}

async function create(
  programId: PublicKey,
  vestingSeed: Array<Buffer | Uint8Array>,
  payer: Account,
  sourceTokenOwner: Account,
  possibleSourceTokenPubkey: PublicKey | null,
  destinationTokenPubkey: PublicKey,
  mintAddress: PublicKey,
  schedules: Array<Schedule>,
) {
  // If no source token account was given, use the associated source account
  if (possibleSourceTokenPubkey == null) {
    possibleSourceTokenPubkey = await findAssociatedTokenAddress(
      sourceTokenOwner.publicKey,
      mintAddress,
    );
  }

  const numberOfSchedules = schedules.length;

  // Find the non reversible public key for the vesting contract via the seed
  vestingSeed = vestingSeed.slice(0, 31);
  const [pubkey, bump] = await PublicKey.findProgramAddress(
    vestingSeed,
    programId,
  );

  console.log('Vesting token account pubkey: ', pubkey.toBase58());

  let instruction = [
    createInitInstruction(
      SystemProgram.programId,
      programId,
      payer.publicKey,
      pubkey,
      [Buffer.from(vestingSeed.toString() + bump, 'hex')],
      schedules.length,
    ),
  ];
  return instruction;
}

const test = async (): Promise<void> => {
  const instructions = await create(
    VESTING_PROGRAM_ID,
    [Buffer.from('11111111111114512345123451234512', 'hex')],
    account,
    account,
    tokenPubkey,
    destinationPubkey,
    mintAddress,
    [schedule],
  );
  const signed = await signTransactionInstructions(
    connection,
    account,
    instructions,
  );
};

test();
