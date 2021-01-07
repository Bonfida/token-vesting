
import { Layout } from 'buffer-layout';
import {
  option,
  i64,
  publicKey,
  rustEnum,
  u64,
  u32,
  struct,
  u8,
} from '@project-serum/borsh';
import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

type InitInstruction = {
    padding: u8, // ?
    seeds: Array<Buffer | Uint8Array>, // ?
    numberOfSchedules: number
}

const INIT_INSTRUCTION_LAYOUT: Layout<InitInstruction> = struct(
    [
        // TODO
    ]
)

// Same for CreateInstruction, UnlockInstruction, ChangeDestination



export function decode(data: Buffer, layout: Layout<any>): Layout<any> {
    return layout.decode(data);
  }
  
  export function encode(i: any, layout: Layout<any>): Buffer {
    const buffer = Buffer.alloc(1000); // TODO: use a tighter buffer.
    const len = layout.encode(i, buffer);
    return buffer.slice(0, len);
  }