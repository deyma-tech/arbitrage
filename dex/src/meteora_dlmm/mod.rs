pub mod accounts;
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod pda;
pub mod quote;
pub mod typedefs;

pub use accounts::*;
//use constants::*;
use errors::*;
//use events::*;
//use instructions::*;
//use math::*;
//use pda::*;
//use quote::*;
use typedefs::*;

solana_program::declare_id!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

use constants::EXTENSION_BINARRAY_BITMAP_SIZE;
use constants::{BASIS_POINT_MAX, BIN_ARRAY_BITMAP_SIZE, FEE_PRECISION, MAX_BIN_PER_ARRAY, ONE, SCALE_OFFSET};
use math::{pow, safe_mul_shr_cast, safe_shl_div_cast, SafeMath};
use num_integer::Integer;
use ruint::aliases::U1024;
use ruint::aliases::U512;

use anyhow::{Context, Result};
use std::ops::Shl;
use std::ops::Shr;

impl StaticParameters {
    #[inline(always)]
    pub fn get_filter_period(&self) -> u16 {
        self.filter_period
    }

    #[inline(always)]
    pub fn get_decay_period(&self) -> u16 {
        self.decay_period
    }
}

impl VariableParameters {
    /// volatility_accumulator = min(volatility_reference + num_of_bin_crossed, max_volatility_accumulator)
    pub fn update_volatility_accumulator(&mut self, active_id: i32, static_params: &StaticParameters) -> Result<()> {
        // Upscale to prevent overflow caused by swapping from left most bin to right most bin.
        let delta_id = i64::from(self.index_reference)
            .safe_sub(active_id.into())?
            .unsigned_abs();

        let volatility_accumulator =
            u64::from(self.volatility_reference).safe_add(delta_id.safe_mul(constants::BASIS_POINT_MAX as u64)?)?;

        self.volatility_accumulator =
            std::cmp::min(volatility_accumulator, static_params.max_volatility_accumulator.into())
                .try_into()
                .map_err(|_| MeteoraDlmmProgramError::TypeCastFailed)?;

        Ok(())
    }

    /// Update id, and volatility reference
    pub fn update_references(
        &mut self,
        active_id: i32,
        current_timestamp: i64,
        static_params: &StaticParameters,
    ) -> Result<()> {
        let elapsed = current_timestamp.safe_sub(self.last_update_timestamp)?;

        // Not high frequency trade
        if elapsed >= static_params.get_filter_period() as i64 {
            // Update active id of last transaction
            self.index_reference = active_id;
            // filter period < t < decay_period. Decay time window.
            if elapsed < static_params.get_decay_period() as i64 {
                let volatility_reference = self
                    .volatility_accumulator
                    .safe_mul(static_params.reduction_factor as u32)?
                    .safe_div(constants::BASIS_POINT_MAX as u32)?;

                self.volatility_reference = volatility_reference;
            }
            // Out of decay time window
            else {
                self.volatility_reference = 0;
            }
        }

        // self.last_update_timestamp = current_timestamp;

        Ok(())
    }

    pub fn update_volatility_parameter(
        &mut self,
        active_id: i32,
        current_timestamp: i64,
        static_params: &StaticParameters,
    ) -> Result<()> {
        self.update_references(active_id, current_timestamp, static_params)?;
        self.update_volatility_accumulator(active_id, static_params)
    }
}

impl LbPair {
    pub fn status(&self) -> Result<PairStatus> {
        let pair_status: PairStatus = self
            .status
            .try_into()
            .map_err(|_| MeteoraDlmmProgramError::TypeCastFailed)?;

        Ok(pair_status)
    }

    pub fn pair_type(&self) -> Result<PairType> {
        let pair_type: PairType = self
            .pair_type
            .try_into()
            .map_err(|_| MeteoraDlmmProgramError::TypeCastFailed)?;

        Ok(pair_type)
    }

    pub fn update_references(&mut self, current_timestamp: i64) -> Result<()> {
        self.v_parameters
            .update_references(self.active_id, current_timestamp, &self.parameters)
    }

    pub fn update_volatility_accumulator(&mut self) -> Result<()> {
        self.v_parameters
            .update_volatility_accumulator(self.active_id, &self.parameters)
    }

