import * as anchor from "@project-serum/anchor";
import { SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx, MemberUnstakeAll } from "../ctx/ctx";

export async function finishUnstakeAllRPC(ctx: Ctx, memberUnstakeAll: MemberUnstakeAll) {
    await ctx.program.methods.finishUnstakeAll(
        memberUnstakeAll.stakePool.rewardType.value,
    )
    .accounts({
        factory: ctx.PDAS.factory.key,
        stakePool: memberUnstakeAll.stakePool.key,
        beneficiary: memberUnstakeAll.memberStake.member.beneficiary.publicKey,
        vaultBeneficiary: memberUnstakeAll.member.beneficiaryStakeVault,
        member: memberUnstakeAll.member.key,
        memberStake: memberUnstakeAll.memberStake.key,
        vaultStaked: memberUnstakeAll.memberStake.vaultStaked,
        memberPendingUnstake: memberUnstakeAll.key,
        vaultPendingUnstake: memberUnstakeAll.vaultPendingUnstake,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
    })
    .signers([memberUnstakeAll.memberStake.member.beneficiary])
    .rpc();
}
