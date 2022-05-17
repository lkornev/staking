import * as anchor from "@project-serum/anchor";
import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID,ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx, Member } from "../ctx/ctx";

export async function createMemberRPC(ctx: Ctx, member: Member) {
    await ctx.program.methods.createMember()
    .accounts({
        factory: ctx.PDAS.factory.key,
        stakeTokenMint: ctx.PDAS.factory.stakeTokenMint,
        beneficiary: member.beneficiary.publicKey,
        member: member.key,
        vaultFree: member.vaultFree,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([member.beneficiary])
    .rpc();
}
