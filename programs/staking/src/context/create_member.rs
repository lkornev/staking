use anchor_lang::prelude::*;
use crate::account::*;
use anchor_spl::token::{TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;

#[derive(Accounts)]
pub struct CreateMember<'info> {
    #[account(
        seeds = [Factory::PDA_SEED],
        bump = factory.bump,
        has_one = stake_token_mint,
    )]
    pub factory: Account<'info, Factory>,
    pub stake_token_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
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
        init,
        payer = beneficiary,
        associated_token::mint = stake_token_mint,
        associated_token::authority = member,
    )]
    pub vault_free: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}