    /// Total fee rate = base_fee_rate + variable_fee_rate
    pub fn get_total_fee(&self) -> Result<u128> {
        let total_fee_rate = self.get_base_fee()?.safe_add(self.get_variable_fee()?)?;
        let total_fee_rate_cap = std::cmp::min(total_fee_rate, constants::MAX_FEE_RATE.into());
        Ok(total_fee_rate_cap)
    }

    /// Base fee rate = Base fee factor * bin step. This is in 1e9 unit.
    pub fn get_base_fee(&self) -> Result<u128> {
        Ok(u128::from(self.parameters.base_factor)
            .safe_mul(self.bin_step.into())?
            // Make it to be the same as FEE_PRECISION defined for ceil_div later on.
            .safe_mul(10u128)?)
    }

    /// Variable fee rate = variable fee factor * (volatility_accumulator * bin_step)^2
    pub fn compute_variable_fee(&self, volatility_accumulator: u32) -> Result<u128> {
        if self.parameters.variable_fee_control > 0 {
            let volatility_accumulator: u128 = volatility_accumulator.into();
            let bin_step: u128 = self.bin_step.into();
            let variable_fee_control: u128 = self.parameters.variable_fee_control.into();

            let square_vfa_bin = volatility_accumulator
                .safe_mul(bin_step)?
                .checked_pow(2)
                .ok_or(MeteoraDlmmProgramError::MathOverflow)?;

            // Variable fee control, volatility accumulator, bin step are in basis point unit (10_000)
            // This is 1e20. Which > 1e9. Scale down it to 1e9 unit and ceiling the remaining.
            let v_fee = variable_fee_control.safe_mul(square_vfa_bin)?;

            let scaled_v_fee = v_fee.safe_add(99_999_999_999)?.safe_div(100_000_000_000)?;
            return Ok(scaled_v_fee);
        }

        Ok(0)
    }

    /// Variable fee rate = variable_fee_control * (variable_fee_accumulator * bin_step) ^ 2
    pub fn get_variable_fee(&self) -> Result<u128> {
        self.compute_variable_fee(self.v_parameters.volatility_accumulator)
    }

    /// Compute fee for the amount. The fee is not part of the amount. This function is used when you do not know the amount_with_fees
    /// Solve for fee_amount, equation: (amount + fee_amount) * total_fee_rate / 1e9 = fee_amount
    /// fee_amount = (amount * total_fee_rate) / (1e9 - total_fee_rate)
    /// The result is ceil-ed.
    pub fn compute_fee(&self, amount: u64) -> Result<u64> {
        let total_fee_rate = self.get_total_fee()?;
        let denominator = u128::from(constants::FEE_PRECISION).safe_sub(total_fee_rate)?;

        // Ceil division
        let fee = u128::from(amount)
            .safe_mul(total_fee_rate)?
            .safe_add(denominator)?
            .safe_sub(1)?;

        let scaled_down_fee = fee.safe_div(denominator)?;

        Ok(scaled_down_fee
            .try_into()
            .map_err(|_| MeteoraDlmmProgramError::TypeCastFailed)?)
    }

    /// Plus / Minus 1 to the active bin based on the swap direction
    pub fn advance_active_bin(&mut self, swap_for_y: bool) -> Result<()> {
        let next_active_bin_id = if swap_for_y {
            self.active_id.safe_sub(1)?
        } else {
            self.active_id.safe_add(1)?
        };

        /*
        require!(
            next_active_bin_id >= MIN_BIN_ID && next_active_bin_id <= MAX_BIN_ID,
            LBError::PairInsufficientLiquidity
        );
         */
        if !(constants::MIN_BIN_ID..=constants::MAX_BIN_ID).contains(&next_active_bin_id) {
            return Err(MeteoraDlmmProgramError::PairInsufficientLiquidity.into());
        }

        self.active_id = next_active_bin_id;

        Ok(())
    }

