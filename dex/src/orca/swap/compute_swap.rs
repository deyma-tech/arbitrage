// https://github.com/orca-so/whirlpools/blob/main/programs/whirlpool/src/instructions/v2/swap.rs

use utils::quote::QuoteResult;

use crate::orca::{
    swap::{
        fee_rate_manager::FeeRateManager,
        math::{
            add_liquidity_delta, checked_mul_div, checked_mul_div_round_up, sqrt_price_from_tick_index,
            tick_index_from_sqrt_price, MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64, Q64_RESOLUTION,
        },
        oracle::{AdaptiveFeeInfo, Oracle, OracleAccessor},
        swap_tick_sequence::SwapTickSequence,
        token_math::{
            get_amount_delta_a, get_amount_delta_b, get_next_sqrt_price, try_get_amount_delta_a,
            try_get_amount_delta_b, AmountDeltaU64, FEE_RATE_MUL_VALUE, PROTOCOL_FEE_RATE_MUL_VALUE,
        },
        TICK_ARRAY_SIZE,
    },
    OrcaTick, OrcaTickArray, OrcaWhirlpool,
};

pub fn compute_swap_v3(
    token_amount: u64,
    whirlpool: &OrcaWhirlpool,
    tick_arrays: &[&OrcaTickArray],
    a_to_b: bool, // Zero for one
    specified_input: bool,
    oracle: Option<Oracle>,
    //sqrt_price_limit: u128,
    //remaining_accounts_info: Option<RemainingAccountsInfo>,
) -> anyhow::Result<QuoteResult> {
    const OTHER_AMOUNT_THRESHOLD: u64 = 0;
    //let whirlpool = &mut ctx.accounts.whirlpool;

    //let clock = Clock::get()?;
    // Update the global reward growth which increases as a function of time.
    //let timestamp = to_timestamp_u64(clock.unix_timestamp)?;
    let timestamp = utils::now::as_secs();

    // Process remaining accounts
    // let remaining_accounts = parse_remaining_accounts(
    //     ctx.remaining_accounts,
    //     &remaining_accounts_info,
    //     &[
    //         AccountsType::TransferHookA,
    //         AccountsType::TransferHookB,
    //         AccountsType::SupplementalTickArrays,
    //     ],
    // )?;

    // let swap_tick_sequence_builder = SparseSwapTickSequenceBuilder::new(
    //     vec![
    //         ctx.accounts.tick_array_0.to_account_info(),
    //         ctx.accounts.tick_array_1.to_account_info(),
    //         ctx.accounts.tick_array_2.to_account_info(),
    //     ],
    //     remaining_accounts.supplemental_tick_arrays,
    // );
    // let mut swap_tick_sequence = swap_tick_sequence_builder.try_build(whirlpool, a_to_b)?;

    // let oracle_accessor = OracleAccessor::new(whirlpool, ctx.accounts.oracle.to_account_info())?;
    // if !oracle_accessor.is_trade_enabled(timestamp)? {
    //     return Err(anyhow::format_err!("TradeIsNotEnable"));
    // }

    let mut swap_tick_sequence = SwapTickSequence {
        arrays: tick_arrays.to_vec(),
    };

    let oracle_accessor = match oracle {
        Some(oracle) => OracleAccessor {
            oracle_account_info: oracle,
            oracle_account_initialized: true,
        },
        None => OracleAccessor {
            oracle_account_info: Oracle::default(),
            oracle_account_initialized: false,
        },
    };
    let adaptive_fee_info = oracle_accessor.get_adaptive_fee_info()?;

    let mut qoute_result = QuoteResult {
        indices: Some(
            tick_arrays
                .iter()
                .map(|tick_array| tick_array.start_tick_index)
                .collect(),
        ),
        compute_units: 75_000,
        ..Default::default()
    };

    let swap_update = swap_with_transfer_fee_extension(
        whirlpool,
        // &whirlpool.token_mint_a,
        // &whirlpool.token_mint_b,
        &mut swap_tick_sequence,
        token_amount,
        //sqrt_price_limit,
        specified_input,
        a_to_b,
        timestamp,
        &adaptive_fee_info,
    )?;

    if specified_input {
        // let transfer_fee_excluded_output_amount = if a_to_b {
        //     //calculate_transfer_fee_excluded_amount(&whirlpool.token_mint_b, swap_update.amount_b)?.amount
        //     swap_update.amount_b
        // } else {
        //     //calculate_transfer_fee_excluded_amount(&whirlpool.token_mint_a, swap_update.amount_a)?.amount
        //     swap_update.amount_a
        // };
        //if transfer_fee_excluded_output_amount < OTHER_AMOUNT_THRESHOLD {
        //    return Err(anyhow::format_err!("AmountOutBelowMinimum"));
        //}
    } else {
        let transfer_fee_included_input_amount = if a_to_b {
            swap_update.amount_a
        } else {
            swap_update.amount_b
        };
        if transfer_fee_included_input_amount > OTHER_AMOUNT_THRESHOLD {
            return Err(anyhow::format_err!("AmountInAboveMaximum"));
        }
    }

    //oracle_accessor.update_adaptive_fee_variables(&swap_update.next_adaptive_fee_info)?;

    // let pre_sqrt_price = whirlpool.sqrt_price;
    // let (input_amount, output_amount) = if a_to_b {
    //     (swap_update.amount_a, swap_update.amount_b)
    // } else {
    //     (swap_update.amount_b, swap_update.amount_a)
    // };
    // let (token_mint_input, token_mint_output) = if a_to_b {
    //     (&whirlpool.token_mint_a, &whirlpool.token_mint_b)
    // } else {
    //     (&whirlpool.token_mint_b, &whirlpool.token_mint_a)
    // };

    // let input_transfer_fee = calculate_transfer_fee_excluded_amount(token_mint_input, input_amount)?.transfer_fee;
    // let output_transfer_fee = calculate_transfer_fee_excluded_amount(token_mint_output, output_amount)?.transfer_fee;
    // let (lp_fee, protocol_fee) = (swap_update.lp_fee, swap_update.next_protocol_fee);

    // update_and_swap_whirlpool_v2(
    //     whirlpool,
    //     &ctx.accounts.token_authority,
    //     &ctx.accounts.token_mint_a,
    //     &ctx.accounts.token_mint_b,
    //     &ctx.accounts.token_owner_account_a,
    //     &ctx.accounts.token_owner_account_b,
    //     &ctx.accounts.token_vault_a,
    //     &ctx.accounts.token_vault_b,
    //     &remaining_accounts.transfer_hook_a,
    //     &remaining_accounts.transfer_hook_b,
    //     &ctx.accounts.token_program_a,
    //     &ctx.accounts.token_program_b,
    //     &ctx.accounts.memo_program,
    //     &swap_update,
    //     a_to_b,
    //     timestamp,
    //     transfer_memo::TRANSFER_MEMO_SWAP.as_bytes(),
    // )?;

    // emit!(Traded {
    //     whirlpool: whirlpool.key(),
    //     a_to_b,
    //     pre_sqrt_price,
    //     post_sqrt_price: whirlpool.sqrt_price,
    //     input_amount,
    //     output_amount,
    //     input_transfer_fee,
    //     output_transfer_fee,
    //     lp_fee,
    //     protocol_fee,
    // });

    if a_to_b && specified_input {
        qoute_result.amount_a_in = token_amount;
        qoute_result.amount_a_out = swap_update.amount_a;
        qoute_result.amount_b_out = swap_update.amount_b;
    } else if a_to_b && !specified_input {
        qoute_result.amount_b_out = token_amount; // amount_remaining je 0
        qoute_result.amount_a_in = swap_update.amount_a;
    } else if !a_to_b && specified_input {
        qoute_result.amount_b_in = token_amount;
        qoute_result.amount_b_out = swap_update.amount_b;
        qoute_result.amount_a_out = swap_update.amount_a;
    } else if !a_to_b && !specified_input {
        qoute_result.amount_a_out = token_amount; //  amount_remaining je 0
        qoute_result.amount_b_in = swap_update.amount_b;
    }

    // qoute_result.indices = Some(indices);

    Ok(qoute_result)
}

