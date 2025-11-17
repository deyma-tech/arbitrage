use decimal::*;

use crate::{err, from_result, ok_or_mark_trace};
use std::{cell::RefMut, convert::TryInto};

use super::{
    decimals::*,
    structs::{get_search_limit, Pool, Tick, Tickmap, MAX_TICK, TICK_LIMIT},
    //utils::{TrackableError, TrackableResult},
};

use anyhow::Result;

#[derive(PartialEq, Debug)]
pub struct SwapResult {
    pub next_price_sqrt: Price,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee_amount: TokenAmount,
}

// converts ticks to price with reduced precision
pub fn calculate_price_sqrt(tick_index: i32) -> Price {
    // checking if tick be converted to price (overflows if more)
    let tick = tick_index.abs();
    assert!(tick <= MAX_TICK, "tick over bounds");

    let mut price = FixedPoint::from_integer(1);

    if tick & 0x1 != 0 {
        price *= FixedPoint::new(1000049998750);
    }
    if tick & 0x2 != 0 {
        price *= FixedPoint::new(1000100000000);
    }
    if tick & 0x4 != 0 {
        price *= FixedPoint::new(1000200010000);
    }
    if tick & 0x8 != 0 {
        price *= FixedPoint::new(1000400060004);
    }
    if tick & 0x10 != 0 {
        price *= FixedPoint::new(1000800280056);
    }
    if tick & 0x20 != 0 {
        price *= FixedPoint::new(1001601200560);
    }
    if tick & 0x40 != 0 {
        price *= FixedPoint::new(1003204964963);
    }
    if tick & 0x80 != 0 {
        price *= FixedPoint::new(1006420201726);
    }
    if tick & 0x100 != 0 {
        price *= FixedPoint::new(1012881622442);
    }
    if tick & 0x200 != 0 {
        price *= FixedPoint::new(1025929181080);
    }
    if tick & 0x400 != 0 {
        price *= FixedPoint::new(1052530684591);
    }
    if tick & 0x800 != 0 {
        price *= FixedPoint::new(1107820842005);
    }
    if tick & 0x1000 != 0 {
        price *= FixedPoint::new(1227267017980);
    }
    if tick & 0x2000 != 0 {
        price *= FixedPoint::new(1506184333421);
    }
    if tick & 0x4000 != 0 {
        price *= FixedPoint::new(2268591246242);
    }
    if tick & 0x8000 != 0 {
        price *= FixedPoint::new(5146506242525);
    }
    if tick & 0x0001_0000 != 0 {
        price *= FixedPoint::new(26486526504348);
    }
    if tick & 0x0002_0000 != 0 {
        price *= FixedPoint::new(701536086265529);
    }

    // Parsing to the Price type by the end by convention (should always have 12 zeros at the end)
    if tick_index >= 0 {
        Price::from_decimal(price)
    } else {
        Price::from_decimal(FixedPoint::from_integer(1).big_div(price))
    }
}

