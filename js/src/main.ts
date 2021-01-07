import { Account, PublicKey, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, u64 } from '@solana/spl-token';
import nacl from 'tweetnacl';
import * as bip32 from 'bip32';

import { createInitInstruction, Schedule } from './instructions';

const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

const walletSeed = Buffer.from(
  'cf556a77183c563b77986835d39d600a8d56998254d42d95888f91df9bb20fabc5da8e06f59a202bf23fb99e3cd10d2ea292437baa80d9d78c7e0f6f2eaf5621',
  'hex',
);

const getDerivedSeed = (seed: Buffer) => {
  const derivedSeed = bip32.fromSeed(seed).derivePath(`m/501'/0'/0/0`)
    .privateKey;
  return nacl.sign.keyPair.fromSeed(derivedSeed).secretKey;
};

const getAccountFromSeed = (seed: Buffer) => {
  const derivedSeed = bip32.fromSeed(seed).derivePath(`m/501'/0'/0/0`)
    .privateKey;
  return new Account(nacl.sign.keyPair.fromSeed(derivedSeed).secretKey);
};

const seed = getDerivedSeed(walletSeed);
const account = getAccountFromSeed(walletSeed);
const tokenPubkey = new PublicKey(
  '4PkZGUcaQoW7o138fUyn2xi1PfBNH2RFEavxyoKfJvtG',
);
const mintAddress = new PublicKey(
  'GAVRiTwa55gNrVZwsRzLGkCmLC1qvrFtUAfD1ARz5spP',
);
const schedule = new Schedule();

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
      SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
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
  console.log(vestingSeed);
  // Find the non reversible public key for the vesting contract via the seed
  let vestingPubkey = await PublicKey.createProgramAddress(
    vestingSeed,
    programId,
  );

  console.log('Vesting token account pubkey: ', vestingPubkey);

  let instruction = [
    createInitInstruction(
      SystemProgram.programId,
      programId,
      payer.publicKey,
      vestingPubkey,
      vestingSeed,
      schedules.length,
    ),
  ];
}

create(
  SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
  [Buffer.from('11111111111114512345123451234512', 'hex')],
  account,
  account,
  tokenPubkey,
  tokenPubkey,
  mintAddress,
  [schedule],
);
