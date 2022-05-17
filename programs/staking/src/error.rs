use anchor_lang::prelude::*;

#[error_code]
pub enum SPError {
    #[msg("A new instance of the stake pool can be created only by the owner of the program")]
    NewPoolOwnerMismatch,
    #[msg("The reward type is unknown. Please check the RewardType.")]
    RewardTypeMismatch,
    #[msg("The stake pool PDA account is invalid.")]
    StakePoolPDAInvalid,
    #[msg("Not enough tokens for the deposition. Please add more tokens to your account.")]
    InsufficientAmountOfTokensToDeposit,
    #[msg("Current stake is zero tokens")]
    UserStakeZero,
    #[msg("You should wait at least full reward period to claim reward")]
    RewardPeriodNotPassed,
    #[msg("Not enough reward tokens in the staking factory")]
    InsufficientAmountOfTokensToClaim,
    #[msg("Reward rate should be greater than 1")]
    RewardRateTooSmall,
    #[msg("Reward rate should be lower than 100")]
    RewardRateTooHigh,
    #[msg("No reward tokens available for sharing between stakers")]
    TokensToShareEmpty,
    #[msg("No staked tokens")]
    NoStakedTokens,
    #[msg("Unstake delay hasn't passed yet")]
    NotAllowedFinishUnstakeYet,
    #[msg("Owner interest should be between 1% and 99%")]
    OwnerInterestWrong,
    #[msg("Before staking you should add enough tokens to the free_vault")]
    NotEnoughFreeVaultAmount,
}
