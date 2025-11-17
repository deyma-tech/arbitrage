use super::math::U128;
use super::state::AmmInfo;
use super::{math, state};
use solana_program::pubkey::Pubkey;

use anyhow::{format_err, Result};

use super::math::CheckedCeilDiv;
use solana_program::pubkey;
use std::convert::TryFrom;

pub const TEN_THOUSAND: u128 = 10000;
pub const RAYDIUM_AUTHORITY_V4: Pubkey = pubkey!("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1");
//
pub fn amount_with_slippage(amount: u64, slippage_bps: u64, up_towards: bool) -> anyhow::Result<u64> {
    let amount = amount as u128;
    let slippage_bps = slippage_bps as u128;
    let amount_with_slippage = if up_towards {
        amount
            .checked_mul(
                slippage_bps
                    .checked_add(TEN_THOUSAND)
                    .ok_or_else(|| anyhow::format_err!("RaydiumAMMAmountWithSlippageOverflow"))?,
            )
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMAmountWithSlippageOverflow"))?
            .checked_div(TEN_THOUSAND)
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMAmountWithSlippageOverflow"))?
    } else {
        amount
            .checked_mul(
                TEN_THOUSAND
                    .checked_sub(slippage_bps)
                    .ok_or_else(|| anyhow::format_err!("RaydiumAMMAmountWithSlippageOverflow"))?,
            )
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMAmountWithSlippageOverflow"))?
            .checked_div(TEN_THOUSAND)
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMAmountWithSlippageOverflow"))?
    };
    u64::try_from(amount_with_slippage).map_err(|_| format_err!("failed to read keypair from {}", amount_with_slippage))
}

pub fn swap_exact_amount(
    pc_vault_amount: u64,
    coin_vault_amount: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
    swap_direction: state::SwapDirection,
    amount_specified: u64,
    swap_base_in: bool,
) -> Result<u64> {
    let other_amount_threshold = if swap_base_in {
        let swap_fee = U128::from(amount_specified)
            .checked_mul(swap_fee_numerator.into())
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMComputeSwap1"))?
            .checked_ceil_div(swap_fee_denominator.into())
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMComputeSwap2"))?
            .0;
        let swap_in_after_deduct_fee = U128::from(amount_specified)
            .checked_sub(swap_fee)
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMComputeSwap3"))?;
        math::Calculator::swap_token_amount_base_in(
            swap_in_after_deduct_fee,
            pc_vault_amount.into(),
            coin_vault_amount.into(),
            swap_direction,
        )
        .map_err(|e| format_err!("RaydiumAMMSwapExactAmount1: {}", e))?
        .as_u64()
    } else {
        let swap_in_before_add_fee = math::Calculator::swap_token_amount_base_out(
            amount_specified.into(),
            pc_vault_amount.into(),
            coin_vault_amount.into(),
            swap_direction,
        )
        .map_err(|e| format_err!("RaydiumAMMSwapExactAmount2: {}", e))?;
        swap_in_before_add_fee
            .checked_mul(swap_fee_denominator.into())
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMComputeSwap4"))?
            .checked_ceil_div(
                (swap_fee_denominator
                    .checked_sub(swap_fee_numerator)
                    .ok_or_else(|| anyhow::format_err!("RaydiumAMMComputeSwap5"))?)
                .into(),
            )
            .ok_or_else(|| anyhow::format_err!("RaydiumAMMComputeSwap6"))?
            .0
            .as_u64()
    };

    Ok(other_amount_threshold)
}

