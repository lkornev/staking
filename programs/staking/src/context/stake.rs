use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;

#[derive(Accounts)]
pub struct Stake<'info> {
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
            stake_pool.name.as_ref(),
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
        has_one = beneficiary,
        has_one = vault_free,
    )]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub vault_free: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = beneficiary,
        space = 8 + MemberStake::SPACE,
        seeds = [
            stake_pool.to_account_info().key.as_ref(),
            member.to_account_info().key.as_ref(),
        ],
        bump,
    )]
    pub member_stake: Account<'info, MemberStake>,
    #[account(
        init,
        payer = beneficiary,
        associated_token::mint = stake_token_mint,
        associated_token::authority = member_stake,
    )]
    pub vault_staked: Box<Account<'info, TokenAccount>>, 
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    pub fn transfer_tokens_to_staked_vault(&self, amount_to_stake: u64) -> Result<()> {
        let token_program = self.token_program.to_account_info();
        let from = self.vault_free.to_account_info();
        let to = self.vault_staked.to_account_info();
        let authority = self.member.to_account_info();

        let seeds = &[
            self.beneficiary.to_account_info().key.as_ref(),
            self.factory.to_account_info().key.as_ref(),
            &[self.member.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                token_program,
                token::Transfer { from, to, authority },
                &[&seeds[..]],
            ),
            amount_to_stake
        )
    }
}
