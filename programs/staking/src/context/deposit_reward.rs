use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::{TokenAccount, Token};

#[derive(Accounts)]
pub struct DepositReward<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        mut,
        constraint = owner.key() == factory.owner,
    )]
    pub owner: Signer<'info>,
    #[account(
        mut,
        constraint = vault_owner.owner == owner.key(),
        constraint = vault_owner.mint == factory.reward_token_mint
    )]
    pub vault_owner: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = vault_reward.owner == factory.key(),
        constraint = vault_reward.mint == factory.reward_token_mint
    )]
    pub vault_reward: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