#[derive(Debug)]
pub struct PostSwapUpdate {
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_fee: u64,
    pub next_liquidity: u128,
    pub next_tick_index: i32,
    pub next_sqrt_price: u128,
    pub next_fee_growth_global: u128,
    //pub next_reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS],
    pub next_protocol_fee: u64,
    //pub next_adaptive_fee_info: Option<AdaptiveFeeInfo>,
}

#[allow(clippy::too_many_arguments)]
pub fn swap_with_transfer_fee_extension(
    whirlpool: &OrcaWhirlpool,
    // token_mint_a: &InterfaceAccount<'info, Mint>,
    // token_mint_b: &InterfaceAccount<'info, Mint>,
    swap_tick_sequence: &mut SwapTickSequence,
    amount: u64,
    //sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
    timestamp: u64,
    adaptive_fee_info: &Option<AdaptiveFeeInfo>,
) -> anyhow::Result<PostSwapUpdate> {
    // let (input_token_mint, output_token_mint) = if a_to_b {
    //     (token_mint_a, token_mint_b)
    // } else {
    //     (token_mint_b, token_mint_a)
    // };

    // ExactIn
    if amount_specified_is_input {
        let transfer_fee_included_input = amount;
        let transfer_fee_excluded_input = amount;
        //calculate_transfer_fee_excluded_amount(input_token_mint, transfer_fee_included_input)?.amount;

        let swap_update = swap(
            whirlpool,
            swap_tick_sequence,
            transfer_fee_excluded_input,
            //sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
            timestamp,
            adaptive_fee_info,
        )?;

        let (swap_update_amount_input, swap_update_amount_output) = if a_to_b {
            (swap_update.amount_a, swap_update.amount_b)
        } else {
            (swap_update.amount_b, swap_update.amount_a)
        };

        let fullfilled = swap_update_amount_input == transfer_fee_excluded_input;

        let adjusted_transfer_fee_included_input = if fullfilled {
            transfer_fee_included_input
        } else {
            amount
            //calculate_transfer_fee_included_amount(input_token_mint, swap_update_amount_input)?.amount
        };

        let transfer_fee_included_output = swap_update_amount_output;

        let (amount_a, amount_b) = if a_to_b {
            (adjusted_transfer_fee_included_input, transfer_fee_included_output)
        } else {
            (transfer_fee_included_output, adjusted_transfer_fee_included_input)
        };
        return Ok(PostSwapUpdate {
            amount_a, // updated (transfer fee included)
            amount_b, // updated (transfer fee included)
            lp_fee: swap_update.lp_fee,
            next_liquidity: swap_update.next_liquidity,
            next_tick_index: swap_update.next_tick_index,
            next_sqrt_price: swap_update.next_sqrt_price,
            next_fee_growth_global: swap_update.next_fee_growth_global,
            //next_reward_infos: swap_update.next_reward_infos,
            next_protocol_fee: swap_update.next_protocol_fee,
            //next_adaptive_fee_info: swap_update.next_adaptive_fee_info,
        });
    }

    // ExactOut
    //let transfer_fee_excluded_output = amount;
    let transfer_fee_included_output = amount;
    // calculate_transfer_fee_included_amount(output_token_mint, transfer_fee_excluded_output)?.amount;

    let swap_update = swap(
        whirlpool,
        swap_tick_sequence,
        transfer_fee_included_output,
        //sqrt_price_limit,
        amount_specified_is_input,
        a_to_b,
        timestamp,
        adaptive_fee_info,
    )?;

    let (_swap_update_amount_input, swap_update_amount_output) = if a_to_b {
        (swap_update.amount_a, swap_update.amount_b)
    } else {
        (swap_update.amount_b, swap_update.amount_a)
    };

    let transfer_fee_included_input = amount;
    //calculate_transfer_fee_included_amount(input_token_mint, swap_update_amount_input)?.amount;

    let adjusted_transfer_fee_included_output = swap_update_amount_output;

    let (amount_a, amount_b) = if a_to_b {
        (transfer_fee_included_input, adjusted_transfer_fee_included_output)
    } else {
        (adjusted_transfer_fee_included_output, transfer_fee_included_input)
    };
    Ok(PostSwapUpdate {
        amount_a, // updated (transfer fee included)
        amount_b, // updated (transfer fee included)
        lp_fee: swap_update.lp_fee,
        next_liquidity: swap_update.next_liquidity,
        next_tick_index: swap_update.next_tick_index,
        next_sqrt_price: swap_update.next_sqrt_price,
        next_fee_growth_global: swap_update.next_fee_growth_global,
        //next_reward_infos: swap_update.next_reward_infos,
        next_protocol_fee: swap_update.next_protocol_fee,
        //next_adaptive_fee_info: swap_update.next_adaptive_fee_info,
    })
}

