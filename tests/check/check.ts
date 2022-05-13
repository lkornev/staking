import * as anchor from "@project-serum/anchor";
import { 
    getAccount as getTokenAccount,
} from '@solana/spl-token';
import { expect } from "chai";
import { Ctx, Member, MemberStake, StakePool } from '../ctx/ctx';

export namespace Check {

    export async function factory(ctx: Ctx) {
        const factory = await ctx.program.account.factory.fetch(ctx.PDAS.factory.key);
        expect(`${factory.owner}`).to.be.eq(`${ctx.owner.publicKey}`);
        expect(`${factory.rewardTokenMint}`).to.be.eq(`${ctx.PDAS.factory.rewardTokenMint}`);
        expect(`${factory.stakeTokenMint}`).to.be.eq(`${ctx.PDAS.factory.stakeTokenMint}`);
        expect(`${factory.vaultReward}`).to.be.eq(`${ctx.PDAS.factory.vaultReward}`);
    }

    export async function newStakePool(ctx: Ctx, stakePool: StakePool) {
        const stakePoolAcc = await ctx.program.account.stakePool.fetch(stakePool.key);

        expect(`${stakePoolAcc.totalStakedTokens}`).to.be.eq(`${0}`);
        expect(`${stakePoolAcc.unstakeDelay}`).to.be.eq(`${stakePool.unstakeDelay}`);
        expect(`${stakePoolAcc.unstakeForceFeePercent}`).to.be.eq(`${stakePool.unstakeForceFeePercent}`);
        expect(stakePoolAcc.rewardType).to.be.deep.eq(stakePool.rewardType.value);
        expect(`${stakePoolAcc.rewardMetadata}`).to.be.eq(`${stakePool.rewardMetadata}`);
        expect(`${stakePoolAcc.ownerInterestPercent}`).to.be.eq(`${stakePool.ownerInterestPercent}`);
        expect(`${stakePoolAcc.rewardPeriod}`).to.be.eq(`${stakePool.rewardPeriod}`);
    }

    export async function memberDeposit(ctx: Ctx, member: Member, deposit: (ctx: Ctx, member: Member) => Promise<void>) {
        const beneficiaryAccountStateBefore = await getTokenAccount(ctx.connection, member.beneficiaryTokenAccount.address);
        const memberVaultFreeBefore = await getTokenAccount(ctx.connection, member.vaultFree.publicKey);
        const memberVaultPUBefore = await getTokenAccount(ctx.connection, member.vaultPendingUnstaking.publicKey);
        expect(`${beneficiaryAccountStateBefore.amount}`).to.be.eq(`${member.stakeTokenAmount}`);
        expect(`${memberVaultFreeBefore.amount}`).to.be.eq(`0`);
        expect(`${memberVaultPUBefore.amount}`).to.be.eq(`0`);

        await deposit(ctx, member);

        const memberAcc = await ctx.program.account.member.fetch(member.key);
        expect(`${memberAcc.beneficiary}`).to.be.eq(`${member.beneficiary.publicKey}`);
        expect(`${memberAcc.vaultFree}`).to.be.eq(`${member.vaultFree.publicKey}`);
        expect(`${memberAcc.vaultPendingUnstaking}`).to.be.eq(`${member.vaultPendingUnstaking.publicKey}`);
        expect(memberAcc.bump).to.be.eq(member.bump);

        const beneficiaryAccountState = await getTokenAccount(ctx.connection, member.beneficiaryTokenAccount.address);
        const memberVaultFree = await getTokenAccount(ctx.connection, member.vaultFree.publicKey);
        expect(`${beneficiaryAccountState.amount}`).to.be.eq(`0`);
        expect(`${memberVaultFree.amount}`).to.be.eq(`${member.stakeTokenAmount}`);
    }

    export async function newMember(ctx: Ctx, member: Member) {
        const memberAcc = await ctx.program.account.member.fetch(member.key);
        expect(`${memberAcc.bump}`).to.be.eq(`${member.bump}`);
        expect(`${memberAcc.beneficiary}`).to.be.eq(`${member.beneficiary.publicKey}`);
        expect(`${memberAcc.vaultFree}`).to.be.eq(`${member.vaultFree.publicKey}`);
        expect(`${memberAcc.vaultPendingUnstaking}`).to.be.eq(`${member.vaultPendingUnstaking.publicKey}`);

        const memberVaultFree = await getTokenAccount(ctx.connection, member.vaultFree.publicKey);
        expect(`${memberVaultFree.amount}`).to.be.eq(`${0}`);
    }

    export async function memberStake(
        ctx: Ctx,
        stakePool: StakePool, 
        member: Member, 
        memberStake: MemberStake,
        stake: (ctx: Ctx, stakePool: StakePool, member: Member, memberStake: MemberStake) => Promise<void>) 
    {
        const stakeholderVault = await getTokenAccount(ctx.connection, memberStake.vaultStaked.publicKey);
        expect(`${stakeholderVault.amount}`).to.be.eq(`0`);

        await stake(ctx, stakePool, member, memberStake);

        // TODO check stakeholder fields

        const stakeholderVaultChanged = await getTokenAccount(ctx.connection, memberStake.vaultStaked.publicKey);
        expect(`${stakeholderVaultChanged.amount}`).to.be.eq(`${memberStake.amountToStake}`);
    }

    export async function depositReward(
        ctx: Ctx, 
        depositReward: (ctx: Ctx, rewardTokensAmount: number) => Promise<void>, 
        checks: { rewardAmountBefore: number, rewardAmountAfter: number }
    ) {
        const factory = await ctx.program.account.factory.fetch(ctx.PDAS.factory.key);
        const factoryRewardVault = await getTokenAccount(ctx.connection, factory.vaultReward);

        expect(`${factoryRewardVault.amount}`).to.be.eq(`${checks.rewardAmountBefore}`);

        await depositReward(ctx, Number(ctx.owner.initialRewardTokensAmount));

        const factoryRewardVaultChanged = await getTokenAccount(ctx.connection, factory.vaultReward);
        expect(`${factoryRewardVaultChanged.amount}`).to.be.eq(`${checks.rewardAmountAfter}`);
    }

    export async function claimReward(
        ctx: Ctx,
        memberStake: MemberStake,
        claimReward: (ctx: Ctx, memberStake: MemberStake) => Promise<void>
    ) {
        const factory = await ctx.program.account.factory.fetch(ctx.PDAS.factory.key);
        const userRewardBefore = (await getTokenAccount(ctx.connection, ctx.PDAS.member.beneficiaryRewardVault.address)).amount;
        const ownerFeeBefore = (await getTokenAccount(ctx.connection, ctx.owner.feeRewardVault)).amount;

        await claimReward(ctx, memberStake);

        const userRewardAfter = (await getTokenAccount(ctx.connection, ctx.PDAS.member.beneficiaryRewardVault.address)).amount;
        expect(Number(userRewardAfter)).to.be.above(Number(userRewardBefore));

        const factoryRewardVaultAfter = await getTokenAccount(ctx.connection, factory.vaultReward);
        expect(Number(factoryRewardVaultAfter.amount)).to.be.below(Number(ctx.owner.initialRewardTokensAmount));

        const ownerFeeAfter = (await getTokenAccount(ctx.connection, ctx.owner.feeRewardVault)).amount;
        expect(Number(ownerFeeAfter)).to.be.above(Number(ownerFeeBefore));
    }
}
