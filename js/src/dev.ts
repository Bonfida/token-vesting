import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import fs from 'fs';
import {
  Numberu64,
  generateRandomSeed,
  signTransactionInstructions,
} from './utils';
import { Schedule } from './state';
import { create, TOKEN_VESTING_PROGRAM_ID } from './main';

/**
 *
 * Simple example of a linear unlock.
 *
 * This is just an example, please be careful using the vesting contract and test it first with test tokens.
 *
 */

/** Path to your wallet */
const WALLET_PATH = '';
const wallet = Keypair.fromSecretKey(
  new Uint8Array(JSON.parse(fs.readFileSync(WALLET_PATH).toString())),
);

/** There are better way to generate an array of dates but be careful as it's irreversible */
const DATES = [
  new Date(2022, 12),
  new Date(2023, 1),
  new Date(2023, 2),
  new Date(2023, 3),
  new Date(2023, 4),
  new Date(2023, 5),
  new Date(2023, 6),
  new Date(2023, 7),
  new Date(2023, 8),
  new Date(2023, 9),
  new Date(2023, 10),
  new Date(2023, 11),
  new Date(2024, 12),
  new Date(2024, 2),
  new Date(2024, 3),
  new Date(2024, 4),
  new Date(2024, 5),
  new Date(2024, 6),
  new Date(2024, 7),
  new Date(2024, 8),
  new Date(2024, 9),
  new Date(2024, 10),
  new Date(2024, 11),
  new Date(2024, 12),
];

/** Info about the desintation */
const DESTINATION_OWNER = new PublicKey('');
const DESTINATION_TOKEN_ACCOUNT = new PublicKey('');

/** Token info */
const MINT = new PublicKey('');
const DECIMALS = 0;

/** Info about the source */
const SOURCE_TOKEN_ACCOUNT = new PublicKey('');

/** Amount to give per schedule */
const AMOUNT_PER_SCHEDULE = 0;

/** Your RPC connection */
const connection = new Connection('');

/** Do some checks before sending the tokens */
const checks = async () => {
  const tokenInfo = await connection.getParsedAccountInfo(
    DESTINATION_TOKEN_ACCOUNT,
  );

  // @ts-ignore
  const parsed = tokenInfo.value.data.parsed;
  if (parsed.info.mint !== MINT.toBase58()) {
    throw new Error('Invalid mint');
  }
  if (parsed.info.owner !== DESTINATION_OWNER.toBase58()) {
    throw new Error('Invalid owner');
  }
  if (parsed.info.tokenAmount.decimals !== DECIMALS) {
    throw new Error('Invalid decimals');
  }
};

/** Function that locks the tokens */
const lock = async () => {
  await checks();
  const schedules: Schedule[] = [];
  for (let date of DATES) {
    schedules.push(
      new Schedule(
        /** Has to be in seconds */
        new Numberu64(date.getTime() / 1_000),
        /** Don't forget to add decimals */
        new Numberu64(AMOUNT_PER_SCHEDULE * Math.pow(10, DECIMALS)),
      ),
    );
  }
  const seed = generateRandomSeed();

  console.log(`Seed: ${seed}`);

  const instruction = await create(
    connection,
    TOKEN_VESTING_PROGRAM_ID,
    Buffer.from(seed),
    wallet.publicKey,
    wallet.publicKey,
    SOURCE_TOKEN_ACCOUNT,
    DESTINATION_TOKEN_ACCOUNT,
    MINT,
    schedules,
  );

  const tx = await signTransactionInstructions(
    connection,
    [wallet],
    wallet.publicKey,
    instruction,
  );

  console.log(`Transaction: ${tx}`);
};

lock();
