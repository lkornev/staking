use anchor_lang::prelude::*;
use crate::account::*;
use crate::reward::Reward;
use anchor_spl::token::{TokenAccount, Token};

#[derive(Accounts)]
#[instruction(reward: Reward)]
pub struct Stake<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        seeds = [&[reward as u8]],
        bump = stake_pool.bump,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        seeds = [
            beneficiary.to_account_info().key.as_ref(),
            factory.to_account_info().key.as_ref(),
        ],
        bump = member.bump,
    )]
    pub member: Account<'info, Member>,
    #[account(
        mut, 
        // TODO add an error with useful text
        constraint = vault_free.owner == member.to_account_info().key(),
    )]
    pub vault_free: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = beneficiary,
        space = 8 + Stakeholder::SPACE,
        seeds = [
            member.to_account_info().key.as_ref(),
            stake_pool.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub stakeholder: Account<'info, Stakeholder>,
    #[account(
        mut,
        // TODO add an error with useful text
        constraint = vault_staked.owner == stakeholder.to_account_info().key(),
    )]
    pub vault_staked: Box<Account<'info, TokenAccount>>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
