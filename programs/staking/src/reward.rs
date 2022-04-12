use std::convert::TryFrom; 
use crate::SPError;
use anchor_lang::err;
use anchor_lang::prelude::Error;

#[derive(PartialEq, Eq)]
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

impl TryFrom<u8> for RewardType {
    type Error = Error;

    fn try_from(orig: u8) -> Result<Self, Self::Error> {
        match orig {
            0 => Ok(RewardType::Fixed),
            1 => Ok(RewardType::Unfixed),
            _ => err!(SPError::RewardTypeMismatch),
        }
    }
}
