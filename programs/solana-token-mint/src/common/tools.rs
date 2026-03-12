use crate::{
    config, Configuration, CustomError, StakingAccount, APY_PRECISION, PER_SHARE_PRECISION,
    SECONDS_PER_YEAR,
};
use anchor_lang::prelude::*;

pub fn update_pool(config: &mut Account<Configuration>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_elapsed = current_time - config.last_update_time;

    msg!(
        "stake amount in update pool : {}",
        config.total_staked_amount
    );
    if config.total_staked_amount == 0 {
        msg!("stake amount is zero or no time elapsed");
        config.last_update_time = current_time;
        return Ok(());
    }

    // Calculate how many reward tokens the pool has accumulated during this period
    let total_staked = config.total_staked_amount;
    let apy: u128 = config.apy.into();

    // Handle time rollback: if current time is less than last update time, a time rollback occurred
    // In this case, reset the timestamp and skip this reward calculation to prevent abnormal rewards
    if time_elapsed <= 0 {
        if time_elapsed < 0 {
            msg!(
                "WARNING: Time rollback detected! last_update_time: {}, current_time: {}",
                config.last_update_time,
                current_time
            );
        }
        config.last_update_time = current_time;
        return Ok(());
    }

    // Safe conversion: ensure time_elapsed is positive before converting to u64
    let time: u128 = u64::try_from(time_elapsed)
        .map_err(|_| CustomError::ArithmeticOverflow)?
        .into();

    let reward_gained = calc_reward(total_staked, apy, time)?;
    msg!("calc reward gained in update pool: {}", reward_gained);

    let grps_add = reward_gained
        .checked_mul(PER_SHARE_PRECISION)
        .ok_or(CustomError::ArithmeticOverflow)?
        .checked_div(total_staked)
        .ok_or(CustomError::ArithmeticOverflow)?;
    msg!("calc grps add in update pool: {}", grps_add);

    config.global_reward_per_share = config
        .global_reward_per_share
        .checked_add(grps_add)
        .ok_or(CustomError::ArithmeticOverflow)?;
    msg!(
        "new grps in update pool: {}",
        config.global_reward_per_share
    );

    config.last_update_time = current_time;
    Ok(())
}

fn calc_reward(principal: u128, apy: u128, time_for_stake: u128) -> Result<u128> {
    let numerator: u128 = principal
        .checked_mul(apy)
        .ok_or(CustomError::ArithmeticOverflow)?
        .checked_mul(time_for_stake)
        .ok_or(CustomError::ArithmeticOverflow)?;
    let denominator: u128 = (SECONDS_PER_YEAR as u128)
        .checked_mul(APY_PRECISION)
        .ok_or(CustomError::ArithmeticOverflow)?;
    let reward: u128 = numerator
        .checked_div(denominator)
        .ok_or(CustomError::ArithmeticOverflow)?;
    Ok(reward)
}

pub fn calc_total_reward(
    config: &mut Account<Configuration>,
    stake_acc: &mut Account<StakingAccount>,
) -> Result<u128> {
    let reward_accrued_index = config
        .global_reward_per_share
        .checked_sub(stake_acc.user_reward_debt)
        .ok_or(CustomError::ArithmeticOverflow)?;

    let reward_accrued: u128 = (reward_accrued_index
        .checked_mul(stake_acc.amount)
        .ok_or(CustomError::ArithmeticOverflow)?)
    .checked_div(config::PER_SHARE_PRECISION)
    .ok_or(CustomError::ArithmeticOverflow)?;

    let total_rewards_to_mint = reward_accrued
        .checked_add(stake_acc.claimable_reward)
        .ok_or(CustomError::ArithmeticOverflow)?;

    Ok(total_rewards_to_mint)
}
