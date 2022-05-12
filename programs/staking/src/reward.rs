use anchor_lang::prelude::*;
use crate::error::SPError;

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum Reward {
    /// Member will receive a fixed amount of reward tokens pro rata one's staked tokens.
    /// E.g. staked tokens: 300, `reward_per_token`: 5 %.
    /// Reward: 15 reward tokens per `reward_period`. (300 * 5 / 100)
    Fixed = 0,
    /// Member will receive a part of tokens (`reward_per_period`)
    /// in proportion to the tokens of the all member_stakes in the pool.
    /// E.g. the user's staked tokens 300, 
    /// total staked tokens by every member in the pool: 1000,
    /// `reward_tokens_per_period`: 500.
    /// Reward: 150 reward tokens per `reward_period`. ((300 / 1000) * 500)
    Unfixed = 1,
}

impl Reward {
    /// Returns the reward tokens amount and the new config index in the reward history
    /// (reward tokens amount, config index)
    pub fn calculate<'info>(
        &self,
        current_time: u64,
        program_ends_at: u64,
        staked_at: u64,
        staked_by_user: u64,
        reward_period: u64,
        reward_metadata: u128,
        _total_staked: u128,
    ) -> Result<(u64, u64)> {
        if staked_by_user == 0 || current_time < staked_at.checked_add(reward_period).unwrap() {
            return err!(SPError::NoRewardAvailable);
        }

        match self {
            Reward::Fixed => {
                let last_reward_time = if program_ends_at > current_time { current_time } else { program_ends_at };
                let total_reward_time_passed = last_reward_time.checked_sub(staked_at).unwrap();
                let full_reward_periods_amount = total_reward_time_passed.checked_div(reward_period).unwrap();
                let full_reward_periods_end_at: u64 = full_reward_periods_amount.checked_mul(reward_period).unwrap()
                    .checked_add(staked_at).unwrap();

                let reward_rate = reward_metadata as u64; // %
                let reward_amount: u64 = staked_by_user
                    .checked_mul(100).unwrap()
                    .checked_div(reward_rate).unwrap()
                    .checked_div(100).unwrap()
                    .checked_mul(full_reward_periods_amount).unwrap();

                Ok((reward_amount, full_reward_periods_end_at))
            },
            Reward::Unfixed => {
                unimplemented!();
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_config_one_reward_period() {
        let staked_at: u64 = 1652378565;
        let program_ends_at:u64 = 1652378663;
        let reward = Reward::Fixed;
        let reward_rate: u128 =  10; // % reward_metadata
        let reward_period: u64 = 2; // secs
        let staked_by_user: u64 = 1000; // tokens
        let current_timestamp: u64 = 1652378567; // A little bit more than one reward period
        let total_staked: u128 = 100000000;

        let (reward_amount, reward_payed_for) = reward
            .calculate(
                current_timestamp,
                program_ends_at,
                staked_at,
                staked_by_user,
                reward_period,
                reward_rate as u128,
                total_staked,
            ).unwrap();

        assert_eq!(reward_amount, 100); // 10% from 1000 tokens for one reward periods
        assert_eq!(reward_payed_for, staked_at + reward_period); // paid for exactly one reward periods

        let reward_tokens_for_owner = reward_amount
            .checked_mul(1).unwrap()
            .checked_div(100).unwrap();
        let reward_tokens_for_user = reward_amount.checked_sub(reward_tokens_for_owner).unwrap();

        assert_eq!(reward_tokens_for_owner, 1);
        assert_eq!(reward_tokens_for_user, 99);
    }

    #[test]
    fn fixed_config_two_reward_periods() {
        let staked_at: u64 = 1650106095;
        let program_ends_at:u64 = staked_at * 2;
        let reward = Reward::Fixed;
        let reward_rate: u8 =  10; // %
        let reward_period: u64 = 500; // secs
        let staked_by_user: u64 = 100; // tokens
        let current_timestamp: u64 = staked_at + (reward_period * 2) + 100; // A little bit more than two reward periods
        let total_staked: u128 = staked_by_user as u128;

        let (reward_amount, reward_payed_for) = reward
            .calculate(
                current_timestamp,
                program_ends_at,
                staked_at,
                staked_by_user,
                reward_period,
                reward_rate as u128,
                total_staked,
            ).unwrap();

        assert_eq!(reward_amount, 20); // 10% from 100 tokens for two reward periods
        assert_eq!(reward_payed_for, staked_at + reward_period * 2); // paid for exactly two reward periods
    }
}


