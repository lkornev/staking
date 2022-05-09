import * as anchor from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { Ctx } from "../ctx/ctx";

export async function initializeRPC(ctx: Ctx) {
    await ctx.program.methods.initialize(
        ctx.owner.publicKey,
        ctx.ownerInterest,
        ctx.configChangeDelay,
        ctx.rewardPeriod,
    )
    .accounts({
        factory: ctx.PDAS.factory.key,
        rewardTokenMint: ctx.rewardTokenMint,
        stakeTokenMint: ctx.stakeTokenMint,
        vaultReward: ctx.vaultReward,
        initializer: ctx.owner.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([ctx.owner])
    .rpc();
}
