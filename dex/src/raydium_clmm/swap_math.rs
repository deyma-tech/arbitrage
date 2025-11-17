use super::{constants::FEE_RATE_DENOMINATOR_VALUE, full_math::MulDiv, liquidity_math, sqrt_price_math};

#[derive(Default, Debug)]
pub struct SwapStep {
    /// The price after swapping the amount in/out, not to exceed the price target
    pub sqrt_price_next_x64: u128,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
}

/// Computes the result of swapping some amount in, or amount out, given the parameters of the swap
pub fn compute_swap_step(
    sqrt_price_current_x64: u128,
    sqrt_price_target_x64: u128,
    liquidity: u128,
    amount_remaining: u64,
    fee_rate: u32,
    is_base_input: bool,
    zero_for_one: bool,
) -> anyhow::Result<SwapStep> {
    // let exact_in = amount_remaining >= 0;
    let mut swap_step = SwapStep::default();
    if is_base_input {
        // round up amount_in
        // In exact input case, amount_remaining is positive
        if let Some(amount_remaining_less_fee) = amount_remaining.mul_div_floor(
            (FEE_RATE_DENOMINATOR_VALUE - fee_rate).into(),
            u64::from(FEE_RATE_DENOMINATOR_VALUE),
        ) {
            let amount_in = calculate_amount_in_range(
                sqrt_price_current_x64,
                sqrt_price_target_x64,
                liquidity,
                zero_for_one,
                is_base_input,
            )?;
            if let Some(amount_in) = amount_in {
                swap_step.amount_in = amount_in;
            }

            swap_step.sqrt_price_next_x64 = if amount_in.is_some() && amount_remaining_less_fee >= swap_step.amount_in {
                sqrt_price_target_x64
            } else {
                sqrt_price_math::get_next_sqrt_price_from_input(
                    sqrt_price_current_x64,
                    liquidity,
                    amount_remaining_less_fee,
                    zero_for_one,
                )?
            };
        } else {
            return Err(anyhow::format_err!("RaydiumCLMMComputeSwap1"));
        }
    } else {
        let amount_out = calculate_amount_in_range(
            sqrt_price_current_x64,
            sqrt_price_target_x64,
            liquidity,
            zero_for_one,
            is_base_input,
        )?;
        if let Some(amount_out) = amount_out {
            swap_step.amount_out = amount_out;
        }
        // In exact output case, amount_remaining is negative
        swap_step.sqrt_price_next_x64 = if amount_out.is_some() && amount_remaining >= swap_step.amount_out {
            sqrt_price_target_x64
        } else {
            sqrt_price_math::get_next_sqrt_price_from_output(
                sqrt_price_current_x64,
                liquidity,
                amount_remaining,
                zero_for_one,
            )?
        }
    }

    // whether we reached the max possible price for the given ticks
    let max = sqrt_price_target_x64 == swap_step.sqrt_price_next_x64;
    //println!("max: {}", max);
    // get the input / output amounts when target price is not reached
    if zero_for_one {
        // if max is reached for exact input case, entire amount_in is needed
        #[allow(clippy::nonminimal_bool)] // Warning!!! Don't listening to clippy
        if !(max && is_base_input) {
            swap_step.amount_in = liquidity_math::get_delta_amount_0_unsigned(
                swap_step.sqrt_price_next_x64,
                sqrt_price_current_x64,
                liquidity,
                true,
            )?
        };
        // if max is reached for exact output case, entire amount_out is needed
        #[allow(clippy::nonminimal_bool)] // Warning!!! Don't listening to clippy
        if !(max && !is_base_input) {
            swap_step.amount_out = liquidity_math::get_delta_amount_1_unsigned(
                swap_step.sqrt_price_next_x64,
                sqrt_price_current_x64,
                liquidity,
                false,
            )?;
            //println!("1. swap_step.amount_out: {}", swap_step.amount_out);
        };
    } else {
        #[allow(clippy::nonminimal_bool)] // Warning!!! Don't listening to clippy
        if !(max && is_base_input) {
            swap_step.amount_in = liquidity_math::get_delta_amount_1_unsigned(
                sqrt_price_current_x64,
                swap_step.sqrt_price_next_x64,
                liquidity,
                true,
            )?
        };
        #[allow(clippy::nonminimal_bool)] // Warning!!! Don't listening to clippy
        if !(max || !is_base_input) {
            swap_step.amount_out = liquidity_math::get_delta_amount_0_unsigned(
                sqrt_price_current_x64,
                swap_step.sqrt_price_next_x64,
                liquidity,
                false,
            )?;
            //println!("1. swap_step.amount_out: {}", swap_step.amount_out);
        };
    }

    // For exact output case, cap the output amount to not exceed the remaining output amount
    if !is_base_input && swap_step.amount_out > amount_remaining {
        swap_step.amount_out = amount_remaining;
    }

    swap_step.fee_amount = if is_base_input && swap_step.sqrt_price_next_x64 != sqrt_price_target_x64 {
        // we didn't reach the target, so take the remainder of the maximum input as fee
        // swap dust is granted as fee
        amount_remaining
            .checked_sub(swap_step.amount_in)
            .ok_or_else(|| anyhow::format_err!("RaydiumCLMMComputeSwap2"))?
    } else {
        // take pip percentage as fee
        swap_step
            .amount_in
            .mul_div_ceil(fee_rate.into(), (FEE_RATE_DENOMINATOR_VALUE - fee_rate).into())
            .ok_or_else(|| anyhow::format_err!("RaydiumCLMMComputeSwap3"))?
    };

    Ok(swap_step)
}

/// Pre calcumate amount_in or amount_out for the specified price range
/// The amount maybe overflow of u64 due to the `sqrt_price_target_x64` maybe unreasonable.
/// Therefore, this situation needs to be handled in `compute_swap_step` to recalculate the price that can be reached based on the amount.
fn calculate_amount_in_range(
    sqrt_price_current_x64: u128,
    sqrt_price_target_x64: u128,
    liquidity: u128,
    zero_for_one: bool,
    is_base_input: bool,
) -> anyhow::Result<Option<u64>> {
    if is_base_input {
        let result = if zero_for_one {
            liquidity_math::get_delta_amount_0_unsigned(sqrt_price_target_x64, sqrt_price_current_x64, liquidity, true)
        } else {
            liquidity_math::get_delta_amount_1_unsigned(sqrt_price_current_x64, sqrt_price_target_x64, liquidity, true)
        };

        if let Ok(result) = result {
            Ok(Some(result))
        } else if result
            .err()
            .ok_or_else(|| anyhow::format_err!("RaydiumCLMMCalculateAmountInRange1"))?
            .to_string()
            .contains("MaxTokenOverflow")
        {
            Ok(None)
        } else {
            Err(anyhow::format_err!("SqrtPriceLimitOverflow"))
        }
    } else {
        let result = if zero_for_one {
            liquidity_math::get_delta_amount_1_unsigned(sqrt_price_target_x64, sqrt_price_current_x64, liquidity, false)
        } else {
            liquidity_math::get_delta_amount_0_unsigned(sqrt_price_current_x64, sqrt_price_target_x64, liquidity, false)
        };
        if let Ok(result) = result {
            Ok(Some(result))
        } else if result
            .err()
            .ok_or_else(|| anyhow::format_err!("RaydiumCLMMCalculateAmountInRange2"))?
            .to_string()
            .contains("MaxTokenOverflow")
        {
            Ok(None)
        } else {
            Err(anyhow::format_err!("SqrtPriceLimitOverflow"))
        }
    }
}
