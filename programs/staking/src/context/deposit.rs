use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::{self, TokenAccount, Token};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,
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
        constraint = vault_free.mint == factory.stake_token_mint,
    )]
    pub vault_free: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(mut)]
    pub beneficiary_token_account: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    pub fn transfer_user_tokens_to_program(&self, amount_to_deposit: u64) -> Result<()> {
        let token_program = self.token_program.to_account_info();
        let from = self.beneficiary_token_account.to_account_info();
        let to = self.vault_free.to_account_info();
        let authority = self.beneficiary.to_account_info();

        token::transfer(
            CpiContext::new(
                token_program,
                token::Transfer { from, to, authority },
            ), 
            amount_to_deposit
        )
    }
}
