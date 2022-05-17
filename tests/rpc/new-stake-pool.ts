import * as anchor from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import { Ctx, StakePool } from "../ctx/ctx";

export async function newStakePoolRPC(ctx: Ctx, stakePool: StakePool) {
    await ctx.program.methods.newStakePool(
        stakePool.name,
        stakePool.rewardType.value as any,
        stakePool.endedAt,
        stakePool.minOwnerReward,
        stakePool.ownerInterestPercent, // %
        stakePool.unstakeDelay, // secs
        stakePool.rewardPeriod,
    )
    .accounts({
        factory: ctx.PDAS.factory.key,
        stakePool: stakePool.key,
        owner: ctx.owner.publicKey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
    })
    .signers([ctx.owner])
    .rpc();
}
