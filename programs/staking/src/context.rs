use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::TokenAccount;

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
    #[account(
        init, 
        payer = owner,
        space = 8 + StakePoolConfig::SPACE,
        seeds = [
            b"0", // Index in the Config Histroy
            stake_pool.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub stake_pool_config: Account<'info, StakePoolConfig>,
    #[account(zero)]
    pub config_history: Account<'info, ConfigHistory>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        seeds = [StakePool::PDA_SEED_FIXED],
        bump = stake_pool.bump,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(
        init_if_needed,
        payer = beneficiary,
        space = 8 + Stakeholder::SPACE,
        seeds = [
            beneficiary.to_account_info().key.as_ref(),
            stake_pool.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub stakeholder: Account<'info, Stakeholder>,
    #[account(
        mut, 
        // TODO add error with useful text
        constraint = vault_free.owner == stakeholder.to_account_info().key(),
    )]
    pub vault_free: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        // TODO add error with useful text
        constraint = vault_pending_unstaking.owner == stakeholder.to_account_info().key(), 
        constraint = vault_pending_unstaking.mint == vault_free.mint,
    )]
    pub vault_pending_unstaking: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    pub system_program: Program<'info, System>,
}

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