// https://github.com/orca-so/whirlpools/blob/5278dd46f4648e4041e05b48301ff0fd5747f592/programs/whirlpool/src/manager/swap_manager.rs

fn get_next_sqrt_prices(next_tick_index: i32, sqrt_price_limit: u128, a_to_b: bool) -> (u128, u128) {
    let next_tick_price = sqrt_price_from_tick_index(next_tick_index);
    let next_sqrt_price_limit = if a_to_b {
        sqrt_price_limit.max(next_tick_price)
    } else {
        sqrt_price_limit.min(next_tick_price)
    };
    (next_tick_price, next_sqrt_price_limit)
}

fn calculate_update(
    tick: &OrcaTick,
    a_to_b: bool,
    liquidity: u128,
    //fee_growth_global_a: u128,
    //fee_growth_global_b: u128,
    //reward_infos: &[WhirlpoolRewardInfo; NUM_REWARDS],
) -> anyhow::Result<u128> {
    // Use updated fee_growth for crossing tick
    // Use -liquidity_net if going left, +liquidity_net going right
    let signed_liquidity_net = if a_to_b {
        -tick.liquidity_net
    } else {
        tick.liquidity_net
    };

    //let update = next_tick_cross_update(tick, fee_growth_global_a, fee_growth_global_b, reward_infos)?;

    // Update the global liquidity to reflect the new current tick
    let next_liquidity = add_liquidity_delta(liquidity, signed_liquidity_net)?;

    //Ok((update, next_liquidity))
    Ok(next_liquidity)
}