#[allow(clippy::too_many_arguments)]
pub fn swap_with_slippage(
    pc_vault_amount: u64,
    coin_vault_amount: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
    swap_direction: state::SwapDirection,
    amount_specified: u64,
    swap_base_in: bool,
    slippage_bps: u64,
) -> Result<u64> {
    let other_amount_threshold = swap_exact_amount(
        pc_vault_amount,
        coin_vault_amount,
        swap_fee_numerator,
        swap_fee_denominator,
        swap_direction,
        amount_specified,
        swap_base_in,
    )?;
    let other_amount_threshold = if swap_base_in {
        // min out
        amount_with_slippage(other_amount_threshold, slippage_bps, false)?
    } else {
        // max in
        amount_with_slippage(other_amount_threshold, slippage_bps, true)?
    };
    Ok(other_amount_threshold)
}
//
#[derive(Clone, Debug, PartialEq)]
pub struct AmmSwapInfoResult {
    pub pool_id: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub amm_coin_vault: Pubkey,
    pub amm_pc_vault: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub market_program: Pubkey,
    pub market: Pubkey,
    pub market_coin_vault: Pubkey,
    pub market_pc_vault: Pubkey,
    pub market_vault_signer: Pubkey,
    pub market_event_queue: Pubkey,
    pub market_bids: Pubkey,
    pub market_asks: Pubkey,
    pub amount_specified: u64,
    pub other_amount_threshold: u64,
}

#[allow(clippy::too_many_arguments)]
pub fn calculate_swap_info(
    pool_id: Pubkey,
    amm_info: &AmmInfo,
    pc_vault_amount: u64,   // &TokenAccount,
    coin_vault_amount: u64, //  &TokenAccount,
    input_token: Pubkey,
    amount_specified: u64,
    slippage_bps: u64,
    base_in: bool,
) -> anyhow::Result<AmmSwapInfoResult> {
    // load amm keys
    // reload accounts data to calculate amm pool vault amount
    // get multiple accounts at the same time to ensure data consistency
    // calculate pool vault amount without take pnl
    let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
        math::Calculator::calc_total_without_take_pnl_no_orderbook(pc_vault_amount, coin_vault_amount, amm_info)
            .map_err(|e| {
                format_err!(
                    "calculate_swap_info calc_total_without_take_pnl_no_orderbook error: {}",
                    e
                )
            })?;

    let (swap_direction, input_mint, output_mint) = if input_token == amm_info.coin_vault_mint {
        (
            state::SwapDirection::Coin2PC,
            amm_info.coin_vault_mint,
            amm_info.pc_vault_mint,
        )
    } else if input_token == amm_info.pc_vault_mint {
        (
            state::SwapDirection::PC2Coin,
            amm_info.pc_vault_mint,
            amm_info.coin_vault_mint,
        )
    } else {
        panic!("input tokens not match pool vaults");
    };
    let other_amount_threshold = swap_with_slippage(
        amm_pool_pc_vault_amount,
        amm_pool_coin_vault_amount,
        amm_info.fees.swap_fee_numerator,
        amm_info.fees.swap_fee_denominator,
        swap_direction,
        amount_specified,
        base_in,
        slippage_bps,
    )?;

    Ok(AmmSwapInfoResult {
        pool_id,
        amm_authority: RAYDIUM_AUTHORITY_V4,
        amm_open_orders: amm_info.open_orders,
        amm_coin_vault: amm_info.coin_vault,
        amm_pc_vault: amm_info.pc_vault,
        input_mint,
        output_mint,
        // amm_authority,
        market_program: RAYDIUM_AUTHORITY_V4, //amm_info.market_program, // padding readonly account
        market: RAYDIUM_AUTHORITY_V4,         //amm_info.market,              // padding readwrite account
        market_coin_vault: RAYDIUM_AUTHORITY_V4, //amm_info.open_orders, // padding readwrite account
        market_pc_vault: RAYDIUM_AUTHORITY_V4, //amm_info.open_orders, // padding readwrite account
        // amm_authority,
        market_vault_signer: RAYDIUM_AUTHORITY_V4, //amm_info.open_orders, // padding readonly account
        market_event_queue: RAYDIUM_AUTHORITY_V4,  //amm_info.open_orders,  // padding readwrite account
        market_bids: RAYDIUM_AUTHORITY_V4,         //amm_info.open_orders,         // padding readwrite account
        market_asks: RAYDIUM_AUTHORITY_V4,         //amm_info.open_orders,         // padding readwrite account
        amount_specified,
        other_amount_threshold,
    })
}
