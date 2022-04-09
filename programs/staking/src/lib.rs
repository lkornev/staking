use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod staking {
    use super::*;

    /// Creates a staking instance. 
    /// Must to be called right after deploying the program.
    pub fn initialize(
        ctx: Context<Initialize>,
        owner: Pubkey,
        owner_interest: u8,
        reward_token_mint: Pubkey,
        staked_token_mint: Pubkey,
        config_change_delay: u128,
        reward_queue_length: u32,
    ) -> Result<()> {
        let stake_pool = &mut ctx.accounts.stake_pool;

        stake_pool.bump = *ctx.bumps.get(StakePool::PDA_KEY).unwrap();
        stake_pool.owner = owner;
        stake_pool.owner_interest = owner_interest;
        stake_pool.reward_token_mint = reward_token_mint;
        stake_pool.staked_token_mint = staked_token_mint;
        stake_pool.reward_queue = ctx.accounts.reward_queue.to_account_info().key();
        stake_pool.config_change_delay = config_change_delay;
        stake_pool.last_config_change = ctx.accounts.clock.unix_timestamp;

        ctx.accounts.reward_queue
            .events
            .resize(reward_queue_length as usize, Default::default());

        Ok(())
    }

    /// Transfers tokens from a user's external wallet to the user's internal `free vault`,
    /// that belongs to the user, but controlled by the program.
    /// User can freely deposit and withdraw tokens to/from the `free vault`.
    /// The program cannot transfer any staked tokens without the user's signed request.
    /// 
    /// Tokens inside `free vault` don't bring any rewards.
    /// To start getting rewards user can stake his tokens
    /// inside `free vault` by calling the `stake` method.
    pub fn deposit(
        ctx: Context<Deposit>,
    ) -> Result<()> {
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
        ctx: Context<Withdraw>,
    ) -> Result<()> {
        // TODO if all user accounts will be empty after withdraw,
        // than destroy token accounts and return rent-extempt sol to the user
        Ok(())
    }

    /// Moves tokens from the `free vault` to the `staked valut`
    /// Tokens inside `staked valut` allow to get rewards pro rata staked amount.
    /// Stakeholder must claim the rewards before staking more tokens. (TODO Check)
    pub fn stake(
        ctx: Context<Stake>,
        amount: u128, // The amount of tokens to stake
    ) -> Result<()> {
        // TODO set timestamp when the staking begins
        Ok(())
    }

    /// Moves tokens from the `staked vault` to the `pending unstaking vault`.
    /// Saves data to finish unstaking in the `pending unstaking` account provided by the user.
    /// The `pending unstaking` account must belongs to the user.
    pub fn start_unstake(
        ctx: Context<StartUnstake>,
    ) -> Result<()> {
        Ok(())
    }

    /// Moves tokens from `pending unstaking vault` to `free vault`.
    pub fn finish_unstake(
        ctx: Context<FinishUnstake>,
        // Unstake immediately without waiting for `unstake_delay` by paying the `unstake_forse_fee`
        forse: bool, 
    ) -> Result<()> {
        // TODO if unstake_forse_fee === 0 or unstake_delay passed, than ignore forse flag
        Ok(())
    }

    /// Claims a reward for staked tokens.
    pub fn claim_reward(
        ctx: Context<ClaimReward>,
    ) -> Result<()> {
        Ok(())
    }

    /// Drop a reward for stakers.
    /// The reward is distributed pro rata to staked beneficiaries.
    pub fn drop_reward(
        ctx: Context<DropReward>,
    ) -> Result<()> {
        // Create a Reward Vendor account with an associated token vault holding the reward.
        // Assign the Reward Vendor the next available position in a Reward Event Queue. 
        // Then, to retrieve a reward, a staker invokes the ClaimReward command,
        // providing a proof that the funds were staked at the time of the reward being dropped, 
        // and in response, the program transfers the proportion of the dropped reward 
        // to the polling stakeholder. TODO rename?
        // The operation completes by incrementing the stakeholder's queue cursor,
        // ensuring that a given reward can only be processed once.
        Ok(())
    }

    /// Change the config of the stake pool. 
    /// Changing the config means replacing the config account in the StakePool struct.
    /// 
    /// The configuration of the stake program can be changed on the fly,
    /// therefore, before changing the configuration, 
    /// the owner needs to give the pool stakeholders the promised reward
    /// for the time they staked their tokens using the current configuration.
    pub fn change_config(
        ctx: Context<ChangeConfig>,
    ) -> Result<()> {
        // TODO withdraw the reward tokens from the owner's account
        // and change the config in one transaction.
        Ok(())
    }
}