    pub fn get_amount_in(amount_out: u64, price: u128, swap_for_y: bool) -> Result<u64> {
        if swap_for_y {
            // (amount_y << SCALE_OFFSET) / price
            // Convert amount_y into Q64x0, if not the result will always in 0 as price is in Q64x64
            // Division between same Q number format cancel out, result in integer
            // amount_y / price = amount_in_token_x (integer [Rounding::Down])
            safe_shl_div_cast(amount_out.into(), price, SCALE_OFFSET, Rounding::Up)
        } else {
            // (Q64x64(price) * Q64x0(amount_x)) >> SCALE_OFFSET
            // price * amount_x = amount_in_token_y (Q64x64)
            // amount_in_token_y >> SCALE_OFFSET (convert it back to integer form [Rounding::Down])
            safe_mul_shr_cast(amount_out.into(), price, SCALE_OFFSET, Rounding::Up)
        }
    }

    /// Get out token amount from the bin based in amount in. The result is floor-ed.
    /// X -> Y: inX * bin_price
    /// Y -> X: inY / bin_price
    pub fn get_amount_out(amount_in: u64, price: u128, swap_for_y: bool) -> Result<u64> {
        if swap_for_y {
            // (Q64x64(price) * Q64x0(amount_in)) >> SCALE_OFFSET
            // price * amount_in = amount_out_token_y (Q64x64)
            // amount_out_in_token_y >> SCALE_OFFSET (convert it back to integer form, with some loss of precision [Rounding::Down])
            safe_mul_shr_cast(price, amount_in.into(), SCALE_OFFSET, Rounding::Down)
        } else {
            // (amount_in << SCALE_OFFSET) / price
            // Convert amount_in into Q64x0, if not the result will always in 0 as price is in Q64x64
            // Division between same Q number format cancel out, result in integer
            // amount_in / price = amount_out_token_x (integer [Rounding::Down])
            safe_shl_div_cast(amount_in.into(), price, SCALE_OFFSET, Rounding::Down)
        }
    }

    /// Compute protocol fee
    pub fn compute_protocol_fee(&self, fee_amount: u64) -> Result<u64> {
        let protocol_fee = u128::from(fee_amount)
            .safe_mul(self.parameters.protocol_share.into())?
            .safe_div(BASIS_POINT_MAX as u128)?;

        Ok(protocol_fee
            .try_into()
            .map_err(|_| MeteoraDlmmProgramError::TypeCastFailed)?)
    }

    /// Compute fee from amount, where fee is part of the amount. The result is ceil-ed.
    pub fn compute_fee_from_amount(&self, amount_with_fees: u64) -> Result<u64> {
        // total_fee_rate 1e9 unit
        let total_fee_rate = self.get_total_fee()?;
        // Ceil division
        let fee_amount = u128::from(amount_with_fees)
            .safe_mul(total_fee_rate)?
            .safe_add((FEE_PRECISION - 1).into())?;
        let scaled_down_fee = fee_amount.safe_div(FEE_PRECISION.into())?;

        Ok(scaled_down_fee
            .try_into()
            .map_err(|_| MeteoraDlmmProgramError::TypeCastFailed)?)
    }

    pub fn bitmap_range() -> (i32, i32) {
        (-BIN_ARRAY_BITMAP_SIZE, BIN_ARRAY_BITMAP_SIZE - 1)
    }

    pub fn is_overflow_default_bin_array_bitmap(&self, bin_array_index: i32) -> bool {
        let (min_bitmap_id, max_bitmap_id) = LbPair::bitmap_range();
        bin_array_index > max_bitmap_id || bin_array_index < min_bitmap_id
    }

    fn get_bin_array_offset(bin_array_index: i32) -> usize {
        (bin_array_index + BIN_ARRAY_BITMAP_SIZE) as usize
    }