#[allow(clippy::too_many_arguments)]
pub fn swap(
    whirlpool: &OrcaWhirlpool,
    swap_tick_sequence: &mut SwapTickSequence,
    amount: u64,
    //sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
    timestamp: u64,
    adaptive_fee_info: &Option<AdaptiveFeeInfo>,
) -> anyhow::Result<PostSwapUpdate> {
    let adjusted_sqrt_price_limit = if a_to_b { MIN_SQRT_PRICE_X64 } else { MAX_SQRT_PRICE_X64 };

    // if !(MIN_SQRT_PRICE_X64..=MAX_SQRT_PRICE_X64).contains(&adjusted_sqrt_price_limit) {
    //     return Err(anyhow::format_err!("SqrtPriceOutOfBounds"));
    // }

    // if a_to_b && adjusted_sqrt_price_limit >= whirlpool.sqrt_price
    //     || !a_to_b && adjusted_sqrt_price_limit <= whirlpool.sqrt_price
    // {
    //     return Err(ErrorCode::InvalidSqrtPriceLimitDirection.into());
    // }

    // if amount == 0 {
    //     return Err(ErrorCode::ZeroTradableAmount.into());
    // }

    let tick_spacing = whirlpool.tick_spacing;
    let fee_rate = whirlpool.fee_rate;
    let protocol_fee_rate = whirlpool.protocol_fee_rate;
    //let next_reward_infos = next_whirlpool_reward_infos(whirlpool, timestamp)?;

    let mut amount_remaining: u64 = amount;
    let mut amount_calculated: u64 = 0;
    let mut curr_sqrt_price = whirlpool.sqrt_price;
    let mut curr_tick_index = whirlpool.tick_current_index;
    let mut curr_liquidity = whirlpool.liquidity;
    let mut curr_protocol_fee: u64 = 0;
    let mut curr_array_index: usize = 0;
    let mut curr_fee_growth_global_input = if a_to_b {
        whirlpool.fee_growth_global_a
    } else {
        whirlpool.fee_growth_global_b
    };
    let mut fee_sum: u64 = 0;

    let mut fee_rate_manager = FeeRateManager::new(
        a_to_b,
        whirlpool.tick_current_index, // note:  -1 shift is acceptable
        timestamp,
        fee_rate,
        adaptive_fee_info,
    )?;

    while amount_remaining > 0 && adjusted_sqrt_price_limit != curr_sqrt_price {
        let (next_array_index, next_tick_index) = swap_tick_sequence.get_next_initialized_tick_index(
            curr_tick_index,
            tick_spacing,
            a_to_b,
            curr_array_index,
        )?;

        let (next_tick_sqrt_price, sqrt_price_target) =
            get_next_sqrt_prices(next_tick_index, adjusted_sqrt_price_limit, a_to_b);

        loop {
            fee_rate_manager.update_volatility_accumulator()?;

            let total_fee_rate = fee_rate_manager.get_total_fee_rate();
            let (bounded_sqrt_price_target, adaptive_fee_update_skipped) =
                fee_rate_manager.get_bounded_sqrt_price_target(sqrt_price_target, curr_liquidity);

            let swap_computation = compute_swap(
                amount_remaining,
                total_fee_rate,
                curr_liquidity,
                curr_sqrt_price,
                bounded_sqrt_price_target,
                amount_specified_is_input,
                a_to_b,
            )?;

            if amount_specified_is_input {
                amount_remaining = amount_remaining
                    .checked_sub(swap_computation.amount_in)
                    .ok_or(anyhow::format_err!("AmountRemainingOverflow"))?;
                amount_remaining = amount_remaining
                    .checked_sub(swap_computation.fee_amount)
                    .ok_or(anyhow::format_err!("AmountRemainingOverflow"))?;

                amount_calculated = amount_calculated
                    .checked_add(swap_computation.amount_out)
                    .ok_or(anyhow::format_err!("AmountCalcOverflow"))?;
            } else {
                amount_remaining = amount_remaining
                    .checked_sub(swap_computation.amount_out)
                    .ok_or(anyhow::format_err!("AmountRemainingOverflow"))?;

                amount_calculated = amount_calculated
                    .checked_add(swap_computation.amount_in)
                    .ok_or(anyhow::format_err!("AmountCalcOverflow"))?;
                amount_calculated = amount_calculated
                    .checked_add(swap_computation.fee_amount)
                    .ok_or(anyhow::format_err!("AmountCalcOverflow"))?;
            }

            fee_sum = fee_sum
                .checked_add(swap_computation.fee_amount)
                .ok_or(anyhow::format_err!("AmountCalcOverflow"))?;

            let (next_protocol_fee, next_fee_growth_global_input) = calculate_fees(
                swap_computation.fee_amount,
                protocol_fee_rate,
                curr_liquidity,
                curr_protocol_fee,
                curr_fee_growth_global_input,
            );
            curr_protocol_fee = next_protocol_fee;
            curr_fee_growth_global_input = next_fee_growth_global_input;

            if swap_computation.next_price == next_tick_sqrt_price {
                let (next_tick, next_tick_initialized) = swap_tick_sequence
                    .get_tick(next_array_index, next_tick_index, tick_spacing)
                    .map_or_else(|_| (None, false), |tick| (Some(tick), tick.initialized));

                if next_tick_initialized {
                    let (_fee_growth_global_a, _fee_growth_global_b) = if a_to_b {
                        (curr_fee_growth_global_input, whirlpool.fee_growth_global_b)
                    } else {
                        (whirlpool.fee_growth_global_a, curr_fee_growth_global_input)
                    };

                    let next_liquidity = calculate_update(
                        &next_tick.unwrap(),
                        a_to_b,
                        curr_liquidity,
                        //fee_growth_global_a,
                        //fee_growth_global_b,
                        //&next_reward_infos,
                    )?;

                    curr_liquidity = next_liquidity;
                    //swap_tick_sequence.update_tick(next_array_index, next_tick_index, tick_spacing, &update)?;
                }

                let tick_offset =
                    swap_tick_sequence.get_tick_offset(next_array_index, next_tick_index, tick_spacing)?;

                // Increment to the next tick array if either condition is true:
                //  - Price is moving left and the current tick is the start of the tick array
                //  - Price is moving right and the current tick is the end of the tick array
                curr_array_index =
                    if (a_to_b && tick_offset == 0) || (!a_to_b && tick_offset == TICK_ARRAY_SIZE as isize - 1) {
                        next_array_index + 1
                    } else {
                        next_array_index
                    };

                // The get_init_tick search is inclusive of the current index in an a_to_b trade.
                // We therefore have to shift the index by 1 to advance to the next init tick to the left.
                curr_tick_index = if a_to_b { next_tick_index - 1 } else { next_tick_index };
            } else if swap_computation.next_price != curr_sqrt_price {
                curr_tick_index = tick_index_from_sqrt_price(&swap_computation.next_price);
            }

            curr_sqrt_price = swap_computation.next_price;

            if !adaptive_fee_update_skipped {
                // Note: curr_sqrt_price != bounded_sqrt_price_target implies the end of the loop.
                //       tick_group_index counter exists only in the memory of the FeeRateManager,
                //       so even if it is incremented one extra time at the end of the loop, there is no real harm.
                fee_rate_manager.advance_tick_group();
            } else {
                fee_rate_manager.advance_tick_group_after_skip(
                    curr_sqrt_price,
                    next_tick_sqrt_price,
                    next_tick_index,
                )?;
            }

            // do while loop
            if amount_remaining == 0 || curr_sqrt_price == sqrt_price_target {
                break;
            }
        }
    }

    // // Reject partial fills if no explicit sqrt price limit is set and trade is exact out mode
    // if amount_remaining > 0 && !amount_specified_is_input && sqrt_price_limit == NO_EXPLICIT_SQRT_PRICE_LIMIT {
    //     return Err(ErrorCode::PartialFillError.into());
    // }

    let (amount_a, amount_b) = if a_to_b == amount_specified_is_input {
        (amount - amount_remaining, amount_calculated)
    } else {
        (amount_calculated, amount - amount_remaining)
    };

    //fee_rate_manager.update_major_swap_timestamp(timestamp, whirlpool.sqrt_price, curr_sqrt_price)?;

    Ok(PostSwapUpdate {
        amount_a,
        amount_b,
        lp_fee: fee_sum - curr_protocol_fee,
        next_liquidity: curr_liquidity,
        next_tick_index: curr_tick_index,
        next_sqrt_price: curr_sqrt_price,
        next_fee_growth_global: curr_fee_growth_global_input,
        //next_reward_infos,
        next_protocol_fee: curr_protocol_fee,
        //next_adaptive_fee_info: fee_rate_manager.get_next_adaptive_fee_info(),
    })
}

