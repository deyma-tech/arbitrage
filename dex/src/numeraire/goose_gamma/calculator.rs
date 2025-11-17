#![allow(warnings)]
use crate::goose_gamma::accounts::ObservationState;
use crate::goose_gamma::accounts::{AmmConfig, PoolState};
use crate::goose_gamma::fee::{DynamicFee, FeeType, StaticFee};
use crate::raydium_amm::CheckedCeilDiv;
use anyhow::{format_err, Context};
//use spl_token_swap::curve::calculator::{RoundDirection, TradingTokenResult};

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
    /// Dynamic fee charged for trade
    pub dynamic_fee: u128,
    /// Amount of source tokens going to protocol
    pub protocol_fee: u128,
    /// Amount of source tokens going to protocol team
    pub fund_fee: u128,
    /// Dynamic fee rate
    pub dynamic_fee_rate: u64,
}

/// Concrete struct to wrap around the trait object which performs calculation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurveCalculator {}

impl CurveCalculator {
    pub fn validate_supply(token_0_amount: u64, token_1_amount: u64) -> anyhow::Result<()> {
        if token_0_amount == 0 {
            return Err(format_err!("InvalidSupplyA"));
        }
        if token_1_amount == 0 {
            return Err(format_err!("InvalidSupplyB"));
        }
        Ok(())
    }

    /// Subtract fees and calculate how much destination token will be received
    /// for a given amount of source token

    pub fn swap_base_input(
        source_amount_to_be_swapped: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        amm_config: &AmmConfig,
        pool_state: &PoolState,
        block_timestamp: u64,
        observation_state: &ObservationState,
        // This is to indicate that the trade is not a toxic trade and is coming to us from a signed dflow segmenter.
        // It is planed to charge an additional fee for this trade if it is false in future.
        is_invoked_by_signed_segmenter: bool,
        // TODO: add fee type here once that is configurable on pool level/ or we can use it from pool_state
    ) -> anyhow::Result<SwapResult> {
        let (dynamic_fee, dynamic_fee_rate) = DynamicFee::dynamic_fee(
            source_amount_to_be_swapped,
            block_timestamp,
            observation_state,
            FeeType::Volatility,
            amm_config.trade_fee_rate,
            pool_state,
            is_invoked_by_signed_segmenter,
        )?;

        let protocol_fee = StaticFee::protocol_fee(dynamic_fee, amm_config.protocol_fee_rate).context("Overflow")?;
        let fund_fee = StaticFee::fund_fee(dynamic_fee, amm_config.fund_fee_rate).context("Overflow")?;

        let source_amount_after_fees = source_amount_to_be_swapped
            .checked_sub(dynamic_fee)
            .context("Overflow")?;
        let destination_amount_swapped = ConstantProductCurve::swap_base_input_without_fees(
            source_amount_after_fees,
            swap_source_amount,
            swap_destination_amount,
        )?;

        Ok(SwapResult {
            new_swap_source_amount: swap_source_amount
                .checked_add(source_amount_to_be_swapped)
                .context("Overflow")?,
            new_swap_destination_amount: swap_destination_amount
                .checked_sub(destination_amount_swapped)
                .context("Overflow")?,
            source_amount_swapped: source_amount_to_be_swapped,
            destination_amount_swapped,
            dynamic_fee,
            protocol_fee,
            fund_fee,
            dynamic_fee_rate,
        })
    }

    /// Subtract fees and calculate how much source token will be required
    pub fn swap_base_output(
        destination_amount_to_be_swapped: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        amm_config: &AmmConfig,
        pool_state: &PoolState,
        block_timestamp: u64,
        observation_state: &ObservationState,
        is_invoked_by_signed_segmenter: bool,
    ) -> anyhow::Result<SwapResult> {
        let source_amount_swapped = ConstantProductCurve::swap_base_output_without_fees(
            destination_amount_to_be_swapped,
            swap_source_amount,
            swap_destination_amount,
        )?;

        let (source_amount, dynamic_fee_rate) = DynamicFee::calculate_pre_fee_amount(
            block_timestamp,
            source_amount_swapped,
            observation_state,
            FeeType::Volatility,
            amm_config.trade_fee_rate,
            pool_state,
            is_invoked_by_signed_segmenter,
        )?;

        let dynamic_fee = source_amount.checked_sub(source_amount_swapped).context("Overflow")?;
        let protocol_fee = StaticFee::protocol_fee(dynamic_fee, amm_config.protocol_fee_rate).context("Overflow")?;
        let fund_fee = StaticFee::fund_fee(dynamic_fee, amm_config.fund_fee_rate).context("Overflow")?;

        Ok(SwapResult {
            new_swap_source_amount: swap_source_amount.checked_add(source_amount).context("Overflow")?,
            new_swap_destination_amount: swap_destination_amount
                .checked_sub(destination_amount_to_be_swapped)
                .context("Overflow")?,
            source_amount_swapped: source_amount,
            destination_amount_swapped: destination_amount_to_be_swapped,
            protocol_fee,
            fund_fee,
            dynamic_fee,
            dynamic_fee_rate,
        })
    }
}

