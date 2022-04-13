use anchor_lang::prelude::*;

/// The program main state.
#[account]
pub struct Factory {
    pub bump: u8,
    /// The owner of the stake pool program instance.
    pub owner: Pubkey,
    /// The percentage of reward tokens the owner will receive for users stakes.
    pub owner_interest: u8,
    /// The amount of seconds should be passed before the next config change is allowed.
    pub config_change_delay: u128,
    /// Describes the type of the reward tokens.
    /// The mint itself does not need to be under control of the stake pool owner.
    /// It could be the wrapped Sol mint or any other spl token mint.
    pub reward_token_mint: Pubkey,
    /// Describes the type of the tokens that are allowed to be staked.
    /// The mint itself does not need to be under control of the stake pool owner or a Member.
    /// It could be the wrapped Sol mint or any other spl token mint.
    pub stake_token_mint: Pubkey,
}

impl Factory {
    pub const SPACE: usize = 1 + 32 + 1 + 32 + 16 + 16 + 16; // TODO remove extra 16
    pub const PDA_KEY: &'static str = "factory";
    pub const PDA_SEED: & 'static [u8] = Self::PDA_KEY.as_bytes();
}

#[account]
pub struct StakePool {
    /// Storage of the config changes
    pub config_history: Pubkey,
    pub bump: u8,
}

impl StakePool {
    pub const SPACE: usize = 32 + 8;
}

#[account]
pub struct ConfigHistory {
    // Invariant: index is position of the next available slot.
    pub head: u32,
    // Invariant: index is position of the first (oldest) taken slot.
    // Invariant: head == tail => queue is initialized.
    // Invariant: index_of(head + 1) == index_of(tail) => queue is full.
    pub tail: u32,
    // Although a vec is used, the size is immutable
    // and defines during initialization.
    pub history: Vec<Option<Pubkey>>,
}

impl ConfigHistory {
    pub fn append(&mut self, config: Pubkey) -> u32 {
        let cursor = self.head;

        // Insert into next available slot.
        let h_idx = self.index_of(self.head);
        self.history[h_idx] = Some(config);

        // Update head and tail counters.
        let is_full = self.index_of(self.head + 1) == self.index_of(self.tail);
        if is_full {
            self.tail += 1;
        }
        self.head += 1;

        cursor
    }

    pub fn index_of(&self, counter: u32) -> usize {
        counter as usize % self.capacity()
    }

    pub fn capacity(&self) -> usize {
        self.history.len()
    }

    pub fn get(&self, cursor: u32) -> Option<Pubkey> {
        self.history[cursor as usize % self.capacity()]
    }

    pub fn head(&self) -> u32 {
        self.head
    }

    pub fn tail(&self) -> u32 {
        self.tail
    }
}

#[account]
pub struct StakePoolConfig{
    // creating timestamp
    pub created_at: i64,
    /// Last time the config changed. Unix timestamp.
    pub last_config_change: i64,
    // The total amount of tokens that been staked by all users of the pool
    // when this config was active.
    pub total_staked_tokens: u128,
    /// The time in seconds a Member have to wait 
    /// to finish unstaking without paying the fee (`unstake_forse_fee_percent`).
    pub unstake_delay: u64,
    /// If a user wants unstake without waiting `unstake_delay`
    /// the user can pay the `unstake_forse_fee_percent` and receives the amount of tokens equals to 
    /// staked_tokens - (staked_tokens * unstake_forse_fee_percent) / 100.
    /// Should be in the range 0 - 100. (TODO check range)
    pub unstake_forse_fee_percent: u8,
    /// The time in seconds a Member have to wait to receive the next reward.
    /// After each `reward_period` the Member are allowed to claim the reward.
    pub reward_period: u64,
    /// If the `reward_type` is Fixed
    /// `reward_metadata` is the fixed percentage of the income.
    /// Should be greather than 0 and less or equal to 100. (TODO check the range only if the reward_type is Fixed).
    /// The reward is granted as a fixed percentage of the staked tokens.
    /// 
    /// If the `reward_type` is Unfixed
    /// `reward_metadata` is the amount of reward tokens that will be shared 
    /// in proportion to the user's staked tokens among all members.
    /// Should be greather than 0.
    pub reward_type: u8,
    pub reward_metadata: u128,
}

impl StakePoolConfig {
    pub const SPACE: usize = 8 + 8 + 16 + 8 + 1 + 8 + 1 + 16;
}

/// Member account represents a user of the stake pool factory program.
#[account]
pub struct Member {
    /// The owner and beneficiary of the Member account.
    pub owner: Pubkey,
    /// User can freely deposit and withdraw tokens to or from the `vault_free`.
    /// Tokens on the `vault_free` do not produce rewards.
    /// Used as a transit zone between external and internal wallets/vaults.
    pub vault_free: Pubkey,
    /// Tokens transfered to `vault_pending_unstaking` after calling `start_unstake` method.
    /// The tokens inside `vault_pending_unstaking` are not giving the rewards any more.
    /// The tokens could be forsed to be transfered to `vault_free` immediately
    /// by paying the `unstake_forse_fee_percent` penalty.
    /// Or they could be transferred for free after the period of time
    /// defined in the `unstake_delay` variable in the StakePoolConfig. 
    pub vault_pending_unstaking: Pubkey,
    pub bump: u8,
}

impl Member {
    pub const SPACE: usize = 32 * 3 + 8;
}

/// It's leterally holds a stake
#[account]
pub struct Stakeholder {
    /// The owner and beneficiary of the stake and the Member account.
    pub owner: Pubkey,
    /// The tokens inside Stakeholder vault gain rewards.
    pub vault: Pubkey,
    // The UNIX timestamp when the staking begins
    pub staked_at: i64,
}

impl Stakeholder {
    pub const SPACE: usize = 32 * 2;
}
