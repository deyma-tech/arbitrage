use super::typedefs::ActivationType;
use super::typedefs::Bin;
use super::typedefs::PairStatus;
use super::typedefs::PairType;
use super::LbPair;
use super::SwapResult;
use super::{BinArray, BinArrayBitmapExtension};

use super::constants::{MAX_BIN_ID, MIN_BIN_ID};
use super::math::SafeMath;
use anyhow::{ensure, Context, Result};
use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;

use utils::quote::QuoteResult;

#[derive(Debug)]
pub struct SwapExactInQuote {
    pub amount_out: u64,
    pub fee: u64,
    pub remaining: u64,
}

#[derive(Debug)]
pub struct SwapExactOutQuote {
    pub amount_in: u64,
    pub fee: u64,
    pub remaining: u64,
}

fn validate_swap_activation(lb_pair: &LbPair, current_timestamp: u64, current_slot: u64) -> Result<()> {
    ensure!(lb_pair.status()?.eq(&PairStatus::Enabled), "Pair is disabled");

    let pair_type = lb_pair.pair_type().context("Invalid pair type")?;
    if pair_type.eq(&PairType::Permission) {
        let activation_type = ActivationType::try_from(lb_pair.activation_type).context("Invalid activation type")?;
        let current_point = match activation_type {
            ActivationType::Slot => current_slot,
            ActivationType::Timestamp => current_timestamp,
        };

        ensure!(current_point >= lb_pair.activation_point, "Pair is disabled");
    }

    Ok(())
}

pub fn advance_active_bin(active_id: i32, swap_for_y: bool) -> Result<i32> {
    let next_active_bin_id = if swap_for_y {
        active_id.safe_sub(1)?
    } else {
        active_id.safe_add(1)?
    };

    if !(MIN_BIN_ID..=MAX_BIN_ID).contains(&next_active_bin_id) {
        return Err(anyhow::format_err!("bin outside of range"));
    }
    Ok(next_active_bin_id)
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteInput<'a> {
    pub bin_arrays: &'a BTreeMap<i32, (Pubkey, BinArray)>,
    pub bitmap_extension: Option<&'a BinArrayBitmapExtension>,
    pub current_slot: u64,
}

const CU_BASE: u32 = 50_500;
const CU_PER_BIN: u32 = 150;
const CU_PER_ARRAY: u32 = 450;
const CU_PER_SWAP: u32 = 5_250;

pub fn quote_exact_out_bitmap(
    lb_pair: &LbPair,
    mut amount_out: u64,
    a_to_b: bool,
    bin_arrays: &BTreeMap<i32, (Pubkey, BinArray)>,
    bitmap_extension: Option<&BinArrayBitmapExtension>,
    current_timestamp: u64,
    current_slot: u64,
) -> Result<QuoteResult> {
    validate_swap_activation(lb_pair, current_timestamp, current_slot)?;

    let mut lb_pair = *lb_pair;
    lb_pair.update_references(current_timestamp as i64)?;

    let mut total_amount_in: u64 = 0;
    let mut total_fee: u64 = 0;

    let mut quote_result = QuoteResult {
        indices: Some(vec![]),
        compute_units: CU_BASE,
        ..Default::default()
    };

    let token_amount = amount_out;

    while amount_out > 0 {
        quote_result.compute_units += CU_PER_ARRAY;

        let active_bin_array_index = get_bin_array_indices_for_swap(&lb_pair, bitmap_extension, a_to_b, 1)?
            .pop()
            .context("MeteoraPoolOutOfLiquidity quote_exact_out_bitmap")?;

        if let Some(indices) = &mut quote_result.indices {
            indices.push(active_bin_array_index);
            let has_duplicity = indices.windows(2).any(|win| win[0] == win[1]);
            /*indices.len() > 6*/
            if has_duplicity {
                indices.dedup();
                break;
            }
        }

        let mut active_bin_array = bin_arrays
            .get(&active_bin_array_index)
            .cloned()
            .context("Active bin array not found")?
            .1;

        loop {
            quote_result.compute_units += CU_PER_BIN;
            if active_bin_array.is_bin_id_within_range(lb_pair.active_id).is_err() || amount_out == 0 {
                break;
            }

            //lb_pair.update_volatility_accumulator()?;

            let active_bin = active_bin_array.get_bin_mut(lb_pair.active_id)?;
            let price = active_bin.get_or_store_bin_price(lb_pair.active_id, lb_pair.bin_step)?;

            if !active_bin.is_empty(!a_to_b) {
                quote_result.compute_units += CU_PER_SWAP;

                let bin_max_amount_out = active_bin.get_max_amount_out(a_to_b);
                if amount_out >= bin_max_amount_out {
                    let max_amount_in = active_bin.get_max_amount_in(price, a_to_b)?;
                    let max_fee = lb_pair.compute_fee(max_amount_in)?;

                    total_amount_in = total_amount_in.checked_add(max_amount_in).context("MathOverflow")?;

                    total_fee = total_fee.checked_add(max_fee).context("MathOverflow")?;

                    amount_out = amount_out.checked_sub(bin_max_amount_out).context("MathOverflow")?;
                } else {
                    let amount_in = Bin::get_amount_in(amount_out, price, a_to_b)?;
                    let fee = lb_pair.compute_fee(amount_in)?;

                    total_amount_in = total_amount_in.checked_add(amount_in).context("MathOverflow")?;

                    total_fee = total_fee.checked_add(fee).context("MathOverflow")?;

                    amount_out = 0;
                }
            }

            if amount_out > 0 {
                lb_pair.advance_active_bin(a_to_b)?;
            }
        }
    }

    // exact_out
    if a_to_b {
        quote_result.amount_a_in = total_amount_in + total_fee;
        quote_result.amount_b_out = token_amount;
    } else {
        quote_result.amount_b_in = total_amount_in + total_fee;
        quote_result.amount_a_out = token_amount;
    }

    Ok(quote_result)
}

