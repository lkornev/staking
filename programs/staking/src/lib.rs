use anchor_lang::prelude::*;
mod reward; use reward::*;
mod account; use account::*;
mod context; use context::*;
mod error; use error::SPError;
use std::convert::TryFrom;

declare_id!("5E1FrMGJa9S1qJHXVZKdhuu3WF8BrwzNdx1JKARyNbVm");

#[program]
pub mod staking {
    use super::*;

    /// Creates the staking factory. 
    /// The method must to be called right after deploying the program.
    pub fn initialize(
        ctx: Context<Initialize>,
        owner: Pubkey,
        owner_interest: u8,
        config_change_delay: u128,
        reward_token_mint: Pubkey,
        stake_token_mint: Pubkey,
    ) -> Result<()> {
        msg!("HELLO!");
        let factory = &mut ctx.accounts.factory;

        factory.bump = *ctx.bumps.get(Factory::PDA_KEY).unwrap();
        factory.owner = owner;
        factory.owner_interest = owner_interest;
        factory.config_change_delay = config_change_delay;
        factory.reward_token_mint = reward_token_mint;
        factory.stake_token_mint = stake_token_mint;

        Ok(())
    }

    /// Creates a new stake pool instance
    pub fn new(
        ctx: Context<NewStakePool>,
        unstake_delay: u64,
        unstake_forse_fee_percent: u8,
        reward_period: u64,
        reward_type: u8, // RewardType enum
        reward_metadata: u128, // Could be any data depending on the `reward_type`
        config_history_length: u32,
        bump: u8,
    ) -> Result<()> {
        if ctx.accounts.owner.key() != ctx.accounts.factory.owner {
            return err!(SPError::NewPoolOwnerMistmatch)
        } else if RewardType::try_from(reward_type).is_err() {
            return err!(SPError::RewardTypeMismatch)
        }

        let stake_pool_config = &mut ctx.accounts.stake_pool_config;

        stake_pool_config.unstake_delay = unstake_delay;
        stake_pool_config.unstake_forse_fee_percent = unstake_forse_fee_percent;
        stake_pool_config.reward_period = reward_period;
        stake_pool_config.created_at = ctx.accounts.clock.unix_timestamp;
        stake_pool_config.last_config_change = ctx.accounts.clock.unix_timestamp;
        stake_pool_config.reward_metadata = reward_metadata;
        stake_pool_config.reward_type = reward_type;

        let stake_pool = &mut ctx.accounts.stake_pool;
        stake_pool.bump = bump;
    
        stake_pool.config_history = *ctx.accounts.config_history.to_account_info().key;

        let config_history = &mut ctx.accounts.config_history;

        config_history
            .history
            .resize(config_history_length as usize, None);

        config_history.append(stake_pool_config.to_account_info().key());

        Ok(())
    }

    /// Transfers tokens from a user's external wallet to the user's internal `free vault`,
    /// that belongs to the user, but controlled by the program.
    /// User can freely deposit and withdraw tokens to/from the `free vault`.
    /// The program cannot transfer any staked tokens without the user's signed request.
    /// 
    /// Tokens inside `free vault` don't bring any rewards.
    /// To start getting rewards user can stake one's tokens
    /// inside `free vault` by calling the `stake` method.
    pub fn deposit(
        _ctx: Context<Deposit>,
    ) -> Result<()> {
        // TODO implement
        Ok(())
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

    /// Moves tokens from the `free vault` to the `staked valut`
    /// Tokens inside `staked valut` allow to get rewards pro rata staked amount.
    /// Stakeholder must claim the rewards before staking more tokens. (TODO Check)
    pub fn stake(
        _ctx: Context<Stake>,
        _amount: u128, // The amount of tokens to stake
    ) -> Result<()> {
        // TODO set timestamp when the staking begins
        // TODO update total_staked_tokens in the stake pool config
        unimplemented!()
    }

    /// Moves tokens from the `staked vault` to the `pending unstaking vault`.
    /// Saves data to finish unstaking in the `pending unstaking` account provided by the user.
    /// The `pending unstaking` account must belongs to the user.
    /// Stakeholder must claim the rewards before unstaking tokens. (TODO Check)
    pub fn start_unstake(
        _ctx: Context<StartUnstake>,
    ) -> Result<()> {
        unimplemented!()
    }

    /// Moves tokens from `pending unstaking vault` to `free vault`.
    pub fn finish_unstake(
        _ctx: Context<FinishUnstake>,
        // Unstake immediately without waiting for `unstake_delay` by paying the `unstake_forse_fee`
        _forse: bool,
    ) -> Result<()> {
        // TODO if unstake_forse_fee === 0 or unstake_delay passed, than ignore forse flag
        unimplemented!()
    }

    /// Claims a reward for staked tokens.
    pub fn claim_reward(
        _ctx: Context<ClaimReward>,
    ) -> Result<()> {
        unimplemented!()
    }

    /// Deposit a reward for stakers.
    /// The reward is distributed pro rata to staked beneficiaries.
    pub fn give_reward(
        _ctx: Context<DropReward>,
    ) -> Result<()> {
        unimplemented!()
    }

    /// Change the config of the stake pool. 
    /// Changing the config means pushing the new config account in the ConfigHistory.
    /// 
    /// The configuration of the stake program can be changed on the fly,
    /// therefore, before changing the configuration, 
    /// the owner needs to give the pool stakeholders the promised reward
    /// for the time they staked their tokens using the current configuration.
    pub fn change_config(
        _ctx: Context<ChangeConfig>,
    ) -> Result<()> {
        // TODO withdraw the reward tokens from the owner's account
        // and change the config in one transaction.
        unimplemented!()
    }
}
