import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { Staking } from "../target/types/staking";
import { RewardType } from "./reward";
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
import { expect, assert } from 'chai';

describe("staking", () => {
    anchor.setProvider(anchor.Provider.env());

    const connection = new Connection("http://localhost:8899", 'recent');

    const program = anchor.workspace.Staking as Program<Staking>;
    const provider = program.provider;

    let owner: Signer;
    const ownerInterest = 1; // %
    const confifChangeDelay = new BN(20); // secs
    const rewardQueueLength = 10;

    const rewardQueueKeypair: Keypair = anchor.web3.Keypair.generate();
    let rewardQueueElSize: number = 1; // TODO get from the program (if it's possible)
    let rewardQueueMetadata: number = 12; // TODO get from the program (if it's possible)

    let rewardTokenMint: PublicKey;
    let stakeTokenMint: PublicKey;

    let factoryPDA: PublicKey;
    let stakePoolFixedPDA: PublicKey;

    it("Initializes factory!", async () => {
        owner = await createUserWithLamports(connection, 10);

        // TODO get the seed value form the program Factory.PDA_KEY
        const [_factoryPDA, _factoryBump] = await PublicKey.findProgramAddress(
            [anchor.utils.bytes.utf8.encode("factory")],
            program.programId
        );
        factoryPDA = _factoryPDA;

        await program.rpc.initialize(
            owner.publicKey,
            ownerInterest,
            confifChangeDelay,
            rewardQueueLength,
            {
                accounts: {
                    factory: factoryPDA,
                    rewardQueue: rewardQueueKeypair.publicKey,
                    initializer: owner.publicKey,
                    systemProgram: SystemProgram.programId,
                },
                signers: [owner, rewardQueueKeypair],
                preInstructions: [
                    await program.account.rewardQueue.createInstruction(
                        rewardQueueKeypair, 
                        rewardQueueLength * rewardQueueElSize + rewardQueueMetadata
                    ),
                ]
            }
        );

        const factory = await program.account.factory.fetch(factoryPDA);

        expect(`${factory.owner}`).to.be.eq(`${owner.publicKey}`);
        expect(factory.ownerInterest).to.be.eq(ownerInterest);
        expect(`${factory.rewardQueue}`).to.be.eq(`${rewardQueueKeypair.publicKey}`);
        expect(`${factory.configChangeDelay}`).to.be.eq(`${confifChangeDelay}`);
    });

    it("Creates reward and stake mints", async () => {
        rewardTokenMint = await createTokenMint(connection, owner.publicKey, owner, 6);
        stakeTokenMint = await createTokenMint(connection, owner.publicKey, owner, 9);
    });

    it("Creates new staking pool instance with fixed rewards", async () => {
        // TODO get the seed value form the program StakePool.PDA_SEED_FIXED
        const [_stakePoolFixedPDA, _spfBump] = await PublicKey.findProgramAddress(
            [anchor.utils.bytes.utf8.encode("stake-pool-fixed")],
            program.programId
        );
        stakePoolFixedPDA = _stakePoolFixedPDA;

        const stakePoolConfig: StakePoolConfig = newSPFixedConfig(rewardTokenMint, stakeTokenMint);

        await program.rpc.new(
            ...Object.values(stakePoolConfig),
            {
                accounts: {
                    factory: factoryPDA,
                    stakePool: stakePoolFixedPDA,
                    owner: owner.publicKey,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                    systemProgram: SystemProgram.programId,
                },
                signers: [owner]
            }
        );

        const stakePool = await program.account.stakePool.fetch(stakePoolFixedPDA);

        expect(`${stakePool.unstakeDelay}`).to.be.eq(`${stakePoolConfig.unstakeDelay}`);
        expect(stakePool.unstakeForseFeePercent).to.be.eq(stakePoolConfig.unstakeForseFeePercent);
        expect(`${stakePool.rewardPeriod}`).to.be.eq(`${stakePoolConfig.rewardPeriod}`);
        expect(`${stakePool.rewardTokenMint}`).to.be.eq(`${stakePoolConfig.rewardTokenMint}`);
        expect(`${stakePool.stakeTokenMint}`).to.be.eq(`${stakePoolConfig.stakeTokenMint}`);
        expect(stakePool.rewardType).to.be.eq(stakePoolConfig.rewardType);
        expect(`${stakePool.rewardMetadata}`).to.be.eq(`${stakePoolConfig.rewardMetadata}`);
    });
});
