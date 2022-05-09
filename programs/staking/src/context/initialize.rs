use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{TokenAccount, Token, Mint};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = initializer,
        space = 8 + Factory::SPACE,
        seeds = [Factory::PDA_SEED], 
        bump,
    )]
    pub factory: Account<'info, Factory>,
    pub reward_token_mint: Box<Account<'info, Mint>>,
    pub stake_token_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = reward_token_mint,
        associated_token::authority = factory,
    )]
    pub vault_reward: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
