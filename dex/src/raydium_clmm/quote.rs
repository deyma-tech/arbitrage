//#![allow(warnings)]
use super::constants::FEE_RATE_DENOMINATOR_VALUE;
use super::{big_num::U128, liquidity_math, swap_math, tick_math};
use super::{AmmConfig, PoolState, TickArrayBitmapExtension, TickArrayState, TickState};
use anyhow::{format_err, Context, Result};
use num_traits::SaturatingMul;
use std::collections::VecDeque;
use std::ops::Neg;
use utils::now;
use utils::quote::QuoteResult;

// the top level state of the swap, the results of which are recorded in storage at the end
#[derive(Debug)]
pub struct SwapState {
    // the amount remaining to be swapped in/out of the input/output asset
    pub amount_specified_remaining: u64,
    // the amount already swapped out/in of the output/input asset
    pub amount_calculated: u64,
    // current sqrt(price)
    pub sqrt_price_x64: u128,
    // the tick associated with the current price
    pub tick: i32,
    // the global fee growth of the input token
    pub fee_growth_global_x64: u128,
    // the global fee of the input token
    pub fee_amount: u64,
    // amount of input token paid as protocol fee
    pub protocol_fee: u64,
    // amount of input token paid as fund fee
    pub fund_fee: u64,
    // the current liquidity in range
    pub liquidity: u128,
}

#[derive(Default, Debug)]
struct StepComputations {
    // the price at the beginning of the step
    sqrt_price_start_x64: u128,
    // the next tick to swap to from the current tick in the swap direction
    tick_next: i32,
    // whether tick_next is initialized or not
    initialized: bool,
    // sqrt(price) for the next tick (1/0)
    sqrt_price_next_x64: u128,
    // how much is being swapped in in this step
    amount_in: u64,
    // how much is being swapped out
    amount_out: u64,
    // how much fee is being paid in
    fee_amount: u64,
}

const CU_BASE: u32 = 72_250;
const CU_PER_TICK: u32 = 17_100;
const CU_SATURATION: u32 = 360_750;

