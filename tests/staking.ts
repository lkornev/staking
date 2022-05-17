import * as anchor from "@project-serum/anchor";
import { Ctx, createCtx } from './ctx/ctx';
import { initializeRPC } from './rpc/initialize';
import { newStakePoolRPC } from './rpc/new-stake-pool';
import { depositRPC } from './rpc/deposit';
import { stakeRPC } from './rpc/stake';
import { depositRewardRPC } from "./rpc/deposit-reward";
import { createMemberRPC } from "./rpc/create-member";
import { claimRewardRPC } from "./rpc/claim-reward";
import { startUnstakeAllRPC } from "./rpc/start-unstake-all";
import { Check } from "./check/check";
import { sleepTill } from "./helpers/general";
import { Reward } from "./types/reward";
import { finishUnstakeAllRPC } from "./rpc/finish-unstake-all";
import { withdrawalAllRPC } from "./rpc/withdraw-all";

describe("staking", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    let ctx: Ctx;

    it("Initializes factory!", async () => {
        ctx = await createCtx();
        await initializeRPC(ctx);
        await Check.factory(ctx);        
    });

    it("Deposits reward tokens", async () => {
        await Check.depositReward(ctx, depositRewardRPC, { 
            rewardAmountBefore: 0,
            rewardAmountAfter: ctx.owner.initialRewardTokensAmount,
        });
    });

    it("Creates member", async () => {
        await createMemberRPC(ctx, ctx.PDAS.member);
        await Check.newMember(ctx, ctx.PDAS.member);
    });

    it("Deposits tokens for future stakes", async () => {
        await Check.memberDeposit(ctx, ctx.PDAS.member, depositRPC);
    });

    it("Stakes", async () => {
        await stakeSuite(ctx, Reward.Fixed.name);
        await stakeSuite(ctx, Reward.Unfixed.name)
    });

    it("Unstakes and withdraws tokens", async () => {
        await unstakeSuite(ctx, Reward.Fixed.name);
        await Check.withdrawAll(ctx, ctx.PDAS.member, withdrawalAllRPC);

        await unstakeSuite(ctx, Reward.Unfixed.name);
        await Check.withdrawAll(ctx, ctx.PDAS.member, withdrawalAllRPC);
    });
});

async function stakeSuite (ctx: Ctx, reward: "fixed" | "unfixed") {
    await newStakePoolRPC(ctx, ctx.PDAS[reward].stakePool);
    await Check.newStakePool(ctx, ctx.PDAS[reward].stakePool);

    await Check.memberStake(ctx, ctx.PDAS[reward].stakePool, ctx.PDAS.member, ctx.PDAS[reward].memberStake, stakeRPC);

    const stakedAt = Number((await ctx.program.account.memberStake.fetch(ctx.PDAS[reward].memberStake.key)).stakedAt);
    const rewardPeriod = Number(ctx.PDAS[reward].stakePool.rewardPeriod);
    await sleepTill((stakedAt + rewardPeriod + rewardPeriod * 0.5) * 1000);
    await Check.claimReward(ctx, ctx.PDAS[reward].memberStake, claimRewardRPC);
};

async function unstakeSuite (ctx: Ctx, reward: "fixed" | "unfixed") {
    await Check.startUnstakeAll(ctx, ctx.PDAS[reward].memberUnstakeAll, startUnstakeAllRPC);

    const unstakedAcc = await ctx.program.account.memberPendingUnstake.fetch(ctx.PDAS[reward].memberUnstakeAll.key);
    const unstakedAt = Number((unstakedAcc).unstakedAt);
    const unstakeDelay = Number(ctx.PDAS[reward].stakePool.unstakeDelay);
    await sleepTill((unstakedAt + unstakeDelay + 2) * 1000);
    await Check.finishUnstakeAll(ctx, ctx.PDAS[reward].memberUnstakeAll, finishUnstakeAllRPC);
}
