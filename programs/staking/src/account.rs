use anchor_lang::prelude::*;
use crate::reward::Reward;

/// The program main state.
/// These parameters cannot be changed after the initialization.
#[account]
pub struct Factory {
    pub bump: u8,
    /// The owner of the stake pool factory program.
    pub owner: Pubkey,
    /// Describes the type of the reward tokens.
    /// The mint itself does not need to be under control of the stake pool owner.
    /// It could be the wrapped Sol mint or any other spl token mint.
    pub reward_token_mint: Pubkey,
    /// Describes the type of the tokens that are allowed to be staked.
    /// The mint itself does not need to be under control of the stake pool owner or a Member.
    /// It could be the wrapped Sol mint or any other spl token mint.
    pub stake_token_mint: Pubkey,
    /// The vault with the reward tokens to be transferred to the members on demand.
    pub vault_reward: Pubkey,
}

impl Factory {
    pub const SPACE: usize = 1 + 32 * 4;
    pub const PDA_KEY: &'static str = "factory";
    pub const PDA_SEED: & 'static [u8] = Self::PDA_KEY.as_bytes();
}

#[account]
pub struct StakePool {
    /// If the `reward_type` is Fixed
    /// `reward_metadata` is the fixed percentage of the income.
    /// Should be greater than 0 and less or equal to 100. (TODO check the range only if the reward_type is Fixed).
    /// The reward is granted as a fixed percentage of the staked tokens.
    /// 
    /// If the `reward_type` is Unfixed
    /// `reward_metadata` is the amount of reward tokens that will be shared 
    /// in proportion to a user's staked tokens among all members.
    /// Should be greater than 0.
    pub reward_type: Reward,
    pub reward_metadata: u128,
    /// The UNIX time when this config was created. 
    // Invariant: The config that is added later has a timestamp greater than previous configs.
    pub started_at: u64,
    /// The time when the stake pool is no longer gains any rewards
    pub ends_at: u64,
    /// The total amount of tokens that been staked by all users of the pool
    /// when this config was active.
    pub total_staked_tokens: u128,
    pub bump: u8,
    /// The percentage of reward tokens the owner will receive from each user's reward.
    pub owner_interest_percent: u8,
    /// Minimum amount of reward tokens the owner will receive from each user's reward.
    pub min_owner_reward: u32,
    /// The time in seconds a Member have to wait to receive the next reward.
    /// After each `reward_period` the Member are allowed to claim the reward.
    pub reward_period: u64,
}

impl StakePool {
    pub const SPACE: usize = 1 + 16 + 8 + 8 + 16 + 1 + 1 + 4 + 8;
}

/// Member account represents a user of the stake pool factory program.
#[account]
pub struct Member {
    /// The owner and beneficiary of the Member account.
    pub beneficiary: Pubkey,
    /// User can freely deposit and withdraw tokens to or from the `vault_free`.
    /// Tokens on the `vault_free` do not produce rewards.
    /// Used as a transit zone between external and internal wallets/vaults.
    pub vault_free: Pubkey,
    pub bump: u8,
}

impl Member {
    pub const SPACE: usize = 32 * 2 + 8;
}

#[account]
pub struct MemberStake {
    /// StakePool the member has the stake in
    pub stake_pool: Pubkey,
    /// The owner and beneficiary of the stake and the Member account.
    pub beneficiary: Pubkey,
    /// The tokens inside Stakeholder vault gain rewards.
    pub vault_staked: Pubkey,
    /// The UNIX timestamp when the staking started
    pub staked_at: u64,
    /// When the last reward was payed
    pub reward_payed_for: u64,
    pub bump: u8,
}

impl MemberStake {
    pub const SPACE: usize = 32 * 3 + 8 + 8 + 1;
}

#[account]
pub struct MemberPendingUnstake {
    /// StakePool the member has the stake in
    pub stake_pool: Pubkey,
    /// The owner and beneficiary of the stake and the Member account.
    pub beneficiary: Pubkey,
    /// The tokens transferred to `vault_pending_unstaking` after calling `start_unstake` method.
    /// The tokens inside `vault_pending_unstaking` are not giving the rewards any more.
    /// The tokens could be transferred for free after the period of time
    /// defined in the `unstake_delay` variable in the StakePool. 
    pub vault_pending_unstake: Pubkey,
    /// The UNIX timestamp when the unstaking started
    pub unstaked_at: u64,
    pub bump: u8,
}

impl MemberPendingUnstake {
    pub const SPACE: usize = 32 * 3 + 8 + 1;
}
