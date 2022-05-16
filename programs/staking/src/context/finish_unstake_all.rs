use anchor_lang::prelude::*;
use crate::account::*;
use crate::reward::Reward;
use anchor_spl::token::{self, TokenAccount, Token};


#[derive(Accounts)]
#[instruction(reward: Reward)]
pub struct FinishUnstakeAll<'info> {
    #[account(
        seeds = [Factory::PDA_SEED], 
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
    #[account(
        mut,
        seeds = [&[reward as u8]],
        bump = stake_pool.bump,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        constraint = vault_beneficiary.owner == beneficiary.key(),
        constraint = vault_beneficiary.mint == factory.stake_token_mint
    )]
    pub vault_beneficiary: Box<Account<'info, TokenAccount>>,
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
        seeds = [
            stake_pool.to_account_info().key.as_ref(),
            member.to_account_info().key.as_ref(),
        ],
        bump,
        has_one = vault_staked,
    )]
    pub member_stake: Account<'info, MemberStake>,
    #[account(mut)]
    pub vault_staked: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [
            stake_pool.to_account_info().key.as_ref(),
            member_stake.to_account_info().key.as_ref(),
        ],
        bump = member_pending_unstake.bump,
        has_one = vault_pending_unstake,
    )]
    pub member_pending_unstake: Account<'info, MemberPendingUnstake>,
    #[account(
        mut,
        constraint = vault_pending_unstake.owner == stake_pool.key(),
        constraint = vault_pending_unstake.mint == factory.stake_token_mint
    )]
    pub vault_pending_unstake: Box<Account<'info, TokenAccount>>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> FinishUnstakeAll<'info> {
    pub fn transfer_pu_tokens_to_beneficiary_vault(&self, amount: u64) -> Result<()> {
        // let seeds = &[
        //     self.stake_pool.to_account_info().key.as_ref(),
        //     self.member.to_account_info().key.as_ref(),
        //     &[self.member_stake.bump]
        // ];

        // token::transfer(
        //     CpiContext::new_with_signer(
        //         self.token_program.to_account_info(),
        //         token::Transfer {
        //             from: self.vault_staked.to_account_info(),
        //             to: self.vault_pending_unstake.to_account_info(),
        //             authority: self.member_stake.to_account_info(),
        //         },
        //         &[&seeds[..]],
        //     ),
        //     amount
        // )

        unimplemented!()
    }
}
