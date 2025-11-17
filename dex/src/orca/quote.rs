#![allow(suspicious_double_ref_op)]

use orca_whirlpools_client::{Tick, TickArray};
use orca_whirlpools_core::{
    get_tick_array_start_tick_index, sqrt_price_to_tick_index, tick_index_to_sqrt_price, CoreError,
    ARITHMETIC_OVERFLOW, INVALID_SQRT_PRICE_LIMIT_DIRECTION, INVALID_TICK_INDEX, MAX_SQRT_PRICE, MAX_TICK_INDEX,
    MIN_SQRT_PRICE, MIN_TICK_INDEX, SQRT_PRICE_LIMIT_OUT_OF_BOUNDS, TICK_ARRAY_SIZE, ZERO_TRADABLE_AMOUNT,
};
use orca_whirlpools_core::{
    try_apply_swap_fee, try_get_amount_delta_a, try_get_amount_delta_b, try_get_next_sqrt_price_from_a,
    try_get_next_sqrt_price_from_b, try_reverse_apply_swap_fee, AMOUNT_EXCEEDS_MAX_U64,
};
use utils::quote::QuoteResult;

use super::accounts::{
    get_next_init_tick_index, get_tick, is_max_tick_array, is_min_tick_array, OrcaTick, OrcaTickArray, OrcaWhirlpool,
};