/// ConstantProductCurve struct implementing CurveCalculator
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConstantProductCurve;

impl ConstantProductCurve {
    /// ConstantProduct swap ensures x * y = constant
    /// The constant product swap calculation, factored out of it's class for reuse.
    ///
    /// This is guaranteed to work for all the values such that
    /// 1 <= swap_source_amount * swap_destination_amount <= u128::MAX
    /// 1 <= source_amount <= u64::MAX
    pub fn swap_base_input_without_fees(
        source_amount_to_be_swapped: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
    ) -> anyhow::Result<u128> {
        // (x + delta_x) * (y - delta_y) = x * y
        // delta_y = (delta_x * y) / (x + delta_x)
        let numerator = source_amount_to_be_swapped
            .checked_mul(swap_destination_amount)
            .context("Overflow")?;
        let denominator = swap_source_amount
            .checked_add(source_amount_to_be_swapped)
            .context("Overflow")?;
        let destination_amount_swapped = numerator.checked_div(denominator).context("Overflow")?;
        Ok(destination_amount_swapped)
    }

    pub fn swap_base_output_without_fees(
        destination_amount_to_be_swapped: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
    ) -> anyhow::Result<u128> {
        // (x + delta_x) * (y - delta_y) = x * y
        // delta_x = (x * delta_y) / (y - delta_y)
        let numerator = swap_source_amount
            .checked_mul(destination_amount_to_be_swapped)
            .context("Overflow")?;
        let denominator = swap_destination_amount
            .checked_sub(destination_amount_to_be_swapped)
            .context("Overflow")?;
        let (source_amount_swapped, _) = numerator.checked_ceil_div(denominator).context("Overflow")?;
        Ok(source_amount_swapped)
    }

    /*
    /// Get the amount of trading tokens(token_0 and token_1) for a given amount of pool tokens(lp_tokens)
    /// provided the total trading tokens and supply of pool tokens
    ///
    /// The constant product implementation is a simple ratio calcluations for the amount of trading tokens
    /// corresponding to a certain number of pool tokens.
    pub fn lp_tokens_to_trading_tokens(
        lp_token_amount: u128,
        lp_token_supply: u128,
        swap_token_0_amount: u128,
        swap_token_1_amount: u128,
        round_direction: RoundDirection,
    ) -> Option<TradingTokenResult> {
        // token_0_amount = (lp_token_amount * swap_token_0_amount) / lp_token_supply
        // lp_token_amount - Amount of pool tokens to be exchanged
        // swap_token_0_amount - Total token_0 amount in the pool
        let mut token_0_amount = lp_token_amount
            .checked_mul(swap_token_0_amount)?
            .checked_div(lp_token_supply)?;
        let mut token_1_amount = lp_token_amount
            .checked_mul(swap_token_1_amount)?
            .checked_div(lp_token_supply)?;
        let (token_a_amount, token_b_amount) = match round_direction {
            RoundDirection::Floor => (token_0_amount, token_1_amount),
            RoundDirection::Ceiling => {
                let token_0_remainder = lp_token_amount
                    .checked_mul(swap_token_0_amount)?
                    .checked_rem(lp_token_supply)?;
                // Also check for 0 token A and B amount to avoid taking too much
                // for tiny amounts of pool tokens.  For example, if someone asks
                // for 1 pool token, which is worth 0.01 token A, we avoid the
                // ceiling of taking 1 token A and instead return 0, for it to be
                // rejected later in processing.
                if token_0_remainder > 0 && token_0_amount > 0 {
                    token_0_amount = token_0_amount.checked_add(1)?;
                }
                let token_1_remainder = lp_token_amount
                    .checked_mul(swap_token_1_amount)?
                    .checked_rem(lp_token_supply)?;
                if token_1_remainder > 0 && token_1_amount > 0 {
                    token_1_amount = token_1_amount.checked_add(1)?;
                }
                (token_0_amount, token_1_amount)
            }
        };
        Some(TradingTokenResult {
            token_a_amount,
            token_b_amount,
        })
    }

    /// Get the amount of lp-tokens for a given amount of trading tokens
    pub fn token_0_to_lp_tokens(
        trading_token_0_amount: u128,
        total_token_0_amount: u128,
        lp_token_supply: u128,
    ) -> Option<u128> {
        let lp_token_amount = trading_token_0_amount
            .checked_mul(lp_token_supply)?
            .checked_div(total_token_0_amount)?;
        Some(lp_token_amount)
    }

    pub fn token_1_to_lp_tokens(
        trading_token_1_amount: u128,
        total_token_1_amount: u128,
        lp_token_supply: u128,
    ) -> Option<u128> {
        let lp_token_amount = trading_token_1_amount
            .checked_mul(lp_token_supply)?
            .checked_div(total_token_1_amount)?;
        Some(lp_token_amount)
    }
    */
}
