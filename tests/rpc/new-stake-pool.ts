import * as anchor from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import { Ctx, StakePool } from "../ctx/ctx";

export async function newStakePoolRPC(ctx: Ctx, stakePool: StakePool) {
    await ctx.program.methods.newStakePool(
        stakePool.rewardType.value,
        stakePool.bump,
        stakePool.endedAt,
        stakePool.unstakeDelay, // secs
        stakePool.unstakeForceFeePercent, // %,
        stakePool.minOwnerReward,
        stakePool.rewardMetadata,
        stakePool.ownerInterestPercent, // %
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
