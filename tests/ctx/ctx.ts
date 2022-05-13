import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { Staking } from "../../target/types/staking";
import {
    Connection, 
    PublicKey, 
    Signer,
    Keypair,
} from '@solana/web3.js';
import {
    createMint,
    getOrCreateAssociatedTokenAccount,
    Account as TokenAccount,
    createAccount as createTokenAccount,
    mintTo,
    getAssociatedTokenAddress,
} from '@solana/spl-token';
import { createUserWithLamports } from "../helpers/general";
import { Reward, RewardType } from "../types/reward";

// The interface passed to every function in the Check namespace
export interface Ctx {
    connection: Connection,
    program: Program<Staking>,
    PDAS: {
        factory: Factory,
        member: Member,
        fixed: StakeGroup,
        unfixed: StakeGroup,
    },
    owner: Owner,
}

export interface StakeGroup  {
    stakePool: StakePool,
    memberStake: MemberStake,
}

export async function createCtx(): Promise<Ctx> {
    const program = anchor.workspace.Staking as Program<Staking>;
    const connection = new Connection("http://localhost:8899", 'recent');

    const owner = await createOwner({program, connection});
    const factory = await createFactory({program, connection, owner});
    const member = await createMember({program, connection, factory});

    let stakeGroup = async (rewardType: RewardType, rewardData: BN, amountToStake: BN): Promise<StakeGroup> => {
        const stakePool = await createStakePool({program, rewardType, rewardMetadata: rewardData});
        const memberStake = await createMemberStake({ connection, program, factory, member, stakePool}, amountToStake);
        return { stakePool, memberStake };
    }

    return {
        connection,
        program,
        PDAS: {
            factory,
            member,
            fixed: await stakeGroup(Reward.Fixed, new BN(10), member.amountToStake.fixed),
            unfixed: await stakeGroup(Reward.Unfixed,  new BN(200), member.amountToStake.unfixed),
        },
        owner,
    }
}

export interface CtxPDA {
    key: PublicKey,
    bump: number,
}

export interface Owner extends Signer {
    rewardTokenVault: PublicKey,
    feeRewardVault: PublicKey,
    rewardTokenMint: PublicKey,
    initialRewardTokensAmount: number,
}

export interface OwnerCtx {
    program: Program<Staking>,
    connection: Connection,
}

export async function createOwner(ctx: OwnerCtx): Promise<Owner> {
    const signer = await createUserWithLamports(ctx.connection, 10);
    const rewardTokenMint = await createMint(ctx.connection, signer, signer.publicKey, signer.publicKey, 6);
    const feeRewardTokenAccount = await getOrCreateAssociatedTokenAccount(ctx.connection, signer, rewardTokenMint, signer.publicKey);
    const initialRewardTokensAmount = 100000000;
    const rewardTokenAccount = await getOrCreateAssociatedTokenAccount(
        ctx.connection,
        signer, // Payer
        rewardTokenMint,
        signer.publicKey, // Owner
    );
    await mintTo(
        ctx.connection,
        signer, // Payer
        rewardTokenMint,
        rewardTokenAccount.address, // mint to
        signer.publicKey, // Authority
        initialRewardTokensAmount,
    );

    return { 
        publicKey: signer.publicKey,
        secretKey: signer.secretKey,
        rewardTokenVault: rewardTokenAccount.address,
        feeRewardVault: feeRewardTokenAccount.address,
        rewardTokenMint,
        initialRewardTokensAmount,
    };
}

export interface Factory extends CtxPDA {
    vaultReward: PublicKey,
    rewardTokenMint: PublicKey,
    stakeTokenMint: PublicKey,
    owner: Owner,
}

export interface FactoryCtx {
    program: Program<Staking>,
    connection: Connection,
    owner: Owner,
}

export async function createFactory(ctx: FactoryCtx): Promise<Factory> {
    const [key, bump] = await PublicKey.findProgramAddress(
        [anchor.utils.bytes.utf8.encode("factory")],
        ctx.program.programId
    );
    const stakeTokenMint = await createMint(ctx.connection, ctx.owner, ctx.owner.publicKey, ctx.owner.publicKey, 9);
    const vaultReward = await getAssociatedTokenAddress(ctx.owner.rewardTokenMint, key, true);

    return { 
        key, 
        bump,
        vaultReward, 
        rewardTokenMint: ctx.owner.rewardTokenMint, 
        stakeTokenMint, 
        owner: ctx.owner 
    };
}

export interface StakePool extends CtxPDA {
    endedAt: BN, // secs
    rewardPeriod: BN, // secs
    ownerInterestPercent: number, // %
    rewardType: RewardType,
    unstakeDelay: BN, // secs
    unstakeForceFeePercent: number, // %,
    minOwnerReward: number,
    rewardMetadata: BN,
}

export interface StakePoolCtx {
    program: Program<Staking>,
    rewardType: RewardType,
    rewardMetadata: BN,
    unstakeDelay?: BN, // secs
    unstakeForceFeePercent?: number, // %,
    endedAt?: BN,
    ownerInterestPercent?: number, // %
    rewardPeriod?: BN, // secs
    minOwnerReward?: number,
}

