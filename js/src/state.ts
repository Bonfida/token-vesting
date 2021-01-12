import { PublicKey } from '@solana/web3.js';
import { Numberu64 } from './utils';

export class Schedule {
  releaseHeight!: Numberu64;
  amount!: Numberu64;

  constructor(releaseHeight: Numberu64, amount: Numberu64) {
    this.releaseHeight = releaseHeight;
    this.amount = amount;
  }

  public toBuffer(): Buffer {
    return Buffer.concat([
      this.releaseHeight.toBuffer(),
      this.amount.toBuffer(),
    ]);
  }

  static fromBuffer(buf: Buffer): Schedule {
    const releaseHeight: Numberu64 = Numberu64.fromBuffer(buf.slice(0, 8));
    const amount: Numberu64 = Numberu64.fromBuffer(buf.slice(8, 16));
    return new Schedule(releaseHeight, amount);
  }
}

export class VestingScheduleHeader {
  destinationAddress!: PublicKey;
  mintAddress!: PublicKey;
  isInitialized!: boolean;

  constructor(
    destinationAddress: PublicKey,
    mintAddress: PublicKey,
    isInitialized: boolean,
  ) {
    this.destinationAddress = destinationAddress;
    this.mintAddress = mintAddress;
    this.isInitialized = isInitialized;
  }

  static fromBuffer(buf: Buffer): VestingScheduleHeader {
    const destinationAddress = new PublicKey(buf.slice(0, 32));
    const mintAddress = new PublicKey(buf.slice(32, 64));
    const isInitialized = buf[65] == 1;
    const header: VestingScheduleHeader = {
      destinationAddress,
      mintAddress,
      isInitialized,
    };
    return header;
  }
}

export class ContractInfo {
  destinationAddress!: PublicKey;
  mintAddress!: PublicKey;
  schedules!: Array<Schedule>;

  constructor(
    destinationAddress: PublicKey,
    mintAddress: PublicKey,
    schedules: Array<Schedule>,
  ) {
    this.destinationAddress = destinationAddress;
    this.mintAddress = mintAddress;
    this.schedules = schedules;
  }

  static fromBuffer(buf: Buffer): ContractInfo | undefined {
    const header = VestingScheduleHeader.fromBuffer(buf.slice(0, 65));
    if (!header.isInitialized) {
      return undefined;
    }
    const schedules: Array<Schedule> = [];
    for (let i = 65; i < buf.length; i += 16) {
      schedules.push(Schedule.fromBuffer(buf.slice(i, i + 16)));
    }
    return new ContractInfo(
      header.destinationAddress,
      header.mintAddress,
      schedules,
    );
  }
}
