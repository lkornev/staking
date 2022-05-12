import * as anchor from "@project-serum/anchor";
import { Ctx, createCtx } from './ctx/ctx';
import { initializeRPC } from './rpc/initialize';
import { newStakePoolRPC } from './rpc/new-stake-pool';
import { depositRPC } from './rpc/deposit';
import { stakeRPC } from './rpc/stake';
import { depositRewardRPC } from "./rpc/deposit-reward";
import { createMemberRPC } from "./rpc/create-member";
import { claimRewardRPC } from "./rpc/claim-reward";
import { Check } from "./check/check";
import { sleepTill } from "./helpers/general";
import { getAccount } from "@solana/spl-token";

describe("staking", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    let ctx: Ctx;

    it("Initializes factory!", async () => {
        ctx = await createCtx();
        await initializeRPC(ctx);
        await Check.factory(ctx);        
    });

    it("Creates new staking pool instance with fixed rewards", async () => {
        await newStakePoolRPC(ctx, ctx.PDAS.stakePoolFixed);
        await Check.newStakePool(ctx, ctx.PDAS.stakePoolFixed);
    });

    it("Creates member", async () => {
        await createMemberRPC(ctx, ctx.PDAS.member);
        await Check.newMember(ctx, ctx.PDAS.member);
    });

    it("Deposits tokens", async () => {
        await Check.memberDeposit(ctx, ctx.PDAS.member, depositRPC);
    });

    it("Stake tokens", async () => { 
        await Check.memberStake(ctx, ctx.PDAS.stakePoolFixed, ctx.PDAS.member, ctx.PDAS.memberStakeFixed, stakeRPC);
    });

    it("Deposits tokens reward", async () => {
        await Check.depositReward(ctx, depositRewardRPC, { rewardAmountBefore: 0 });
    });

    it("Claim reward", async () => {
        let stakedAt = (await ctx.program.account.memberStake.fetch(ctx.PDAS.memberStakeFixed.key)).stakedAt;

        // Wait one reward period + extra sec, just in case.
        sleepTill(((Number(stakedAt.add(ctx.PDAS.stakePoolFixed.rewardPeriod)) + 1000) * 1000));
        await Check.claimReward(ctx, ctx.PDAS.stakePoolFixed, ctx.PDAS.member, ctx.PDAS.memberStakeFixed, claimRewardRPC);
    });
});
