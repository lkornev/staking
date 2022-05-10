import * as anchor from "@project-serum/anchor";
import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { CtxRPC } from "../types/ctx-rpc";

export async function depositRewardRPC(ctx: CtxRPC, rewardTokensAmount: number) {
    await ctx.program.methods.depositReward(new anchor.BN(rewardTokensAmount))
        .accounts({
            factory: ctx.factory,
            owner: ctx.owner.publicKey,
            vaultOwner: ctx.ownerTokenAccount.address,
            vaultReward: ctx.vaultReward,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([ctx.owner])
        .rpc();
}