// Finds closes initialized tick in direction of trade
// and compares its price to the price limit of the trade
pub fn get_closer_limit(
    sqrt_price_limit: Price,
    x_to_y: bool,
    current_tick: i32, // tick already scaled by tick_spacing
    tick_spacing: u16,
    tickmap: &Tickmap,
) -> Result<(Price, Option<(i32, bool)>)> {
    // find initalized tick (None also for virtual tick limiated by search scope)
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
                Err(anyhow::format_err!("InvariantErrorCode::LimitReached"))?;
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

pub fn compute_swap_step(
    current_price_sqrt: Price,
    target_price_sqrt: Price,
    liquidity: Liquidity, // pool.liquidity
    amount: TokenAmount,  // reaming_amount (input or output depending on by_amount_in)
    by_amount_in: bool,
    fee: FixedPoint, // pool.fee
                     //) -> TrackableResult<SwapResult> {
) -> Result<SwapResult> {
    if liquidity.is_zero() {
        return Ok(SwapResult {
            next_price_sqrt: target_price_sqrt,
            amount_in: TokenAmount(0),
            amount_out: TokenAmount(0),
            fee_amount: TokenAmount(0),
        });
    }

    let x_to_y = current_price_sqrt >= target_price_sqrt;

    let next_price_sqrt;
    let mut amount_in = TokenAmount(0);
    let mut amount_out = TokenAmount(0);

    if by_amount_in {
        // take fee in input_amount
        // U256(2^64) * U256(1e12) - no overflow in intermediate operations
        // no overflow in token_amount result
        let amount_after_fee = amount.big_mul(
            FixedPoint::from_integer(1u8)
                .checked_sub(fee)
                .map_err(|_| err!("sub underflow"))?,
        );

        amount_in = if x_to_y {
            get_delta_x(target_price_sqrt, current_price_sqrt, liquidity, true)
        } else {
            get_delta_y(current_price_sqrt, target_price_sqrt, liquidity, true)
        }
        .unwrap_or(TokenAmount(u64::MAX));

        // if target price was hit it will be the next price
        if amount_after_fee >= amount_in {
            next_price_sqrt = target_price_sqrt
        } else {
            // DOMAIN:
            // liquidity = U128::MAX
            // amount_after_fee = U64::MAX
            // current_price_sqrt = entire price space
            next_price_sqrt = ok_or_mark_trace!(get_next_sqrt_price_from_input(
                current_price_sqrt,
                liquidity,
                amount_after_fee,
                x_to_y,
            ))?;
        };
    } else {
        amount_out = if x_to_y {
            get_delta_y(target_price_sqrt, current_price_sqrt, liquidity, false)
        } else {
            get_delta_x(current_price_sqrt, target_price_sqrt, liquidity, false)
        }
        .unwrap_or(TokenAmount(u64::MAX));

        if amount >= amount_out {
            next_price_sqrt = target_price_sqrt
        } else {
            next_price_sqrt = ok_or_mark_trace!(get_next_sqrt_price_from_output(
                current_price_sqrt,
                liquidity,
                amount,
                x_to_y
            ))?;
        }
    }

    let not_max = target_price_sqrt != next_price_sqrt;

    if x_to_y {
        if not_max || !by_amount_in {
            amount_in = get_delta_x(next_price_sqrt, current_price_sqrt, liquidity, true)
                .ok_or_else(|| err!("get_delta_x overflow"))?;
        };
        if not_max || by_amount_in {
            amount_out = get_delta_y(next_price_sqrt, current_price_sqrt, liquidity, false)
                .ok_or_else(|| err!("get_delta_y overflow"))?;
        }
    } else {
        if not_max || !by_amount_in {
            amount_in = get_delta_y(current_price_sqrt, next_price_sqrt, liquidity, true)
                .ok_or_else(|| err!("get_delta_y overflow"))?;
        };
        if not_max || by_amount_in {
            amount_out = get_delta_x(current_price_sqrt, next_price_sqrt, liquidity, false)
                .ok_or_else(|| err!("get_delta_x overflow"))?;
        };
    }

    // Amount out can not exceed amount
    if !by_amount_in && amount_out > amount {
        amount_out = amount;
    }

    let fee_amount = if by_amount_in && next_price_sqrt != target_price_sqrt {
        // no possible to overflow in intermediate operations
        // edge case occurs when the next_price is target_price (minimal distance to target)
        amount
            .checked_sub(amount_in)
            .map_err(|_| err!("sub underflow"))?
    } else {
        // no possible to overflow in intermediate operations
        // edge case when amount_in is maximum and fee is maximum
        amount_in.big_mul_up(fee)
    };

    Ok(SwapResult {
        next_price_sqrt,
        amount_in,
        amount_out,
        fee_amount,
    })
}

// delta x = (L * delta_sqrt_price) / (lower_sqrt_price * higher_sqrt_price)
pub fn get_delta_x(
    sqrt_price_a: Price,
    sqrt_price_b: Price,
    liquidity: Liquidity,
    up: bool,
) -> Option<TokenAmount> {
    let delta_price = if sqrt_price_a > sqrt_price_b {
        sqrt_price_a - sqrt_price_b
    } else {
        sqrt_price_b - sqrt_price_a
    };

    let nominator = delta_price.big_mul_to_value(liquidity);
    match up {
        true => Price::big_div_values_to_token_up(
            nominator,
            sqrt_price_a.big_mul_to_value(sqrt_price_b),
        ),
        false => Price::big_div_values_to_token(
            nominator,
            sqrt_price_a.big_mul_to_value_up(sqrt_price_b),
        ),
    }
}

// delta y = L * delta_sqrt_price
pub fn get_delta_y(
    sqrt_price_a: Price,
    sqrt_price_b: Price,
    liquidity: Liquidity,
    up: bool,
) -> Option<TokenAmount> {
    let delta_price = if sqrt_price_a > sqrt_price_b {
        sqrt_price_a - sqrt_price_b
    } else {
        sqrt_price_b - sqrt_price_a
    };

    match match up {
        true => delta_price
            .big_mul_to_value_up(liquidity)
            .checked_add(Price::almost_one())
            .unwrap()
            .checked_div(Price::one())
            .unwrap()
            .try_into(),
        false => delta_price
            .big_mul_to_value(liquidity)
            .checked_div(Price::one())
            .unwrap()
            .try_into(),
    } {
        Ok(x) => Some(TokenAmount(x)),
        Err(_) => None,
    }
}

fn get_next_sqrt_price_from_input(
    price_sqrt: Price,
    liquidity: Liquidity,
    amount: TokenAmount,
    x_to_y: bool,
    //) -> TrackableResult<Price> {
) -> Result<Price> {
    if liquidity.is_zero() {
        return Err(err!("getting next price from input with zero liquidity"));
    }
    if price_sqrt.is_zero() {
        return Err(err!("getting next price from input with zero price"));
    }
    // DOMAIN:
    // price_sqrt <sqrt_price_at_min_tick, sqrt_price_at_max_tick>
    // pool.liquidity <1, u128::MAX>
    // amount <1, u64::MAX>

    let result = if x_to_y {
        // checked
        get_next_sqrt_price_x_up(price_sqrt, liquidity, amount, true)
    } else {
        // checked
        get_next_sqrt_price_y_down(price_sqrt, liquidity, amount, true)
    };
    ok_or_mark_trace!(result)
}

fn get_next_sqrt_price_from_output(
    price_sqrt: Price,
    liquidity: Liquidity,
    amount: TokenAmount,
    x_to_y: bool,
) -> Result<Price> {
    // DOMAIN:
    // price_sqrt <sqrt_price_at_min_tick, sqrt_price_at_max_tick>
    // pool.liquidity <1, u128::MAX>
    // amount <1, u64::MAX>

    if liquidity.is_zero() {
        return Err(err!("getting next price from output with zero liquidity"));
    }
    if price_sqrt.is_zero() {
        return Err(err!("getting next price from output with zero price"));
    }

    let result = if x_to_y {
        get_next_sqrt_price_y_down(price_sqrt, liquidity, amount, false)
    } else {
        get_next_sqrt_price_x_up(price_sqrt, liquidity, amount, false)
    };
    ok_or_mark_trace!(result)
}

// L * price / (L +- amount * price)
fn get_next_sqrt_price_x_up(
    price_sqrt: Price,
    liquidity: Liquidity,
    amount: TokenAmount,
    add: bool,
) -> Result<Price> {
    // DOMAIN:
    // In case add always true
    // pool.liquidity = U128::MAX
    // amount = U64::MAX
    // price_sqrt = entire price space

    if amount.is_zero() {
        return Ok(price_sqrt);
    };

    // PRICE_LIQUIDITY_DENOMINATOR = 10 ^ (24 - 6)
    // max_big_liquidity -> ceil(log2(2^128 * 10^18)) = 188
    // no possibility of overflow here
    let big_liquidity = liquidity
        .here::<U256>()
        .checked_mul(U256::from(PRICE_LIQUIDITY_DENOMINATOR)) // extends liquidity precision (operation on U256, so there is no dividing by denominator)
        .ok_or_else(|| err!("mul overflow"))?;

    // max(price * amount)
    // ceil(log2(max_price * 2^64))= 160
    // U256::from(max_price) * U256::from(2^64) / U256::(1)
    // so not possible to overflow here
    let denominator = from_result!(match add {
        // max_denominator = L + amount * price [maximize all parameters]
        // max_denominator 2^128 + 2^64 * 2^96 = 2^161 <- no possible to overflow
        true => big_liquidity.checked_add(price_sqrt.big_mul_to_value(amount)),
        false => big_liquidity.checked_sub(price_sqrt.big_mul_to_value(amount)),
    }
    .ok_or_else(|| "big_liquidity -/+ price_sqrt * amount"))?; // never should be triggered

    // max_nominator = (U256::from(max_price) * U256::from(max_liquidity) + 10^6) / 10^6
    // max_nominator = (2^96 * 2^128 + 10^6) / 10^6
    // ceil(log2(2^96 * 2^128 + 10^6)) = 225
    // ceil(log2((2^96 * 2^128 + 10^6)/10^6)) = 205
    // ceil(lg2(max_nominator)) = 205
    // no possibility of overflowing in the result or in intermediate calculations

    // result = div_up(nominator, denominator) -> so maximizing nominator while minimizing denominator
    // max_results = (max_nominator * Price::one + min_denominator) / min_denominator
    // (2^205 * 10^24 + 1) / 1 = 2^285 <- possible to overflow in result

    // maximize nominator -> (max_nominator * Price::one + max_denominator)
    // 2^205 * 10^24 + 2^161 = 2^285 <- possible to overflow in intermediate operations
    ok_or_mark_trace!(Price::checked_big_div_values_up(
        price_sqrt.big_mul_to_value_up(liquidity),
        denominator
    ))
}

// price +- (amount / L)
fn get_next_sqrt_price_y_down(
    price_sqrt: Price,
    liquidity: Liquidity,
    amount: TokenAmount,
    add: bool,
) -> Result<Price> {
    // DOMAIN:
    // price_sqrt <sqrt_price_at_min_tick, sqrt_price_at_max_tick>
    // pool.liquidity <1, u128::MAX> (zero liquidity not possible)
    // amount <1, u64::MAX>

    // quotient= amount / L
    // PRICE_LIQUIDITY_DENOMINATOR = 10 ^ (24 - 6)

    if add {
        // Price::from_scale(amount, TokenAmount::scale())
        // max_nominator = max_amount * 10^24 => 2^144 so possible to overflow here

        // max_denominator = max_liquidity
        // max_denominator = U256(u128::MAX) * U256(10^18)
        // max_denominator = U256(2^128 * 10^18) ~ 2^188 so no possible to overflow

        // quotient - max quotient nominator
        // quotient_max_nominator = U256(max_nominator) * U256(10^24)
        // quotient_max_nominator = 2^128 * 10^24 ~ 2^208 so no possible to overflow in intermediate operations

        // max_quotient = max_nominator / min_denominator
        // max_quotient = 2^128 * 10^24 / 10^18 ~ 2^148 so possible to overflow in max_quote
        let quotient = from_result!(Price::checked_from_decimal(amount)
            .map_err(|err| err!(&err))? // TODO: add util macro to map str -> TrackableError
            .checked_big_div_by_number(
                U256::from(liquidity.get())
                    .checked_mul(U256::from(PRICE_LIQUIDITY_DENOMINATOR))
                    .ok_or_else(|| err!("mul overflow"))?,
            ))?;
        // max_quotient = 2^128
        // price_sqrt = 2^96
        // possible to overflow in result
        from_result!(price_sqrt.checked_add(quotient))
    } else {
        // Price::from_scale - same as case above
        let quotient = from_result!(Price::checked_from_decimal(amount)
            .map_err(|err| err!(&err))? // TODO: add util macro to map str -> TrackableError
            .checked_big_div_by_number_up(
                U256::from(liquidity.get())
                    .checked_mul(U256::from(PRICE_LIQUIDITY_DENOMINATOR))
                    .ok_or_else(|| err!("mul overflow"))?,
            ))?;
        from_result!(price_sqrt.checked_sub(quotient))
    }
}

pub fn is_enough_amount_to_push_price(
    amount: TokenAmount,
    current_price_sqrt: Price,
    liquidity: Liquidity,
    fee: FixedPoint,
    by_amount_in: bool,
    x_to_y: bool,
) -> Result<bool> {
    if liquidity.is_zero() {
        return Ok(true);
    }

    let next_price_sqrt = ok_or_mark_trace!(if by_amount_in {
        let amount_after_fee = amount.big_mul(
            FixedPoint::from_integer(1)
                .checked_sub(fee)
                .map_err(|_| err!("sub underflow"))?,
        );
        get_next_sqrt_price_from_input(current_price_sqrt, liquidity, amount_after_fee, x_to_y)
    } else {
        get_next_sqrt_price_from_output(current_price_sqrt, liquidity, amount, x_to_y)
    })?;

    Ok(current_price_sqrt.ne(&next_price_sqrt))
}

pub fn cross_tick(tick: &mut RefMut<Tick>, pool: &mut Pool) -> Result<()> {
    tick.fee_growth_outside_x = pool
        .fee_growth_global_x
        .unchecked_sub(tick.fee_growth_outside_x);
    tick.fee_growth_outside_y = pool
        .fee_growth_global_y
        .unchecked_sub(tick.fee_growth_outside_y);

    // When going to higher tick net_liquidity should be added and for going lower subtracted
    let new_liquidity = if (pool.current_tick_index >= tick.index) ^ tick.sign {
        pool.liquidity.checked_add(tick.liquidity_change)
    } else {
        pool.liquidity.checked_sub(tick.liquidity_change)
    };

    pool.liquidity = new_liquidity.map_err(|_| anyhow::format_err!("InvalidPoolLiquidity"))?;
    Ok(())
}

pub fn get_max_tick(tick_spacing: u16) -> Result<i32> {
    let limit_by_space = TICK_LIMIT
        .checked_sub(1)
        .ok_or_else(|| err!("sub underflow"))?
        .checked_mul(tick_spacing.into())
        .ok_or_else(|| err!("mul overflow"))?;
    Ok(limit_by_space.min(MAX_TICK))
}

pub fn get_min_tick(tick_spacing: u16) -> Result<i32> {
    let limit_by_space = (-TICK_LIMIT)
        .checked_add(1)
        .ok_or_else(|| err!("add overflow"))?
        .checked_mul(tick_spacing.into())
        .ok_or_else(|| err!("mul overflow"))?;
    Ok(limit_by_space.max(-MAX_TICK))
}

pub fn get_max_sqrt_price(tick_spacing: u16) -> Result<Price> {
    let max_tick = get_max_tick(tick_spacing);
    Ok(calculate_price_sqrt(max_tick?))
}

pub fn get_min_sqrt_price(tick_spacing: u16) -> Result<Price> {
    let min_tick = get_min_tick(tick_spacing);
    Ok(calculate_price_sqrt(min_tick?))
}
