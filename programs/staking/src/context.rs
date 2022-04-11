use anchor_lang::prelude::*;
use crate::account::*;

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
    #[account(zero)]
    pub reward_queue: Account<'info, RewardQueue>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(reward_type: u8)]
pub struct NewStakePool<'info> {
    #[account(mut, seeds = [Factory::PDA_SEED], bump = factory.bump)]
    pub factory: Account<'info, Factory>,
    #[account(
        init, 
        payer = owner,
        space = 8 + StakePool::SPACE,
        seeds = [StakePool::PDA_SEED_FIXED],
        bump,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit {}

#[derive(Accounts)]
pub struct Withdraw {}

#[derive(Accounts)]
pub struct Stake {}

#[derive(Accounts)]
pub struct StartUnstake {}

#[derive(Accounts)]
pub struct FinishUnstake {}

#[derive(Accounts)]
pub struct ClaimReward {}

#[derive(Accounts)]
pub struct DropReward {
    // TODO USE multisig
}

#[derive(Accounts)]
pub struct ChangeConfig {}
