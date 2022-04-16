use anchor_lang::prelude::*;
mod reward; use reward::*;
mod account; use account::*;
mod context; use context::*;
mod error; use error::SPError;
use anchor_spl::token;
use std::convert::TryFrom;

declare_id!("5E1FrMGJa9S1qJHXVZKdhuu3WF8BrwzNdx1JKARyNbVm");

#[program]
pub mod staking {
    use super::*;

    /// Create the stake factory.
    pub fn initialize(
        ctx: Context<Initialize>,
        owner: Pubkey,
        owner_interest: u8,
        config_change_delay: u128,
        reward_period: u64,
    ) -> Result<()> {
        let factory = &mut ctx.accounts.factory;

        factory.bump = *ctx.bumps.get(Factory::PDA_KEY).unwrap();
        factory.owner = owner;
        factory.owner_interest = owner_interest;
        factory.config_change_delay = config_change_delay;
        factory.reward_period = reward_period;
        factory.reward_token_mint = ctx.accounts.reward_token_mint.key();
        factory.stake_token_mint = ctx.accounts.stake_token_mint.key();
        factory.vault_reward = ctx.accounts.vault_reward.key();

        Ok(())
    }

    /// Create a new stake pool instance
    #[access_control(valid_reward_type(reward_type))]
    pub fn new(
        ctx: Context<NewStakePool>,
        reward_type: u8, // Reward enum
        unstake_delay: u64,
        unstake_forse_fee_percent: u8,
        reward_metadata: u128, // Could be any data depending on the `reward_type`
        config_history_length: u32,
        bump: u8,
    ) -> Result<()> {
        let stake_pool_config = StakePoolConfig {
            started_at: u64::try_from(ctx.accounts.clock.unix_timestamp)
                .expect("Creation time should be positive integer"),
            ended_at: None,
            total_staked_tokens: 0,
            unstake_delay,
            unstake_forse_fee_percent,
            reward_type,
            reward_metadata,
        };

        let stake_pool = &mut ctx.accounts.stake_pool;

        stake_pool.bump = bump;
        stake_pool.config_history = *ctx.accounts.config_history.to_account_info().key;

        let config_history = &mut ctx.accounts.config_history;

        config_history
            .history
            .resize(config_history_length as usize, None);

        config_history.append(stake_pool_config);

        Ok(())
    }

    /// Transfer tokens from a user's external wallet to the user's internal `vault_free`,
    /// that belongs to the user, but controlled by the program.
    /// User can freely deposit and withdraw tokens to/from the `vault_free`.
    /// The program cannot transfer any staked tokens without the user's signed request.
    /// 
    /// Tokens inside `vault_free` don't gain any rewards.
    /// To start getting rewards user can stake one's tokens
    /// inside `vault_free` by calling the `stake` method.
    pub fn deposit(
        ctx: Context<Deposit>,
        member_bump: u8,
        amount: u64, // The amount of tokens to deposit
    ) -> Result<()> {
        let member = &mut ctx.accounts.member;

        member.beneficiary = *ctx.accounts.beneficiary.key;
        member.vault_free = (*ctx.accounts.vault_free).key();
        member.vault_pending_unstaking = (*ctx.accounts.vault_pending_unstaking).key();
        member.bump = member_bump;

        let token_program = ctx.accounts.token_program.to_account_info();
        let from = ctx.accounts.beneficiary_token_account.to_account_info();
        let to = (*ctx.accounts.vault_free).to_account_info();
        let authority = ctx.accounts.beneficiary.to_account_info();

        // TODO check the amount is less or equals to the beneficiary_token_account amount of tokens 
        // and throw an erorr if needed

        token::transfer(
            CpiContext::new(
                token_program,
                token::Transfer { from, to, authority },
            ), 
            amount
        )
    }

    /// Move tokens from the `vault free` to the `stakeholder valut`
    /// Tokens inside `stakeholder valut` allow to get rewards pro rata staked amount.
    /// Member can stake coins from one's `vault free` to any stake.
    /// Member must claim the rewards before staking more tokens to the same pool. (TODO Check)
    #[access_control(valid_reward_type(reward_type))]
    pub fn stake(
        ctx: Context<Stake>,
        reward_type: u8, // Reward enum. It's used in stake pool seeds
        stakeholder_bump: u8,
        amount: u64, // The amount of the tokens to stake
    ) -> Result<()> {
        let stakeholder = &mut ctx.accounts.stakeholder;
        let token_program = ctx.accounts.token_program.to_account_info();
        let from = (*ctx.accounts.vault_free).to_account_info();
        let to = (*ctx.accounts.vault_staked).to_account_info();
        let authority = ctx.accounts.member.to_account_info();

        // TODO check the amount is less or equals to the vault_free amount of tokens 
        // and throw an erorr if needed

        stakeholder.owner = *ctx.accounts.beneficiary.owner;
        stakeholder.vault = to.key();
        stakeholder.staked_at =  u64::try_from(ctx.accounts.clock.unix_timestamp)
            .expect("Staking time should be positive integer");
        stakeholder.bump = stakeholder_bump;

        let seeds = &[
            ctx.accounts.beneficiary.to_account_info().key.as_ref(),
            ctx.accounts.factory.to_account_info().key.as_ref(),
            &[ctx.accounts.member.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                token_program,
                token::Transfer { to, from, authority },
                &[&seeds[..]],
            ),
            amount
        )?;

        // TODO update total_staked_tokens in the stake pool config

        Ok(())
    }