fn calculate_fees(
    fee_amount: u64,
    protocol_fee_rate: u16,
    curr_liquidity: u128,
    curr_protocol_fee: u64,
    curr_fee_growth_global_input: u128,
) -> (u64, u128) {
    let mut next_protocol_fee = curr_protocol_fee;
    let mut next_fee_growth_global_input = curr_fee_growth_global_input;
    let mut global_fee = fee_amount;
    if protocol_fee_rate > 0 {
        let delta = calculate_protocol_fee(global_fee, protocol_fee_rate);
        global_fee -= delta;
        next_protocol_fee = next_protocol_fee.wrapping_add(delta);
    }

    if curr_liquidity > 0 {
        next_fee_growth_global_input =
            next_fee_growth_global_input.wrapping_add(((global_fee as u128) << Q64_RESOLUTION) / curr_liquidity);
    }
    (next_protocol_fee, next_fee_growth_global_input)
}

fn calculate_protocol_fee(global_fee: u64, protocol_fee_rate: u16) -> u64 {
    ((global_fee as u128) * (protocol_fee_rate as u128) / PROTOCOL_FEE_RATE_MUL_VALUE)
        .try_into()
        .unwrap()
}

// https://github.com/orca-so/whirlpools/blob/5278dd46f4648e4041e05b48301ff0fd5747f592/programs/whirlpool/src/math/swap_math.rs

