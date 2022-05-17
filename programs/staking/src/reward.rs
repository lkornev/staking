use anchor_lang::prelude::*;
use crate::error::SPError;
use std::convert::TryFrom;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum Reward {
    /// Member will receive a fixed amount of reward tokens pro rata one's staked tokens.
    /// E.g. staked tokens: 300, `reward_per_token`: 5 %.
    /// Reward: 15 reward tokens per `reward_period`. (300 * 5 / 100)
    Fixed {
        data: u128, // reward_rate
    },
    /// Member will receive a part of tokens (`reward_per_period`)
    /// in proportion to the tokens of the all member_stakes in the pool.
    /// E.g. the user's staked tokens 300, 
    /// total staked tokens by every member in the pool: 1000,
    /// `reward_tokens_per_period`: 500.
    /// Reward: 150 reward tokens per `reward_period`. ((300 / 1000) * 500)
    Unfixed {
        data: u128, // reward_tokens_per_period
    },
}

impl Reward {
    pub fn new_fixed(reward_rate: u8) -> Reward {
        Reward::Fixed { data: reward_rate as u128 }
    }

    pub fn new_unfixed(reward_tokens_per_period: u128) -> Reward {
        Reward::Unfixed { data: reward_tokens_per_period }
    }

    pub fn calculate<'info>(
        &self,
        current_time: u64,
        program_ends_at: u64,
        staked_at: u64,
        staked_by_user: u64,
        reward_period: u64,
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
            Reward::Fixed{ data: reward_rate} => {
                let reward_rate = reward_rate.to_owned() as u64; // % 
                require!(reward_rate > 1, SPError::RewardRateTooSmall);
                require!(reward_rate < 100, SPError::RewardRateTooHigh);

                staked_by_user
                    .checked_mul(100).unwrap()
                    .checked_div(reward_rate).unwrap()
                    .checked_div(100).unwrap()
                    .checked_mul(full_reward_periods_amount).unwrap()
            },
            Reward::Unfixed{ data: reward_tokens_per_period } => {
                require!(reward_tokens_per_period > &0, SPError::TokensToShareEmpty);

                const PRECISENESS: u128 = 10000;
                let user_reward_rate: u128 = (staked_by_user as u128).checked_mul(PRECISENESS).unwrap()
                    .checked_div(total_staked as u128).unwrap();

                u64::try_from(
                    reward_tokens_per_period
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
        let reward_amount = one_reward_period(Reward::new_fixed(10), 1000, 1000).unwrap();
        assert_eq!(reward_amount, 100); 
    }

    #[test]
    fn fixed_config_two_reward_periods() {
        // 10% from 100 tokens for two reward periods
        let reward_amount = two_reward_periods(Reward::new_fixed(10), 100, 100).unwrap();
        assert_eq!(reward_amount, 20); 
    }

    #[test]
    fn unfixed_config_one_reward_period_multiple_stakers() {
        // 50 tokens staked by user it's 5% of total_staked tokens (1000) 
        // tokens_to_share among all stakers is 200
        // so 5% of 200 tokens is 10 tokens
        let reward_amount = one_reward_period(Reward::new_unfixed(200), 50, 1000).unwrap();
        assert_eq!(reward_amount, 10);
    }

    #[test]
    fn unfixed_config_one_reward_period_single_staker() {
        let reward_amount = one_reward_period(Reward::new_unfixed(200), 50, 50).unwrap();
        // 50 tokens staked by user it's 100% of total_staked (50) tokens
        // tokens_to_share among all stakers is 200
        // so 100% of 200 tokens is 200 tokens
        assert_eq!(reward_amount, 200); 
    }

    fn one_reward_period(reward: Reward, staked_by_user: u64, total_staked: u128) -> Result<u64> {
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
                total_staked,
            )?;

        assert_eq!(reward_payed_for, staked_at + reward_period); // paid for exactly one reward periods

        Ok(reward_amount)
    }

    fn two_reward_periods(reward: Reward, staked_by_user: u64, total_staked: u128) -> Result<u64> {
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
                total_staked,
            )?;

        assert_eq!(reward_payed_for, staked_at + reward_period * 2); // paid for exactly two reward periods

        Ok(reward_amount)
    }
}