// 1 active bin = 5_200, base = 60_200 + ((#active_bin - 1) * 5_200)
pub fn quote_exact_in_bitmap(
    lb_pair: &LbPair,
    mut amount_in: u64,
    a_to_b: bool,
    bin_arrays: &BTreeMap<i32, (Pubkey, BinArray)>,
    bitmap_extension: Option<&BinArrayBitmapExtension>,
    current_timestamp: u64,
    current_slot: u64,
) -> Result<QuoteResult> {
    //let mut counter = 0;

    validate_swap_activation(lb_pair, current_timestamp, current_slot)
        .map_err(|e| anyhow::format_err!("quote_exact_in_bitmap: validate_swap_activation error: {:?}", e))?;

    let mut lb_pair = *lb_pair;
    lb_pair
        .update_references(current_timestamp as i64)
        .context("Update references")?;

    let mut total_amount_out: u64 = 0;
    let mut total_fee: u64 = 0;

    let mut qoute_result = QuoteResult {
        indices: Some(vec![]),
        compute_units: CU_BASE,
        ..Default::default()
    };

    let token_amount = amount_in;

    while amount_in > 0 {
        qoute_result.compute_units += CU_PER_ARRAY;

        let active_bin_array_index = get_bin_array_indices_for_swap(&lb_pair, bitmap_extension, a_to_b, 1)?
            .pop()
            .context("MeteoraPoolOutOfLiquidity quote_exact_in_bitmap")?;

        if let Some(indices) = &mut qoute_result.indices {
            indices.push(active_bin_array_index);
            let has_duplicity = indices.windows(2).any(|win| win[0] == win[1]);
            if has_duplicity {
                indices.dedup();
                break;
            }
        }

        let mut active_bin_array = bin_arrays
            .get(&active_bin_array_index)
            .cloned()
            .context("Active bin array not found")?
            .1;

        loop {
            qoute_result.compute_units += CU_PER_BIN;

            if active_bin_array.is_bin_id_within_range(lb_pair.active_id).is_err() || amount_in == 0 {
                break;
            }

            lb_pair.update_volatility_accumulator()?;

            let active_bin = active_bin_array
                .get_bin_mut(lb_pair.active_id)
                .context("Get active bin")?;
            let price = active_bin
                .get_or_store_bin_price(lb_pair.active_id, lb_pair.bin_step)
                .context("Get bin price")?;

            if !active_bin.is_empty(!a_to_b) {
                qoute_result.compute_units += CU_PER_SWAP;

                let SwapResult {
                    amount_in_with_fees,
                    amount_out,
                    fee,
                    ..
                } = active_bin
                    .swap(amount_in, price, a_to_b, &lb_pair, None)
                    .context("Swap")?;

                amount_in = amount_in.checked_sub(amount_in_with_fees).context("MathOverflow")?;

                total_amount_out = total_amount_out.checked_add(amount_out).context("MathOverflow")?;
                total_fee = total_fee.checked_add(fee).context("MathOverflow")?;
            }

            if amount_in > 0 {
                lb_pair.advance_active_bin(a_to_b).context("Advance active bin")?;
            }
        }
    }

    // out
    if a_to_b {
        qoute_result.amount_a_in = token_amount;
        qoute_result.amount_b_out = total_amount_out;
    } else {
        qoute_result.amount_b_in = token_amount;
        qoute_result.amount_a_out = total_amount_out;
    }

    Ok(qoute_result)
}

pub fn get_bin_array_indices_for_swap(
    lb_pair: &LbPair,
    bitmap_extension: Option<&BinArrayBitmapExtension>,
    swap_for_y: bool,
    take_count: u8,
) -> Result<Vec<i32>> {
    let mut start_bin_array_idx = BinArray::bin_id_to_bin_array_index(lb_pair.active_id)?;
    let mut bin_array_idx = vec![];
    let increment = if swap_for_y { -1 } else { 1 };
    loop {
        if bin_array_idx.len() == take_count as usize {
            break;
        }

        if lb_pair.is_overflow_default_bin_array_bitmap(start_bin_array_idx) {
            let Some(bitmap_extension) = bitmap_extension else {
                break;
            };
            let Ok((next_bin_array_idx, has_liquidity)) =
                bitmap_extension.next_bin_array_index_with_liquidity(swap_for_y, start_bin_array_idx)
            else {
                // Out of search range. No liquidity.
                break;
            };
            if has_liquidity {
                bin_array_idx.push(next_bin_array_idx);
                start_bin_array_idx = next_bin_array_idx + increment;
            } else {
                // Switch to internal bitmap
                start_bin_array_idx = next_bin_array_idx;
            }
        } else {
            let Ok((next_bin_array_idx, has_liquidity)) =
                lb_pair.next_bin_array_index_with_liquidity_internal(swap_for_y, start_bin_array_idx)
            else {
                break;
            };
            if has_liquidity {
                bin_array_idx.push(next_bin_array_idx);
                start_bin_array_idx = next_bin_array_idx + increment;
            } else {
                // Switch to external bitmap
                start_bin_array_idx = next_bin_array_idx;
            }
        }
    }
    Ok(bin_array_idx)
}
// eof
