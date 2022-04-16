use std::convert::TryFrom; 
use crate::SPError;
use crate::account::{ConfigHistory, Stakeholder};
use anchor_lang::prelude::{Error};
use anchor_lang::err;

#[derive(PartialEq, Eq, Debug)]
pub enum Reward {
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

impl Reward {
    /// Returns the reward tokens amount and the new config index in the reward history
    /// (reward tokens amount, config index)
    pub fn calculate<'info>(
        &self,
        reward_period: u64,
        staked_by_user: u64,
        current_timestamp: u64,
        config_history: &ConfigHistory, 
        stakeholder: &Stakeholder,
    ) -> (u64, u32) {
        match self {
            Reward::Fixed => {
                // Config cursor is an index of the active config in the config history 
                // that stakeholder account will use to retrieve actual coefficients
                let mut config_cursor = stakeholder.config_cursor;
                let mut reward_coeff: f64 = 0.0;

                // If the stakeholder's config_cursor is less then the tail, then the ring buffer has
                // overwritten those entries, so jump to the tail.
                let tail = config_history.tail();
                if config_cursor < tail {
                    config_cursor = tail;
                }

                // The unix time when the last full reward period ends
                let last_full_period_ends_at: u64 = current_timestamp - 
                    ((current_timestamp - stakeholder.staked_at) % reward_period);

                let mut current_period_ends_at: u64 = stakeholder.staked_at + reward_period;
                let mut processing_time_at = stakeholder.staked_at;
 
                while config_cursor < config_history.head() {
                    if processing_time_at >= last_full_period_ends_at {
                        break;
                    }

                    let config = config_history.get(config_cursor)
                        .expect("Config cursor, tail and head indexes are correct");

                    // How much of the reward period this config was active
                    let period_part: f64 = if let Some(config_ended_at) = config.ended_at {
                        if config_ended_at >= current_period_ends_at {
                            // The config lasted till the end of the period
                            let res = (current_period_ends_at - processing_time_at) as f64 / reward_period as f64;
                            processing_time_at = current_period_ends_at;
                            current_period_ends_at += reward_period;
                            config_cursor += 1; // Yes, this line differs this code with the simmilar code below

                            res
                        } else {
                            // The config ends before current period
                            let res = (config_ended_at - processing_time_at) as f64 / reward_period as f64;
                            processing_time_at = config_ended_at;
                            config_cursor += 1;

                           res
                        }
                    } else {
                        // The config is not ended so it lasted the whole reward period
                        let res = (current_period_ends_at - processing_time_at) as f64 / reward_period as f64;
                        processing_time_at = current_period_ends_at;
                        current_period_ends_at += reward_period;
                        
                        res
                    };

                    let reward_rate: f64 = (config.reward_metadata as f64) / 100.0;

                    reward_coeff += reward_rate * period_part;
                }

                let reward_tokens_amount = (staked_by_user as f64 * reward_coeff).floor() as u64;

                return (reward_tokens_amount, config_cursor);
            },
            Reward::Unfixed => {
                unimplemented!();
            },
        }
    }
}

impl TryFrom<u8> for Reward {
    type Error = Error;

    fn try_from(orig: u8) -> Result<Self, Self::Error> {
        match orig {
            0 => Ok(Reward::Fixed),
            1 => Ok(Reward::Unfixed),
            _ => err!(SPError::RewardTypeMismatch),
        }
    }
}

