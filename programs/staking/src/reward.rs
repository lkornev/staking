use anchor_lang::prelude::*;
use crate::error::SPError;
use std::convert::TryFrom;

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
        total_staked: u128,
    ) -> Result<(u64, u64)> {
        require!(staked_by_user > 0, SPError::UserStakeZero);
        require!(current_time >= staked_at.checked_add(reward_period).unwrap(), SPError::RewardPeriodNotPassed);

        let last_reward_time = if program_ends_at > current_time { current_time } else { program_ends_at };
        let total_reward_time_passed = last_reward_time.checked_sub(staked_at).unwrap();
        let full_reward_periods_amount = total_reward_time_passed.checked_div(reward_period).unwrap();
        let full_reward_periods_end_at: u64 = full_reward_periods_amount.checked_mul(reward_period).unwrap()
            .checked_add(staked_at).unwrap();

        let reward_amount: u64 = match self {
            Reward::Fixed => {
                // `reward_metadata` is the fixed percentage of the income.
                let reward_rate = reward_metadata as u64; // % 
                require!(reward_rate > 1, SPError::RewardRateTooSmall);
                require!(reward_rate < 100, SPError::RewardRateTooHigh);

                staked_by_user
                    .checked_mul(100).unwrap()
                    .checked_div(reward_rate).unwrap()
                    .checked_div(100).unwrap()
                    .checked_mul(full_reward_periods_amount).unwrap()
            },
            Reward::Unfixed => {
                // `reward_metadata` is the amount of reward tokens that will be shared 
                // in proportion to a user's staked tokens among all members.
                let tokens_to_share = reward_metadata;
                require!(tokens_to_share > 0, SPError::TokensToShareEmpty);

                const PRECISENESS: u128 = 10000;
                let user_reward_rate: u128 = (staked_by_user as u128).checked_mul(PRECISENESS).unwrap()
                    .checked_div(total_staked as u128).unwrap();

                u64::try_from(
                    tokens_to_share
                        .checked_mul(user_reward_rate).unwrap()
                        .checked_div(PRECISENESS).unwrap()
                ).unwrap()
            },
        };

        Ok((reward_amount, full_reward_periods_end_at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_config_one_reward_period() {
        // 10% from 1000 tokens for one reward periods
        let reward_amount = one_reward_period(Reward::Fixed, 10, 1000, 1000).unwrap();
        assert_eq!(reward_amount, 100); 
    }

    #[test]
    fn fixed_config_two_reward_periods() {
        // 10% from 100 tokens for two reward periods
        let reward_amount = two_reward_periods(Reward::Fixed, 10, 100, 100).unwrap();
        assert_eq!(reward_amount, 20); 
    }

    #[test]
    fn unfixed_config_one_reward_period_multiple_stakers() {
        // 50 tokens staked by user it's 5% of total_staked tokens (1000) 
        // tokens_to_share among all stakers is 200
        // so 5% of 200 tokens is 10 tokens
        let reward_amount = one_reward_period(Reward::Unfixed, 200, 50, 1000).unwrap();
        assert_eq!(reward_amount, 10);
    }

    #[test]
    fn unfixed_config_one_reward_period_single_staker() {
        let reward_amount = one_reward_period(Reward::Unfixed, 200, 50, 50).unwrap();
        // 50 tokens staked by user it's 100% of total_staked (50) tokens
        // tokens_to_share among all stakers is 200
        // so 100% of 200 tokens is 200 tokens
        assert_eq!(reward_amount, 200); 
    }

    fn one_reward_period(reward: Reward, reward_data: u128, staked_by_user: u64, total_staked: u128) -> Result<u64> {
        let staked_at: u64 = 1652378565;
        let program_ends_at:u64 = 1652378663;
        let reward_period: u64 = 2; // secs
        let current_timestamp: u64 = 1652378567; // A little bit more than one reward period

        let (reward_amount, reward_payed_for) = reward
            .calculate(
                current_timestamp,
                program_ends_at,
                staked_at,
                staked_by_user,
                reward_period,
                reward_data,
                total_staked,
            )?;

        assert_eq!(reward_payed_for, staked_at + reward_period); // paid for exactly one reward periods

        Ok(reward_amount)
    }

    fn two_reward_periods(reward: Reward, reward_data: u128, staked_by_user: u64, total_staked: u128) -> Result<u64> {
        let staked_at: u64 = 1650106095;
        let program_ends_at:u64 = staked_at * 2;
        let reward_period: u64 = 500; // secs
        let current_timestamp: u64 = staked_at + (reward_period * 2) + 100; // A little bit more than two reward periods

        let (reward_amount, reward_payed_for) = reward
            .calculate(
                current_timestamp,
                program_ends_at,
                staked_at,
                staked_by_user,
                reward_period,
                reward_data,
                total_staked,
            )?;

        assert_eq!(reward_payed_for, staked_at + reward_period * 2); // paid for exactly two reward periods

        Ok(reward_amount)
    }
}
