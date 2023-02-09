import {
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  TransactionInstruction,
  Connection,
} from '@solana/web3.js';
import {
  createAssociatedTokenAccountInstruction,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from '@solana/spl-token';
import {
  createChangeDestinationInstruction,
  createCreateInstruction,
  createInitInstruction,
  createUnlockInstruction,
} from './instructions';
import { ContractInfo, Schedule } from './state';
import { assert } from 'console';
import bs58 from 'bs58';

/**
 * The vesting schedule program ID on mainnet
 */
export const TOKEN_VESTING_PROGRAM_ID = new PublicKey(
  'CChTq6PthWU82YZkbveA3WDf7s97BWhBK4Vx9bmsT743',
);

/**
 * This function can be used to lock tokens
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param seedWord Seed words used to derive the vesting account
 * @param payer The fee payer of the transaction
 * @param sourceTokenOwner The owner of the source token account (i.e where locked tokens are originating from)
 * @param possibleSourceTokenPubkey The source token account (i.e where locked tokens are originating from), if null it defaults to the ATA
 * @param destinationTokenPubkey The destination token account i.e where unlocked tokens will be transfered
 * @param mintAddress The mint of the tokens being vested
 * @param schedules The array of vesting schedules
 * @returns An array of `TransactionInstruction`
 */
export async function create(
  connection: Connection,
  programId: PublicKey,
  seedWord: Buffer | Uint8Array,
  payer: PublicKey,
  sourceTokenOwner: PublicKey,
  possibleSourceTokenPubkey: PublicKey | null,
  destinationTokenPubkey: PublicKey,
  mintAddress: PublicKey,
  schedules: Array<Schedule>,
): Promise<Array<TransactionInstruction>> {
  // If no source token account was given, use the associated source account
  if (possibleSourceTokenPubkey == null) {
    possibleSourceTokenPubkey = await getAssociatedTokenAddress(
      mintAddress,
      sourceTokenOwner,
      true,
    );
  }

  // Find the non reversible public key for the vesting contract via the seed
  seedWord = seedWord.slice(0, 31);
  const [vestingAccountKey, bump] = await PublicKey.findProgramAddress(
    [seedWord],
    programId,
  );

  const vestingTokenAccountKey = await getAssociatedTokenAddress(
    mintAddress,
    vestingAccountKey,
    true,
  );

  seedWord = Buffer.from(seedWord.toString('hex') + bump.toString(16), 'hex');

  console.log(
    'Vesting contract account pubkey: ',
    vestingAccountKey.toBase58(),
  );

  console.log('contract ID: ', bs58.encode(seedWord));

  const check_existing = await connection.getAccountInfo(vestingAccountKey);
  if (!!check_existing) {
    throw 'Contract already exists.';
  }

  let instruction = [
    createInitInstruction(
      SystemProgram.programId,
      programId,
      payer,
      vestingAccountKey,
      [seedWord],
      schedules.length,
    ),
    createAssociatedTokenAccountInstruction(
      payer,
      vestingTokenAccountKey,
      vestingAccountKey,
      mintAddress,
    ),
    createCreateInstruction(
      programId,
      TOKEN_PROGRAM_ID,
      vestingAccountKey,
      vestingTokenAccountKey,
      sourceTokenOwner,
      possibleSourceTokenPubkey,
      destinationTokenPubkey,
      mintAddress,
      schedules,
      [seedWord],
    ),
  ];
  return instruction;
}

/**
 * This function can be used to unlock vested tokens
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param seedWord Seed words used to derive the vesting account
 * @param mintAddress The mint of the vested tokens
 * @returns An array of `TransactionInstruction`
 */
export async function unlock(
  connection: Connection,
  programId: PublicKey,
  seedWord: Buffer | Uint8Array,
  mintAddress: PublicKey,
): Promise<Array<TransactionInstruction>> {
  seedWord = seedWord.slice(0, 31);
  const [vestingAccountKey, bump] = await PublicKey.findProgramAddress(
    [seedWord],
    programId,
  );
  seedWord = Buffer.from(seedWord.toString('hex') + bump.toString(16), 'hex');

  const vestingTokenAccountKey = await getAssociatedTokenAddress(
    mintAddress,
    vestingAccountKey,
    true,
  );

  const vestingInfo = await getContractInfo(connection, vestingAccountKey);

  let instruction = [
    createUnlockInstruction(
      programId,
      TOKEN_PROGRAM_ID,
      SYSVAR_CLOCK_PUBKEY,
      vestingAccountKey,
      vestingTokenAccountKey,
      vestingInfo.destinationAddress,
      [seedWord],
    ),
  ];

  return instruction;
}

/**
 * This function can be used retrieve information about a vesting account
 * @param connection The Solana RPC connection object
 * @param vestingAccountKey The vesting account public key
 * @returns A `ContractInfo` object
 */
export async function getContractInfo(
  connection: Connection,
  vestingAccountKey: PublicKey,
): Promise<ContractInfo> {
  console.log('Fetching contract ', vestingAccountKey.toBase58());
  const vestingInfo = await connection.getAccountInfo(
    vestingAccountKey,
    'single',
  );
  if (!vestingInfo) {
    throw new Error('Vesting contract account is unavailable');
  }
  const info = ContractInfo.fromBuffer(vestingInfo!.data);
  if (!info) {
    throw new Error('Vesting contract account is not initialized');
  }
  return info!;
}

/**
 * This function can be used to transfer a vesting account to a new wallet. It requires the current owner to sign.
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param currentDestinationTokenAccountPublicKey The current token account to which the vested tokens are transfered to as they unlock
 * @param newDestinationTokenAccountOwner The new owner of the vesting account
 * @param newDestinationTokenAccount The new token account to which the vested tokens will be transfered to as they unlock
 * @param vestingSeed Seed words used to derive the vesting account
 * @returns An array of `TransactionInstruction`
 */
export async function changeDestination(
  connection: Connection,
  programId: PublicKey,
  currentDestinationTokenAccountPublicKey: PublicKey,
  newDestinationTokenAccountOwner: PublicKey | undefined,
  newDestinationTokenAccount: PublicKey | undefined,
  vestingSeed: Array<Buffer | Uint8Array>,
): Promise<Array<TransactionInstruction>> {
  let seedWord = vestingSeed[0];
  seedWord = seedWord.slice(0, 31);
  const [vestingAccountKey, bump] = await PublicKey.findProgramAddress(
    [seedWord],
    programId,
  );
  seedWord = Buffer.from(seedWord.toString('hex') + bump.toString(16), 'hex');

  const contractInfo = await getContractInfo(connection, vestingAccountKey);
  if (!newDestinationTokenAccount) {
    assert(
      !!newDestinationTokenAccountOwner,
      'At least one of newDestinationTokenAccount and newDestinationTokenAccountOwner must be provided!',
    );
    newDestinationTokenAccount = await getAssociatedTokenAddress(
      contractInfo.mintAddress,
      newDestinationTokenAccountOwner!,
      true,
    );
  }

  return [
    createChangeDestinationInstruction(
      programId,
      vestingAccountKey,
      currentDestinationTokenAccountPublicKey,
      contractInfo.destinationAddress,
      newDestinationTokenAccount,
      [seedWord],
    ),
  ];
}
