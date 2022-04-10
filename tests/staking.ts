import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { Staking } from "../target/types/staking";
import { 
    PublicKey, 
    SystemProgram, 
    LAMPORTS_PER_SOL, 
    Keypair,
    Connection,
    Signer,
} from '@solana/web3.js';
import { 
    createMint,
    TOKEN_PROGRAM_ID, 
    getOrCreateAssociatedTokenAccount,
    Account as TokenAccount,
    mintTo,
    getAccount,
    createApproveInstruction,
    NATIVE_MINT,
} from '@solana/spl-token';
import { Config } from './types'
import { expect, assert } from 'chai';

const CONNECTION = new Connection("http://localhost:8899", 'recent');

describe("staking", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.Provider.env());

    const program = anchor.workspace.Staking as Program<Staking>;
    const provider = program.provider;

    let owner: Signer;
    const ownerInterest = 1; // %
    const confifChangeDelay = new BN(20); // secs
    const rewardQueueLength = 10;

    const rewardQueueKeypair: Keypair = anchor.web3.Keypair.generate();
    let rewardQueueElSize: number = 1; // TODO
    let rewardQueueMetadata: number = 12; // TODO get from the program (if it's possible)

    let rewardTokenMint: PublicKey;
    let stakedTokenMint: PublicKey;

    it("Initializes factory!", async () => {
        owner = await createUserWithLamports(3);

        // TODO get the seed value form the program Factory.PDA_KEY
        const [factoryPDA, _factoryBump] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("factory")
            ],
            program.programId
        );

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
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
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

        expect(factory.owner.toString()).to.be.eq(`${owner.publicKey}`);
        expect(factory.ownerInterest).to.be.eq(ownerInterest);
        expect(factory.rewardQueue.toString()).to.be.eq(`${rewardQueueKeypair.publicKey}`);
        expect(factory.configChangeDelay.toString()).to.be.eq(`${confifChangeDelay}`);
    });

    it("Creates mints", async () => {
        rewardTokenMint = await createTokenMint(owner.publicKey, owner, 6);
        stakedTokenMint = await createTokenMint(owner.publicKey, owner, 9);
    });

    it("Creates a new staking pool instance with fixed reward", async () => {
        // await program.rpc.new(
            // TODO
        // );
    });
});

/**
* A stakeholder will receive 10% (`rewardPerToken`) of the reward tokens for each staked token
* every 30 seconds (`rewardPeriod`).
*
* If the user want to unstake the tokens one should wait for 40 seconds (`unstakeDelay`)
* or loose 50% (`unstakeForseFeePercent`) of the tokens.
*/
// function createConfigFixed(): Config {
//     return {
//         rewardType: 0, // Fixed reward.
//         unstakeDelay: new BN(40), // secs
//         unstakeForseFeePercent: 50, // %
//         rewardPeriod: new BN(30), // secs
//         rewardPerToken: 10, // %
//         rewardTokensPerPeriod: new BN(0), // Not used with the fixed reward type.
//     }
// }

export async function createUserWithLamports(lamports: number): Promise<Signer> {
    const account = Keypair.generate();
    const signature = await CONNECTION.requestAirdrop(
        account.publicKey, 
        lamports * LAMPORTS_PER_SOL
    );
    await CONNECTION.confirmTransaction(signature);
    return account;
}

async function createTokenMint(authority: PublicKey, feePayer: Signer, decimals = 0): Promise<PublicKey> {
    return await createMint(
        CONNECTION,
        feePayer,
        authority,
        authority,
        decimals,
    );
}

async function getOrCreateATA(mint: PublicKey, feePayer: Signer, owner: PublicKey): Promise<TokenAccount> {
    return await getOrCreateAssociatedTokenAccount(
        CONNECTION,
        feePayer,
        mint,
        owner,
        true,
    );
}
