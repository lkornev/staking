use anchor_lang::prelude::*;

/// The factory program main state.
#[account]
pub struct Factory {
    pub bump: u8,
    /// The owner of the stake pool program instance.
    pub owner: Pubkey,
    /// The percentage of reward tokens the owner will receive for users stakes.
    pub owner_interest: u8,
    /// Global event queue for reward vendoring.
    pub reward_queue: Pubkey,
    /// The amount of seconds should be passed before the next config change is allowed.
    pub config_change_delay: u128,
}

impl Factory {
    pub const SPACE: usize = 1 + 32 + 1 + 32 + 32 + 32 + 32 + 16 + 8;
    pub const PDA_KEY: &'static str = "factory";
    pub const PDA_SEED: & 'static [u8] = Factory::PDA_KEY.as_bytes();
}

/// The config describes the behaviour of the staking pool instance.
#[account]
pub struct StakePool {
    /// The time in seconds a stakeholder have to wait 
    /// to finish unstaking without paying the fee (`unstake_forse_fee_percent`).
    pub unstake_delay: u64,
    /// If a user wants unstake without waiting `unstake_delay`
    /// the user can pay the `unstake_forse_fee_percent` and receives the amount of tokens equals to 
    /// staked_tokens - (staked_tokens * unstake_forse_fee_percent) / 100.
    /// Should be in the range 0 - 100. (TODO check range)
    pub unstake_forse_fee_percent: u8,
    /// The time in seconds a stakeholder have to wait to receive the next reward.
    /// After each `reward_period` the stakeholders are allowed to claim the reward.
    pub reward_period: u64,
    /// Describes the type of the reward tokens.
    /// The mint itself does not need to be under control of the stake pool owner.
    /// It could be the wrapped Sol mint or any other spl token mint.
    pub reward_token_mint: Pubkey,
    /// Describes the type of the tokens that are allowed to be staked.
    /// The mint itself does not need to be under control of the stake pool owner or a stakeholder.
    /// It could be the wrapped Sol mint or any other spl token mint.
    pub stake_token_mint: Pubkey,
    /// Last time the config changed. Unix timestamp.
    pub last_config_change: i64,
    /// If the `reward_type` is Fixed
    /// `reward_metadata` is the fixed percentage of the income.
    /// Should be greather than 0 and less or equal to 100. (TODO check the range only if the reward_type is Fixed).
    /// The reward is granted as a fixed percentage of the staked tokens.
    /// 
    /// If the `reward_type` is Unfixed
    /// `reward_metadata` is the amount of reward tokens that will be shared 
    /// in proportion to the user's staked tokens among all stakeholders.
    /// Should be greather than 0.
    pub reward_type: u8,
    pub reward_metadata: u128, // TODO change to Pubkey ?
}

impl StakePool {
    pub const SPACE: usize = 8 + 1 + 8 + 16 + 16 + 8 + 1 + 16 + 32; // TODO Why we need extra 32?
    pub const PDA_SEED_FIXED: & 'static [u8] = b"stake-pool-fixed";
    pub const PDA_SEED_UNFIXED: & 'static [u8] = b"stake-pool-unfixed";
}

#[derive(Default, Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct RewardEvent {
    // /// The config could be changed by the owner of the stake pool, so we store
    // /// the reference to the state snapshot that was actual at the time when the reward was created.
    // /// That state snapshot is used to calculate the appopriate reward for the user.
    // state_snapshot: Pubkey,

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
    pub free_vault: Pubkey,
    /// The amount of staked tokens that belongs to the user.
    /// This valult is used to calculate the reward.
    /// The tokens could be unstaked and transfered to `pending_unstaking_vault`.
    pub stake_vault: Pubkey,
    /// The tokens inside `pending_unstaking_vault` are not giving the rewards.
    /// The tokens could be forsed to be transfered to `free_vault`
    /// by paying the `unstake_forse_fee_percent` penalty.
    /// Or they could be transferred for free after the period of time 
    /// defined in the `unstake_delay` variable in the Config. 
    pub pending_unstaking_vault: Pubkey,
    /// The owner of the Stakeholder account.
    pub beneficiary: Pubkey,
    /// StakePool the Stakeholder belongs to.
    pub stake_pool: Pubkey,
    /// Next position in the rewards event queue to process.
    pub rewards_cursor: u32,
    /// The clock timestamp of the last time this account staked or switched
    /// entities. Used as a proof to reward vendors that the Member account
    /// was staked at a given point in time.
    pub last_stake_timestamp: u128,
}

#[account]
pub struct RewardQueue {
    // Invariant: index is position of the next available slot.
    pub head: u32,
    // Invariant: index is position of the first (oldest) taken slot.
    // Invariant: head == tail => queue is initialized.
    // Invariant: index_of(head + 1) == index_of(tail) => queue is full.
    pub tail: u32,
    // Although a vec is used, the size is immutable
    // and defines during initialization.
    pub events: Vec<RewardEvent>,
}
