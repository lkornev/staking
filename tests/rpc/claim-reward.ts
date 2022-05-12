import * as anchor from "@project-serum/anchor";
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Ctx, MemberStake } from "../ctx/ctx";

export async function claimRewardRPC(ctx: Ctx, memberStake: MemberStake) {
    await ctx.program.methods.claimReward(memberStake.stakePool.rewardType.value)
        .accounts({
            factory: ctx.PDAS.factory.key,
            stakePool: memberStake.stakePool.key,
            memberStake: memberStake.key,
            member: memberStake.member.key,
            vaultStaked: memberStake.vaultStaked.publicKey,
            vaultReward: ctx.PDAS.factory.vaultReward,
            beneficiary: memberStake.member.beneficiary.publicKey,
            beneficiaryRewardVault: memberStake.member.beneficiaryRewardVault.address,
            ownerFeeRewardVault: ctx.owner.feeRewardVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([memberStake.member.beneficiary])
        .rpc();
}
