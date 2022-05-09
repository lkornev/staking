import * as anchor from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx } from "../ctx/ctx";
import { Reward } from "../types/reward";

export async function stakeRPC(ctx: Ctx, rewardType: number) {
    await ctx.program.methods.stake(
        Reward.Fixed.value,
        ctx.PDAS.stakePoolFixed.bump,
        ctx.amountToStake
    )
    .accounts({
        factory: ctx.PDAS.factory.key,
        stakePool: ctx.PDAS.stakePoolFixed.key,
        beneficiary: ctx.beneficiary.publicKey,
        member: ctx.PDAS.memberFixed.key,
        vaultFree: ctx.vaultFree.publicKey,
        stakeholder: ctx.PDAS.stakeholderFixed.key,
        vaultStaked: ctx.vaultStakedFixed.publicKey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([ctx.beneficiary])
    .rpc();
}
