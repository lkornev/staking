import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx, Member } from "../ctx/ctx";

export async function withdrawalAllRPC(ctx: Ctx, member: Member) {
    await ctx.program.methods.withdrawAll()
    .accounts({
        factory: ctx.PDAS.factory.key,
        beneficiary: member.beneficiary.publicKey,
        beneficiaryTokenAccount: member.beneficiaryStakeVault,
        member: member.key,
        vaultFree: member.vaultFree,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([member.beneficiary])
    .rpc();
}
