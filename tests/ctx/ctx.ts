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
import { Reward } from "../types/reward";

// The interface passed to every RPC test function
export interface Ctx {
    connection: Connection,
    program: Program<Staking>,
    owner: Signer,
    ownerInterest: number, // %
    configChangeDelay: BN, // secs
    configHistoryLength: number,
    configHistoryKeypair: Keypair,
    configHistoryElSize: number, // TODO get from the program
    configHistoryMetadata: number, // TODO get from the program
    rewardPeriod: BN, // secs
    rewardTokenMint: PublicKey,
    stakeTokenMint: PublicKey,
    vaultReward: PublicKey,
    unstakeDelay: BN,
    unstakeForseFeePercent: number,
    rewardMetadata: BN,
    PDAS: {
        factory: CtxPDA,
        stakePoolFixed: CtxPDA,
        memberFixed: CtxPDA,
        stakeholderFixed: CtxPDA,
    },
    beneficiary: Signer,
    beneficiaryTokenAccount: TokenAccount,
    stakeTokenAmount: number,
    amountToDeposit: BN,
    amountToStake: BN,
    vaultFree: Keypair,
    vaultPendingUnstaking: Keypair,
    vaultStakedFixed: Keypair,
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
    const [stakePoolFixedPDA, spfBump] = await PublicKey.findProgramAddress(
        [ Uint8Array.from([Reward.Fixed.index]) ],
        program.programId
    );
    const beneficiary = await createUserWithLamports(connection, 1);
    const beneficiaryTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        beneficiary, // Payer
        stakeTokenMint,
        beneficiary.publicKey, // Owner
    );
    const stakeTokenAmount = 1000;
    const amountToDeposit = new BN(stakeTokenAmount);
    const amountToStake = new BN(stakeTokenAmount);
    await mintTo(
        connection,
        owner,  // Payer
        stakeTokenMint,
        beneficiaryTokenAccount.address, // mint to
        owner, // Authority
        stakeTokenAmount,
    );
    const vaultFree = anchor.web3.Keypair.generate();
    const vaultPendingUnstaking = anchor.web3.Keypair.generate();
    const [memberFixedPDA, memberFixedBump] = await PublicKey.findProgramAddress(
        [
            beneficiary.publicKey.toBuffer(),
            factoryPDA.toBuffer(),
        ],
        program.programId
    );
    await createTokenAccount(
        connection,
        beneficiary, // Payer
        stakeTokenMint,
        memberFixedPDA, // Owner
        vaultFree, // Keypair
    );
    await createTokenAccount(
        connection,
        beneficiary, // Payer
        stakeTokenMint,
        memberFixedPDA, // Owner
        vaultPendingUnstaking, // Keypair
    );
    const [stakeholderFixedPDA, stakeholderFixedBump] = await PublicKey.findProgramAddress(
        [
            memberFixedPDA.toBuffer(),
            stakePoolFixedPDA.toBuffer(),
        ],
        program.programId
    );
    const vaultStakedFixed = anchor.web3.Keypair.generate();
    await createTokenAccount(
        connection,
        beneficiary, // Payer
        stakeTokenMint,
        stakeholderFixedPDA, // Owner
        vaultStakedFixed, // Keypair
    );
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
            factory: {
                key: factoryPDA,
                bump: factoryBump,
            },
            stakePoolFixed: {
                key: stakePoolFixedPDA,
                bump: spfBump,
            },
            memberFixed: {
                key: memberFixedPDA,
                bump: memberFixedBump,
            },
            stakeholderFixed: {
                key: stakeholderFixedPDA,
                bump: stakeholderFixedBump,
            }
        },
        owner,
        ownerInterest: 1, // %
        configChangeDelay: new BN(20), // secs
        rewardPeriod: new BN(30), // secs
        configHistoryLength: 10,
        configHistoryKeypair: anchor.web3.Keypair.generate(), // TODO replace to PDA
        configHistoryElSize: 32, // TODO get from the program
        configHistoryMetadata: 4 + 4 + 8 * 3, // TODO get from the program
        rewardTokenMint,
        stakeTokenMint,
        vaultReward,
        unstakeDelay: new BN(40),
        unstakeForseFeePercent: 50,
        rewardMetadata: new BN(10),
        beneficiary,
        beneficiaryTokenAccount,
        stakeTokenAmount,
        amountToDeposit,
        amountToStake,
        vaultFree,
        vaultPendingUnstaking,
        vaultStakedFixed,
        rewardTokensAmount,
        ownerTokenAccount,
    }
}