// https://github.com/orca-so/whirlpools/blob/5278dd46f4648e4041e05b48301ff0fd5747f592/rust-sdk/core/src/quote/swap.rs#L198
pub fn compute_swap_v2(
    token_amount: u64,
    sqrt_price_limit: u128,
    whirlpool: &OrcaWhirlpool,
    tick_array: &[&OrcaTickArray],
    a_to_b: bool,
    specified_input: bool,
) -> Result<QuoteResult, CoreError> {
    //let mut tick_sequence = VecDeque::from_iter(tick_array.iter().map(|tick_array| *tick_array));
    //let mut tick_sequence = VecDeque::from_iter(tick_array.iter().copied());

    let start_index = get_tick_array_start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing);

    // special case
    if let Some(ta) = tick_array.first() {
        if ta.start_tick_index != start_index {
            return Err(INVALID_TICK_INDEX);
        }
    }

    let sqrt_price_limit = if sqrt_price_limit == 0 {
        if a_to_b {
            MIN_SQRT_PRICE
        } else {
            MAX_SQRT_PRICE
        }
    } else {
        sqrt_price_limit
    };

    if !(MIN_SQRT_PRICE..=MAX_SQRT_PRICE).contains(&sqrt_price_limit) {
        return Err(SQRT_PRICE_LIMIT_OUT_OF_BOUNDS);
    }

    if a_to_b && sqrt_price_limit > whirlpool.sqrt_price || !a_to_b && sqrt_price_limit < whirlpool.sqrt_price {
        return Err(INVALID_SQRT_PRICE_LIMIT_DIRECTION);
    }

    if token_amount == 0 {
        return Err(ZERO_TRADABLE_AMOUNT);
    }

    let mut amount_remaining = token_amount;
    let mut amount_calculated = 0u64;
    let mut current_sqrt_price = whirlpool.sqrt_price;
    let mut current_tick_index = whirlpool.tick_current_index;
    let mut current_liquidity = whirlpool.liquidity;
    //let mut trade_fee = 0u64;

    // we have case 3 or 5 - in the case of 5 check if all 5 accounts where used ...
    let mut qoute_result = QuoteResult {
        indices: Some(
            tick_array
                .iter()
                .map(|tick_array| tick_array.start_tick_index)
                .collect(),
        ),
        compute_units: 75_000,
        ..Default::default()
    };

    while amount_remaining > 0 && sqrt_price_limit != current_sqrt_price {
        let result = get_next_initialized_tick_index(
            tick_array,
            current_tick_index,
            whirlpool.tick_spacing,
            a_to_b,
            start_index,
        );
        if result.is_err() {
            break;
        }
        let (next_array_index, next_tick_index) = result.unwrap();

        let next_tick_sqrt_price: u128 = tick_index_to_sqrt_price(next_tick_index);
        let target_sqrt_price = if a_to_b {
            next_tick_sqrt_price.max(sqrt_price_limit)
        } else {
            next_tick_sqrt_price.min(sqrt_price_limit)
        };

        let step_quote = compute_swap_step(
            amount_remaining,
            whirlpool.fee_rate,
            current_liquidity,
            current_sqrt_price,
            target_sqrt_price,
            a_to_b,
            specified_input,
        )?;

        //trade_fee += step_quote.fee_amount;

        if specified_input {
            amount_remaining = amount_remaining
                .checked_sub(step_quote.amount_in)
                .ok_or(ARITHMETIC_OVERFLOW)?
                .checked_sub(step_quote.fee_amount)
                .ok_or(ARITHMETIC_OVERFLOW)?;
            amount_calculated = amount_calculated
                .checked_add(step_quote.amount_out)
                .ok_or(ARITHMETIC_OVERFLOW)?;
        } else {
            amount_remaining = amount_remaining
                .checked_sub(step_quote.amount_out)
                .ok_or(ARITHMETIC_OVERFLOW)?;
            amount_calculated = amount_calculated
                .checked_add(step_quote.amount_in)
                .ok_or(ARITHMETIC_OVERFLOW)?
                .checked_add(step_quote.fee_amount)
                .ok_or(ARITHMETIC_OVERFLOW)?;
        }

        if step_quote.next_sqrt_price == next_tick_sqrt_price {
            let next_tick = get_tick(tick_array, next_array_index, next_tick_index, whirlpool.tick_spacing);
            match next_tick {
                Ok(tick) => {
                    current_liquidity =
                        get_next_liquidity_v2(current_liquidity, Some(tick), a_to_b).ok_or(ARITHMETIC_OVERFLOW)?;
                    current_tick_index = if a_to_b { next_tick_index - 1 } else { next_tick_index }
                }
                Err(_) => {
                    break;
                }
            }
        } else if step_quote.next_sqrt_price != current_sqrt_price {
            current_tick_index = sqrt_price_to_tick_index(step_quote.next_sqrt_price);
        }

        current_sqrt_price = step_quote.next_sqrt_price;
        //
    }

    if a_to_b && specified_input {
        qoute_result.amount_a_in = token_amount;
        qoute_result.amount_a_out = amount_remaining;
        qoute_result.amount_b_out = amount_calculated;
    } else if a_to_b && !specified_input {
        qoute_result.amount_b_out = token_amount; // amount_remaining je 0
        qoute_result.amount_a_in = amount_calculated;
    } else if !a_to_b && specified_input {
        qoute_result.amount_b_in = token_amount;
        qoute_result.amount_b_out = amount_remaining;
        qoute_result.amount_a_out = amount_calculated;
    } else if !a_to_b && !specified_input {
        qoute_result.amount_a_out = token_amount; //  amount_remaining je 0
        qoute_result.amount_b_in = amount_calculated;
    }

    // qoute_result.indices = Some(indices);

    Ok(qoute_result)
}

fn get_next_liquidity_v2(current_liquidity: u128, next_tick: Option<&OrcaTick>, a_to_b: bool) -> Option<u128> {
    let liquidity_net = next_tick.map(|tick| tick.liquidity_net).unwrap_or(0);
    let liquidity_net_unsigned = liquidity_net.unsigned_abs();
    if a_to_b {
        if liquidity_net < 0 {
            current_liquidity.checked_add(liquidity_net_unsigned)
        } else {
            current_liquidity.checked_sub(liquidity_net_unsigned)
        }
    } else if liquidity_net < 0 {
        current_liquidity.checked_sub(liquidity_net_unsigned)
    } else {
        current_liquidity.checked_add(liquidity_net_unsigned)
    }
}

pub fn get_next_initializable_tick_index(tick_index: i32, tick_spacing: u16) -> i32 {
    let tick_spacing_i32 = tick_spacing as i32;
    let remainder = tick_index.rem_euclid(tick_spacing_i32);
    tick_index - remainder + tick_spacing_i32
}

// b_a

