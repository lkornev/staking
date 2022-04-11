import { BN } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';

export interface StakePoolConfig {
    /// The time in seconds a stakeholder have to wait 
    /// to finish unstaking without paying the fee (`unstake_forse_fee_percent`).
    unstakeDelay: BN,
    /// If a user wants unstake without waiting `unstake_delay`
    /// the user can pay the `unstake_forse_fee_percent` and receives the amount of tokens equals to 
    /// staked_tokens - (staked_tokens * unstake_forse_fee_percent) / 100.
    /// Should be in the range 0 - 100. (TODO check range)
    unstakeForseFeePercent: number,
    /// The time in seconds a stakeholder have to wait to receive the next reward.
    /// After each `reward_period` the stakeholders are allowed to claim the reward.
    rewardPeriod: BN,
    /// Describes the type of the reward tokens.
    /// The mint itself does not need to be under control of the stake pool owner.
    /// It could be the wrapped Sol mint or any other spl token mint.
    rewardTokenMint: PublicKey,
    /// Describes the type of the tokens that are allowed to be staked.
    /// The mint itself does not need to be under control of the stake pool owner or a stakeholder.
    /// It could be the wrapped Sol mint or any other spl token mint.
    stakeTokenMint: PublicKey,
    // Could be Fixed (0) or Unfixed (1). See RewardType enum for more details.
    rewardType: number,
    /// If the `reward_type` is Fixed
    /// `reward_metadata` is the fixed percentage of the income.
    /// Should be greather than 0 and less or equal to 100. (TODO check the range only if the reward_type is Fixed).
    /// The reward is granted as a fixed percentage of the staked tokens.
    /// 
    /// If the `reward_type` is Unfixed
    /// `reward_metadata` is the amount of reward tokens that will be shared 
    /// in proportion to the user's staked tokens among all stakeholders.
    /// Should be greather than 0.
    rewardMetadata: BN,
}

/**
* A stakeholder will receive 10% (`rewardPerToken`) of the reward tokens 
* for each staked token every 30 seconds (`rewardPeriod`).
*
* If the user wants to unstake the tokens one should wait for 40 seconds (`unstakeDelay`)
* or lose 50% (`unstakeForseFeePercent`) of the reward tokens.
*/
export function newSPFixedConfig( 
    rewardTokenMint: PublicKey,
    stakeTokenMint: PublicKey,
): StakePoolConfig {
    return {
        unstakeDelay: new BN(40), // secs,
        unstakeForseFeePercent: 50, // %,
        rewardPeriod: new BN(30), // secs
        rewardTokenMint,
        stakeTokenMint,
        rewardType: 0, // Fixed reward.
        rewardMetadata: new BN(10), // %
    }
}
