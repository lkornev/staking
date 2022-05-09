import * as anchor from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import { Ctx } from "../ctx/ctx";
import { Reward } from "../types/reward";

export async function newStakePoolRPC(ctx: Ctx) {
    await ctx.program.methods.new(
        Reward.Fixed.value,
        ctx.unstakeDelay, // secs
        ctx.unstakeForseFeePercent, // %,
        ctx.rewardMetadata, // %,
        ctx.configHistoryLength,
        ctx.PDAS.stakePoolFixed.bump,
    )
    .accounts({
        factory: ctx.PDAS.factory.key,
        stakePool: ctx.PDAS.stakePoolFixed.key,
        configHistory: ctx.configHistoryKeypair.publicKey,
        owner: ctx.owner.publicKey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
    })
    .signers([ctx.owner, ctx.configHistoryKeypair])
    .preInstructions([
        await ctx.program.account.configHistory.createInstruction(
            ctx.configHistoryKeypair, 
            ctx.configHistoryLength * ctx.configHistoryElSize + ctx.configHistoryMetadata
        ),
    ])
    .rpc();
}
