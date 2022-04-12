use std::convert::TryFrom;

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
    type Error = &'static str;

    fn try_from(orig: u8) -> Result<Self, Self::Error> {
        match orig {
            0 => return Ok(RewardType::Fixed),
            1 => return Ok(RewardType::Unfixed),
            _ => Err("This reward type is not supported"),
        }
    }
}