    // return bin_array_index that it's liquidity is non-zero
    // if cannot find one, return false
    pub fn next_bin_array_index_with_liquidity_internal(
        &self,
        swap_for_y: bool,
        start_array_index: i32,
    ) -> Result<(i32, bool)> {
        let bin_array_bitmap = U1024::from_limbs(self.bin_array_bitmap);
        let array_offset: usize = Self::get_bin_array_offset(start_array_index);
        let (min_bitmap_id, max_bitmap_id) = LbPair::bitmap_range();
        if swap_for_y {
            let binmap_range: usize = max_bitmap_id
                .safe_sub(min_bitmap_id)?
                .try_into()
                .map_err(|_| MeteoraDlmmProgramError::TypeCastFailed)?;
            let offset_bit_map = bin_array_bitmap.shl(binmap_range.safe_sub(array_offset)?);

            if offset_bit_map.eq(&U1024::ZERO) {
                Ok((min_bitmap_id.safe_sub(1)?, false))
            } else {
                let next_bit = offset_bit_map.leading_zeros();
                Ok((start_array_index.safe_sub(next_bit as i32)?, true))
            }
        } else {
            let offset_bit_map = bin_array_bitmap.shr(array_offset);
            if offset_bit_map.eq(&U1024::ZERO) {
                Ok((max_bitmap_id.safe_add(1)?, false))
            } else {
                let next_bit = offset_bit_map.trailing_zeros();
                Ok((
                    start_array_index.checked_add(next_bit as i32).context("Overflow")?,
                    true,
                ))
            }
        }
    }
}

impl BinArrayBitmapExtension {
    fn get_bitmap_offset(bin_array_index: i32) -> Result<usize> {
        // bin_array_index starts from 512 in positive side and -513 in negative side
        let offset = if bin_array_index > 0 {
            bin_array_index / BIN_ARRAY_BITMAP_SIZE - 1
        } else {
            -(bin_array_index + 1) / BIN_ARRAY_BITMAP_SIZE - 1
        };
        Ok(offset as usize)
    }

    fn bin_array_offset_in_bitmap(bin_array_index: i32) -> Result<usize> {
        if bin_array_index > 0 {
            Ok(bin_array_index.safe_rem(BIN_ARRAY_BITMAP_SIZE)? as usize)
        } else {
            Ok((-(bin_array_index + 1)).safe_rem(BIN_ARRAY_BITMAP_SIZE)? as usize)
        }
    }

