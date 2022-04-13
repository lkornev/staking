import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { Staking } from "../target/types/staking";
import { newSPFixedConfig, StakePoolConfig } from "./stake-pool";
import {
    createUserWithLamports,
    createTokenMint,
} from "./helpers";
import {
    PublicKey,
    SystemProgram,
    Keypair,
    Connection,
    Signer,
} from '@solana/web3.js';
import { 
    createMint,
    TOKEN_PROGRAM_ID, 
    createAccount as createTokenAccount,
    getOrCreateAssociatedTokenAccount,
    Account as TokenAccount,
    mintTo,
    getAccount as getTokenAccount,
    createApproveInstruction,
    NATIVE_MINT,
} from '@solana/spl-token';
import { expect, assert } from 'chai';
import { RewardType } from "./reward";


describe("staking", () => {
    anchor.setProvider(anchor.Provider.env());

    const connection = new Connection("http://localhost:8899", 'recent');

    const program = anchor.workspace.Staking as Program<Staking>;

    let owner: Signer;
    const ownerInterest = 1; // %
    const confifChangeDelay = new BN(20); // secs

    const configHistoryLength = 10;
    // TODO replace to PDA
    const configHistoryKeypair: Keypair = anchor.web3.Keypair.generate();
    let configHistoryElSize: number = 16; // TODO get from the program
    let configHistoryMetadata: number = 8 + 16 * 3; // TODO get from the program

    let rewardTokenMint: PublicKey;
    let stakeTokenMint: PublicKey;

    let factoryPDA: PublicKey;
    let stakePoolFixedPDA: PublicKey;
    let stakePoolConfigPDA: PublicKey;

    it("Creates reward and stake mints", async () => {
        owner = await createUserWithLamports(connection, 10);
        rewardTokenMint = await createTokenMint(connection, owner.publicKey, owner, 6);
        stakeTokenMint = await createTokenMint(connection, owner.publicKey, owner, 9);
    });

    it("Initializes factory!", async () => {
        const [_factoryPDA, _factoryBump] = await PublicKey.findProgramAddress(
            [anchor.utils.bytes.utf8.encode("factory")],
            program.programId
        );
        factoryPDA = _factoryPDA;

        await program.rpc.initialize(
            owner.publicKey,
            ownerInterest,
            confifChangeDelay,
            rewardTokenMint, 
            stakeTokenMint,
            {
                accounts: {
                    factory: factoryPDA,
                    initializer: owner.publicKey,
                    systemProgram: SystemProgram.programId,
                },
                signers: [owner],
            }
        );

        const factory = await program.account.factory.fetch(factoryPDA);

        expect(`${factory.owner}`).to.be.eq(`${owner.publicKey}`);
        expect(factory.ownerInterest).to.be.eq(ownerInterest);
        expect(`${factory.configChangeDelay}`).to.be.eq(`${confifChangeDelay}`);
        expect(`${factory.rewardTokenMint}`).to.be.eq(`${rewardTokenMint}`);
        expect(`${factory.stakeTokenMint}`).to.be.eq(`${stakeTokenMint}`);
    });

    it("Creates new staking pool instance with fixed rewards", async () => {
        const [_stakePoolFixedPDA, spfBump] = await PublicKey.findProgramAddress(
            [anchor.utils.bytes.utf8.encode("fixed")],
            program.programId
        );
        stakePoolFixedPDA = _stakePoolFixedPDA;

        const [_stakePoolConfigPDA, _spcBump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("0"), // Index in the Config Histroy
                stakePoolFixedPDA.toBuffer()
            ],
            program.programId
        );
        stakePoolConfigPDA = _stakePoolConfigPDA;

        const configTemplate: StakePoolConfig = newSPFixedConfig(configHistoryLength);

        await program.rpc.new(
            ...Object.values(configTemplate),
            spfBump,
            {
                accounts: {
                    factory: factoryPDA,
                    stakePool: stakePoolFixedPDA,
                    stakePoolConfig: stakePoolConfigPDA,
                    configHistory: configHistoryKeypair.publicKey,
                    owner: owner.publicKey,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                    systemProgram: SystemProgram.programId,
                },
                signers: [owner, configHistoryKeypair],
                preInstructions: [
                    await program.account.configHistory.createInstruction(
                        configHistoryKeypair, 
                        configHistoryLength * configHistoryElSize + configHistoryMetadata
                    ),
                ]
            }
        );

        const stakePool = await program.account.stakePool.fetch(stakePoolFixedPDA);
        const configHistory = await program.account.configHistory.fetch(configHistoryKeypair.publicKey);
        const stakePoolConfig = await program.account.stakePoolConfig.fetch(stakePoolConfigPDA);

        expect(`${stakePool.configHistory}`).to.be.eq(`${configHistoryKeypair.publicKey}`);
        expect(`${configHistory.history[0]}`).to.be.eq(`${stakePoolConfigPDA}`);

        for (let i = 1; i < configHistoryLength; i++) {
            expect(configHistory.history[i], `el № ${i}`).to.be.eq(null);
        }

        expect(`${stakePoolConfig.totalStakedTokens}`).to.be.eq(`${0}`);
        expect(`${stakePoolConfig.unstakeDelay}`).to.be.eq(`${configTemplate.unstakeDelay}`);
        expect(`${stakePoolConfig.unstakeForseFeePercent}`).to.be
            .eq(`${configTemplate.unstakeForseFeePercent}`);
        expect(`${stakePoolConfig.rewardPeriod}`).to.be.eq(`${configTemplate.rewardPeriod}`);
        expect(stakePoolConfig.rewardType).to.be.eq(configTemplate.rewardType);
        expect(`${stakePoolConfig.rewardMetadata}`).to.be
            .eq(`${configTemplate.rewardMetadata}`);
    });

    let beneficiary: Signer;
    let beneficiaryTokenAccount: TokenAccount;
    const stakeTokenAmount = 1000;
    const amountToDeposit = new BN(stakeTokenAmount);

    let vaultFree: Keypair;
    let vaultPendingUnstaking: Keypair;
    let memberPDA: PublicKey;
    let memberBump: number;

    it("Create member PDA and token vaults", async () => {
        beneficiary = await createUserWithLamports(connection, 1);

        beneficiaryTokenAccount = await getOrCreateAssociatedTokenAccount(
            connection,
            beneficiary, // Payer
            stakeTokenMint,
            beneficiary.publicKey, // Owner
        );

        await mintTo(
            connection,
            owner,  // Payer
            stakeTokenMint,
            beneficiaryTokenAccount.address, // mint to
            owner, // Authority
            stakeTokenAmount,
        );

        vaultFree = anchor.web3.Keypair.generate();
        vaultPendingUnstaking = anchor.web3.Keypair.generate();

        const [_memberPDA, _memberBump] = await PublicKey.findProgramAddress(
            [
                beneficiary.publicKey.toBuffer(),
                factoryPDA.toBuffer(),
            ],
            program.programId
        );
        memberPDA = _memberPDA;
        memberBump = _memberBump;

        await createTokenAccount(
            connection,
            beneficiary, // Payer
            stakeTokenMint,
            memberPDA, // Owner
            vaultFree, // Keypair
        );

        await createTokenAccount(
            connection,
            beneficiary, // Payer
            stakeTokenMint,
            memberPDA, // Owner
            vaultPendingUnstaking, // Keypair
        );

        const beneficiaryAccountState = await getTokenAccount(connection, beneficiaryTokenAccount.address);
        const memberVaultFree = await getTokenAccount(connection, vaultFree.publicKey);
        const memberVaultPU = await getTokenAccount(connection, vaultPendingUnstaking.publicKey);

        expect(`${beneficiaryAccountState.amount}`).to.be.eq(`${stakeTokenAmount}`);
        expect(`${memberVaultFree.amount}`).to.be.eq(`0`);
        expect(`${memberVaultPU.amount}`).to.be.eq(`0`);
    });

    it("Deposits tokens", async () => {
       await program.rpc.deposit(
            amountToDeposit,
            memberBump,
            {
                accounts: {
                    factory: factoryPDA,
                    beneficiary: beneficiary.publicKey,
                    beneficiaryTokenAccount: beneficiaryTokenAccount.address,
                    member: memberPDA,
                    vaultFree: vaultFree.publicKey,
                    vaultPendingUnstaking: vaultPendingUnstaking.publicKey,
                    systemProgram: SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                },
                signers: [beneficiary],
            }
        );

        const member = await program.account.member.fetch(memberPDA);

        expect(`${member.owner}`).to.be.eq(`${beneficiary.publicKey}`);
        expect(`${member.vaultFree}`).to.be.eq(`${vaultFree.publicKey}`);
        expect(`${member.vaultPendingUnstaking}`).to.be.eq(`${vaultPendingUnstaking.publicKey}`);
        expect(member.bump).to.be.eq(memberBump);

        const beneficiaryAccountState = await getTokenAccount(connection, beneficiaryTokenAccount.address);
        const memberVaultFree = await getTokenAccount(connection, member.vaultFree);

        expect(`${beneficiaryAccountState.amount}`).to.be.eq(`0`);
        expect(`${memberVaultFree.amount}`).to.be.eq(`${stakeTokenAmount}`);
    });
});
