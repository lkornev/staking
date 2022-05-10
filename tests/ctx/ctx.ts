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
    getAssociatedTokenAddress,
    getOrCreateAssociatedTokenAccount,
    Account as TokenAccount,
    createAccount as createTokenAccount,
    mintTo,
} from '@solana/spl-token';
import { createUserWithLamports } from "../helpers/general";
import { Reward, RewardType } from "../types/reward";

// The interface passed to every function in the Check namespace
export interface Ctx {
    connection: Connection,
    program: Program<Staking>,
    owner: Signer,
    ownerInterest: number, // %
    configChangeDelay: BN, // secs
    rewardPeriod: BN, // secs
    rewardTokenMint: PublicKey,
    stakeTokenMint: PublicKey,
    vaultReward: PublicKey,
    PDAS: {
        factory: CtxPDA,
        stakePoolFixed: StakePool,
        member: Member,
        memberStakeFixed: MemberStake,
    },
    rewardTokensAmount: number,
    ownerTokenAccount: TokenAccount,
}

export interface CtxPDA {
    key: PublicKey,
    bump: number,
}

export async function createCtx(): Promise<Ctx> {
    const connection = new Connection("http://localhost:8899", 'recent');
    const program = anchor.workspace.Staking as Program<Staking>;
    const owner = await createUserWithLamports(connection, 10);
    const [factoryPDA, factoryBump] = await PublicKey.findProgramAddress(
        [anchor.utils.bytes.utf8.encode("factory")],
        program.programId
    );
    const rewardTokenMint = await createMint(connection, owner, owner.publicKey, owner.publicKey, 6);
    const stakeTokenMint = await createMint(connection, owner, owner.publicKey, owner.publicKey, 9);
    const vaultReward = await getAssociatedTokenAddress(rewardTokenMint, factoryPDA, true);

    const stakePoolFixed = await createStakePool({
        program,
        rewardType: Reward.Fixed,
        rewardMetadata: new BN(10),
    });
    
    const member: Member = await createMember({
        connection,
        program,
        owner,
        stakeTokenMint,
        PDAS: {
            factory: { key: factoryPDA, bump: factoryBump },
        },
    });

    const memberStakeFixed: MemberStake = await createMemberStake({
        connection,
        program,
        owner,
        rewardTokenMint,
        stakeTokenMint,
        amountToStake: member.amountToDeposit,
        PDAS: {
            factory: { key: factoryPDA, bump: factoryBump },
            stakePool: stakePoolFixed,
            member,
        },
    });

    const rewardTokensAmount = 100000000;
    const ownerTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        owner, // Payer
        rewardTokenMint,
        owner.publicKey, // Owner
    );

    await mintTo(
        connection,
        owner,  // Payer
        rewardTokenMint,
        ownerTokenAccount.address, // mint to
        owner, // Authority
        rewardTokensAmount,
    );

    return {
        connection,
        program,
        PDAS: {
            factory: { key: factoryPDA, bump: factoryBump },
            stakePoolFixed,
            member,
            memberStakeFixed,
        },
        owner,
        ownerInterest: 1, // %
        configChangeDelay: new BN(20), // secs
        rewardPeriod: new BN(30), // secs
        rewardTokenMint,
        stakeTokenMint,
        vaultReward,
        rewardTokensAmount,
        ownerTokenAccount,
    }
}

export interface StakePool extends CtxPDA {
    rewardType: RewardType,
    unstakeDelay: BN, // secs
    unstakeForseFeePercent: number, // %,
    rewardMetadata: BN,
    configHistoryKeypair: Keypair,
    configHistoryLength: number,
    configHistoryElSize: number,
    configHistoryMetadata: number,
}

export interface StakePoolCtx {
    program: Program<Staking>,
    rewardType: RewardType,
    rewardMetadata: BN,
    unstakeDelay?: BN, // secs
    unstakeForseFeePercent?: number, // %,
    configHistoryLength?: number,
    configHistoryElSize?: number,
    configHistoryMetadata?: number,
}

