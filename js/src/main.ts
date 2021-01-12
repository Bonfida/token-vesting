import {
  Account,
  PublicKey,
  SystemProgram,
  Transaction,
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
  connection,
  account,
  VESTING_PROGRAM_ID,
  tokenPubkey,
  destinationPubkey,
  mintAddress,
  schedule,
  signTransactionInstructions,
} from './utils';
import { ContractInfo, Schedule, VestingScheduleHeader } from './state';
import { assert } from 'console';

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

export async function create(
  programId: PublicKey,
  vestingSeed: Array<Buffer | Uint8Array>,
  payer: Account,
  sourceTokenOwner: Account,
  possibleSourceTokenPubkey: PublicKey | null,
  destinationTokenPubkey: PublicKey,
  mintAddress: PublicKey,
  schedules: Array<Schedule>,
): Promise<Array<TransactionInstruction>> {
  // If no source token account was given, use the associated source account
  if (possibleSourceTokenPubkey == null) {
    possibleSourceTokenPubkey = await findAssociatedTokenAddress(
      sourceTokenOwner.publicKey,
      mintAddress,
    );
  }

  let seedWord = vestingSeed[0];

  const numberOfSchedules = schedules.length;

  // Find the non reversible public key for the vesting contract via the seed
  seedWord = seedWord.slice(0, 31);
  const [pubkey, bump] = await PublicKey.findProgramAddress(
    [seedWord],
    programId,
  );

  const vestingTokenAccountKey = await findAssociatedTokenAddress(
    pubkey,
    mintAddress,
  );

  seedWord = Buffer.from(seedWord.toString('hex') + bump.toString(16), 'hex');

  console.log('Vesting token account pubkey: ', pubkey.toBase58());
  console.log('contract ID: ', seedWord.toString('hex'));

  let instruction = [
    createInitInstruction(
      SystemProgram.programId,
      programId,
      payer.publicKey,
      pubkey,
      [seedWord],
      schedules.length,
    ),
    createCreateInstruction(
      programId,
      TOKEN_PROGRAM_ID,
      pubkey,
      vestingTokenAccountKey,
      sourceTokenOwner.publicKey,
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
  vestingSeed: Array<Buffer | Uint8Array>,
  payer: Account,
): Promise<Array<TransactionInstruction>> {
  let seedWord = vestingSeed[0];
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

  const vestingInfo = await getContractInfo(connection, programId, [seedWord]);

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
  programId: PublicKey,
  vestingSeed: Array<Buffer | Uint8Array>,
): Promise<ContractInfo> {
  const [vestingAccountKey, bump] = await PublicKey.findProgramAddress(
    vestingSeed,
    programId,
  );
  const vestingInfo = await connection.getAccountInfo(vestingAccountKey);
  assert(vestingInfo != null, 'Vesting contract account is unavailable');
  const info = ContractInfo.fromBuffer(vestingInfo!.data);
  assert(info != undefined, "Vesting contract isn't initialized");
  return info!;
}

export async function changeDestination(
  connection: Connection,
  programId: PublicKey,
  currentDestinationTokenAccount: Account,
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

  const contractInfo = await getContractInfo(connection, programId, [seedWord]);
  if (newDestinationTokenAccount == undefined) {
    assert(
      newDestinationTokenAccountOwner != undefined,
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
      currentDestinationTokenAccount.publicKey,
      contractInfo.destinationAddress,
      newDestinationTokenAccount,
      [seedWord],
    ),
  ];
}

const test = async (): Promise<void> => {
  const pre_instructions = [];
  const instructions = await create(
    VESTING_PROGRAM_ID,
    [
      Buffer.from(
        '1111111111111481234512345123451211111111111114512345123451234512',
        'hex',
      ),
    ],
    account,
    account,
    tokenPubkey,
    destinationPubkey,
    mintAddress,
    [schedule],
  );
  const signed = await signTransactionInstructions(
    connection,
    [account],
    account.publicKey,
    instructions,
  );
};

test();