    fn to_bin_array_index(offset: usize, bin_array_offset: usize, is_positive: bool) -> Result<i32> {
        let offset = offset as i32;
        let bin_array_offset = bin_array_offset as i32;
        if is_positive {
            Ok((offset + 1) * BIN_ARRAY_BITMAP_SIZE + bin_array_offset)
        } else {
            Ok(-((offset + 1) * BIN_ARRAY_BITMAP_SIZE + bin_array_offset) - 1)
        }
    }
    pub fn iter_bitmap(&self, start_index: i32, end_index: i32) -> Result<Option<i32>> {
        let offset: usize = Self::get_bitmap_offset(start_index)?;
        let bin_array_offset = Self::bin_array_offset_in_bitmap(start_index)?;
        if start_index < 0 {
            // iter in negative_bin_array_bitmap
            if start_index <= end_index {
                for i in (0..=offset).rev() {
                    let mut bin_array_bitmap = U512::from_limbs(self.negative_bin_array_bitmap[i]);

                    if i == offset {
                        bin_array_bitmap <<= BIN_ARRAY_BITMAP_SIZE as usize - bin_array_offset - 1;
                        if bin_array_bitmap.eq(&U512::ZERO) {
                            continue;
                        }

                        let bin_array_offset_in_bitmap = bin_array_offset - bin_array_bitmap.leading_zeros();

                        return Ok(Some(BinArrayBitmapExtension::to_bin_array_index(
                            i,
                            bin_array_offset_in_bitmap,
                            false,
                        )?));
                    }
                    if bin_array_bitmap.eq(&U512::ZERO) {
                        continue;
                    }
                    let bin_array_offset_in_bitmap =
                        BIN_ARRAY_BITMAP_SIZE as usize - bin_array_bitmap.leading_zeros() - 1;
                    return Ok(Some(BinArrayBitmapExtension::to_bin_array_index(
                        i,
                        bin_array_offset_in_bitmap,
                        false,
                    )?));
                }
            } else {
                for i in offset..EXTENSION_BINARRAY_BITMAP_SIZE {
                    let mut bin_array_bitmap = U512::from_limbs(self.negative_bin_array_bitmap[i]);
                    if i == offset {
                        bin_array_bitmap >>= bin_array_offset;
                        if bin_array_bitmap.eq(&U512::ZERO) {
                            continue;
                        }

                        let bin_array_offset_in_bitmap = bin_array_offset + bin_array_bitmap.trailing_zeros();

                        return Ok(Some(BinArrayBitmapExtension::to_bin_array_index(
                            i,
                            bin_array_offset_in_bitmap,
                            false,
                        )?));
                    }

                    if bin_array_bitmap.eq(&U512::ZERO) {
                        continue;
                    }
                    let bin_array_offset_in_bitmap = bin_array_bitmap.trailing_zeros();

                    return Ok(Some(BinArrayBitmapExtension::to_bin_array_index(
                        i,
                        bin_array_offset_in_bitmap,
                        false,
                    )?));
                }
            }
        } else {
            // iter in possitive_bin_array_bitmap
            if start_index <= end_index {
                for i in offset..EXTENSION_BINARRAY_BITMAP_SIZE {
                    let mut bin_array_bitmap = U512::from_limbs(self.positive_bin_array_bitmap[i]);
                    if i == offset {
                        bin_array_bitmap >>= bin_array_offset;
                        if bin_array_bitmap.eq(&U512::ZERO) {
                            continue;
                        }

                        let bin_array_offset_in_bitmap = bin_array_offset + bin_array_bitmap.trailing_zeros();
                        return Ok(Some(BinArrayBitmapExtension::to_bin_array_index(
                            i,
                            bin_array_offset_in_bitmap,
                            true,
                        )?));
                    }

                    if bin_array_bitmap.eq(&U512::ZERO) {
                        continue;
                    }

                    let bin_array_offset_in_bitmap = bin_array_bitmap.trailing_zeros();
                    return Ok(Some(BinArrayBitmapExtension::to_bin_array_index(
                        i,
                        bin_array_offset_in_bitmap,
                        true,
                    )?));
                }
            } else {
                for i in (0..=offset).rev() {
                    let mut bin_array_bitmap = U512::from_limbs(self.positive_bin_array_bitmap[i]);

                    if i == offset {
                        bin_array_bitmap <<= BIN_ARRAY_BITMAP_SIZE as usize - bin_array_offset - 1;

                        if bin_array_bitmap.eq(&U512::ZERO) {
                            continue;
                        }
                        let bin_array_offset_in_bitmap = bin_array_offset - bin_array_bitmap.leading_zeros();
                        return Ok(Some(BinArrayBitmapExtension::to_bin_array_index(
                            i,
                            bin_array_offset_in_bitmap,
                            true,
                        )?));
                    }

                    if bin_array_bitmap.eq(&U512::ZERO) {
                        continue;
                    }
                    let bin_array_offset_in_bitmap =
                        BIN_ARRAY_BITMAP_SIZE as usize - bin_array_bitmap.leading_zeros() - 1;
                    return Ok(Some(BinArrayBitmapExtension::to_bin_array_index(
                        i,
                        bin_array_offset_in_bitmap,
                        true,
                    )?));
                }
            }
        }
        Ok(None)
    }

    pub fn bitmap_range() -> (i32, i32) {
        (
            -BIN_ARRAY_BITMAP_SIZE * (EXTENSION_BINARRAY_BITMAP_SIZE as i32 + 1),
            BIN_ARRAY_BITMAP_SIZE * (EXTENSION_BINARRAY_BITMAP_SIZE as i32 + 1) - 1,
        )
    }

    pub fn next_bin_array_index_with_liquidity(&self, swap_for_y: bool, start_index: i32) -> Result<(i32, bool)> {
        let (min_bitmap_id, max_bit_map_id) = BinArrayBitmapExtension::bitmap_range();
        if start_index > 0 {
            if swap_for_y {
                match self.iter_bitmap(start_index, BIN_ARRAY_BITMAP_SIZE)? {
                    Some(value) => Ok((value, true)),
                    None => Ok((BIN_ARRAY_BITMAP_SIZE - 1, false)),
                }
            } else {
                match self.iter_bitmap(start_index, max_bit_map_id)? {
                    Some(value) => Ok((value, true)),
                    None => Err(MeteoraDlmmProgramError::CannotFindNonZeroLiquidityBinArrayId.into()),
                }
            }
        } else if swap_for_y {
            match self.iter_bitmap(start_index, min_bitmap_id)? {
                Some(value) => Ok((value, true)),
                None => Err(MeteoraDlmmProgramError::CannotFindNonZeroLiquidityBinArrayId.into()),
            }
        } else {
            match self.iter_bitmap(start_index, -BIN_ARRAY_BITMAP_SIZE - 1)? {
                Some(value) => Ok((value, true)),
                None => Ok((-BIN_ARRAY_BITMAP_SIZE, false)),
            }
        }
    }
}

