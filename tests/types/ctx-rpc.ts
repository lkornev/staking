import { Program } from "@project-serum/anchor";
import { Staking } from "../../target/types/staking";
import {
    Connection,
    PublicKey,
    Signer,
} from '@solana/web3.js';
import { Account } from '@solana/spl-token';
import { Ctx } from "../ctx/ctx";

export interface CtxRPC {
    connection: Connection,
    program: Program<Staking>,
    owner: Signer,
    ownerTokenAccount: Account,
    factory: PublicKey,
    vaultReward: PublicKey,
}

export function sliceCtxRpc(ctx: Ctx): CtxRPC {
    return {
        connection: ctx.connection,
        program: ctx.program,
        owner: ctx.owner,
        ownerTokenAccount: ctx.ownerTokenAccount,
        factory: ctx.PDAS.factory.key,
        vaultReward: ctx.vaultReward,
    }
}