export async function createStakePool(ctx: StakePoolCtx): Promise<StakePool> {
    const [stakePoolPDA, stakePoolBump] = await PublicKey.findProgramAddress(
        [ Uint8Array.from([Reward.Fixed.index]) ],
        ctx.program.programId
    );

    return {
        key: stakePoolPDA,
        bump: stakePoolBump,
        rewardType: ctx.rewardType,
        rewardMetadata: ctx.rewardMetadata,
        unstakeDelay: ctx.unstakeDelay || new BN(40), // secs
        unstakeForseFeePercent: ctx.unstakeForseFeePercent || 50, // %,
        configHistoryKeypair: anchor.web3.Keypair.generate(),
        configHistoryLength: ctx.configHistoryLength || 10,
        configHistoryElSize: ctx.configHistoryElSize || 32,
        configHistoryMetadata: ctx.configHistoryMetadata || 4 + 4 + 8 * 3,
    }
}

export interface Member extends CtxPDA {
    beneficiary: Signer,
    beneficiaryTokenAccount: TokenAccount,
    vaultFree: Keypair,
    vaultPendingUnstaking: Keypair,
    stakeTokenAmount: number,
    amountToDeposit: BN,
}

export interface MemberCtx {
    connection: Connection,
    program: Program<Staking>,
    owner: Signer,
    stakeTokenMint: PublicKey,
    PDAS: {
        factory: CtxPDA,
    },
}

export async function createMember(memberCtx: MemberCtx): Promise<Member> {
    const stakeTokenAmount = 1000;
    const amountToDeposit = new BN(stakeTokenAmount);

    const beneficiary = await createUserWithLamports(memberCtx.connection, 1);
    const beneficiaryTokenAccount = await getOrCreateAssociatedTokenAccount(
        memberCtx.connection,
        beneficiary, // Payer
        memberCtx.stakeTokenMint,
        beneficiary.publicKey, // Owner
    );
    await mintTo(
        memberCtx.connection,
        memberCtx.owner,  // Payer
        memberCtx.stakeTokenMint,
        beneficiaryTokenAccount.address, // mint to
        memberCtx.owner, // Authority
        stakeTokenAmount,
    );
    const vaultFree = anchor.web3.Keypair.generate();
    const vaultPendingUnstaking = anchor.web3.Keypair.generate();
    const [memberPDA, memberBump] = await PublicKey.findProgramAddress(
        [
            beneficiary.publicKey.toBuffer(),
            memberCtx.PDAS.factory.key.toBuffer(),
        ],
        memberCtx.program.programId
    );
    await createTokenAccount(
        memberCtx.connection,
        beneficiary, // Payer
        memberCtx.stakeTokenMint,
        memberPDA, // Owner
        vaultFree, // Keypair
    );
    await createTokenAccount(
        memberCtx.connection,
        beneficiary, // Payer
        memberCtx.stakeTokenMint,
        memberPDA, // Owner
        vaultPendingUnstaking, // Keypair
    );

    return {
        key: memberPDA,
        bump: memberBump,
        beneficiary,
        beneficiaryTokenAccount,
        vaultFree,
        vaultPendingUnstaking,
        stakeTokenAmount,
        amountToDeposit,
    }
}

export interface MemberStake extends CtxPDA  {
    member: Member,
    stakePool: CtxPDA,
    amountToStake: BN,
    vaultStaked: Keypair,
}

export interface MemberStakeCtx {
    connection: Connection,
    program: Program<Staking>,
    owner: Signer,
    rewardTokenMint: PublicKey,
    stakeTokenMint: PublicKey,
    amountToStake: BN,
    PDAS: {
        factory: CtxPDA,
        stakePool: CtxPDA,
        member: Member,
    },
}

export async function createMemberStake(ctx: MemberStakeCtx): Promise<MemberStake> {
    const [memberStake, memberStakeBump] = await PublicKey.findProgramAddress(
        [
            ctx.PDAS.member.key.toBuffer(),
            ctx.PDAS.stakePool.key.toBuffer(),
        ],
        ctx.program.programId
    );
    const vaultStaked = anchor.web3.Keypair.generate();
    await createTokenAccount(
        ctx.connection,
        ctx.PDAS.member.beneficiary, // Payer
        ctx.stakeTokenMint,
        memberStake, // Owner
        vaultStaked, // Keypair
    );

    return {
        key: memberStake,
        bump: memberStakeBump,
        member: ctx.PDAS.member,
        stakePool: ctx.PDAS.stakePool,
        amountToStake: ctx.amountToStake,
        vaultStaked: vaultStaked,
    }
}
