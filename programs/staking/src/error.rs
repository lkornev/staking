use anchor_lang::prelude::*;

#[error_code]
pub enum SP {
    #[msg("A new instance of the stake pool can be created only by the owner of the program")]
    NewPoolOwnerMistmatch,
    #[msg("The reward type is unknown. Please check the RewardType.")]
    RewardTypeMismatch,
}