// In Trader Joe, the active_id need to be shifted by 2 ** 23 to get the actual ID.
// The reason is because they mint LP for each bin based on active_id using ERC1155, which the ID do not support negative

/// Calculate price based on the given bin id. Eg: 1.0001 ^ 5555. The returned value is in Q64.64
pub fn get_price_from_id(active_id: i32, bin_step: u16) -> Result<u128> {
    // Make bin_step into Q64x64, and divided by BASIS_POINT_MAX. If bin_step = 1, we get 0.0001 in Q64x64
    let bps = u128::from(bin_step)
        .safe_shl(SCALE_OFFSET.into())?
        .safe_div(BASIS_POINT_MAX as u128)?;
    // Add 1 to bps, we get 1.0001 in Q64.64
    let base = ONE.safe_add(bps)?;
    pow(base, active_id).ok_or_else(|| MeteoraDlmmProgramError::MathOverflow.into())
}

impl Bin {
    #[inline(always)]
    pub fn is_zero_liquidity(&self) -> bool {
        self.liquidity_supply == 0
    }

    #[inline(always)]
    pub fn is_empty(&self, is_x: bool) -> bool {
        if is_x {
            self.amount_x == 0
        } else {
            self.amount_y == 0
        }
    }

    pub fn get_amount_in(amount_out: u64, price: u128, swap_for_y: bool) -> Result<u64> {
        if swap_for_y {
            // (amount_y << SCALE_OFFSET) / price
            // Convert amount_y into Q64x0, if not the result will always in 0 as price is in Q64x64
            // Division between same Q number format cancel out, result in integer
            // amount_y / price = amount_in_token_x (integer [Rounding::Down])
            safe_shl_div_cast(amount_out.into(), price, SCALE_OFFSET, Rounding::Up)
        } else {
            // (Q64x64(price) * Q64x0(amount_x)) >> SCALE_OFFSET
            // price * amount_x = amount_in_token_y (Q64x64)
            // amount_in_token_y >> SCALE_OFFSET (convert it back to integer form [Rounding::Down])
            safe_mul_shr_cast(amount_out.into(), price, SCALE_OFFSET, Rounding::Up)
        }
    }

    /// Get out token amount from the bin based in amount in. The result is floor-ed.
    /// X -> Y: inX * bin_price
    /// Y -> X: inY / bin_price
    pub fn get_amount_out(amount_in: u64, price: u128, swap_for_y: bool) -> Result<u64> {
        if swap_for_y {
            // (Q64x64(price) * Q64x0(amount_in)) >> SCALE_OFFSET
            // price * amount_in = amount_out_token_y (Q64x64)
            // amount_out_in_token_y >> SCALE_OFFSET (convert it back to integer form, with some loss of precision [Rounding::Down])
            safe_mul_shr_cast(price, amount_in.into(), SCALE_OFFSET, Rounding::Down)
        } else {
            // (amount_in << SCALE_OFFSET) / price
            // Convert amount_in into Q64x0, if not the result will always in 0 as price is in Q64x64
            // Division between same Q number format cancel out, result in integer
            // amount_in / price = amount_out_token_x (integer [Rounding::Down])
            safe_shl_div_cast(amount_in.into(), price, SCALE_OFFSET, Rounding::Down)
        }
    }

    #[inline(always)]
    pub fn get_max_amount_out(&self, swap_for_y: bool) -> u64 {
        if swap_for_y {
            self.amount_y
        } else {
            self.amount_x
        }
    }

    /// Get or compute and save bin price if not exists
    pub fn get_or_store_bin_price(&mut self, id: i32, bin_step: u16) -> Result<u128> {
        if self.price == 0 {
            self.price = get_price_from_id(id, bin_step)?;
        }

        Ok(self.price)
    }

    /// Get bin price if not exists
    pub fn get_bin_price(&self, id: i32, bin_step: u16) -> Result<u128> {
        if self.price == 0 {
            return get_price_from_id(id, bin_step);
        }

        Ok(self.price)
    }