pub fn swap_internal(
    amm_config: &AmmConfig,
    pool_state: &PoolState,
    tick_array_states: &[&TickArrayState],
    amount_specified: u64,
    a_to_b: bool,
    exact_in: bool,
) -> Result<QuoteResult> {
    let mut tick_array_states = tick_array_states.iter().rev().collect::<Vec<_>>();
    let sqrt_price_limit_x64 = if a_to_b {
        tick_math::MIN_SQRT_PRICE_X64
    } else {
        tick_math::MAX_SQRT_PRICE_X64
    };

    let liquidity_start = pool_state.liquidity;
    pool_state.check_reward_infos(now::as_secs())?;

    let mut state = SwapState {
        amount_specified_remaining: amount_specified,
        amount_calculated: 0,
        sqrt_price_x64: pool_state.sqrt_price_x64,
        tick: pool_state.tick_current,
        fee_growth_global_x64: if a_to_b {
            pool_state.fee_growth_global0_x64
        } else {
            pool_state.fee_growth_global1_x64
        },
        fee_amount: 0,
        protocol_fee: 0,
        fund_fee: 0,
        liquidity: liquidity_start,
    };

    let mut tick_array_current = tick_array_states.pop().context("first pop not enough")?;

    let pool_tick_array_index = TickArrayState::get_array_start_index(pool_state.tick_current, pool_state.tick_spacing);
    let mut is_match_pool_current_tick_array = pool_tick_array_index == tick_array_current.start_tick_index;

    let mut quote_result = QuoteResult {
        indices: Some(vec![]),
        compute_units: CU_BASE,
        ..Default::default()
    };
    if let Some(indices) = &mut quote_result.indices {
        indices.push(tick_array_current.start_tick_index);
    }

    let mut tick_counter = 0;

    // continue swapping as long as we haven't used the entire input/output and haven't
    // reached the price limit
    while state.amount_specified_remaining != 0 && state.sqrt_price_x64 != sqrt_price_limit_x64 {
        tick_counter += 1;

        // Save these three pieces of information for PriceChangeEvent
        // let tick_before = state.tick;
        // let sqrt_price_x64_before = state.sqrt_price_x64;
        // let liquidity_before = state.liquidity;

        // let mut step = StepComputations {
        //     sqrt_price_start_x64: state.sqrt_price_x64,
        //     ..Default::default()
        // };

        let mut next_initialized_tick = if let Some(tick_state) =
            tick_array_current.next_initialized_tick(state.tick, pool_state.tick_spacing, a_to_b)?
        {
            tick_state
        } else if !is_match_pool_current_tick_array {
            is_match_pool_current_tick_array = true;
            tick_array_current.first_initialized_tick(a_to_b)?
        } else {
            &TickState::default()
        };

        if !next_initialized_tick.is_initialized() {
            tick_array_current = tick_array_states.pop().context("RaydiumCLMMNotEnoughTickArrays")?;

            if let Some(indices) = &mut quote_result.indices {
                indices.push(tick_array_current.start_tick_index);
            }

            let first_initialized_tick = tick_array_current.first_initialized_tick(a_to_b)?;
            next_initialized_tick = first_initialized_tick;
        }

        let mut step = StepComputations {
            sqrt_price_start_x64: state.sqrt_price_x64,
            tick_next: next_initialized_tick.tick,
            initialized: next_initialized_tick.is_initialized(),
            ..Default::default()
        };

        //step.tick_next = next_initialized_tick.tick;
        //step.initialized = next_initialized_tick.is_initialized();

        step.tick_next = step.tick_next.clamp(tick_math::MIN_TICK, tick_math::MAX_TICK);
        step.sqrt_price_next_x64 = tick_math::get_sqrt_price_at_tick(step.tick_next)?;

        let target_price = if (a_to_b && step.sqrt_price_next_x64 < sqrt_price_limit_x64)
            || (!a_to_b && step.sqrt_price_next_x64 > sqrt_price_limit_x64)
        {
            sqrt_price_limit_x64
        } else {
            step.sqrt_price_next_x64
        };

        let swap_step = swap_math::compute_swap_step(
            step.sqrt_price_start_x64,
            target_price,
            state.liquidity,
            state.amount_specified_remaining,
            amm_config.trade_fee_rate,
            exact_in,
            a_to_b,
        )?;

        state.sqrt_price_x64 = swap_step.sqrt_price_next_x64;
        step.amount_in = swap_step.amount_in;
        step.amount_out = swap_step.amount_out;
        step.fee_amount = swap_step.fee_amount;

        if exact_in {
            state.amount_specified_remaining = state
                .amount_specified_remaining
                .checked_sub(step.amount_in + step.fee_amount)
                .ok_or(format_err!("RaydiumCLMMInternalSwap1"))?;
            state.amount_calculated = state
                .amount_calculated
                .checked_add(step.amount_out)
                .ok_or(format_err!("RaydiumCLMMInternalSwap2"))?;
        } else {
            state.amount_specified_remaining = state
                .amount_specified_remaining
                .checked_sub(step.amount_out)
                .ok_or(format_err!("RaydiumCLMMInternalSwap3"))?;
            let step_amount_calculate = step
                .amount_in
                .checked_add(step.fee_amount)
                .ok_or(format_err!("RaydiumCLMMInternalSwap4"))?;
            state.amount_calculated = state
                .amount_calculated
                .checked_add(step_amount_calculate)
                .ok_or(format_err!("RaydiumCLMMInternalSwap4"))?;
        }

        let step_fee_amount = step.fee_amount;
        // if the protocol fee is on, calculate how much is owed, decrement fee_amount, and increment protocol_fee
        if amm_config.protocol_fee_rate > 0 {
            let delta = U128::from(step_fee_amount)
                .checked_mul(amm_config.protocol_fee_rate.into())
                .ok_or(format_err!("RaydiumCLMMInternalSwap5"))?
                .checked_div(FEE_RATE_DENOMINATOR_VALUE.into())
                .ok_or(format_err!("RaydiumCLMMInternalSwap6"))?
                .as_u64();
            step.fee_amount = step
                .fee_amount
                .checked_sub(delta)
                .ok_or(format_err!("RaydiumCLMMInternalSwap7"))?;
        }
        // if the fund fee is on, calculate how much is owed, decrement fee_amount, and increment fund_fee
        if amm_config.fund_fee_rate > 0 {
            let delta = U128::from(step_fee_amount)
                .checked_mul(amm_config.fund_fee_rate.into())
                .ok_or(format_err!("RaydiumCLMMInternalSwap8"))?
                .checked_div(FEE_RATE_DENOMINATOR_VALUE.into())
                .ok_or(format_err!("RaydiumCLMMInternalSwap9"))?
                .as_u64();
            step.fee_amount = step
                .fee_amount
                .checked_sub(delta)
                .ok_or(format_err!("RaydiumCLMMInternalSwap10"))?;
        }

        // update global fee tracker
        if state.liquidity > 0 {
            state.fee_amount = state
                .fee_amount
                .checked_add(step.fee_amount)
                .ok_or_else(|| format_err!("RaydiumCLMMInternalSwap11"))?;
        }
        // shift tick if we reached the next price
        if state.sqrt_price_x64 == step.sqrt_price_next_x64 {
            // if the tick is initialized, run the tick transition
            if step.initialized {
                let mut liquidity_net = next_initialized_tick.liquidity_net;

                if a_to_b {
                    liquidity_net = liquidity_net.neg();
                }
                state.liquidity = liquidity_math::add_delta(state.liquidity, liquidity_net)?;
            }

            state.tick = if a_to_b { step.tick_next - 1 } else { step.tick_next };
        } else if state.sqrt_price_x64 != step.sqrt_price_start_x64 {
            // recompute unless we're on a lower tick boundary (i.e. already transitioned ticks), and haven't moved
            // if only a small amount of quantity is traded, the input may be consumed by fees, resulting in no price change. If state.sqrt_price_x64, i.e., the latest price in the pool, is used to recalculate the tick, some errors may occur.
            // for example, if zero_for_one, and the price falls exactly on an initialized tick t after the first trade, then at this point, pool.sqrtPriceX64 = get_sqrt_price_at_tick(t), while pool.tick = t-1. if the input quantity of the
            // second trade is very small and the pool price does not change after the transaction, if the tick is recalculated, pool.tick will be equal to t, which is incorrect.
            state.tick = tick_math::get_tick_at_sqrt_price(state.sqrt_price_x64)?;
        }
    }

    //println!("Tick counter: {}", tick_counter);

    quote_result.compute_units += tick_counter.saturating_mul(&CU_PER_TICK);

    if quote_result.compute_units > CU_SATURATION {
        quote_result.compute_units = CU_SATURATION;
    }

    if a_to_b && exact_in {
        quote_result.amount_a_in = amount_specified;
        quote_result.amount_a_out = state.amount_specified_remaining;
        quote_result.amount_b_out = state.amount_calculated;
    } else if a_to_b && !exact_in {
        quote_result.amount_b_out = amount_specified;
        quote_result.amount_a_in = state.amount_calculated;
    } else if !a_to_b && exact_in {
        quote_result.amount_b_in = amount_specified;
        quote_result.amount_b_out = state.amount_specified_remaining;
        quote_result.amount_a_out = state.amount_calculated;
    } else if !a_to_b && !exact_in {
        quote_result.amount_a_out = amount_specified;
        quote_result.amount_b_in = state.amount_calculated;
    }

    Ok(quote_result)
}

