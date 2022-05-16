import * as anchor from "@project-serum/anchor";
import { 
    getAccount as getTokenAccount,
    getMinimumBalanceForRentExemptAccount,
} from '@solana/spl-token';
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { Ctx, Member, MemberStake, MemberUnstakeAll, StakePool } from '../ctx/ctx';

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
        expect(stakePoolAcc.rewardType).to.be.deep.eq(stakePool.rewardType.value);
        expect(`${stakePoolAcc.rewardMetadata}`).to.be.eq(`${stakePool.rewardMetadata}`);
        expect(`${stakePoolAcc.ownerInterestPercent}`).to.be.eq(`${stakePool.ownerInterestPercent}`);
        expect(`${stakePoolAcc.rewardPeriod}`).to.be.eq(`${stakePool.rewardPeriod}`);
    }

    export async function memberDeposit(ctx: Ctx, member: Member, deposit: (ctx: Ctx, member: Member) => Promise<void>) {
        const beneficiaryAccountStateBefore = await getTokenAccount(ctx.connection, member.beneficiaryStakeVault);
        const memberVaultFreeBefore = await getTokenAccount(ctx.connection, member.vaultFree);
        expect(`${beneficiaryAccountStateBefore.amount}`).to.be.eq(`${member.stakeTokenAmount}`);
        expect(`${memberVaultFreeBefore.amount}`).to.be.eq(`0`);

        await deposit(ctx, member);

        const memberAcc = await ctx.program.account.member.fetch(member.key);
        expect(`${memberAcc.beneficiary}`).to.be.eq(`${member.beneficiary.publicKey}`);
        expect(`${memberAcc.vaultFree}`).to.be.eq(`${member.vaultFree}`);
        expect(memberAcc.bump).to.be.eq(member.bump);

        const beneficiaryAccountState = await getTokenAccount(ctx.connection, member.beneficiaryStakeVault);
        const memberVaultFree = await getTokenAccount(ctx.connection, member.vaultFree);
        expect(`${beneficiaryAccountState.amount}`).to.be.eq(`0`);
        expect(`${memberVaultFree.amount}`).to.be.eq(`${member.stakeTokenAmount}`);
    }

    export async function newMember(ctx: Ctx, member: Member) {
        const memberAcc = await ctx.program.account.member.fetch(member.key);
        expect(`${memberAcc.bump}`).to.be.eq(`${member.bump}`);
        expect(`${memberAcc.beneficiary}`).to.be.eq(`${member.beneficiary.publicKey}`);
        expect(`${memberAcc.vaultFree}`).to.be.eq(`${member.vaultFree}`)

        const memberVaultFree = await getTokenAccount(ctx.connection, member.vaultFree);
        expect(`${memberVaultFree.amount}`).to.be.eq(`${0}`);
    }

    export async function memberStake(
        ctx: Ctx,
        stakePool: StakePool, 
        member: Member, 
        memberStake: MemberStake,
        stake: (ctx: Ctx, stakePool: StakePool, member: Member, memberStake: MemberStake) => Promise<void>) 
    {
        await stake(ctx, stakePool, member, memberStake);

        // TODO check memberStakeVault fields

        const memberStakeVaultVaultChanged = await getTokenAccount(ctx.connection, memberStake.vaultStaked);
        expect(`${memberStakeVaultVaultChanged.amount}`).to.be.eq(`${memberStake.amountToStake}`);
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
        const userRewardBefore = (await getTokenAccount(ctx.connection, ctx.PDAS.member.beneficiaryRewardVault)).amount;
        const ownerFeeBefore = (await getTokenAccount(ctx.connection, ctx.owner.feeRewardVault)).amount;

        await claimReward(ctx, memberStake);

        const userRewardAfter = (await getTokenAccount(ctx.connection, ctx.PDAS.member.beneficiaryRewardVault)).amount;
        expect(Number(userRewardAfter)).to.be.above(Number(userRewardBefore));

        const factoryRewardVaultAfter = await getTokenAccount(ctx.connection, factory.vaultReward);
        expect(Number(factoryRewardVaultAfter.amount)).to.be.below(Number(ctx.owner.initialRewardTokensAmount));

        const ownerFeeAfter = (await getTokenAccount(ctx.connection, ctx.owner.feeRewardVault)).amount;
        expect(Number(ownerFeeAfter)).to.be.above(Number(ownerFeeBefore));
    }

    export async function startUnstakeAll(
        ctx: Ctx,
        memberUnstakeAll: MemberUnstakeAll,
        startUnstakeAll: (ctx: Ctx, memberUnstakeAll: MemberUnstakeAll) => Promise<void>,
    ) {
        const stakedAmountBefore = (await getTokenAccount(ctx.connection, memberUnstakeAll.memberStake.vaultStaked)).amount;
        const totalStakedBefore = (await ctx.program.account.stakePool.fetch(memberUnstakeAll.stakePool.key)).totalStakedTokens;

        await startUnstakeAll(ctx, memberUnstakeAll);

        // Pending unstake correctly initialized
        const memberPendingUnstakeAcc = await ctx.program.account.memberPendingUnstake.fetch(memberUnstakeAll.key);
        expect((memberPendingUnstakeAcc).bump).to.be.eq(memberUnstakeAll.bump);
        expect(`${(memberPendingUnstakeAcc).stakePool}`).to.be.eq(`${memberUnstakeAll.stakePool.key}`);
        expect(`${(memberPendingUnstakeAcc).beneficiary}`).to.be
            .eq(`${memberUnstakeAll.memberStake.member.beneficiary.publicKey}`);
        expect(`${(memberPendingUnstakeAcc).vaultPendingUnstake}`).to.be.eq(`${memberUnstakeAll.vaultPendingUnstake}`);
        const nowSecs = Math.floor((Date.now() / 1000));
        expect(Number((memberPendingUnstakeAcc).unstakedAt)).to.be.below(nowSecs + 2).to.be.above(nowSecs - 2);

        // Total staked amount reduced
        const totalStakedAfter = (await ctx.program.account.stakePool.fetch(memberUnstakeAll.stakePool.key)).totalStakedTokens;
        expect(`${Number(totalStakedBefore) - Number(stakedAmountBefore)}`).to.be.eq(`${totalStakedAfter}`);
    }

    export async function finishUnstakeAll(
        ctx: Ctx,
        memberUnstakeAll: MemberUnstakeAll,
        finishUnstakeAll: (ctx: Ctx, memberUnstakeAll: MemberUnstakeAll) => Promise<void>,
    ) {
        const beneficiaryKey = memberUnstakeAll.member.beneficiary.publicKey;
        const vaultPUBefore = await getTokenAccount(ctx.connection, memberUnstakeAll.vaultPendingUnstake);
        const vaultFreeBefore = await getTokenAccount(ctx.connection, memberUnstakeAll.member.vaultFree);
        const beneficiaryLamportsBefore = (await ctx.connection.getAccountInfo(beneficiaryKey)).lamports;
        const tokenAccountRent = await getMinimumBalanceForRentExemptAccount(ctx.connection);
        const memberStakeRent = (await ctx.connection.getAccountInfo(memberUnstakeAll.memberStake.key)).lamports;
        const memberPendingUnstakeRent = (await ctx.connection.getAccountInfo(memberUnstakeAll.key)).lamports;
        const rentToBeReturned = 2 * tokenAccountRent + memberStakeRent + memberPendingUnstakeRent;

        await finishUnstakeAll(ctx, memberUnstakeAll);

        const vaultFreeAfter = await getTokenAccount(ctx.connection, memberUnstakeAll.member.vaultFree);
        const beneficiaryLamportsAfter = (await ctx.connection.getAccountInfo(beneficiaryKey)).lamports;

        // All tokens have moved from vaultPU to vaultFree
        expect(Number(vaultFreeAfter.amount) - Number(vaultFreeBefore.amount)).to.be.eq(Number(vaultPUBefore.amount));
        // Rent-exempt lamports for no longer used accounts has returned to the member's beneficiary
        expect(Number(beneficiaryLamportsAfter) - Number(beneficiaryLamportsBefore)).to.be.eq(rentToBeReturned);
    }

    export async function withdrawAll(ctx: Ctx, member: Member, withdrawAll: (ctx: Ctx, member: Member) => Promise<void>) {
        const beneficiaryVaultBefore = Number((await getTokenAccount(ctx.connection, member.beneficiaryStakeVault)).amount);
        const vaultFreeBefore = Number((await getTokenAccount(ctx.connection, member.vaultFree)).amount);

        await withdrawAll(ctx, member);

        const beneficiaryVaultAfter = Number((await getTokenAccount(ctx.connection, member.beneficiaryStakeVault)).amount);
        const vaultFreeAfter = Number((await getTokenAccount(ctx.connection, member.vaultFree)).amount);
        expect(vaultFreeAfter).to.be.eq(0);
        expect(beneficiaryVaultAfter).to.be.eq(beneficiaryVaultBefore + vaultFreeBefore);
    }
}
