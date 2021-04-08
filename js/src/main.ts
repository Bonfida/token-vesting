import {
  Account,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  TransactionInstruction,
  Connection,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  createChangeDestinationInstruction,
  createCreateInstruction,
  createInitInstruction,
  createUnlockInstruction,
} from './instructions';
import {
  findAssociatedTokenAddress,
  createAssociatedTokenAccount,
} from './utils';
import { ContractInfo, Schedule } from './state';
import { assert } from 'console';
import bs58 from 'bs58';

export const TOKEN_VESTING_PROGRAM_ID = new PublicKey(
  '8cdEhSpRAQaUBzDQL84ZQfNHYYXoP9TLSri8pKYXvUV2',
);

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
    possibleSourceTokenPubkey = await findAssociatedTokenAddress(
      sourceTokenOwner,
      mintAddress,
    );
  }

  // Find the non reversible public key for the vesting contract via the seed
  seedWord = seedWord.slice(0, 31);
  const [vestingAccountKey, bump] = await PublicKey.findProgramAddress(
    [seedWord],
    programId,
  );

  const vestingTokenAccountKey = await findAssociatedTokenAddress(
    vestingAccountKey,
    mintAddress,
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
    await createAssociatedTokenAccount(
      SystemProgram.programId,
      SYSVAR_CLOCK_PUBKEY,
      payer,
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

  const vestingTokenAccountKey = await findAssociatedTokenAddress(
    vestingAccountKey,
    mintAddress,
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
    throw 'Vesting contract account is unavailable';
  }
  const info = ContractInfo.fromBuffer(vestingInfo!.data);
  if (!info) {
    throw 'Vesting contract account is not initialized';
  }
  return info!;
}

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
    newDestinationTokenAccount = await findAssociatedTokenAddress(
      newDestinationTokenAccountOwner!,
      contractInfo.mintAddress,
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