//pub const NO_EXPLICIT_SQRT_PRICE_LIMIT: u128 = 0u128;

#[derive(PartialEq, Debug)]
pub struct SwapStepComputation {
    pub amount_in: u64,
    pub amount_out: u64,
    pub next_price: u128,
    pub fee_amount: u64,
}

pub fn compute_swap(
    amount_remaining: u64,
    fee_rate: u32,
    liquidity: u128,
    sqrt_price_current: u128,
    sqrt_price_target: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
) -> anyhow::Result<SwapStepComputation> {
    // Since SplashPool (aka FullRange only pool) has only 2 initialized ticks at both ends,
    // the possibility of exceeding u64 when calculating "delta amount" is higher than concentrated pools.
    // This problem occurs with ExactIn.
    // The reason is that in ExactOut, "fixed delta" never exceeds the amount of tokens present in the pool and is clearly within the u64 range.
    // On the other hand, for ExactIn, "fixed delta" may exceed u64 because it calculates the amount of tokens needed to move the price to the end.
    // However, the primary purpose of initial calculation of "fixed delta" is to determine whether or not the iteration is "max swap" or not.
    // So the info that “the amount of tokens required exceeds the u64 range” is sufficient to determine that the iteration is NOT "max swap".
    //
    // delta <= u64::MAX: AmountDeltaU64::Valid
    // delta >  u64::MAX: AmountDeltaU64::ExceedsMax
    let initial_amount_fixed_delta = try_get_amount_fixed_delta(
        sqrt_price_current,
        sqrt_price_target,
        liquidity,
        amount_specified_is_input,
        a_to_b,
    )?;

    let mut amount_calc = amount_remaining;
    if amount_specified_is_input {
        amount_calc = checked_mul_div(
            amount_remaining as u128,
            FEE_RATE_MUL_VALUE - fee_rate as u128,
            FEE_RATE_MUL_VALUE,
        )?
        .try_into()?;
    }

    let next_sqrt_price = if initial_amount_fixed_delta.lte(amount_calc) {
        sqrt_price_target
    } else {
        get_next_sqrt_price(
            sqrt_price_current,
            liquidity,
            amount_calc,
            amount_specified_is_input,
            a_to_b,
        )?
    };

    let is_max_swap = next_sqrt_price == sqrt_price_target;

    let amount_unfixed_delta = get_amount_unfixed_delta(
        sqrt_price_current,
        next_sqrt_price,
        liquidity,
        amount_specified_is_input,
        a_to_b,
    )?;

    // If the swap is not at the max, we need to readjust the amount of the fixed token we are using
    let amount_fixed_delta = if !is_max_swap || initial_amount_fixed_delta.exceeds_max() {
        // next_sqrt_price is calculated by get_next_sqrt_price and the result will be in the u64 range.
        get_amount_fixed_delta(
            sqrt_price_current,
            next_sqrt_price,
            liquidity,
            amount_specified_is_input,
            a_to_b,
        )?
    } else {
        // the result will be in the u64 range.
        initial_amount_fixed_delta.value()
    };

    let (amount_in, mut amount_out) = if amount_specified_is_input {
        (amount_fixed_delta, amount_unfixed_delta)
    } else {
        (amount_unfixed_delta, amount_fixed_delta)
    };

    // Cap output amount if using output
    if !amount_specified_is_input && amount_out > amount_remaining {
        amount_out = amount_remaining;
    }

    let fee_amount = if amount_specified_is_input && !is_max_swap {
        amount_remaining - amount_in
    } else {
        checked_mul_div_round_up(
            amount_in as u128,
            fee_rate as u128,
            FEE_RATE_MUL_VALUE - fee_rate as u128,
        )?
        .try_into()?
    };

    Ok(SwapStepComputation {
        amount_in,
        amount_out,
        next_price: next_sqrt_price,
        fee_amount,
    })
}