    /// Deposit a reward for stakers.
    /// The reward is distributed on demand pro rata staked tokens.
    pub fn deposit_reward(
        ctx: Context<DepositReward>,
        amount: u64,
    ) -> Result<()> {
        let token_program = ctx.accounts.token_program.to_account_info();
        let from = ctx.accounts.vault_owner.to_account_info();
        let to = ctx.accounts.vault_reward.to_account_info();
        let authority = ctx.accounts.owner.to_account_info();

        // TODO check the amount is less or equals to the amount of tokens inside vault_owner  
        // and throw an erorr if needed

        token::transfer(
            CpiContext::new(
                token_program,
                token::Transfer { from, to, authority },
            ), 
            amount
        )
    }

    /// Claim the reward for staked tokens
    #[access_control(valid_reward_type(reward_type))]
    pub fn claim_reward(
        ctx: Context<ClaimReward>,
        reward_type: u8, // Reward Type enum
    ) -> Result<()> {
        let factory = &ctx.accounts.factory;
        let config_history = &ctx.accounts.config_history;
        let stakeholder = &ctx.accounts.stakeholder;
        let vault_staked = &ctx.accounts.vault_staked;
        let clock = &ctx.accounts.clock;

        let (reward_tokens_amoun, config_cursor) = Reward::try_from(reward_type).unwrap()
            .calculate(
                factory.reward_period, 
                vault_staked.amount,
                u64::try_from(clock.unix_timestamp)
                    .expect("Current time should be positive integer"),
                &config_history,
                &stakeholder,
            );

        // TODO if reward_tokens_amount = 0 return an error
        // TODO set config_cursor to stakeholder
        // TODO send reward tokens from vault_staked to the beneficiary's external_vault

        // TODO Transfer the factory owner one's owner_interest in the reward tokens.
        Ok(())
    }

    /// Move tokens from the `staked vault` to the `pending unstaking vault`.
    /// Saves data to finish unstaking in the `pending unstaking` account provided by the user.
    /// The `pending unstaking` account must belongs to the user.
    /// Member must claim the rewards before unstaking tokens. (TODO Check)
    pub fn start_unstake(
        _ctx: Context<StartUnstake>,
    ) -> Result<()> {
        // TODO remove stakeholder account and return lamports to the stake's owner
        // TODO update total_staked_tokens in the stake pool config
        unimplemented!()
    }

    /// Move tokens from `pending unstaking vault` to `free vault`.
    pub fn finish_unstake(
        _ctx: Context<FinishUnstake>,
        // Unstake immediately without waiting for `unstake_delay` by paying the `unstake_forse_fee`
        _forse: bool,
    ) -> Result<()> {
        // TODO if unstake_forse_fee === 0 or unstake_delay passed, than ignore forse flag
        unimplemented!()
    }

    /// Withdraw tokens from internal `free vault` controlled by the program
    /// to external user's wallet controlled by the user. (TODO check)
    /// 
    /// To withdraw deposited tokens from the stake program user firstly
    /// have to transfer tokens to his `free vault` inside the program 
    /// using start_unstake and finish_unstake methods.
    /// 
    /// To get rewards for staking see the `claim_reward` method.
    pub fn withdraw(
        _ctx: Context<Withdraw>,
    ) -> Result<()> {
        // TODO if all user accounts will be empty after withdraw,
        // than destroy token accounts and return rent-extempt sol to the user
        unimplemented!()
    }

    /// Change the config of the stake pool. 
    /// Changing the config means pushing the new config account in the ConfigHistory.
    /// 
    /// The configuration of the stake program can be changed on the fly,
    /// therefore, before changing the configuration, 
    /// the owner needs to give the pool member the promised reward
    /// for the time they staked their tokens using the current configuration.
    pub fn change_config(
        _ctx: Context<ChangeConfig>,
    ) -> Result<()> {
        // TODO withdraw the reward tokens from the owner's account,
        // add the new config to the config history in a single transaction.
        unimplemented!()
    }
}

// Access control checks

fn valid_reward_type<'info>(reward_type: u8) -> Result<()> {
    Reward::try_from(reward_type).map(|_rew| ())
}
