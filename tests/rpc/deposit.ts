import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx, Member } from "../ctx/ctx";

export async function depositRPC(ctx: Ctx, member: Member) {
    await ctx.program.methods.deposit(
        member.amountToDeposit
    )
    .accounts({
        factory: ctx.PDAS.factory.key,
        beneficiary: member.beneficiary.publicKey,
        beneficiaryTokenAccount: member.beneficiaryTokenAccount.address,
        member: member.key,
        vaultFree: member.vaultFree.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([member.beneficiary])
    .rpc();
}
