use anchor_lang::prelude::*;
use crate::account::*;
use crate::reward::Reward;
use anchor_spl::token::{self, TokenAccount, Token};

#[derive(Accounts)]
#[instruction(reward: Reward)]
pub struct Stake<'info> {
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
        seeds = [
            beneficiary.to_account_info().key.as_ref(),
            factory.to_account_info().key.as_ref(),
        ],
        bump = member.bump,
        has_one = beneficiary,
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
        space = 8 + MemberStake::SPACE,
        seeds = [
            member.to_account_info().key.as_ref(),
            stake_pool.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub member_stake: Account<'info, MemberStake>,
    #[account(
        mut,
        // TODO add an error with useful text
        constraint = vault_staked.owner == member_stake.to_account_info().key(),
    )]
    pub vault_staked: Box<Account<'info, TokenAccount>>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    pub fn transfer_tokens_to_staked_vault(&self, amount_to_stake: u64) -> Result<()> {
        let token_program = self.token_program.to_account_info();
        let from = (*self.vault_free).to_account_info();
        let to = (*self.vault_staked).to_account_info();
        let authority = self.member.to_account_info();

        let seeds = &[
            self.beneficiary.to_account_info().key.as_ref(),
            self.factory.to_account_info().key.as_ref(),
            &[self.member.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                token_program,
                token::Transfer { to, from, authority },
                &[&seeds[..]],
            ),
            amount_to_stake
        )
    }
}
