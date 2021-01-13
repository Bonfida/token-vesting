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
  connection,
  account,
  VESTING_PROGRAM_ID,
  tokenPubkey,
  mintAddress,
  schedule,
  signTransactionInstructions,
  findAssociatedTokenAddress,
  createAssociatedTokenAccount,
  generateRandomSeed,
  sleep,
  destinationPubkey,
  destinationAccount,
  newDestinationTokenAccountOwner,
  newDestinationTokenAccount,
} from './utils';
import { ContractInfo, Schedule, VestingScheduleHeader } from './state';
import { assert } from 'console';

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
  console.log('contract ID: ', seedWord.toString('hex'));

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
      currentDestinationTokenAccount.publicKey,
      contractInfo.destinationAddress,
      newDestinationTokenAccount,
      [seedWord],
    ),
  ];
}

const test = async (): Promise<void> => {
  const seed = generateRandomSeed();
  console.log(`Seed ${seed}`);
  const instructions = await create(
    connection,
    VESTING_PROGRAM_ID,
    Buffer.from(seed, 'hex'),
    account.publicKey,
    account.publicKey,
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
  console.log('✅ Successfully created vesting instructions');
  console.log(`🚚 Transaction signature: ${signed} \n`);
  await sleep(5 * 1000);

  const instructionsUnlock = await unlock(
    connection,
    VESTING_PROGRAM_ID,
    Buffer.from(seed, 'hex'),
  );

  const signedUnlock = await signTransactionInstructions(
    connection,
    [account],
    account.publicKey,
    instructionsUnlock,
  );
  console.log('✅ Successfully created unlocking instructions');
  console.log(`🚚 Transaction signature: ${signedUnlock} \n`);
  await sleep(5 * 1000);

  const instructionsChangeDestination = await changeDestination(
    connection,
    VESTING_PROGRAM_ID,
    destinationAccount,
    newDestinationTokenAccountOwner,
    undefined,
    [Buffer.from(seed, 'hex')],
  );

  const signedChangeDestination = await signTransactionInstructions(
    connection,
    [destinationAccount],
    destinationAccount.publicKey,
    instructionsChangeDestination,
  );

  console.log('✅ Successfully changed destination');
  console.log(`🚚 Transaction signature: ${signedChangeDestination}`);
};

test();
