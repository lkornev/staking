import * as anchor from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { StakePool, Member, MemberStake } from "../ctx/ctx";
import { CtxRPC } from "../types/ctx-rpc";

export async function stakeRPC(ctx: CtxRPC, stakePool: StakePool, member: Member, memberStake: MemberStake) {
    await ctx.program.methods.stake(
        stakePool.rewardType.value,
        stakePool.bump,
        memberStake.amountToStake
    )
    .accounts({
        factory: ctx.factory,
        stakePool: stakePool.key,
        beneficiary: member.beneficiary.publicKey,
        member: member.key,
        vaultFree: member.vaultFree.publicKey,
        stakeholder: memberStake.key,
        vaultStaked: memberStake.vaultStaked.publicKey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([member.beneficiary])
    .rpc();
}
