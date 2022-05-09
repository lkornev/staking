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
import { expect } from 'chai';
import { Reward } from "./types/reward";
import { Ctx, createCtx } from './ctx/ctx';
import { initializeRPC } from './rpc/initialize';
import { newStakePoolRPC } from './rpc/new-stake-pool';
import { depositRPC } from './rpc/deposit';
import { stakeRPC } from './rpc/stake';
import { depositRewardRPC } from "./rpc/deposit-reward";
import { Check } from "./check/check";

describe("staking", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    let ctx: Ctx;

    it("Initializes factory!", async () => {
        ctx = await createCtx();
        await initializeRPC(ctx);
        await Check.factory(ctx);        
    });

    it("Creates new staking pool instance with fixed rewards", async () => {
        await newStakePoolRPC(ctx);
        await Check.newStakePool(ctx);
    });

    it("Deposits tokens", async () => {
        await Check.memberFixedDeposit(ctx, depositRPC);
    });

    it("Stake tokens", async () => { 
        await Check.memberFixedStake(ctx, stakeRPC);
    });

    it("Deposits tokens reward", async () => {
        await Check.depositReward(ctx, depositRewardRPC, { rewardAmountBefore: 0 });
    });
});
