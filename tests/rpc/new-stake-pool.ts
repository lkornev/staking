import * as anchor from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import { StakePool } from "../ctx/ctx";
import { CtxRPC } from "../types/ctx-rpc";

export async function newStakePoolRPC(ctx: CtxRPC, stakePool: StakePool) {
    await ctx.program.methods.new(
        stakePool.rewardType.value,
        stakePool.unstakeDelay, // secs
        stakePool.unstakeForseFeePercent, // %,
        stakePool.rewardMetadata, // %,
        stakePool.configHistoryLength,
        stakePool.bump,
    )
    .accounts({
        factory: ctx.factory,
        stakePool: stakePool.key,
        configHistory: stakePool.configHistoryKeypair.publicKey,
        owner: ctx.owner.publicKey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
    })
    .signers([ctx.owner, stakePool.configHistoryKeypair])
    .preInstructions([
        await ctx.program.account.configHistory.createInstruction(
            stakePool.configHistoryKeypair, 
            stakePool.configHistoryLength * stakePool.configHistoryElSize + stakePool.configHistoryMetadata
        ),
    ])
    .rpc();
}
