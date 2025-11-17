use std::cell::RefMut;

use super::math::calculate_price_sqrt;
use super::structs::pool::PoolState;
use super::structs::tick::Tick;
use super::structs::tickmap::Tickmap;
use super::structs::tickmap::{get_search_limit, MAX_TICK, TICK_LIMIT};
use super::*;

use anyhow::Result;

pub fn check_ticks(tick_lower: i32, tick_upper: i32, tick_spacing: u16) -> Result<()> {
    // Check order
    if !(tick_lower < tick_upper) {
        return Err(anyhow::format_err!("InvalidTickIndex"));
    }

    check_tick(tick_lower, tick_spacing)?;
    check_tick(tick_upper, tick_spacing)?;

    Ok(())
}

pub fn check_tick(tick_index: i32, tick_spacing: u16) -> Result<()> {
    // Check order
    if !(tick_index.checked_rem(tick_spacing.into()) == Some(0)) {
        return Err(anyhow::format_err!("InvalidTickIndex"));
    }

    let tickmap_index = tick_index.checked_div(tick_spacing.into()).unwrap();

    if !(tickmap_index >= (-TICK_LIMIT)) {
        return Err(anyhow::format_err!("InvalidTickIndex"));
    }
    if !(tickmap_index < TICK_LIMIT) {
        return Err(anyhow::format_err!("InvalidTickIndex"));
    }
    if !(tick_index >= (-MAX_TICK)) {
        return Err(anyhow::format_err!("InvalidTickIndex"));
    }
    if !(tick_index <= MAX_TICK) {
        return Err(anyhow::format_err!("InvalidTickIndex"));
    }

    Ok(())
}

// Finds closes initialized tick in direction of trade
// and compares its price to the price limit of the trade
pub fn get_closer_limit(
    sqrt_price_limit: Price,
    x_to_y: bool,
    current_tick: i32,
    tick_spacing: u16,
    tickmap: &Tickmap,
) -> Result<(Price, Option<(i32, bool)>)> {
    let closes_tick_index = if x_to_y {
        tickmap.prev_initialized(current_tick, tick_spacing)
    } else {
        tickmap.next_initialized(current_tick, tick_spacing)
    };

    match closes_tick_index {
        Some(index) => {
            let price = calculate_price_sqrt(index);
            // trunk-ignore(clippy/if_same_then_else)
            if x_to_y && price > sqrt_price_limit {
                Ok((price, Some((index, true))))
            } else if !x_to_y && price < sqrt_price_limit {
                Ok((price, Some((index, true))))
            } else {
                Ok((sqrt_price_limit, None))
            }
        }
        None => {
            let index = get_search_limit(current_tick, tick_spacing, !x_to_y);
            let price = calculate_price_sqrt(index);

            if !(current_tick != index) {
                return Err(anyhow::format_err!("LimitReached"));
            }

            // trunk-ignore(clippy/if_same_then_else)
            if x_to_y && price > sqrt_price_limit {
                Ok((price, Some((index, false))))
            } else if !x_to_y && price < sqrt_price_limit {
                Ok((price, Some((index, false))))
            } else {
                Ok((sqrt_price_limit, None))
            }
        }
    }
}

pub fn cross_tick(tick: &mut RefMut<Tick>, pool: &mut PoolState, current_timestamp: u64) -> Result<()> {
    tick.fee_growth_outside_x = pool
        .fee_growth_global_x
        .unchecked_sub(tick.fee_growth_outside_x);
    tick.fee_growth_outside_y = pool
        .fee_growth_global_y
        .unchecked_sub(tick.fee_growth_outside_y);

    let seconds_passed: u64 = current_timestamp.checked_sub(pool.start_timestamp).unwrap();
    tick.seconds_outside = seconds_passed - tick.seconds_outside;

    if !pool.liquidity.is_zero() {
        pool.update_seconds_per_liquidity_global(current_timestamp);
    } else {
        pool.last_timestamp = current_timestamp;
    }
    tick.seconds_per_liquidity_outside = pool
        .seconds_per_liquidity_global
        .unchecked_sub(tick.seconds_per_liquidity_outside);

    // When going to higher tick net_liquidity should be added and for going lower subtracted
    if (pool.current_tick_index >= tick.index) ^ tick.sign {
        // trunk-ignore(clippy/assign_op_pattern)
        //pool.liquidity = pool.liquidity + tick.liquidity_change;
        //pool.liquidity += tick.liquidity_change;
        let new_liquidity = pool.liquidity + tick.liquidity_change;
        pool.liquidity = new_liquidity;
    } else {
        // trunk-ignore(clippy/assign_op_pattern)
        //pool.liquidity = pool.liquidity - tick.liquidity_change;
        //pool.liquidity -= tick.liquidity_change;
        let new_liquidity = pool.liquidity - tick.liquidity_change;
        pool.liquidity = new_liquidity;
    }

    Ok(())
}

// pub fn get_current_timestamp() -> u64 {
//     Clock::get().unwrap().unix_timestamp.try_into().unwrap()
// }

// pub fn get_current_slot() -> u64 {
//     Clock::get().unwrap().slot
// }

// pub fn close<'info>(
//     info: AccountInfo<'info>,
//     sol_destination: AccountInfo<'info>,
// ) -> ProgramResult {
//     // Transfer tokens from the account to the sol_destination.
//     let dest_starting_lamports = sol_destination.lamports();
//     **sol_destination.lamports.borrow_mut() =
//         dest_starting_lamports.checked_add(info.lamports()).unwrap();
//     **info.lamports.borrow_mut() = 0;

//     // Mark the account discriminator as closed.
//     let mut data = info.try_borrow_mut_data()?;
//     let dst: &mut [u8] = &mut data;
//     let mut cursor = std::io::Cursor::new(dst);
//     cursor
//         .write_all(&CLOSED_ACCOUNT_DISCRIMINATOR)
//         .map_err(|_| ErrorCode::AccountDidNotSerialize)?;
//     Ok(())
// }
