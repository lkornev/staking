import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx } from "../ctx/ctx";
import { Reward } from '../types/reward';

export async function depositRPC(ctx: Ctx, rewardType: number) {
    if (rewardType === Reward.Fixed.index) {
        await ctx.program.methods.deposit(
            ctx.PDAS.memberFixed.bump,
            ctx.amountToDeposit
        )
        .accounts({
            factory: ctx.PDAS.factory.key,
            beneficiary: ctx.beneficiary.publicKey,
            beneficiaryTokenAccount: ctx.beneficiaryTokenAccount.address,
            member: ctx.PDAS.memberFixed.key,
            vaultFree: ctx.vaultFree.publicKey,
            vaultPendingUnstaking: ctx.vaultPendingUnstaking.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([ctx.beneficiary])
        .rpc();
    } else {
        console.error('unimplemented!');
    }
}
