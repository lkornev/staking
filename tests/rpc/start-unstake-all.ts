import * as anchor from "@project-serum/anchor";
import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx, MemberUnstakeAll } from "../ctx/ctx";

export async function startUnstakeAllRPC(ctx: Ctx, memberUnstakeAll: MemberUnstakeAll) {
    await ctx.program.methods.startUnstakeAll(
        memberUnstakeAll.stakePool.rewardType.value,
        memberUnstakeAll.bump,
    )
    .accounts({
        factory: ctx.PDAS.factory.key,
        stakeTokenMint: memberUnstakeAll.stakePool.factory.stakeTokenMint,
        stakePool: memberUnstakeAll.stakePool.key,
        beneficiary: memberUnstakeAll.memberStake.member.beneficiary.publicKey,
        member: memberUnstakeAll.memberStake.member.key,
        memberStake: memberUnstakeAll.memberStake.key,
        vaultStaked: memberUnstakeAll.memberStake.vaultStaked,
        memberPendingUnstake: memberUnstakeAll.key,
        vaultPendingUnstake: memberUnstakeAll.vaultPendingUnstake,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
    })
    .signers([memberUnstakeAll.memberStake.member.beneficiary])
    .rpc();
}
