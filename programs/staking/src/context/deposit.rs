use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::{TokenAccount, Token};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        init_if_needed,
        payer = beneficiary,
        space = 8 + Member::SPACE,
        seeds = [
            beneficiary.to_account_info().key.as_ref(),
            factory.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub member: Account<'info, Member>,
    #[account(
        mut, 
        // TODO add an error with useful text
        constraint = vault_free.owner == member.to_account_info().key(),
        constraint = vault_free.mint == factory.stake_token_mint,
    )]
    pub vault_free: Box<Account<'info, TokenAccount>>,
    #[account(
        // TODO add errors with useful text
        constraint = vault_pending_unstaking.owner == member.to_account_info().key(), 
        constraint = vault_pending_unstaking.mint == vault_free.mint,
    )]
    pub vault_pending_unstaking: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(mut)]
    pub beneficiary_token_account: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
