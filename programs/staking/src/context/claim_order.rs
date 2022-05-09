use anchor_lang::prelude::*;
use crate::account::*;
use crate::reward::Reward;
use anchor_spl::token::{TokenAccount, Token};

#[derive(Accounts)]
#[instruction(reward: Reward)]
pub struct ClaimReward<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        seeds = [&[reward as u8]],
        bump = stake_pool.bump,
        has_one = config_history,
    )]
    pub stake_pool: Account<'info, StakePool>,
    pub config_history: Account<'info, ConfigHistory>,
    #[account(
        seeds = [
            member.to_account_info().key.as_ref(),
            stake_pool.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub stakeholder: Account<'info, Stakeholder>,
    #[account(
        seeds = [
            beneficiary.to_account_info().key.as_ref(),
            factory.to_account_info().key.as_ref(),
        ],
        bump = member.bump,
        has_one = beneficiary,
    )]
    pub member: Account<'info, Member>,
    #[account(
        // TODO add an error with useful text
        constraint = vault_staked.owner == stakeholder.key(),
    )]
    pub vault_staked: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        // TODO add an error with useful text
        constraint = vault_reward.owner == factory.key(),
        constraint = vault_reward.mint == factory.reward_token_mint
    )]
    pub vault_reward: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        // TODO add an error with useful text
        constraint = external_vault.owner == beneficiary.key(),
    )]
    pub external_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}
