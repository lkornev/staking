use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::{self, TokenAccount, Token};

#[derive(Accounts)]
pub struct WithdrawAll<'info> {
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
        has_one = vault_free,
    )]
    pub member: Account<'info, Member>,
    #[account(mut)]
    pub vault_free: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(mut)]
    pub beneficiary_token_account: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawAll<'info> {
    pub fn transfer_free_vault_tokens_to_beneficiary(&self) -> Result<()> {
        let amount = self.vault_free.amount;

        let seeds = &[
            self.beneficiary.to_account_info().key.as_ref(),
            self.factory.to_account_info().key.as_ref(),
            &[self.member.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::Transfer { 
                    from: self.vault_free.to_account_info(), 
                    to: self.beneficiary_token_account.to_account_info(), 
                    authority: self.member.to_account_info() 
                },
                &[&seeds[..]],
            ),
            amount
        )
    }
}
