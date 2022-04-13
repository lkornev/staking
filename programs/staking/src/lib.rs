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
        reward_type: u8, // RewardType enum
        unstake_delay: u64,
        unstake_forse_fee_percent: u8,
        reward_period: u64,
        reward_metadata: u128, // Could be any data depending on the `reward_type`
        config_history_length: u32,
        bump: u8,
    ) -> Result<()> {
        if ctx.accounts.owner.key() != ctx.accounts.factory.owner {
            return err!(SPError::NewPoolOwnerMistmatch)
        }

        RewardType::try_from(reward_type)?;

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

    /// Transfers tokens from a user's external wallet to the user's internal `vault_free`,
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

        member.owner = *ctx.accounts.beneficiary.key;
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

    /// Moves tokens from the `vault free` to the `stakeholder valut`
    /// Tokens inside `stakeholder valut` allow to get rewards pro rata staked amount.
    /// Member can stake coins from one's `vault free` to any stake.
    /// Member must claim the rewards before staking more tokens to the same pool. (TODO Check)
    pub fn stake(
        ctx: Context<Stake>,
        reward_type: u8, // RewardType enum. It's used in stake pool seeds
        stakeholder_bump: u8,
        amount: u64, // The amount of the tokens to stake
    ) -> Result<()> {
        RewardType::try_from(reward_type)?;
        let stakeholder = &mut ctx.accounts.stakeholder;
        let token_program = ctx.accounts.token_program.to_account_info();
        let from = (*ctx.accounts.vault_free).to_account_info();
        let to = (*ctx.accounts.vault_staked).to_account_info();
        let authority = ctx.accounts.member.to_account_info();

        // TODO check the amount is less or equals to the vault_free amount of tokens 
        // and throw an erorr if needed

        stakeholder.owner = *ctx.accounts.beneficiary.owner;
        stakeholder.vault = to.key();
        stakeholder.staked_at = ctx.accounts.clock.unix_timestamp;
        stakeholder.bump = stakeholder_bump;

        let seeds = &[
            ctx.accounts.member.to_account_info().key.as_ref(),
            ctx.accounts.stake_pool.to_account_info().key.as_ref(),
            &[stakeholder_bump]
        ];

        // TODO fix Cross-program invocation with unauthorized signer or writable account
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
        _ctx: Context<DropReward>,
    ) -> Result<()> {
        // Transfer rewards to the stake pool factory.
        unimplemented!()
    }

    /// Claims a reward for staked tokens.
    pub fn claim_reward(
        _ctx: Context<ClaimReward>,
    ) -> Result<()> {
        // Iterate received stake pool config history and get rewards
        // Transfer the factory owner one's owner_interest in the reward tokens.
        unimplemented!()
    }

    /// Moves tokens from the `staked vault` to the `pending unstaking vault`.
    /// Saves data to finish unstaking in the `pending unstaking` account provided by the user.
    /// The `pending unstaking` account must belongs to the user.
    /// Member must claim the rewards before unstaking tokens. (TODO Check)
    pub fn start_unstake(
        _ctx: Context<StartUnstake>,
    ) -> Result<()> {
        // TODO remove stakeholder account and return lamports to the stake's owner
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