export async function createStakePool(ctx: StakePoolCtx): Promise<StakePool> {
    const [stakePoolPDA, stakePoolBump] = await PublicKey.findProgramAddress(
        [ Uint8Array.from([ctx.rewardType.index]) ],
        ctx.program.programId
    );

    let rewardPeriod = ctx.rewardPeriod || new BN(4);

    return {
        key: stakePoolPDA,
        bump: stakePoolBump,
        rewardType: ctx.rewardType,
        rewardMetadata: ctx.rewardMetadata,
        endedAt: ctx.endedAt || new BN(Math.floor(Date.now() / 1000)).add(rewardPeriod.mul(new BN(50))),
        ownerInterestPercent: ctx.ownerInterestPercent || 1, // %
        rewardPeriod: rewardPeriod, // secs
        unstakeDelay: ctx.unstakeDelay || new BN(40), // secs
        unstakeForceFeePercent: ctx.unstakeForceFeePercent || 50, // %,
        minOwnerReward: ctx.minOwnerReward || 1, // tokens
    }
}

export interface Member extends CtxPDA {
    beneficiary: Signer,
    beneficiaryTokenAccount: TokenAccount,
    vaultFree: Keypair,
    vaultPendingUnstaking: Keypair,
    stakeTokenAmount: BN,
    amountToDeposit: BN,
    amountToStake: {
        fixed: BN,
        unfixed: BN,
    },
    beneficiaryRewardVault: TokenAccount,
}

export interface MemberCtx {
    connection: Connection,
    program: Program<Staking>,
    factory: Factory,
}

export async function createMember(ctx: MemberCtx): Promise<Member> {
    const stakeTokenAmount = new BN(1000);
    const amountToDeposit = stakeTokenAmount;

    const beneficiary = await createUserWithLamports(ctx.connection, 10);
    const beneficiaryTokenAccount = await getOrCreateAssociatedTokenAccount(
        ctx.connection,
        beneficiary, // Payer
        ctx.factory.stakeTokenMint,
        beneficiary.publicKey, // Owner
    );
    await mintTo(
        ctx.connection,
        ctx.factory.owner,  // Payer
        ctx.factory.stakeTokenMint,
        beneficiaryTokenAccount.address, // mint to
        ctx.factory.owner, // Authority
        Number(stakeTokenAmount),
    );
    const vaultFree = anchor.web3.Keypair.generate();
    const vaultPendingUnstaking = anchor.web3.Keypair.generate();
    const [memberPDA, memberBump] = await PublicKey.findProgramAddress(
        [
            beneficiary.publicKey.toBuffer(),
            ctx.factory.key.toBuffer(),
        ],
        ctx.program.programId
    );
    await createTokenAccount(
        ctx.connection,
        beneficiary, // Payer
        ctx.factory.stakeTokenMint,
        memberPDA, // Owner
        vaultFree, // Keypair
    );
    await createTokenAccount(
        ctx.connection,
        beneficiary, // Payer
        ctx.factory.stakeTokenMint,
        memberPDA, // Owner
        vaultPendingUnstaking, // Keypair
    );
    const beneficiaryRewardVault = await getOrCreateAssociatedTokenAccount(
        ctx.connection,
        beneficiary,
        ctx.factory.rewardTokenMint,
        beneficiary.publicKey
    );

    const fixedAmountToStake = amountToDeposit.div(new BN(2));
    const unfixedAmountToStake = amountToDeposit.sub(fixedAmountToStake);

    return {
        key: memberPDA,
        bump: memberBump,
        beneficiary,
        beneficiaryTokenAccount,
        vaultFree,
        vaultPendingUnstaking,
        stakeTokenAmount,
        amountToDeposit,
        amountToStake: {
            fixed: fixedAmountToStake,
            unfixed: unfixedAmountToStake,
        },
        beneficiaryRewardVault,
    }
}

export interface MemberStake extends CtxPDA  {
    member: Member,
    stakePool: StakePool,
    amountToStake: BN,
    vaultStaked: Keypair,
}

export interface MemberStakeCtx {
    connection: Connection,
    program: Program<Staking>,
    factory: Factory,
    stakePool: StakePool,
    member: Member,
}

export async function createMemberStake(ctx: MemberStakeCtx, amountToStake: BN): Promise<MemberStake> {
    const [memberStake, memberStakeBump] = await PublicKey.findProgramAddress(
        [
            ctx.member.key.toBuffer(),
            ctx.stakePool.key.toBuffer(),
        ],
        ctx.program.programId
    );
    const vaultStaked = anchor.web3.Keypair.generate();
    await createTokenAccount(
        ctx.connection,
        ctx.member.beneficiary, // Payer
        ctx.factory.stakeTokenMint,
        memberStake, // Owner
        vaultStaked, // Keypair
    );

    return {
        key: memberStake,
        bump: memberStakeBump,
        member: ctx.member,
        stakePool: ctx.stakePool,
        amountToStake: amountToStake,
        vaultStaked: vaultStaked,
    }
}
