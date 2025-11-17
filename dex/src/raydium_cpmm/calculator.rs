use crate::raydium_amm::CheckedCeilDiv;
use crate::raydium_cpmm::fees::Fees;

use anyhow::Result;

/// ConstantProductCurve struct implementing CurveCalculator
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConstantProductCurve;

/// Encodes all results of swapping from a source token to a destination token
#[derive(Debug, PartialEq)]
pub struct SwapResult {
    /// New amount of source token
    pub new_swap_source_amount: u128,
    /// New amount of destination token
    pub new_swap_destination_amount: u128,
    /// Amount of source token swapped (includes fees)
    pub source_amount_swapped: u128,
    /// Amount of destination token swapped
    pub destination_amount_swapped: u128,
    /// Amount of source tokens going to pool holders
    pub trade_fee: u128,
    /// Amount of source tokens going to protocol
    pub protocol_fee: u128,
    /// Amount of source tokens going to protocol team
    pub fund_fee: u128,
}

impl ConstantProductCurve {
    /// Constant product swap ensures x * y = constant
    /// The constant product swap calculation, factored out of its class for reuse.
    ///
    /// This is guaranteed to work for all values such that:
    ///  - 1 <= swap_source_amount * swap_destination_amount <= u128::MAX
    ///  - 1 <= source_amount <= u64::MAX
    pub fn swap_base_input_without_fees(
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
    ) -> Result<u128> {
        // (x + delta_x) * (y - delta_y) = x * y
        // delta_y = (delta_x * y) / (x + delta_x)

        // Original code
        // let numerator = source_amount.checked_mul(swap_destination_amount).un_wrap();
        // let denominator = swap_source_amount.checked_add(source_amount).un_wrap();
        // numerator.checked_div(denominator).un_wrap()

        let numerator = source_amount
            .checked_mul(swap_destination_amount)
            .ok_or_else(|| anyhow::anyhow!("MathOverflow"))?;
        let denominator = swap_source_amount
            .checked_add(source_amount)
            .ok_or_else(|| anyhow::anyhow!("MathOverflow"))?;
        let result = numerator
            .checked_div(denominator)
            .ok_or_else(|| anyhow::anyhow!("MathOverflow"))?;
        Ok(result)
    }

    pub fn swap_base_output_without_fees(
        destination_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
    ) -> Result<u128> {
        // (x + delta_x) * (y - delta_y) = x * y
        // delta_x = (x * delta_y) / (y - delta_y)

        // Original code
        // let numerator = swap_source_amount.checked_mul(destination_amount).un_wrap();
        // let denominator = swap_destination_amount
        //     .checked_sub(destination_amount)
        //     .un_wrap();
        // let (source_amount_swapped, _) = numerator.checked_ceil_div(denominator).un_wrap();
        // source_amount_swapped

        let numerator = swap_source_amount
            .checked_mul(destination_amount)
            .ok_or_else(|| anyhow::anyhow!("MathOverflow"))?;
        let denominator = swap_destination_amount
            .checked_sub(destination_amount)
            .ok_or_else(|| anyhow::anyhow!("MathOverflow"))?;
        let (source_amount_swapped, _) = numerator
            .checked_ceil_div(denominator)
            .ok_or_else(|| anyhow::anyhow!("MathOverflow"))?;
        Ok(source_amount_swapped)
    }
}

pub fn swap_base_input(
    source_amount: u128,
    swap_source_amount: u128,
    swap_destination_amount: u128,
    trade_fee_rate: u64,
    protocol_fee_rate: u64,
    fund_fee_rate: u64,
) -> Option<SwapResult> {
    // debit the fee to calculate the amount swapped
    let trade_fee = Fees::trading_fee(source_amount, trade_fee_rate)?;
    let protocol_fee = Fees::protocol_fee(trade_fee, protocol_fee_rate)?;
    let fund_fee = Fees::fund_fee(trade_fee, fund_fee_rate)?;

    let source_amount_less_fees = source_amount.checked_sub(trade_fee)?;

    if let Ok(destination_amount_swapped) = ConstantProductCurve::swap_base_input_without_fees(
        source_amount_less_fees,
        swap_source_amount,
        swap_destination_amount,
    ) {
        Some(SwapResult {
            new_swap_source_amount: swap_source_amount.checked_add(source_amount)?,
            new_swap_destination_amount: swap_destination_amount.checked_sub(destination_amount_swapped)?,
            source_amount_swapped: source_amount,
            destination_amount_swapped,
            trade_fee,
            protocol_fee,
            fund_fee,
        })
    } else {
        None
    }
}

pub fn swap_base_output(
    destination_amount: u128,
    swap_source_amount: u128,
    swap_destination_amount: u128,
    trade_fee_rate: u64,
    protocol_fee_rate: u64,
    fund_fee_rate: u64,
) -> Option<SwapResult> {
    if let Ok(source_amount_swapped) = ConstantProductCurve::swap_base_output_without_fees(
        destination_amount,
        swap_source_amount,
        swap_destination_amount,
    ) {
        if let Some(source_amount) = Fees::calculate_pre_fee_amount(source_amount_swapped, trade_fee_rate) {
            let trade_fee = Fees::trading_fee(source_amount, trade_fee_rate)?;
            let protocol_fee = Fees::protocol_fee(trade_fee, protocol_fee_rate)?;
            let fund_fee = Fees::fund_fee(trade_fee, fund_fee_rate)?;

            Some(SwapResult {
                new_swap_source_amount: swap_source_amount.checked_add(source_amount)?,
                new_swap_destination_amount: swap_destination_amount.checked_sub(destination_amount)?,
                source_amount_swapped: source_amount,
                destination_amount_swapped: destination_amount,
                trade_fee,
                protocol_fee,
                fund_fee,
            })
        } else {
            None
        }
    } else {
        None
    }
}
