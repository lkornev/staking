use anchor_lang::prelude::*;
use crate::account::*;
use crate::reward::Reward;
use crate::error::SPError;

#[derive(Accounts)]
#[instruction(reward: Reward)]
pub struct NewStakePool<'info> {
    #[account(
        mut, 
        seeds = [Factory::PDA_SEED], 
        bump = factory.bump,
        has_one = owner @ SPError::NewPoolOwnerMistmatch
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        init, 
        payer = owner,
        space = 8 + StakePool::SPACE,
        seeds = [&[reward as u8]],
        bump,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(zero)]
    pub config_history: Account<'info, ConfigHistory>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}