/// The program state.
#[account]
pub struct StakePool {
    bump: u8,
    /// The owner of the stake pool program instance.
    owner: Pubkey,
    /// The percentage of reward tokens the owner will receive for each stake.
    owner_interest: u8,
    /// Describes the type of the reward tokens. 
    /// The mint itself does not need to be under control of the stake pool owner.
    /// It could be the sol mint or any other spl token mint.
    reward_token_mint: Pubkey,
    /// Describes the type of the tokens that are allowed to be staked.
    /// The mint itself does not need to be under control of the stake pool owner or a stakeholder.
    /// It could be the sol mint or any other spl token mint.
    staked_token_mint: Pubkey,
    /// Global event queue for reward vendoring.
    reward_queue: Pubkey,
    /// The config describes the behaviour of the stake_pool. (see Config struct)
    /// Config could be changed by the owner once per `config_change_delay`.
    config: Pubkey,
    /// The amount of seconds should be passed before the next config change is allowed.
    config_change_delay: u128,
    /// Last time the config changed. Unix timestamp.
    last_config_change: i64,
}

impl StakePool {
    pub const LEN: usize = 1 + 32 + 1 + 32 + 32 + 32 + 32 + 16 + 8;
    pub const PDA_KEY: &'static str = "stake-pool";
    pub const PDA_SEED: & 'static [u8] = StakePool::PDA_KEY.as_bytes();
}

pub enum RewardType {
    /// A stakeholder will receive a fixed amount of reward tokens pro rata one's staked tokens.
    /// E.g. staked tokens: 300, `reward_per_token`: 5 %.
    /// Reward: 15 reward tokens per `reward_period`. (300 * 5 / 100)
    Fixed = 0,
    /// A stakeholder will receive a part of tokens (`reward_per_period`)
    /// in proportion to the tokens of the all stakeholders in the pool.
    /// E.g. the user's staked tokens 300, 
    /// total staked tokens by every stakeholder in the pool: 1000,
    /// `reward_tokens_per_period`: 500.
    /// Reward: 150 reward tokens per `reward_period`. ((300 / 1000) * 500)
    Unfixed = 1,
}

/// The config describes the behaviour of the stake_pool.
#[account]
pub struct Config {
    /// The time in seconds a stakeholder have to wait 
    /// to finish unstaking without paying the fee (`unstake_forse_fee_percent`).
    unstake_delay: u64,
    /// If a user wants unstake without waiting `unstake_delay`
    /// the user can pay the `unstake_forse_fee_percent` and receives the amount of tokens equals to 
    /// staked_tokens - (staked_tokens * unstake_forse_fee_percent) / 100.
    /// Should be in the range 0 - 100. (TODO check range)
    unstake_forse_fee_percent: u8,
    /// The time in seconds a stakeholder have to wait to receive the next reward.
    /// After each `reward_period` the stakeholders are allowed to claim the reward.
    reward_period: u64,
    // Could be Fixed (0) or Unfixed (1). See RewardType enum for more details.
    reward_type: u8,
    /// If the reward_type is Fixed (should be 0 otherwise).
    /// Should be greather than 0 and less or equal to 100. (TODO check the range only if the reward_type is Fixed).
    /// The reward is granted as a fixed percentage of the staked tokens.
    reward_per_token: u8,
    /// If the reward_type is Unfixed (should be 0 otherwise).
    /// The fixed amount of reward tokens that will be shared 
    /// in proportion to the user's staked tokens among all stakeholders.
    /// Should be greather than 0.
    reward_tokens_per_period: u128,
}

