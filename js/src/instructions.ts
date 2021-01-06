import { u64 } from "@solana/spl-token";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import * as BufferLayout from "buffer-layout";

export enum Instruction {
    Init,
    Create
}

export function createInitInstruction(
    systemProgramId: PublicKey,
    vestingProgramId: PublicKey,
    payerKey: PublicKey,
    vestingAccountKey: PublicKey,
    seeds:Array<Buffer | Uint8Array>,
    numberOfSchedules: u64
): TransactionInstruction{
    let buffers = [
        Buffer.from(Int8Array.from([0]).buffer),
        Buffer.concat(seeds),
        numberOfSchedules.toBuffer()
    ]
    const data =  Buffer.concat(buffers);
    const keys = [
        {
            pubkey: systemProgramId,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: payerKey,
            isSigner: true,
            isWritable: true
        },
        {
            pubkey: vestingAccountKey,
            isSigner: false,
            isWritable: true
        }
    ];
    return new TransactionInstruction({
        keys,
        programId: vestingProgramId,
        data
    })
}

export function createCreateInstruction(
    vestingProgramId: PublicKey,
    tokenProgramId: PublicKey,
    vestingAccountKey: PublicKey,
    vestingTokenAccountKey: PublicKey,
    sourceTokenAccountOwnerKey: PublicKey,
    sourceTokenAccountKey: PublicKey,
    destinationTokenAccountKey: PublicKey,
    mintAddress: PublicKey,
    schedules: Array<Schedule>,
    seeds:Array<Buffer | Uint8Array>
): TransactionInstruction{
    let buffers = [
        Buffer.from(Int8Array.from([1]).buffer),
        Buffer.concat(seeds),
        mintAddress.toBuffer(),
        destinationTokenAccountKey.toBuffer()
    ]
    schedules.map(s => {buffers.push(s.toBuffer())})
    const data =  Buffer.concat(buffers);
    const keys = [
        {
            pubkey: tokenProgramId,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: vestingAccountKey,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: vestingTokenAccountKey,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: sourceTokenAccountOwnerKey,
            isSigner: true,
            isWritable: false
        },
        {
            pubkey: sourceTokenAccountKey,
            isSigner: false,
            isWritable: true
        },
    ];
    return new TransactionInstruction({
        keys,
        programId: vestingProgramId,
        data
    })
}

export function createUnlockInstruction(
    vestingProgramId: PublicKey,
    tokenProgramId: PublicKey,
    clockSysvarId: PublicKey,
    vestingAccountKey: PublicKey,
    vestingTokenAccountKey: PublicKey,
    destinationTokenAccountKey: PublicKey,
    seeds:Array<Buffer | Uint8Array>
):TransactionInstruction{
    const data = Buffer.concat(seeds);

    const keys = [
        {
            pubkey: tokenProgramId,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: clockSysvarId,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: vestingAccountKey,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: vestingTokenAccountKey,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: destinationTokenAccountKey,
            isSigner: false,
            isWritable: true
        },
    ];
    return new TransactionInstruction({
        keys,
        programId: vestingProgramId,
        data
    })

}



export function createChangeDestinationInstruction(
    vestingProgramId: PublicKey,
    vestingAccountKey: PublicKey,
    currentDestinationTokenAccountOwner: PublicKey,
    currentDestinationTokenAccount: PublicKey,
    targetDestinationTokenAccount: PublicKey,
    seeds:Array<Buffer | Uint8Array>
):TransactionInstruction{
    const data = Buffer.concat(seeds);

    const keys = [
        {
            pubkey: vestingAccountKey,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: currentDestinationTokenAccount,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: currentDestinationTokenAccountOwner,
            isSigner: true,
            isWritable: false
        },
        {
            pubkey: targetDestinationTokenAccount,
            isSigner: false,
            isWritable: false
        },
    ];
    return new TransactionInstruction({
        keys,
        programId: vestingProgramId,
        data
    })

}

class Create {
    seeds : Array<Buffer | Uint8Array>;
    mint_address: PublicKey;
    destination_token_address: PublicKey;
    schedules: Array<Schedule>;
}

class Unlock {
    seeds : Array<Buffer | Uint8Array>;
}

class ChangeDestination {
    seeds : Array<Buffer | Uint8Array>;
}

export class Schedule {
    release_height: u64;
    amount: u64;

    toBuffer() {
        return Buffer.concat([
            this.release_height.toBuffer(),
            this.amount.toBuffer()
        ])
    }
}