use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::{TokenAccount, Token};

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
        space = 8 + Member::SPACE,
        seeds = [
            beneficiary.to_account_info().key.as_ref(),
            stake_pool.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub member: Account<'info, Member>,
    #[account(
        mut, 
        // TODO add an error with useful text
        constraint = vault_free.owner == member.to_account_info().key(),
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
    /// CHECK: used only for transfer tokens from by the signed beneficiary instraction.
    #[account(mut)]
    pub beneficiary_token_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw {}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        seeds = [StakePool::PDA_SEED_FIXED],
        bump = stake_pool.bump,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        seeds = [
            beneficiary.to_account_info().key.as_ref(),
            stake_pool.to_account_info().key.as_ref(),
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
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

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
