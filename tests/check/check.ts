import { 
    getAccount as getTokenAccount,
} from '@solana/spl-token';
import { expect } from "chai";
import { Ctx, Member, MemberStake, StakePool } from '../ctx/ctx';
import { CtxRPC, sliceCtxRpc } from "../types/ctx-rpc";

export namespace Check {

    export async function factory(ctx: Ctx) {
        const factory = await ctx.program.account.factory.fetch(ctx.PDAS.factory.key);
        expect(`${factory.owner}`).to.be.eq(`${ctx.owner.publicKey}`);
        expect(factory.ownerInterest).to.be.eq(ctx.ownerInterest);
        expect(`${factory.configChangeDelay}`).to.be.eq(`${ctx.configChangeDelay}`);
        expect(`${factory.rewardPeriod}`).to.be.eq(`${ctx.rewardPeriod}`);
        expect(`${factory.rewardTokenMint}`).to.be.eq(`${ctx.rewardTokenMint}`);
        expect(`${factory.stakeTokenMint}`).to.be.eq(`${ctx.stakeTokenMint}`);
        expect(`${factory.vaultReward}`).to.be.eq(`${ctx.vaultReward}`);
    }

    export async function newStakePool(ctx: Ctx, stakePool: StakePool) {
        const stakePoolAcc = await ctx.program.account.stakePool.fetch(ctx.PDAS.stakePoolFixed.key);
        const configHistory = await ctx.program.account.configHistory.fetch(stakePoolAcc.configHistory);
        expect(`${stakePoolAcc.configHistory}`).to.be.eq(`${stakePool.configHistoryKeypair.publicKey}`);

        for (let i = 1; i < stakePool.configHistoryLength; i++) {
            expect(configHistory.history[i], `el № ${i}`).to.be.eq(null);
        }

        const stakePoolConfig = configHistory.history[0];
        expect(`${stakePoolConfig.totalStakedTokens}`).to.be.eq(`${0}`);
        expect(`${stakePoolConfig.unstakeDelay}`).to.be.eq(`${stakePool.unstakeDelay}`);
        expect(`${stakePoolConfig.unstakeForseFeePercent}`).to.be.eq(`${stakePool.unstakeForseFeePercent}`);
        expect(stakePoolConfig.rewardType).to.be.deep.eq(stakePool.rewardType.value);
        expect(`${stakePoolConfig.rewardMetadata}`).to.be.eq(`${stakePool.rewardMetadata}`);
    }

    export async function memberDeposit(ctx: Ctx, member: Member, deposit: (ctx: CtxRPC, member: Member) => Promise<void>) {
        const beneficiaryAccountStateBefore = await getTokenAccount(ctx.connection, member.beneficiaryTokenAccount.address);
        const memberVaultFreeBefore = await getTokenAccount(ctx.connection, member.vaultFree.publicKey);
        const memberVaultPUBefore = await getTokenAccount(ctx.connection, member.vaultPendingUnstaking.publicKey);
        expect(`${beneficiaryAccountStateBefore.amount}`).to.be.eq(`${member.stakeTokenAmount}`);
        expect(`${memberVaultFreeBefore.amount}`).to.be.eq(`0`);
        expect(`${memberVaultPUBefore.amount}`).to.be.eq(`0`);

        await deposit(sliceCtxRpc(ctx), member);

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
        stake: (ctx: CtxRPC, stakePool: StakePool, member: Member, memberStake: MemberStake) => Promise<void>) 
    {
        const stakeholderVault = await getTokenAccount(ctx.connection, memberStake.vaultStaked.publicKey);
        expect(`${stakeholderVault.amount}`).to.be.eq(`0`);

        await stake(sliceCtxRpc(ctx), stakePool, member, memberStake);

        // TODO check stakeholder fields

        const stakeholderVaultChanged = await getTokenAccount(ctx.connection, memberStake.vaultStaked.publicKey);
        expect(`${stakeholderVaultChanged.amount}`).to.be.eq(`${memberStake.amountToStake}`);
    }

    export async function depositReward(
        ctx: Ctx, 
        depositReward: (ctx: CtxRPC, rewardTokensAmount: number) => Promise<void>, 
        checks: { rewardAmountBefore: number }
    ) {
        const factory = await ctx.program.account.factory.fetch(ctx.PDAS.factory.key);
        const factoryRewardVault = await getTokenAccount(ctx.connection, factory.vaultReward);

        expect(`${factoryRewardVault.amount}`).to.be.eq(`${checks.rewardAmountBefore}`);

        await depositReward(sliceCtxRpc(ctx), ctx.rewardTokensAmount);

        const factoryRewardVaultChanged = await getTokenAccount(ctx.connection, factory.vaultReward);
        expect(`${factoryRewardVaultChanged.amount}`).to.be.eq(`${ctx.rewardTokensAmount}`);
    }
}
