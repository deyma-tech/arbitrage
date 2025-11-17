use crate::pump_amm::QuoteInput;
use crate::raydium_amm::CheckedCeilDiv;
use anyhow::{format_err, Context};
use num_traits::ToPrimitive;
use utils::quote::QuoteResult;

pub const PRECISION: u128 = 10_000;

// BUY: b -> a (exact_in)
// SELL: a -> b (exact_in)
pub fn quote(a_to_b: bool, exact_in: bool, amount: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
    let mut quote_result = QuoteResult {
        amount_a_in: 0,
        amount_b_in: 0,
        amount_a_out: 0,
        amount_b_out: 0,
        indices: None,
        compute_units: 63_000,
    };
    if a_to_b {
        // from base to quote !!!
        if exact_in {
            if amount > input.base {
                return Err(format_err!("PumpAMMPoolOutOfLiquidity"));
            }
            let mut amount_out = swap_base_input_without_fees(amount as u128, input.base as u128, input.quote as u128)?;
            // minus fee - quote/b
            amount_out = amount_out - ((amount_out / PRECISION) * input.fee as u128);
            quote_result.amount_a_in = amount;
            quote_result.amount_b_out = amount_out.to_u64().context("CastFailed")?;
        } else {
            // real amount - fee because fee is taken from quote
            let amount_minus_fee = amount as u128 - ((amount as u128 / PRECISION) * input.fee as u128);
            // base/a to exact out
            let amount_in = swap_base_output_without_fees(amount_minus_fee, input.base as u128, input.quote as u128)?;
            // minus fee
            quote_result.amount_a_in = amount_in.to_u64().context("CastFailed")?;
            quote_result.amount_b_out = amount.to_u64().context("CastFailed")?;
        }
    } else {
        // from quote to base !!!
        if exact_in {
            // quote - fee
            if amount > input.quote {
                return Err(format_err!("PumpAMMPoolOutOfLiquidity"));
            }
            let amount_minus_fee = amount as u128 - ((amount as u128 / PRECISION) * input.fee as u128);
            // base
            let amount_out = swap_base_input_without_fees(amount_minus_fee, input.quote as u128, input.base as u128)?;
            // minus fee - quote/b
            quote_result.amount_b_in = amount;
            quote_result.amount_a_out = amount_out.to_u64().context("CastFailed")?;
        } else {
            // real amount - fee because fee is taken from quote
            // base
            let mut amount_in = swap_base_output_without_fees(amount as u128, input.base as u128, input.quote as u128)?;
            // quote + fee
            amount_in = amount_in as u128 + ((amount_in / PRECISION) * input.fee as u128);
            // minus fee
            quote_result.amount_b_in = amount_in.to_u64().context("CastFailed")?;
            quote_result.amount_a_out = amount.to_u64().context("CastFailed")?;
        }
    }
    Ok(quote_result)
}

// fee is quote amount!
pub fn swap_base_input_without_fees(
    source_amount: u128,
    swap_source_amount: u128,      // token account
    swap_destination_amount: u128, // token account
) -> anyhow::Result<u128> {
    // (x + delta_x) * (y - delta_y) = x * y
    // delta_y = (delta_x * y) / (x + delta_x)
    let numerator = source_amount.checked_mul(swap_destination_amount).context("overflow")?;
    let denominator = swap_source_amount.checked_add(source_amount).context("overflow")?;
    let destination_amount_swapped = numerator.checked_div(denominator).context("overflow")?;
    Ok(destination_amount_swapped)
}

pub fn swap_base_output_without_fees(
    destination_amount: u128,
    swap_source_amount: u128,      // token account
    swap_destination_amount: u128, // token account
) -> anyhow::Result<u128> {
    // (x + delta_x) * (y - delta_y) = x * y
    // delta_x = (x * delta_y) / (y - delta_y)
    let numerator = swap_source_amount.checked_mul(destination_amount).context("overflow")?;
    let denominator = swap_destination_amount
        .checked_sub(destination_amount)
        .context("overflow")?;
    let (source_amount_swapped, _) = numerator.checked_ceil_div(denominator).context("0verflow")?;
    Ok(source_amount_swapped)
}
