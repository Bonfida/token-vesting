// @ts-nocheck
import BN from 'bn.js';
import assert from 'assert';
import nacl from 'tweetnacl';
import * as bip32 from 'bip32';
import {
  Account,
  Connection,
  Transaction,
  TransactionInstruction,
  PublicKey,
  TransactionInstruction,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import { Schedule } from './state';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

export async function findAssociatedTokenAddress(
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
      ASSOCIATED_TOKEN_PROGRAM_ID,
    )
  )[0];
}

export class Numberu64 extends BN {
  /**
   * Convert to Buffer representation
   */
  toBuffer(): Buffer {
    const a = super.toArray().reverse();
    const b = Buffer.from(a);
    if (b.length === 8) {
      return b;
    }
    assert(b.length < 8, 'Numberu64 too large');

    const zeroPad = Buffer.alloc(8);
    b.copy(zeroPad);
    return zeroPad;
  }

  /**
   * Construct a Numberu64 from Buffer representation
   */
  static fromBuffer(buffer): any {
    assert(buffer.length === 8, `Invalid buffer length: ${buffer.length}`);
    return new BN(
      [...buffer]
        .reverse()
        .map(i => `00${i.toString(16)}`.slice(-2))
        .join(''),
      16,
    );
  }
}

// Connection

const ENDPOINTS = {
  mainnet: 'https://solana-api.projectserum.com',
  devnet: 'https://devnet.solana.com',
};

export const connection = new Connection(ENDPOINTS.devnet);

// For accounts imported from Sollet.io

export const getDerivedSeed = (seed: Buffer): Uint8Array => {
  const derivedSeed = bip32.fromSeed(seed).derivePath(`m/501'/0'/0/0`)
    .privateKey;
  return nacl.sign.keyPair.fromSeed(derivedSeed).secretKey;
};

export const getAccountFromSeed = (seed: Buffer): Account => {
  const derivedSeed = bip32.fromSeed(seed).derivePath(`m/501'/0'/0/0`)
    .privateKey;
  return new Account(nacl.sign.keyPair.fromSeed(derivedSeed).secretKey);
};

// Test params

export const VESTING_PROGRAM_ID: PublicKey = new PublicKey(
  '5eiTBnbpMsioMR7TbFPLxpj7KLi9c8esrZXYzuW9uEgy',
);

export const ASSOCIATED_TOKEN_PROGRAM_ID: PublicKey = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

// Original account

export const walletSeed = Buffer.from('Enter your seed', 'hex');

export const account = getAccountFromSeed(walletSeed);

export const tokenPubkey = new PublicKey('');
export const mintAddress = new PublicKey('');

// 1st Destination account

export const walletDestinationSeed = Buffer.from('Enter your seed', 'hex');

export const destinationAccount = getAccountFromSeed(walletDestinationSeed);

export const destinationPubkey = new PublicKey('Enter your pubkey');

// 2nd Destination account

export const walletNewDestinationSeed = Buffer.from('Enter your seed', 'hex');

export const newDestinationTokenAccountOwner = new PublicKey(
  'Enter your pubkey',
);

export const newDestinationTokenAccount = new PublicKey('Enter your pubkey');

export const schedule: Schedule = new Schedule(
  new Numberu64(29507188), // Enter the slot height for the vesting schedule
  new Numberu64(10), // Enter the amount to be vested
);

export const generateRandomSeed = () => {
  // Generate a random seed
  let seed = '';
  for (let i = 0; i < 64; i++) {
    seed += Math.floor(Math.random() * 10);
  }
  return seed;
};

export const sleep = (ms: number): Promise<void> => {
  return new Promise(resolve => setTimeout(resolve, ms));
};

// Sign transaction

export const signTransactionInstructions = async (
  // sign and send transaction
  connection: Connection,
  signers: Array<Account>,
  feePayer: PublicKey,
  txInstructions: Array<TransactionInstruction>,
): Promise<string> => {
  const tx = new Transaction();
  tx.feePayer = feePayer;
  tx.add(...txInstructions);
  return await connection.sendTransaction(tx, signers, {
    preflightCommitment: 'single',
  });
};

export const createAssociatedTokenAccount = async (
  systemProgramId: PublicKey,
  clockSysvarId: PublicKey,
  fundingAddress: PublicKey,
  walletAddress: PublicKey,
  splTokenMintAddress: PublicKey,
): Promise<TransactionInstruction> => {
  const associatedTokenAddress = await findAssociatedTokenAddress(
    walletAddress,
    splTokenMintAddress,
  );
  const keys = [
    {
      pubkey: fundingAddress,
      isSigner: true,
      isWritable: true,
    },
    {
      pubkey: associatedTokenAddress,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: walletAddress,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: splTokenMintAddress,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: systemProgramId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: TOKEN_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SYSVAR_RENT_PUBKEY,
      isSigner: false,
      isWritable: false,
    },
  ];
  return new TransactionInstruction({
    keys,
    programId: ASSOCIATED_TOKEN_PROGRAM_ID,
    data: Buffer.from([]),
  });
};
