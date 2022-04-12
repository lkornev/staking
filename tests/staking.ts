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
import { expect, assert } from 'chai';

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
        const [_stakePoolFixedPDA, _spfBump] = await PublicKey.findProgramAddress(
            [anchor.utils.bytes.utf8.encode("stake-pool-fixed")],
            program.programId
        );
        stakePoolFixedPDA = _stakePoolFixedPDA;

        const [_stakePoolConfigPDA, _spcBump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("0"),
                anchor.utils.bytes.utf8.encode("stake-pool-config-fixed")
            ],
            program.programId
        );
        stakePoolConfigPDA = _stakePoolConfigPDA;

        const configTemplate: StakePoolConfig = newSPFixedConfig(configHistoryLength);

        await program.rpc.new(
            ...Object.values(configTemplate),
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
});
