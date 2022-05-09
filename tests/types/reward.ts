export const Reward = {
    /// A stakeholder will receive a fixed amount of reward tokens pro rata one's staked tokens.
    /// E.g. staked tokens: 300, `reward_per_token`: 5 %.
    /// Reward: 15 reward tokens per `reward_period`. (300 * 5 / 100)
    Fixed: {
        index: 0,
        value: { fixed: {} },
    },
    /// A stakeholder will receive a part of tokens (`reward_per_period`)
    /// in proportion to the tokens of the all stakeholders in the pool.
    /// E.g. the user's staked tokens 300, 
    /// total staked tokens by every stakeholder in the pool: 1000,
    /// `reward_tokens_per_period`: 500.
    /// Reward: 150 reward tokens per `reward_period`. ((300 / 1000) * 500)
    Unfixed: {
        index: 1,
        value: { unfixed: {} },
    },
}