impl Into<u8> for Reward {
    fn into(self: Reward) -> u8 {
        match self {
            Reward::Fixed => 0,
            Reward::Unfixed => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reward::Reward;
    use anchor_lang::prelude::{Pubkey};
    use crate::account::{StakePoolConfig};

    #[test]
    fn one_fixed_config() {
        let reward = Reward::Fixed;
        let config_history_length: usize = 10;
        let staked_at: u64 = 1650106095;

        let reward_period = 500; // secs
        let staked_by_user = 100; // tokens
        let current_timestamp = staked_at + 1100; // Little bit more than two reward periods

        let mut config_history = ConfigHistory {
            head: 0,
            tail: 0,
            history: vec![None; config_history_length],
        };

        let config = StakePoolConfig {
            started_at: staked_at - 1000, // started some time before the stake happend
            ended_at: None, // Not ended
            reward_metadata: 10, // % reward rate
            total_staked_tokens: 1000, // it does not matter for the test
            unstake_delay: 100, // it does not matter for the test
            unstake_forse_fee_percent: 10, // it does not matter for the test
            reward_type: Reward::Fixed.into()
        };

        config_history.append(config);

        assert_eq!(config_history.head(), 1);
        assert_eq!(config_history.tail(), 0);

        let stakeholder = Stakeholder {
            owner: Pubkey::default(), // it does not matter for the test
            vault: Pubkey::default(), // it does not matter for the test
            bump: u8::default(), // it does not matter for the test
            staked_at,
            config_cursor: 0, // we start from the beggining for the config history
        };

        let (reward, cursor) = reward
            .calculate(
                reward_period, 
                staked_by_user, 
                current_timestamp,
                &config_history, 
                &stakeholder,
            );

        assert_eq!(reward, 20);
        assert_eq!(cursor, 0);
    }

    #[test]
    fn two_fixed_configs() {
        let reward = Reward::Fixed;
        let config_history_length: usize = 10;
        let staked_at: u64 = 1650106095;

        let reward_period = 500; // secs
        let staked_by_user = 100; // tokens
        let current_timestamp = staked_at + 1100; // Little bit more than two reward periods

        let mut config_history = ConfigHistory {
            head: 0,
            tail: 0,
            history: vec![None; config_history_length],
        };

        let config1_ended_at = staked_at + 250; 

        let config1 = StakePoolConfig {
            started_at: staked_at - 1000, // started some time before the stake happend
            ended_at: Some(config1_ended_at), // Ended on the half way of the first period
            reward_metadata: 10, // % reward rate
            total_staked_tokens: 1000, // it does not matter for the test
            unstake_delay: 100, // it does not matter for the test
            unstake_forse_fee_percent: 10, // it does not matter for the test
            reward_type: Reward::Fixed.into()
        };

        let config2 = StakePoolConfig {
            started_at: config1_ended_at, // started when the first config ended
            ended_at: None, // not ended
            reward_metadata: 5, // % reward rate now is LESS
            total_staked_tokens: 1000, // it does not matter for the test
            unstake_delay: 100, // it does not matter for the test
            unstake_forse_fee_percent: 10, // it does not matter for the test
            reward_type: Reward::Fixed.into()
        };

        config_history.append(config1);
        config_history.append(config2);

        assert_eq!(config_history.head(), 2);
        assert_eq!(config_history.tail(), 0);

        let stakeholder = Stakeholder {
            owner: Pubkey::default(), // it does not matter for the test
            vault: Pubkey::default(), // it does not matter for the test
            bump: u8::default(), // it does not matter for the test
            staked_at,
            config_cursor: 0, // we start from the beggining of the config history
        };

        let (reward, cursor) = reward
            .calculate(
                reward_period, 
                staked_by_user, 
                current_timestamp,
                &config_history, 
                &stakeholder,
            );

        assert_eq!(reward, 12);
        assert_eq!(cursor, 1);
    }

    #[test]
    fn zero_rewards() {
        let reward = Reward::Fixed;
        let config_history_length: usize = 10;
        let staked_at: u64 = 1650106095;

        let reward_period = 500; // secs
        let staked_by_user = 100; // tokens
        let current_timestamp = staked_at + 400; // Little bit less than the reward period

        let mut config_history = ConfigHistory {
            head: 0,
            tail: 0,
            history: vec![None; config_history_length],
        };

        let config = StakePoolConfig {
            started_at: staked_at - 1000, // started some time before the stake happend
            ended_at: None, // Not ended
            reward_metadata: 10, // % reward rate
            total_staked_tokens: 1000, // it does not matter for the test
            unstake_delay: 100, // it does not matter for the test
            unstake_forse_fee_percent: 10, // it does not matter for the test
            reward_type: Reward::Fixed.into()
        };

        config_history.append(config);

        assert_eq!(config_history.head(), 1);
        assert_eq!(config_history.tail(), 0);

        let stakeholder = Stakeholder {
            owner: Pubkey::default(), // it does not matter for the test
            vault: Pubkey::default(), // it does not matter for the test
            bump: u8::default(), // it does not matter for the test
            staked_at,
            config_cursor: 0, // we start from the beggining for the config history
        };

        let (reward, cursor) = reward
            .calculate(
                reward_period, 
                staked_by_user, 
                current_timestamp,
                &config_history, 
                &stakeholder,
            );

        assert_eq!(reward, 0);
        assert_eq!(cursor, 0);
    }

    // TODO add more tests and rewrite Reward::Fixed::calculate ;)
}