fn get_amount_fixed_delta(
    sqrt_price_current: u128,
    sqrt_price_target: u128,
    liquidity: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
) -> anyhow::Result<u64> {
    if a_to_b == amount_specified_is_input {
        get_amount_delta_a(
            sqrt_price_current,
            sqrt_price_target,
            liquidity,
            amount_specified_is_input,
        )
    } else {
        get_amount_delta_b(
            sqrt_price_current,
            sqrt_price_target,
            liquidity,
            amount_specified_is_input,
        )
    }
}

fn try_get_amount_fixed_delta(
    sqrt_price_current: u128,
    sqrt_price_target: u128,
    liquidity: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
) -> anyhow::Result<AmountDeltaU64> {
    if a_to_b == amount_specified_is_input {
        try_get_amount_delta_a(
            sqrt_price_current,
            sqrt_price_target,
            liquidity,
            amount_specified_is_input,
        )
    } else {
        try_get_amount_delta_b(
            sqrt_price_current,
            sqrt_price_target,
            liquidity,
            amount_specified_is_input,
        )
    }
}

fn get_amount_unfixed_delta(
    sqrt_price_current: u128,
    sqrt_price_target: u128,
    liquidity: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
) -> anyhow::Result<u64> {
    if a_to_b == amount_specified_is_input {
        get_amount_delta_b(
            sqrt_price_current,
            sqrt_price_target,
            liquidity,
            !amount_specified_is_input,
        )
    } else {
        get_amount_delta_a(
            sqrt_price_current,
            sqrt_price_target,
            liquidity,
            !amount_specified_is_input,
        )
    }
}
