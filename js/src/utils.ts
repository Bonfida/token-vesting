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
} from '@solana/web3.js';
import { Schedule } from './state';

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
export const walletSeed = Buffer.from(
  'cf556a77183c563b77986835d39d600a8d56998254d42d95888f91df9bb20fabc5da8e06f59a202bf23fb99e3cd10d2ea292437baa80d9d78c7e0f6f2eaf5621',
  'hex',
);
export const account = getAccountFromSeed(walletSeed);
export const tokenPubkey = new PublicKey(
  '4PkZGUcaQoW7o138fUyn2xi1PfBNH2RFEavxyoKfJvtG',
);
export const mintAddress = new PublicKey(
  'GAVRiTwa55gNrVZwsRzLGkCmLC1qvrFtUAfD1ARz5spP',
);

export const destinationPubkey = new PublicKey(
  '4F9NzDF3Z1PbJizbGJdZ3KvQJMrkK1GEBaN6BVmnmkzG',
);

export const schedule: Schedule = new Schedule(
  new Numberu64(29507188),
  new Numberu64(10),
);

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
