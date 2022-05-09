import * as anchor from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import {
    SystemProgram,
} from '@solana/web3.js';
import { 
    TOKEN_PROGRAM_ID, 
    createAccount as createTokenAccount,
    Account as TokenAccount,
    getAccount as getTokenAccount,
} from '@solana/spl-token';
import { expect } from "chai";
import { Ctx, createCtx } from '../ctx/ctx';
import { Reward } from '../types/reward';

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

    export async function newStakePool(ctx: Ctx) {
        const stakePool = await ctx.program.account.stakePool.fetch(ctx.PDAS.stakePoolFixed.key);
        const configHistory = await ctx.program.account.configHistory.fetch(ctx.configHistoryKeypair.publicKey);
        expect(`${stakePool.configHistory}`).to.be.eq(`${ctx.configHistoryKeypair.publicKey}`);

        for (let i = 1; i < ctx.configHistoryLength; i++) {
            expect(configHistory.history[i], `el № ${i}`).to.be.eq(null);
        }

        const stakePoolConfig = configHistory.history[0];
        expect(`${stakePoolConfig.totalStakedTokens}`).to.be.eq(`${0}`);
        expect(`${stakePoolConfig.unstakeDelay}`).to.be.eq(`${ctx.unstakeDelay}`);
        expect(`${stakePoolConfig.unstakeForseFeePercent}`).to.be.eq(`${ctx.unstakeForseFeePercent}`);
        expect(stakePoolConfig.rewardType).to.be.deep.eq(Reward.Fixed.value);
        expect(`${stakePoolConfig.rewardMetadata}`).to.be.eq(`${ctx.rewardMetadata}`);
    }

    export async function memberFixedDeposit(ctx: Ctx, deposit: (ctx: Ctx, rewardType: number) => Promise<void>) {
        const beneficiaryAccountStateBefore = await getTokenAccount(ctx.connection, ctx.beneficiaryTokenAccount.address);
        const memberVaultFreeBefore = await getTokenAccount(ctx.connection, ctx.vaultFree.publicKey);
        const memberVaultPUBefore = await getTokenAccount(ctx.connection, ctx.vaultPendingUnstaking.publicKey);
        expect(`${beneficiaryAccountStateBefore.amount}`).to.be.eq(`${ctx.stakeTokenAmount}`);
        expect(`${memberVaultFreeBefore.amount}`).to.be.eq(`0`);
        expect(`${memberVaultPUBefore.amount}`).to.be.eq(`0`);

        await deposit(ctx, Reward.Fixed.index);

        const member = await ctx.program.account.member.fetch(ctx.PDAS.memberFixed.key);
        expect(`${member.beneficiary}`).to.be.eq(`${ctx.beneficiary.publicKey}`);
        expect(`${member.vaultFree}`).to.be.eq(`${ctx.vaultFree.publicKey}`);
        expect(`${member.vaultPendingUnstaking}`).to.be.eq(`${ctx.vaultPendingUnstaking.publicKey}`);
        expect(member.bump).to.be.eq(ctx.PDAS.memberFixed.bump);

        const beneficiaryAccountState = await getTokenAccount(ctx.connection, ctx.beneficiaryTokenAccount.address);
        const memberVaultFree = await getTokenAccount(ctx.connection, member.vaultFree);
        expect(`${beneficiaryAccountState.amount}`).to.be.eq(`0`);
        expect(`${memberVaultFree.amount}`).to.be.eq(`${ctx.stakeTokenAmount}`);
    }

    export async function memberFixedStake(ctx: Ctx, stake: (ctx: Ctx, rewardType: number) => Promise<void>) {
        const stakeholderVault = await getTokenAccount(ctx.connection, ctx.vaultStakedFixed.publicKey);
        expect(`${stakeholderVault.amount}`).to.be.eq(`0`);

        await stake(ctx, Reward.Fixed.index);

        // TODO check stakeholder fields

        const member = await ctx.program.account.member.fetch(ctx.PDAS.memberFixed.key);
        const memberVaultFree = await getTokenAccount(ctx.connection, member.vaultFree);
        expect(`${memberVaultFree.amount}`).to.be.eq(`${0}`);

        const stakeholderVaultChanged = await getTokenAccount(ctx.connection, ctx.vaultStakedFixed.publicKey);
        expect(`${stakeholderVaultChanged.amount}`).to.be.eq(`${ctx.amountToStake}`);
    }

    export async function depositReward(ctx: Ctx, depositReward: (ctx: Ctx) => Promise<void>, checks: { rewardAmountBefore: number }) {
        const factory = await ctx.program.account.factory.fetch(ctx.PDAS.factory.key);
        const factoryRewardVault = await getTokenAccount(ctx.connection, factory.vaultReward);

        expect(`${factoryRewardVault.amount}`).to.be.eq(`${checks.rewardAmountBefore}`);

        await depositReward(ctx);

        const factoryRewardVaultChanged = await getTokenAccount(ctx.connection, factory.vaultReward);
        expect(`${factoryRewardVaultChanged.amount}`).to.be.eq(`${ctx.rewardTokensAmount}`);
    }
}
