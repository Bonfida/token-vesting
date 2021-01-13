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
  'Hj9R6bEfrULLNrApMsKCEaHR9QJ2JgRtM381xgYcjFmQ',
);

export const ASSOCIATED_TOKEN_PROGRAM_ID: PublicKey = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

export const walletSeed = Buffer.from(
  'cf556a77183c563b77986835d39d600a8d56998254d42d95888f91df9bb20fabc5da8e06f59a202bf23fb99e3cd10d2ea292437baa80d9d78c7e0f6f2eaf5621',
  'hex',
);

// Original account

export const account = getAccountFromSeed(walletSeed);

export const tokenPubkey = new PublicKey(
  '4PkZGUcaQoW7o138fUyn2xi1PfBNH2RFEavxyoKfJvtG',
);
export const mintAddress = new PublicKey(
  'GAVRiTwa55gNrVZwsRzLGkCmLC1qvrFtUAfD1ARz5spP',
);

// 1st Destination account

export const walletDestinationSeed = Buffer.from(
  'afafa04478843a6d82c8d1127307f2fec2eb5f4272bbd4dee5e27696876ce155641132a08426b5548e395f647cc164511eac18de87a692e31b1d11fa8619300f',
  'hex',
);

export const destinationAccount = getAccountFromSeed(walletDestinationSeed);

export const destinationPubkey = new PublicKey(
  'FZUK34uF1LkYtbjacynSPBZ4aeTCsvZ3R9VZGjUiSu27',
);

// 2nd Destination account

export const walletNewDestinationSeed = Buffer.from(
  '23934c36c5464b4370ed2976057f734f0cbe2d838fbf56b5b739eac9c75ee50f044a8ab917dc1f096b53cbbd8d4c98af045eb0f30cba1cb228a766413f3410ba',
  'hex',
);

export const newDestinationTokenAccountOwner = new PublicKey(
  '5ViWRxr4dxsWAEkVvjD6mm5hSKby5ooYxM6taFEzp1Q9',
);

export const newDestinationTokenAccount = new PublicKey(
  'CrYqDj1S44Hi7vbuz6kaXXbCWYe5PJJCF2LZjN26V9K5',
);

export const schedule: Schedule = new Schedule(
  new Numberu64(29507188),
  new Numberu64(10),
);

export const generateRandomSeed = () => {
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
