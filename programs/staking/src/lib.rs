use anchor_lang::prelude::*;
mod reward; use reward::*;
mod account; use account::*;
mod context; use context::*;
mod access_control; use access_control::*;
mod error; use error::SPError;

declare_id!("5E1FrMGJa9S1qJHXVZKdhuu3WF8BrwzNdx1JKARyNbVm");

#[program]
pub mod staking {
    use super::*;

    /// Create the stake factory.
    pub fn initialize(ctx: Context<Initialize>, owner: Pubkey) -> Result<()> {
        let factory = &mut ctx.accounts.factory;
        factory.bump = *ctx.bumps.get(Factory::PDA_KEY).unwrap();
        factory.owner = owner;
        factory.reward_token_mint = ctx.accounts.reward_token_mint.key();
        factory.stake_token_mint = ctx.accounts.stake_token_mint.key();
        factory.vault_reward = ctx.accounts.vault_reward.key();

        Ok(())
    }

    /// Create a new stake pool instance
    pub fn new_stake_pool(
        ctx: Context<NewStakePool>,
        reward: Reward,
        bump: u8,
        ends_at: u64,
        min_owner_reward: u32,
        reward_metadata: u128, // Could be any data depending on the `Reward` // TODO combine with `reward` into a single struct
        owner_interest_percent: u8,
        unstake_delay: u64,
        reward_period: u64,
    ) -> Result<()> {
        require!(owner_interest_percent > 0 && owner_interest_percent < 100, SPError::OwnerInterestWrong);

        let stake_pool = &mut ctx.accounts.stake_pool;
        stake_pool.started_at = ctx.accounts.clock.unix_timestamp as u64;
        stake_pool.ends_at = ends_at;
        stake_pool.total_staked_tokens = 0;
        stake_pool.min_owner_reward = min_owner_reward;
        stake_pool.reward_type = reward;
        stake_pool.reward_metadata = reward_metadata;
        stake_pool.bump = bump;
        stake_pool.owner_interest_percent = owner_interest_percent;
        stake_pool.unstake_delay = unstake_delay;
        stake_pool.reward_period = reward_period;

        Ok(())
    }

    /// To interact with the program a user has to have a member account.
    pub fn create_member(
        ctx: Context<CreateMember>,
        member_bump: u8,
    ) -> Result<()> {
        let member = &mut ctx.accounts.member;
        member.beneficiary = *ctx.accounts.beneficiary.key;
        member.vault_free = ctx.accounts.vault_free.key();
        member.bump = member_bump;

        Ok(())
    }

    /// Transfer tokens from a member's external wallet to the member's internal `vault_free`,
    /// that belongs to the member, but controlled by the program.
    /// Member can freely deposit and withdraw tokens to/from the `vault_free`.
    /// The program cannot transfer any staked tokens without the member's owner signed request.
    /// 
    /// Tokens inside `vault_free` don't gain any rewards.
    /// To start getting rewards member can stake one's tokens
    /// inside `vault_free` by calling the `stake` method.
    pub fn deposit(
        ctx: Context<Deposit>,
        amount: u64, // The amount of tokens to deposit
    ) -> Result<()> {
        let beneficiary_tokens = ctx.accounts.beneficiary_token_account.amount;
        require!(amount <= beneficiary_tokens, SPError::InsufficientAmountOfTokensToDeposit);
        ctx.accounts.transfer_user_tokens_to_program(amount)
    }

