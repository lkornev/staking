import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Member } from "../ctx/ctx";
import { CtxRPC } from "../types/ctx-rpc";

export async function createMemberRPC(ctx: CtxRPC, member: Member) {
    await ctx.program.methods.createMember(
        member.bump,
    )
    .accounts({
        factory: ctx.factory,
        beneficiary: member.beneficiary.publicKey,
        member: member.key,
        vaultFree: member.vaultFree.publicKey,
        vaultPendingUnstaking: member.vaultPendingUnstaking.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([member.beneficiary])
    .rpc();
}
