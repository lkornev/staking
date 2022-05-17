import * as anchor from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx, StakePool, Member, MemberStake } from "../ctx/ctx";

export async function stakeRPC(ctx: Ctx, stakePool: StakePool, member: Member, memberStake: MemberStake) {
    await ctx.program.methods.stake(memberStake.amountToStake)
    .accounts({
        factory: ctx.PDAS.factory.key,
        stakeTokenMint: ctx.PDAS.factory.stakeTokenMint,
        stakePool: stakePool.key,
        beneficiary: member.beneficiary.publicKey,
        member: member.key,
        vaultFree: member.vaultFree,
        memberStake: memberStake.key,
        vaultStaked: memberStake.vaultStaked,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([member.beneficiary])
    .rpc();
}
