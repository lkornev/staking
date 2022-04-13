use anchor_lang::prelude::*;

#[error_code]
pub enum SPError {
    #[msg("A new instance of the stake pool can be created only by the owner of the program")]
    NewPoolOwnerMistmatch,
    #[msg("The reward type is unknown. Please check the RewardType.")]
    RewardTypeMismatch,
    #[msg("The stake pool PDA account is invalid.")]
    StakePoolPDAInvalid,
    // TODO use in the deposit instruction
    #[msg("Not enough tokens for the deposition. Please add more tokens to your account.")]
    InsufficientAmountOfTokensToDeposit,
}