pub fn swap_internal_v2(
    amm_config: &AmmConfig,
    pool_state: &PoolState,
    tick_array_states: &[&TickArrayState],
    tickarray_bitmap_extension: Option<&TickArrayBitmapExtension>,
    amount_specified: u64,
    a_to_b: bool,
    exact_in: bool,
) -> Result<QuoteResult> {
    let mut tick_array_states = VecDeque::from(tick_array_states.to_vec());

    let liquidity_start = pool_state.liquidity;

    // ! let updated_reward_infos = pool_state.update_reward_infos(block_timestamp as u64)?;

    let mut state = SwapState {
        amount_specified_remaining: amount_specified,
        amount_calculated: 0,
        sqrt_price_x64: pool_state.sqrt_price_x64,
        tick: pool_state.tick_current,
        fee_growth_global_x64: 0,
        fee_amount: 0,
        protocol_fee: 0,
        fund_fee: 0,
        liquidity: liquidity_start,
    };

    let (mut is_match_pool_current_tick_array, first_vaild_tick_array_start_index) =
        pool_state.get_first_initialized_tick_array(tickarray_bitmap_extension, a_to_b)?;
    let mut current_vaild_tick_array_start_index = first_vaild_tick_array_start_index;

    let mut tick_array_current = tick_array_states
        .pop_front()
        .ok_or(format_err!("RaydiumCLMMSwapInternalV2"))?;
    // find the first active tick array account
    for _ in 0..tick_array_states.len() {
        if tick_array_current.start_tick_index == current_vaild_tick_array_start_index {
            break;
        }
        tick_array_current = tick_array_states.pop_front().context("NotEnoughTickArrayAccount")?;
    }

    let mut qoute_result = QuoteResult {
        indices: Some(vec![]),
        compute_units: CU_BASE,
        ..Default::default()
    };
    if let Some(indices) = &mut qoute_result.indices {
        indices.push(tick_array_current.start_tick_index);
    }

    let sqrt_price_limit_x64 = if a_to_b {
        tick_math::MIN_SQRT_PRICE_X64 + 1
    } else {
        tick_math::MAX_SQRT_PRICE_X64 - 1
    };

    // continue swapping as long as we haven't used the entire input/output and haven't
    // reached the price limit
    while state.amount_specified_remaining != 0 && state.sqrt_price_x64 != sqrt_price_limit_x64 {
        println!(
            "amount_specified_remaining: {}, tick: {}",
            state.amount_specified_remaining, state.tick,
        );

        // Save these three pieces of information for PriceChangeEvent
        // let tick_before = state.tick;
        // let sqrt_price_x64_before = state.sqrt_price_x64;
        // let liquidity_before = state.liquidity;

        let mut step = StepComputations {
            sqrt_price_start_x64: state.sqrt_price_x64,
            ..Default::default()
        };

        let mut next_initialized_tick = if let Some(tick_state) =
            tick_array_current.next_initialized_tick(state.tick, pool_state.tick_spacing, a_to_b)?
        {
            Box::new(*tick_state)
        } else if !is_match_pool_current_tick_array {
            is_match_pool_current_tick_array = true;
            Box::new(*tick_array_current.first_initialized_tick(a_to_b)?)
        } else {
            Box::new(TickState::default())
        };
        if !next_initialized_tick.is_initialized() {
            let next_initialized_tickarray_index = pool_state.next_initialized_tick_array_start_index(
                tickarray_bitmap_extension,
                current_vaild_tick_array_start_index,
                a_to_b,
            )?;

            if let Some(next_initialized_tickarray_index) = next_initialized_tickarray_index {
                while tick_array_current.start_tick_index != next_initialized_tickarray_index {
                    tick_array_current = tick_array_states.pop_front().context("NotEnoughTickArrayAccount")?;
                    // check the tick_array account is owned by the pool
                }

                if let Some(indices) = &mut qoute_result.indices {
                    indices.push(tick_array_current.start_tick_index);
                }

                current_vaild_tick_array_start_index = next_initialized_tickarray_index;

                let first_initialized_tick = tick_array_current.first_initialized_tick(a_to_b)?;
                next_initialized_tick = Box::new(*first_initialized_tick);
            } else {
                return Err(anyhow::format_err!("RaydiumCLMMLiquidityInsufficient"));
            }
        }
        step.tick_next = next_initialized_tick.tick;
        step.initialized = next_initialized_tick.is_initialized();

        // if step.tick_next < tick_math::MIN_TICK {
        //     step.tick_next = tick_math::MIN_TICK;
        // } else if step.tick_next > tick_math::MAX_TICK {
        //     step.tick_next = tick_math::MAX_TICK;
        // }
        step.tick_next = step.tick_next.clamp(tick_math::MIN_TICK, tick_math::MAX_TICK);
        step.sqrt_price_next_x64 = tick_math::get_sqrt_price_at_tick(step.tick_next)?;

        let target_price = if (a_to_b && step.sqrt_price_next_x64 < sqrt_price_limit_x64)
            || (!a_to_b && step.sqrt_price_next_x64 > sqrt_price_limit_x64)
        {
            sqrt_price_limit_x64
        } else {
            step.sqrt_price_next_x64
        };

        println!("S: {step:?}");

        let swap_step = swap_math::compute_swap_step(
            step.sqrt_price_start_x64,
            target_price,
            state.liquidity,
            state.amount_specified_remaining,
            amm_config.trade_fee_rate,
            exact_in,
            a_to_b,
        )?;

        println!("SS: {swap_step:?}");

        state.sqrt_price_x64 = swap_step.sqrt_price_next_x64;
        step.amount_in = swap_step.amount_in;
        step.amount_out = swap_step.amount_out;
        step.fee_amount = swap_step.fee_amount;

        println!("step.amount_out: {}", step.amount_out);

        if exact_in {
            state.amount_specified_remaining = state
                .amount_specified_remaining
                .checked_sub(step.amount_in + step.fee_amount)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_1"))?;
            state.amount_calculated = state
                .amount_calculated
                .checked_add(step.amount_out)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_2"))?;
        } else {
            state.amount_specified_remaining = state
                .amount_specified_remaining
                .checked_sub(step.amount_out)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_3"))?;
            let step_amount_calculate = step
                .amount_in
                .checked_add(step.fee_amount)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_4"))?;
            state.amount_calculated = state
                .amount_calculated
                .checked_add(step_amount_calculate)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_5"))?;
        }

        let step_fee_amount = step.fee_amount;
        // if the protocol fee is on, calculate how much is owed, decrement fee_amount, and increment protocol_fee
        if amm_config.protocol_fee_rate > 0 {
            let delta = U128::from(step_fee_amount)
                .checked_mul(amm_config.protocol_fee_rate.into())
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_6"))?
                .checked_div(FEE_RATE_DENOMINATOR_VALUE.into())
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_7"))?
                .as_u64();
            step.fee_amount = step
                .fee_amount
                .checked_sub(delta)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_8"))?;
            state.protocol_fee = state
                .protocol_fee
                .checked_add(delta)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_9"))?;
        }
        // if the fund fee is on, calculate how much is owed, decrement fee_amount, and increment fund_fee
        if amm_config.fund_fee_rate > 0 {
            let delta = U128::from(step_fee_amount)
                .checked_mul(amm_config.fund_fee_rate.into())
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_10"))?
                .checked_div(FEE_RATE_DENOMINATOR_VALUE.into())
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_11"))?
                .as_u64();
            step.fee_amount = step
                .fee_amount
                .checked_sub(delta)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_12"))?;
            state.fund_fee = state
                .fund_fee
                .checked_add(delta)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_13"))?;
        }

        // update global fee tracker
        if state.liquidity > 0 {
            // let fee_growth_global_x64_delta = U128::from(step.fee_amount)
            //     .mul_div_floor(U128::from(fixed_point_64::Q64), U128::from(state.liquidity))
            //     .un_wrap()
            //     .as_u128();

            // state.fee_growth_global_x64 = state
            //     .fee_growth_global_x64
            //     .checked_add(fee_growth_global_x64_delta)
            //     .un_wrap();

            // state.fee_amount = state.fee_amount.checked_add(step.fee_amount).un_wrap();
            if let Some(fee) = state.fee_amount.checked_add(step.fee_amount) {
                state.fee_amount = fee;
            } else {
                return Err(format_err!("RaydiumCLMMInternalSwapV2_14"));
            }
        }

        // shift tick if we reached the next price
        if state.sqrt_price_x64 == step.sqrt_price_next_x64 {
            // if the tick is initialized, run the tick transition
            if step.initialized {
                // let mut liquidity_net = next_initialized_tick.cross(
                //     if a_to_b {
                //         state.fee_growth_global_x64
                //     } else {
                //         pool_state.fee_growth_global_0_x64
                //     },
                //     if a_to_b {
                //         pool_state.fee_growth_global_1_x64
                //     } else {
                //         state.fee_growth_global_x64
                //     },
                //     &updated_reward_infos,
                // );
                let mut liquidity_net = next_initialized_tick.liquidity_net;

                // update tick_state to tick_array account
                // tick_array_current.update_tick_state(
                //     next_initialized_tick.tick,
                //     pool_state.tick_spacing.into(),
                //     *next_initialized_tick,
                // )?;

                if a_to_b {
                    liquidity_net = liquidity_net.neg();
                }
                state.liquidity = liquidity_math::add_delta(state.liquidity, liquidity_net)?;
            }

            state.tick = if a_to_b { step.tick_next - 1 } else { step.tick_next };
        } else if state.sqrt_price_x64 != step.sqrt_price_start_x64 {
            // recompute unless we're on a lower tick boundary (i.e. already transitioned ticks), and haven't moved
            // if only a small amount of quantity is traded, the input may be consumed by fees, resulting in no price change. If state.sqrt_price_x64, i.e., the latest price in the pool, is used to recalculate the tick, some errors may occur.
            // for example, if a_to_b, and the price falls exactly on an initialized tick t after the first trade, then at this point, pool.sqrtPriceX64 = get_sqrt_price_at_tick(t), while pool.tick = t-1. if the input quantity of the
            // second trade is very small and the pool price does not change after the transaction, if the tick is recalculated, pool.tick will be equal to t, which is incorrect.
            state.tick = tick_math::get_tick_at_sqrt_price(state.sqrt_price_x64)?;
        }

        println!("step: {step:?}");
    }

    println!(
        "amount_specified_remaining: {}, tick: {}",
        state.amount_specified_remaining, state.tick,
    );

    // update tick
    // if state.tick != pool_state.tick_current {
    //     // update the previous tick to the observation
    //     observation_state.update(block_timestamp, pool_state.tick_current);
    //     pool_state.tick_current = state.tick;
    // }
    //pool_state.sqrt_price_x64 = state.sqrt_price_x64;

    // if liquidity_start != state.liquidity {
    //     pool_state.liquidity = state.liquidity;
    // }

    let (amount_0, amount_1) = if a_to_b == exact_in {
        (
            amount_specified
                .checked_sub(state.amount_specified_remaining)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_15"))?,
            state.amount_calculated,
        )
    } else {
        (
            state.amount_calculated,
            amount_specified
                .checked_sub(state.amount_specified_remaining)
                .ok_or(format_err!("RaydiumCLMMInternalSwapV2_16"))?,
        )
    };

    println!("state.amount_calculated: {}, ", state.amount_calculated);
    println!("amount_0: {amount_0}, amount_1: {amount_1}");

    // if a_to_b {
    //     pool_state.fee_growth_global_0_x64 = state.fee_growth_global_x64;
    //     pool_state.total_fees_token_0 = pool_state
    //         .total_fees_token_0
    //         .checked_add(state.fee_amount)
    //         .un_wrap();

    //     if state.protocol_fee > 0 {
    //         pool_state.protocol_fees_token_0 = pool_state
    //             .protocol_fees_token_0
    //             .checked_add(state.protocol_fee)
    //             .un_wrap();
    //     }
    //     if state.fund_fee > 0 {
    //         pool_state.fund_fees_token_0 = pool_state
    //             .fund_fees_token_0
    //             .checked_add(state.fund_fee)
    //             .un_wrap();
    //     }
    //     pool_state.swap_in_amount_token_0 = pool_state
    //         .swap_in_amount_token_0
    //         .checked_add(u128::from(amount_0))
    //         .un_wrap();
    //     pool_state.swap_out_amount_token_1 = pool_state
    //         .swap_out_amount_token_1
    //         .checked_add(u128::from(amount_1))
    //         .un_wrap();
    // } else {
    //     pool_state.fee_growth_global_1_x64 = state.fee_growth_global_x64;
    //     pool_state.total_fees_token_1 = pool_state
    //         .total_fees_token_1
    //         .checked_add(state.fee_amount)
    //         .un_wrap();

    //     if state.protocol_fee > 0 {
    //         pool_state.protocol_fees_token_1 = pool_state
    //             .protocol_fees_token_1
    //             .checked_add(state.protocol_fee)
    //             .un_wrap();
    //     }
    //     if state.fund_fee > 0 {
    //         pool_state.fund_fees_token_1 = pool_state
    //             .fund_fees_token_1
    //             .checked_add(state.fund_fee)
    //             .un_wrap();
    //     }
    //     pool_state.swap_in_amount_token_1 = pool_state
    //         .swap_in_amount_token_1
    //         .checked_add(u128::from(amount_1))
    //         .un_wrap();
    //     pool_state.swap_out_amount_token_0 = pool_state
    //         .swap_out_amount_token_0
    //         .checked_add(u128::from(amount_0))
    //         .un_wrap();
    // }

    if a_to_b && exact_in {
        qoute_result.amount_a_in = amount_specified;
        qoute_result.amount_a_out = state.amount_specified_remaining;
        qoute_result.amount_b_out = state.amount_calculated;
    } else if a_to_b && !exact_in {
        qoute_result.amount_b_out = amount_specified;
        qoute_result.amount_a_in = state.amount_calculated;
    } else if !a_to_b && exact_in {
        qoute_result.amount_b_in = amount_specified;
        qoute_result.amount_b_out = state.amount_specified_remaining;
        qoute_result.amount_a_out = state.amount_calculated;
    } else if !a_to_b && !exact_in {
        qoute_result.amount_a_out = amount_specified;
        qoute_result.amount_b_in = state.amount_calculated;
    }

    Ok(qoute_result)
}
