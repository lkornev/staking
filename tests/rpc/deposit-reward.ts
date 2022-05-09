import * as anchor from "@project-serum/anchor";
import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx } from "../ctx/ctx";

export async function depositRewardRPC(ctx: Ctx) {
    await ctx.program.methods.depositReward(new anchor.BN(ctx.rewardTokensAmount))
        .accounts({
            factory: ctx.PDAS.factory.key,
            owner: ctx.owner.publicKey,
            vaultOwner: ctx.ownerTokenAccount.address,
            vaultReward: ctx.vaultReward,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([ctx.owner])
        .rpc();
}