    /// Move tokens from the `vault free` to the `MemberStake vault`
    /// Tokens inside `MemberStake vault` allow to get rewards pro rata staked amount.
    /// Member can stake coins from one's `vault free` to any stake.
    /// Member must claim the rewards before staking more tokens to the same pool.
    #[allow(unused_variables)] // but it's not
    pub fn stake(
        ctx: Context<Stake>,
        reward: Reward,
        member_stake_bump: u8,
        amount: u64, // The amount of the tokens to stake
    ) -> Result<()> {
        require!(amount <= ctx.accounts.vault_free.amount, SPError::NotEnoughFreeVaultAmount);

        let member_stake = &mut ctx.accounts.member_stake;
        member_stake.beneficiary = ctx.accounts.beneficiary.key();
        member_stake.vault_staked = ctx.accounts.vault_staked.key();
        member_stake.staked_at = ctx.accounts.clock.unix_timestamp as u64;
        member_stake.bump = member_stake_bump;
        member_stake.stake_pool = ctx.accounts.stake_pool.key();

        ctx.accounts.transfer_tokens_to_staked_vault(amount)?;
        ctx.accounts.stake_pool.total_staked_tokens += amount as u128;

        Ok(())
    }

    /// Deposit a reward for stakers.
    /// The reward is distributed on demand pro rata staked tokens.
    pub fn deposit_reward(ctx: Context<DepositReward>, amount: u64) -> Result<()> {
        // TODO check the amount is less or equals to the amount of tokens inside vault_owner

        ctx.accounts.transfer_tokens_to_reward_vault(amount)
    }

    /// Claim the reward for staked tokens
    // TODO remove reward from params and accounts
    pub fn claim_reward(ctx: Context<ClaimReward>, reward: Reward) -> Result<()> {
        let (reward_tokens_to_transfer, reward_payed_for) = ctx.accounts.calculate_reward_tokens()?;

        let reward_tokens_available = ctx.accounts.vault_reward.amount;
        require!(reward_tokens_to_transfer <= reward_tokens_available, SPError::InsufficientAmountOfTokensToClaim);

        ctx.accounts.transfer_reward_tokens(reward_tokens_to_transfer)?;
        ctx.accounts.member_stake.reward_payed_for = reward_payed_for;

        Ok(())
    }

    /// Move tokens from the `staked vault` to the `pending unstaking vault`.
    /// Save data to finish unstaking in the `pending unstaking` account provided by the user.
    pub fn start_unstake_all(
        ctx: Context<StartUnstakeAll>,
        reward: Reward, // TODO remove from here and from the accounts
        member_unstake_bump: u8,
    ) -> Result<()> {
        require!(ctx.accounts.vault_staked.amount > 0, SPError::NoStakedTokens);

        let unstake = &mut ctx.accounts.member_pending_unstake;
        unstake.bump = member_unstake_bump;
        unstake.stake_pool = ctx.accounts.stake_pool.key();
        unstake.beneficiary = ctx.accounts.beneficiary.key();
        unstake.vault_pending_unstake = ctx.accounts.vault_pending_unstake.key();
        unstake.unstaked_at = ctx.accounts.clock.unix_timestamp as u64;

        let stake_amount = ctx.accounts.vault_staked.amount;
        ctx.accounts.transfer_staked_tokens_to_pu_vault(stake_amount)?;

        ctx.accounts.stake_pool.total_staked_tokens -= stake_amount as u128;

        Ok(())
    }

    /// Moves tokens from `pending unstaking vault` to `free vault`.
    /// Destroys Stake and Unstake accounts and vaults
    #[access_control(allow_finish_unstake(&ctx))]
    pub fn finish_unstake_all(
        ctx: Context<FinishUnstakeAll>,
        reward: Reward, // TODO remove from here and from the accounts
    ) -> Result<()> {
        let unstake_amount = ctx.accounts.vault_pending_unstake.amount;
        ctx.accounts.transfer_pu_tokens_to_free_vault(unstake_amount)?;
        ctx.accounts.close_pending_unstake_vault()?;
        ctx.accounts.close_stake_vault()
    }

    /// Withdraw tokens from internal `free vault` controlled by the program
    /// to external user's wallet controlled by the user.
    /// 
    /// To withdraw deposited tokens from the stake program user firstly
    /// have to transfer tokens to his `free vault` inside the program 
    /// using start_unstake and finish_unstake methods.
    pub fn withdraw_all(ctx: Context<WithdrawAll>) -> Result<()> {
        ctx.accounts.transfer_free_vault_tokens_to_beneficiary()
    }

}
