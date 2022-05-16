use anchor_lang::prelude::*;
use crate::context::*;
use crate::error::SPError;

pub fn allow_finish_unstake<'info>(ctx: &Context<FinishUnstakeAll>) -> Result<()> {
    let unstaked_at: u64 = ctx.accounts.member_pending_unstake.unstaked_at;
    let unstake_delay: u64 = ctx.accounts.stake_pool.unstake_delay;
    let current_time: u64 = ctx.accounts.clock.unix_timestamp as u64; 

    require!(current_time >= unstaked_at + unstake_delay, SPError::NotAllowedFinishUnstakeYet);

    Ok(())
}
