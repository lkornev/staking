use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::{TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::error::SPError;

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

#[derive(Accounts)]
#[instruction(reward_type: u8)]
pub struct NewStakePool<'info> {
    #[account(
        mut, 
        seeds = [Factory::PDA_SEED], 
        bump = factory.bump,
        // factory.owner == owner.key()
        has_one = owner @ SPError::NewPoolOwnerMistmatch
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        init, 
        payer = owner,
        space = 8 + StakePool::SPACE,
        seeds = [&[reward_type]],
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

#[derive(Accounts)]
#[instruction(reward_type: u8)]
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
    /// CHECK: used only for transfer tokens from by the signed beneficiary instraction.
    #[account(mut)]
    pub beneficiary_token_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(reward_type: u8)]
pub struct Stake<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        seeds = [&[reward_type]],
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


#[derive(Accounts)]
pub struct DepositReward<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
    // The owner of the program. TODO Use multisig instead.
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

#[derive(Accounts)]
#[instruction(reward_type: u8)]
pub struct ClaimReward<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        seeds = [&[reward_type]],
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

#[derive(Accounts)]
pub struct StartUnstake {}

#[derive(Accounts)]
pub struct FinishUnstake {}

#[derive(Accounts)]
pub struct Withdraw {}

#[derive(Accounts)]
pub struct ChangeConfig {}
