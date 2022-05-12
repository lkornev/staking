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
    #[msg("No reward available")]
    NoRewardAvailable,
    #[msg("Not enough reward tokens in the staking factory")]
    InsufficientAmountOfTokensToClaim,
}
