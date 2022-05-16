use anchor_lang::prelude::*;
use crate::account::*;
use crate::reward::Reward;
use anchor_spl::token::{self, TokenAccount, Token};

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
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(
        mut,
        seeds = [
            stake_pool.to_account_info().key.as_ref(),
            member.to_account_info().key.as_ref(),
        ],
        bump,
        has_one = vault_staked,
        has_one = stake_pool,
        has_one = beneficiary,
    )]
    pub member_stake: Account<'info, MemberStake>,
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
        constraint = vault_staked.owner == member_stake.key(),
    )]
    pub vault_staked: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = vault_reward.owner == factory.key(),
        constraint = vault_reward.mint == factory.reward_token_mint
    )]
    pub vault_reward: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        constraint = beneficiary_reward_vault.owner == beneficiary.key(),
        constraint = beneficiary_reward_vault.mint == factory.reward_token_mint
    )]
    pub beneficiary_reward_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = owner_fee_reward_vault.owner == factory.owner,
        constraint = owner_fee_reward_vault.mint == factory.reward_token_mint
    )]
    pub owner_fee_reward_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> ClaimReward<'info> {
    pub fn calculate_reward_tokens(&self) -> Result<(u64, u64)> {
        self.stake_pool.reward_type
            .calculate(
                self.clock.unix_timestamp as u64,
                self.stake_pool.ends_at,
                self.stake_pool.started_at,
                self.vault_staked.amount,
                self.stake_pool.reward_period,
                self.stake_pool.reward_metadata,
                self.stake_pool.total_staked_tokens,
            )
    }

    pub fn transfer_reward_tokens(&self, reward_tokens: u64) -> Result<()> {
        let mut reward_tokens_for_owner = reward_tokens
            .checked_mul(self.stake_pool.owner_interest_percent as u64).unwrap()
            .checked_div(100).unwrap();

        if reward_tokens_for_owner == 0 {
            reward_tokens_for_owner = self.stake_pool.min_owner_reward as u64;
        }

        let reward_tokens_for_user = reward_tokens.checked_sub(reward_tokens_for_owner).unwrap();

        self.transfer_reward_tokens_to_user(reward_tokens_for_user)?;
        self.transfer_reward_tokens_to_owner(reward_tokens_for_owner)
    }

    fn transfer_reward_tokens_to_user(&self, amount: u64) -> Result<()> {
        let seeds = &[
            Factory::PDA_SEED,
            &[self.factory.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::Transfer { 
                    from: self.vault_reward.to_account_info(), 
                    to: self.beneficiary_reward_vault.to_account_info(), 
                    authority: self.factory.to_account_info() 
                },
                &[&seeds[..]],
            ),
            amount
        )
    }

    fn transfer_reward_tokens_to_owner(&self, amount: u64) -> Result<()> {
        let seeds = &[
            Factory::PDA_SEED,
            &[self.factory.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::Transfer { 
                    from: self.vault_reward.to_account_info(), 
                    to: self.owner_fee_reward_vault.to_account_info(), 
                    authority: self.factory.to_account_info() 
                },
                &[&seeds[..]],
            ),
            amount
        )
    }
}