pub fn next_initialized_tick(tick_array: &TickArray, tick_index: i32, tick_spacing: u16) -> (Option<&Tick>, i32) {
    let array_start_index = get_tick_array_start_tick_index(tick_index, tick_spacing); //tick_array.start_tick_index;
    let array_end_index = tick_array.start_tick_index + (TICK_ARRAY_SIZE as i32 * tick_spacing as i32);
    if array_start_index == tick_array.start_tick_index {
        for (idx, tick) in tick_array.ticks.iter().enumerate() {
            let tick_index_from_array = tick_array.start_tick_index + (idx as i32 * tick_spacing as i32);
            // initialized??? - or without liquidity
            if tick_index_from_array >= tick_index && tick.initialized {
                return (Some(tick), tick_index_from_array);
            }
        }
    }
    (None, array_end_index)
}

// a_b
pub fn prev_initialized_tick(tick_array: &TickArray, tick_index: i32, tick_spacing: u16) -> (Option<&Tick>, i32) {
    let array_start_index = get_tick_array_start_tick_index(tick_index, tick_spacing); //tick_array.start_tick_index;
    let array_end_index = tick_array.start_tick_index + (TICK_ARRAY_SIZE as i32 * tick_spacing as i32);
    if array_start_index == tick_array.start_tick_index {
        for (idx, tick) in tick_array.ticks.iter().rev().enumerate() {
            let tick_index_from_array = array_end_index - (idx as i32 * tick_spacing as i32);
            // initialized ...?
            if tick_index_from_array <= tick_index && tick.initialized {
                return (Some(tick), tick_index_from_array);
            }
        }
    }
    (None, tick_array.start_tick_index)
}

//

