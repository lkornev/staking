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

export async function createUserWithLamports(
    connection: Connection, 
    lamports: number,
): Promise<Signer> {
    const account = Keypair.generate();
    const signature = await connection.requestAirdrop(
        account.publicKey, 
        lamports * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(signature);
    return account;
}

export async function createTokenMint(
    connection: Connection,
    authority: PublicKey,
    feePayer: Signer,
    decimals = 0
): Promise<PublicKey> {
    return await createMint(
        connection,
        feePayer,
        authority,
        authority,
        decimals,
    );
}