impl Config {
    pub const LEN: usize = 8 + 1 + 8 + 1 + 1 + 16;
    pub const PDA_SEED: & 'static [u8] = b"config";
}

#[derive(Default, Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct Reward {
    /// The config could be changed by the owner of the stake pool, so we store
    /// the reference to the state snapshot that was actual at the time when the reward was created.
    /// That state snapshot is used to calculate the appopriate reward for the user.
    state_snapshot: Pubkey,

    // vendor: Pubkey, // Reward vendor (TODO Why separate from Reward ?)
    // ts: i64, // timestamp

    // TODO store total staked tokens to calculate the reward amount for a particular stakeholder.
}

/// Stakeholder account represents a stake state.
#[account]
pub struct Stakeholder {
    /// User can freely deposit and withdraw tokens to or from the `free_vault`.
    /// Tokens on the `free_vault` do not produce rewards.
    /// Used as a transit zone between external and internal wallets/vaults.
    free_vault: Pubkey,
    /// The amount of staked tokens that belongs to the user.
    /// This valult is used to calculate the reward.
    /// The tokens could be unstaked and transfered to `pending_unstaking_vault`.
    stake_vault: Pubkey,
    /// The tokens inside `pending_unstaking_vault` are not giving the rewards.
    /// The tokens could be forsed to be transfered to `free_vault`
    /// by paying the `unstake_forse_fee_percent` penalty.
    /// Or they could be transferred for free after the period of time 
    /// defined in the `unstake_delay` variable in the Config. 
    pending_unstaking_vault: Pubkey,
    /// The owner of the Stakeholder account.
    beneficiary: Pubkey,
    /// StakePool the Stakeholder belongs to.
    stake_pool: Pubkey,
    /// Next position in the rewards event queue to process.
    rewards_cursor: u32,
    /// The clock timestamp of the last time this account staked or switched
    /// entities. Used as a proof to reward vendors that the Member account
    /// was staked at a given point in time.
    last_stake_timestamp: u128,
}

#[account]
pub struct RewardQueue {
    // Invariant: index is position of the next available slot.
    head: u32,
    // Invariant: index is position of the first (oldest) taken slot.
    // Invariant: head == tail => queue is initialized.
    // Invariant: index_of(head + 1) == index_of(tail) => queue is full.
    tail: u32,
    // Although a vec is used, the size is immutable
    // and defines during initialization.
    events: Vec<Reward>, // (TODO Or use Reward Event?)
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = initializer,
        space = 8 + StakePool::LEN,
        seeds = [StakePool::PDA_SEED], 
        bump,
    )]
    stake_pool: Account<'info, StakePool>,
    #[account(
        init, 
        payer = initializer,
        space = 8 + Config::LEN,
        seeds = [Config::PDA_SEED], 
        bump,
    )]
    config: Account<'info, Config>,
    #[account(zero)]
    reward_queue: Account<'info, RewardQueue>,
    #[account(mut)]
    initializer: Signer<'info>,
    clock: Sysvar<'info, Clock>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit {}

#[derive(Accounts)]
pub struct Withdraw {}

#[derive(Accounts)]
pub struct Stake {}

#[derive(Accounts)]
pub struct StartUnstake {}

#[derive(Accounts)]
pub struct FinishUnstake {}

#[derive(Accounts)]
pub struct ClaimReward {}

#[derive(Accounts)]
pub struct DropReward {
    // TODO USE multisig
}

#[derive(Accounts)]
pub struct ChangeConfig {}