    /// Get maximum token amount needed to deposit into bin, in order to withdraw out all the opposite token from the bin. The result is ceil-ed.
    /// X -> Y: reserve_y / bin_price
    /// Y -> X: reserve_x * bin_price
    pub fn get_max_amount_in(&self, price: u128, swap_for_y: bool) -> Result<u64> {
        if swap_for_y {
            // (amount_y << SCALE_OFFSET) / price
            // Convert amount_y into Q64x0, if not the result will always in 0 as price is in Q64x64
            // Division between same Q number format cancel out, result in integer
            // amount_y / price = amount_in_token_x (integer [Rounding::Up])
            safe_shl_div_cast(self.amount_y.into(), price, SCALE_OFFSET, Rounding::Up)
        } else {
            // (Q64x64(price) * Q64x0(amount_x)) >> SCALE_OFFSET
            // price * amount_x = amount_in_token_y (Q64x64)
            // amount_in_token_y >> SCALE_OFFSET (convert it back to integer form [Rounding::Up])
            safe_mul_shr_cast(self.amount_x.into(), price, SCALE_OFFSET, Rounding::Up)
        }
    }

    /// Swap
    pub fn swap(
        //&mut self,
        &self,
        amount_in: u64,
        price: u128,
        swap_for_y: bool,
        lb_pair: &LbPair,
        host_fee_bps: Option<u16>,
    ) -> Result<SwapResult> {
        // Get maximum out token amount can be swapped out from the bin.
        let max_amount_out = self.get_max_amount_out(swap_for_y);
        // Get maximum in token amount needed to swap out all of the opposite token from the bin.
        let mut max_amount_in = self.get_max_amount_in(price, swap_for_y)?;

        // The fee was deducted from the amount_in if the swap will not move the active bin. So, the amount_in include fees
        // When the amount_in > max_amount_in, it will swap finish all the current bin token X/Y based on the swap direction.
        // However, max_amount_in is amount that required to swap finish the current bin without fee
        // Therefore, we need find max_amount_in_include_fees, where max_amount_in_include_fees - fee = max_amount_in
        let max_fee = lb_pair.compute_fee(max_amount_in)?;
        max_amount_in = max_amount_in.safe_add(max_fee)?;

        // If the in token amount > maximum token amount needed to swap out all of the opposite token from the bin.
        let (amount_in_with_fees, amount_out, fee, protocol_fee) = if amount_in > max_amount_in {
            (
                max_amount_in,
                max_amount_out,
                max_fee,
                lb_pair.compute_protocol_fee(max_fee)?,
            )
        } else {
            // TODO: User possible to bypass fee by swapping small amount ? User do a "normal" swap by just bundling all small swap that bypass fee ?
            let fee = lb_pair.compute_fee_from_amount(amount_in)?;
            let amount_in_after_fee = amount_in.safe_sub(fee)?;
            let amount_out = Bin::get_amount_out(amount_in_after_fee, price, swap_for_y)?;
            (
                amount_in,
                std::cmp::min(amount_out, max_amount_out),
                fee,
                lb_pair.compute_protocol_fee(fee)?,
            )
        };

        let host_fee = match host_fee_bps {
            Some(bps) => protocol_fee.safe_mul(bps.into())?.safe_div(BASIS_POINT_MAX as u64)?,
            None => 0,
        };

        let protocol_fee_after_host_fee = protocol_fee.safe_sub(host_fee)?;

        // Exclude fee and protocol fee. Protocol fee already part of fee. User need to claim the fee later.
        /*

        let amount_into_bin = amount_in_with_fees.safe_sub(fee)?;

        if swap_for_y {
            self.amount_x = self.amount_x.safe_add(amount_into_bin)?;
            self.amount_y = self.amount_y.safe_sub(amount_out)?;
        } else {
            self.amount_y = self.amount_y.safe_add(amount_into_bin)?;
            self.amount_x = self.amount_x.safe_sub(amount_out)?;
        }
        */

        Ok(SwapResult {
            amount_in_with_fees,
            amount_out,
            fee,
            protocol_fee_after_host_fee,
            host_fee,
            is_exact_out_amount: false,
        })
    }
}

