use anchor_lang::prelude::*;
use crate::account::*;
use crate::reward::Reward;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;


#[derive(Accounts)]
#[instruction(reward: Reward)]
pub struct StartUnstakeAll<'info> {
    #[account(
        seeds = [Factory::PDA_SEED], 
        bump = factory.bump,
        has_one = stake_token_mint,
    )]
    pub factory: Account<'info, Factory>,
    pub stake_token_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            factory.to_account_info().key.as_ref(),
            &[reward as u8] // TODO replace with some random pubkey
        ],
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
        init,
        payer = beneficiary,
        space = 8 + MemberPendingUnstake::SPACE,
        seeds = [
            stake_pool.to_account_info().key.as_ref(),
            member_stake.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub member_pending_unstake: Account<'info, MemberPendingUnstake>,
    #[account(
        init,
        payer = beneficiary,
        associated_token::mint = stake_token_mint,
        associated_token::authority = stake_pool,
    )]
    pub vault_pending_unstake: Box<Account<'info, TokenAccount>>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> StartUnstakeAll<'info> {
    pub fn transfer_staked_tokens_to_pu_vault(&self, amount: u64) -> Result<()> {
        let seeds = &[
            self.stake_pool.to_account_info().key.as_ref(),
            self.member.to_account_info().key.as_ref(),
            &[self.member_stake.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.vault_staked.to_account_info(),
                    to: self.vault_pending_unstake.to_account_info(),
                    authority: self.member_stake.to_account_info(),
                },
                &[&seeds[..]],
            ),
            amount
        )
    }
}
