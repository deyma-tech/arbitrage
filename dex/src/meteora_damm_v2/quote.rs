use anyhow::Ok;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Type of the activation
pub enum ActivationType {
    Slot,
    Timestamp,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum TradeDirection {
    /// Input token A, output token B
    AtoB,
    /// Input token B, output token A
    BtoA,
}

/// collect fee mode
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum CollectFeeMode {
    /// Both token, in this mode only out token is collected
    BothToken,
    /// Only token B, we just need token B, because if user want to collect fee in token A, they just need to flip order of tokens
    OnlyB,
}

#[derive(Default, Debug)]
pub struct FeeMode {
    pub fees_on_input: bool,
    pub fees_on_token_a: bool,
    pub has_referral: bool,
}

impl FeeMode {
    pub fn get_fee_mode(
        collect_fee_mode: u8,
        trade_direction: TradeDirection,
        has_referral: bool,
    ) -> anyhow::Result<FeeMode> {
        let collect_fee_mode = CollectFeeMode::try_from(collect_fee_mode)
            .map_err(|_| anyhow::format_err!("MeteoraDAMMInvalidCollectFeeMode"))?;

        let (fees_on_input, fees_on_token_a) = match (collect_fee_mode, trade_direction) {
            // When collecting fees on output token
            (CollectFeeMode::BothToken, TradeDirection::AtoB) => (false, false),
            (CollectFeeMode::BothToken, TradeDirection::BtoA) => (false, true),

            // When collecting fees on tokenB
            (CollectFeeMode::OnlyB, TradeDirection::AtoB) => (false, false),
            (CollectFeeMode::OnlyB, TradeDirection::BtoA) => (true, false),
        };

        Ok(FeeMode {
            fees_on_input,
            fees_on_token_a,
            has_referral,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct FeeOnAmountResult {
    pub amount: u64,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

// pub fn get_quote(
//     pool: &PoolState,
//     current_timestamp: u64,
//     current_slot: u64,
//     actual_amount_in: u64,
//     a_to_b: bool,
//     has_referral: bool,
// ) -> Result<SwapResult> {
//     ensure!(actual_amount_in > 0, "amount is zero");

//     let result = if pool.pool_fees.dynamic_fee.is_dynamic_fee_enable() {
//         let mut pool = pool.clone();
//         pool.update_pre_swap(current_timestamp)?;
//         get_internal_quote(
//             &pool,
//             current_timestamp,
//             current_slot,
//             actual_amount_in,
//             a_to_b,
//             has_referral,
//         )
//     } else {
//         get_internal_quote(
//             pool,
//             current_timestamp,
//             current_slot,
//             actual_amount_in,
//             a_to_b,
//             has_referral,
//         )
//     };

//     result
// }

// fn get_internal_quote(
//     pool: &PoolState,
//     current_timestamp: u64,
//     current_slot: u64,
//     actual_amount_in: u64,
//     a_to_b: bool,
//     has_referral: bool,
// ) -> anyhow::Result<SwapResult> {
//     let activation_type =
//         ActivationType::try_from(pool.activation_type).context("invalid activation type")?;

//     let current_point = match activation_type {
//         ActivationType::Slot => current_slot,
//         ActivationType::Timestamp => current_timestamp,
//     };

//     let trade_direction = if a_to_b {
//         TradeDirection::AtoB
//     } else {
//         TradeDirection::BtoA
//     };

//     let fee_mode = &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, has_referral)?;

//     let swap_result =
//         pool.get_swap_result(actual_amount_in, fee_mode, trade_direction, current_point)?;

//     Ok(swap_result)
// }