impl BinArray {
    pub fn is_zero_liquidity(&self) -> bool {
        for bin in self.bins.iter() {
            if !bin.is_zero_liquidity() {
                return false;
            }
        }
        true
    }

    fn get_bin_index_in_array(&self, bin_id: i32) -> Result<usize> {
        self.is_bin_id_within_range(bin_id)?;

        let (lower_bin_id, upper_bin_id) = BinArray::get_bin_array_lower_upper_bin_id(self.index as i32)?;

        let index = if bin_id.is_positive() {
            // When bin id is positive, the index is ascending
            bin_id.safe_sub(lower_bin_id)?
        } else {
            // When bin id is negative, the index is descending. Eg: bin id -1 will be located at last index of the bin array
            ((MAX_BIN_PER_ARRAY as i32).safe_sub(upper_bin_id.safe_sub(bin_id)?)?).safe_sub(1)?
        };

        if index >= 0 && index < MAX_BIN_PER_ARRAY as i32 {
            Ok(index as usize)
        } else {
            Err(MeteoraDlmmProgramError::InvalidBinId.into())
        }
    }

    /// Get bin from bin array
    pub fn get_bin_mut(&mut self, bin_id: i32) -> Result<&mut Bin> {
        Ok(&mut self.bins[self.get_bin_index_in_array(bin_id)?])
    }

    pub fn get_bin(&self, bin_id: i32) -> Result<&Bin> {
        Ok(&self.bins[self.get_bin_index_in_array(bin_id)?])
    }

    /// Check whether the bin id is within the bin array range
    pub fn is_bin_id_within_range(&self, bin_id: i32) -> Result<()> {
        let (lower_bin_id, upper_bin_id) = BinArray::get_bin_array_lower_upper_bin_id(self.index as i32)?;

        /*
        require!(
            bin_id >= lower_bin_id && bin_id <= upper_bin_id,
            MeteoraDlmmProgramError::InvalidBinId
        );
        */
        if !(bin_id >= lower_bin_id && bin_id <= upper_bin_id) {
            return Err(MeteoraDlmmProgramError::InvalidBinId.into());
        }

        Ok(())
    }

    /// Get lower and upper bin id of the given bin array index
    pub fn get_bin_array_lower_upper_bin_id(index: i32) -> Result<(i32, i32)> {
        let lower_bin_id = index.safe_mul(MAX_BIN_PER_ARRAY as i32)?;
        let upper_bin_id = lower_bin_id.safe_add(MAX_BIN_PER_ARRAY as i32)?.safe_sub(1)?;

        Ok((lower_bin_id, upper_bin_id))
    }

    /// Check that the index within MAX and MIN bin id
    pub fn check_valid_index(index: i32) -> Result<()> {
        let (lower_bin_id, upper_bin_id) = BinArray::get_bin_array_lower_upper_bin_id(index)?;

        /*
        require!(
            lower_bin_id >= MIN_BIN_ID && upper_bin_id <= MAX_BIN_ID,
            MeteoraDlmmProgramError::InvalidStartBinIndex
        );
        */
        if !(lower_bin_id >= constants::MIN_BIN_ID && upper_bin_id <= constants::MAX_BIN_ID) {
            return Err(MeteoraDlmmProgramError::InvalidStartBinIndex.into());
        }

        Ok(())
    }

    /// Get bin array index from bin id
    pub fn bin_id_to_bin_array_index(bin_id: i32) -> Result<i32> {
        let (idx, rem) = bin_id.div_rem(&(MAX_BIN_PER_ARRAY as i32));

        if bin_id.is_negative() && rem != 0 {
            Ok(idx.safe_sub(1)?)
        } else {
            Ok(idx)
        }
    }
}

#[derive(Debug)]
pub struct SwapResult {
    /// Amount of token swap into the bin
    pub amount_in_with_fees: u64,
    /// Amount of token swap out from the bin
    pub amount_out: u64,
    /// Swap fee, includes protocol fee
    pub fee: u64,
    /// Part of fee
    pub protocol_fee_after_host_fee: u64,
    /// Part of protocol fee
    pub host_fee: u64,
    /// Indicate whether reached exact out amount
    pub is_exact_out_amount: bool,
}

// eof