pub fn get_next_initialized_tick_index(
    tick_arrays: &[&OrcaTickArray],
    tick_index: i32,
    tick_spacing: u16,
    a_to_b: bool,
    start_array_index: i32,
) -> anyhow::Result<(i32, i32)> {
    let ticks_in_array = (TICK_ARRAY_SIZE as i32) * tick_spacing as i32;
    let mut search_index = tick_index;
    let mut array_index = start_array_index;

    // Keep looping the arrays until an initialized tick index in the subsequent tick-arrays found.
    loop {
        // If we get to the end of the array sequence and next_index is still not found, throw error
        let next_array = match tick_arrays
            .iter()
            .find(|tick_array| tick_array.start_tick_index == array_index)
        {
            Some(array) => array,
            None => return Err(anyhow::format_err!("TickArraySequenceInvalidIndex")),
        };

        let next_index = get_next_init_tick_index(next_array, search_index, tick_spacing, a_to_b)?;

        match next_index {
            Some(next_index) => {
                return Ok((array_index, next_index));
            }
            None => {
                // If we are at the last valid tick array, return the min/max tick index
                if a_to_b && is_min_tick_array(next_array) {
                    return Ok((array_index, MIN_TICK_INDEX));
                } else if !a_to_b && is_max_tick_array(next_array, tick_spacing) {
                    return Ok((array_index, MAX_TICK_INDEX));
                }

                // If we are at the last tick array in the sequencer, return the last tick
                if array_index + 1 == (tick_arrays.len() as i32) {
                    if a_to_b {
                        return Ok((array_index, next_array.start_tick_index));
                    } else {
                        let last_tick = next_array.start_tick_index + ticks_in_array - 1;
                        return Ok((array_index, last_tick));
                    }
                }

                // No initialized index found. Move the search-index to the 1st search position
                // of the next array in sequence.
                search_index = if a_to_b {
                    next_array.start_tick_index - 1
                } else {
                    next_array.start_tick_index + ticks_in_array - 1
                };

                array_index += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapStepQuote {
    pub amount_in: u64,
    pub amount_out: u64,
    pub next_sqrt_price: u128,
    pub fee_amount: u64,
}

pub fn compute_swap_step(
    amount_remaining: u64,
    fee_rate: u16,
    current_liquidity: u128,
    current_sqrt_price: u128,
    target_sqrt_price: u128,
    a_to_b: bool,
    specified_input: bool,
) -> Result<SwapStepQuote, CoreError> {
    // Any error that is not AMOUNT_EXCEEDS_MAX_U64 is not recoverable
    let initial_amount_fixed_delta = try_get_amount_fixed_delta(
        current_sqrt_price,
        target_sqrt_price,
        current_liquidity,
        a_to_b,
        specified_input,
    );
    let is_initial_amount_fixed_overflow = initial_amount_fixed_delta == Err(AMOUNT_EXCEEDS_MAX_U64);

    let amount_calculated = if specified_input {
        try_apply_swap_fee(amount_remaining, fee_rate)?
    } else {
        amount_remaining
    };

    let next_sqrt_price = if !is_initial_amount_fixed_overflow && initial_amount_fixed_delta? <= amount_calculated {
        target_sqrt_price
    } else {
        try_get_next_sqrt_price(
            current_sqrt_price,
            current_liquidity,
            amount_calculated,
            a_to_b,
            specified_input,
        )?
    };

    let is_max_swap = next_sqrt_price == target_sqrt_price;

    let amount_unfixed_delta = try_get_amount_unfixed_delta(
        current_sqrt_price,
        next_sqrt_price,
        current_liquidity,
        a_to_b,
        specified_input,
    )?;

    // If the swap is not at the max, we need to readjust the amount of the fixed token we are using
    let amount_fixed_delta = if !is_max_swap || is_initial_amount_fixed_overflow {
        try_get_amount_fixed_delta(
            current_sqrt_price,
            next_sqrt_price,
            current_liquidity,
            a_to_b,
            specified_input,
        )?
    } else {
        initial_amount_fixed_delta?
    };

    let (amount_in, mut amount_out) = if specified_input {
        (amount_fixed_delta, amount_unfixed_delta)
    } else {
        (amount_unfixed_delta, amount_fixed_delta)
    };

    // Cap output amount if using output
    if !specified_input && amount_out > amount_remaining {
        amount_out = amount_remaining;
    }

    let fee_amount = if specified_input && !is_max_swap {
        amount_remaining - amount_in
    } else {
        let pre_fee_amount = try_reverse_apply_swap_fee(amount_in, fee_rate)?;
        pre_fee_amount - amount_in
    };

    Ok(SwapStepQuote {
        amount_in,
        amount_out,
        next_sqrt_price,
        fee_amount,
    })
}

fn try_get_amount_fixed_delta(
    current_sqrt_price: u128,
    target_sqrt_price: u128,
    current_liquidity: u128,
    a_to_b: bool,
    specified_input: bool,
) -> Result<u64, CoreError> {
    if a_to_b == specified_input {
        try_get_amount_delta_a(
            current_sqrt_price,
            target_sqrt_price,
            current_liquidity,
            specified_input,
        )
    } else {
        try_get_amount_delta_b(
            current_sqrt_price,
            target_sqrt_price,
            current_liquidity,
            specified_input,
        )
    }
}

fn try_get_amount_unfixed_delta(
    current_sqrt_price: u128,
    target_sqrt_price: u128,
    current_liquidity: u128,
    a_to_b: bool,
    specified_input: bool,
) -> Result<u64, CoreError> {
    if specified_input == a_to_b {
        try_get_amount_delta_b(
            current_sqrt_price,
            target_sqrt_price,
            current_liquidity,
            !specified_input,
        )
    } else {
        try_get_amount_delta_a(
            current_sqrt_price,
            target_sqrt_price,
            current_liquidity,
            !specified_input,
        )
    }
}

fn try_get_next_sqrt_price(
    current_sqrt_price: u128,
    current_liquidity: u128,
    amount_calculated: u64,
    a_to_b: bool,
    specified_input: bool,
) -> Result<u128, CoreError> {
    if specified_input == a_to_b {
        try_get_next_sqrt_price_from_a(
            current_sqrt_price,
            current_liquidity,
            amount_calculated,
            specified_input,
        )
        // .map(|x| x.into())
    } else {
        try_get_next_sqrt_price_from_b(
            current_sqrt_price,
            current_liquidity,
            amount_calculated,
            specified_input,
        )
        // .map(|x| x)
    }
}
