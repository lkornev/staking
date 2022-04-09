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

    const owner: PublicKey = provider.wallet.publicKey;
    const ownerInterest = 1; // %
    let feePayer: Signer;
    let rewardTokenMint: PublicKey;
    const stakedTokenMint: PublicKey = NATIVE_MINT;
    const rewardQueueKeypair: Keypair = anchor.web3.Keypair.generate();
    const confifChangeDelay = new BN(20); // secs
    const configFixedPayment: Config = createConfigFixed();
    const rewardQueueLength = 10;

    it("Sets up reward token mint and the fee payer", async () => {
        feePayer = await createUserWithLamports(1);
        rewardTokenMint = await createTokenMint({
            authority: provider.wallet.publicKey,
            feePayer,
            decimals: 6,
        });
    });

    it("It initialized!", async () => {
        // TODO create config PDA and use configFixed

        const tx = await program.rpc.initialize(
            owner,
            ownerInterest,
            rewardTokenMint,
            stakedTokenMint,
            confifChangeDelay,
            rewardQueueLength,
            {
                // TODO specify accounts
            }
        );
        console.log("Your transaction signature", tx);
    });
});

/**
* A stakeholder will receive 10% (`rewardPerToken`) of the reward tokens for each staked token
* every 30 seconds (`rewardPeriod`).
*
* If the user want to unstake the tokens one should wait for 40 seconds (`unstakeDelay`)
* or loose 50% (`unstakeForseFeePercent`) of the tokens.
*/
function createConfigFixed(): Config {
    return {
        rewardType: 0, // Fixed reward.
        unstakeDelay: new BN(40), // secs
        unstakeForseFeePercent: 50, // %
        rewardPeriod: new BN(30), // secs
        rewardPerToken: 10, // %
        rewardTokensPerPeriod: new BN(0), // Not used with the fixed reward type.
    }
}


async function createTokenMint(params: { authority: PublicKey, feePayer: Signer, decimals: number }): Promise<PublicKey> {
    return await createMint(
        CONNECTION,
        params.feePayer,
        params. authority,
        null,
        params.decimals || 0,
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

export async function createUserWithLamports(lamports: number): Promise<Signer> {
    const account = Keypair.generate();
    const signature = await CONNECTION.requestAirdrop(account.publicKey, lamports);
    await CONNECTION.confirmTransaction(signature);
    return account;
}
