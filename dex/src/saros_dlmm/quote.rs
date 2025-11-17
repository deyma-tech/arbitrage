use anyhow::Result;

use crate::saros_dlmm::{
    accounts::BinArrayPair,
    bin_math::get_price_from_id,
    constants::{MAX_ACTIVE_ID, MAX_BIN_CROSSING, SCALE_OFFSET, VARIABLE_FEE_PRECISION},
    math_utils::{convert_math_result, get_fee_amount, get_fee_for_amount, get_protocol_fee},
    typedefs::Bin,
    u128x128_math::{mul_shr, shl_div, Rounding},
    Pair,
};

pub enum SwapMode {
    ExactIn,
    ExactOut,
}

pub fn get_swap_result(
    pair: &Pair,
    bin_array: &BinArrayPair,
    amount: u64,
    swap_for_y: bool,
    swap_type: SwapMode,
    //block_timestamp: u64,
) -> anyhow::Result<(u64, u64)> {
    let mut active_id = pair.active_id;

    let bin_array = bin_array.clone();
    //pair.update_references(block_timestamp)?;

    match swap_type {
        SwapMode::ExactIn => {
            let mut amount_in_left: u64 = amount;
            let mut amount_out: u64 = 0;
            let mut total_protocol_fee: u64 = 0;
            let mut total_fee_amount: u64 = 0;
            let mut total_bin_used: u32 = 0;

            while amount_in_left > 0 {
                if total_bin_used >= MAX_BIN_CROSSING {
                    return Err(anyhow::format_err!("SwapCrossesTooManyBins"));
                }
                //pair.update_volatility_accumulator()?;

                //let bin = bin_array.get_bin_mut(pair.active_id)?;
                let bin = bin_array.get_bin(active_id)?;

                let fee = pair.get_total_fee()?;

                let (amount_in_with_fees, amount_out_of_bin, fee_amount, protocol_fee) = bin.swap_exact_in(
                    pair.bin_step,
                    active_id,
                    amount_in_left,
                    fee,
                    pair.get_protocol_share(),
                    swap_for_y,
                )?;

                amount_out = amount_out
                    .checked_add(amount_out_of_bin)
                    .ok_or(anyhow::format_err!("AmountOverflow"))?;

                amount_in_left = amount_in_left
                    .checked_sub(amount_in_with_fees)
                    .ok_or(anyhow::format_err!("AmountUnderflow"))?;

                total_protocol_fee = total_protocol_fee
                    .checked_add(protocol_fee)
                    .ok_or(anyhow::format_err!("AmountOverflow"))?;

                total_fee_amount = total_fee_amount
                    .checked_add(fee_amount)
                    .ok_or(anyhow::format_err!("AmountOverflow"))?;

                if amount_in_left == 0 {
                    break;
                } else {
                    Pair::move_active_id_v2(&mut active_id, swap_for_y)?;
                }

                total_bin_used += 1;
            }

            Ok((amount_out, amount_in_left))
        }

        SwapMode::ExactOut => {
            let mut amount_out_left: u64 = amount;
            let mut amount_in: u64 = 0;
            let mut total_protocol_fee: u64 = 0;
            let mut total_fee_amount: u64 = 0;
            let mut total_bin_used: u32 = 0;

            while amount_out_left > 0 {
                if total_bin_used >= MAX_BIN_CROSSING {
                    return Err(anyhow::format_err!("SwapCrossesTooManyBins"));
                }
                //pair.update_volatility_accumulator()?;

                //let bin = bin_array.get_bin_mut(pair.active_id)?;
                let bin = bin_array.get_bin(active_id)?;

                let fee = pair.get_total_fee()?;

                let (amount_in_with_fees, amount_out_of_bin, fee_amount, protocol_fee) = bin.swap_exact_out(
                    pair.bin_step,
                    pair.active_id,
                    amount_out_left,
                    fee,
                    pair.get_protocol_share(),
                    swap_for_y,
                )?;

                amount_in = amount_in
                    .checked_add(amount_in_with_fees)
                    .ok_or(anyhow::format_err!("AmountOverflow"))?;

                amount_out_left = amount_out_left
                    .checked_sub(amount_out_of_bin)
                    .ok_or(anyhow::format_err!("AmountUnderflow"))?;

                total_protocol_fee = total_protocol_fee
                    .checked_add(protocol_fee)
                    .ok_or(anyhow::format_err!("AmountOverflow"))?;

                total_fee_amount = total_fee_amount
                    .checked_add(fee_amount)
                    .ok_or(anyhow::format_err!("AmountOverflow"))?;

                if amount_out_left == 0 {
                    break;
                } else {
                    Pair::move_active_id_v2(&mut active_id, swap_for_y)?;
                }
                total_bin_used += 1;
            }

            Ok((amount_in, amount_out_left))
        }
    }
}

impl Pair {
    pub fn get_total_fee(&self) -> anyhow::Result<u64> {
        Ok(self.get_base_fee()? + self.get_variable_fee()?)
    }

    fn get_base_fee(&self) -> anyhow::Result<u64> {
        // Base factor is in basis points, binStep is in basis points, so we multiply by 10
        u64::from(self.static_fee_parameters.base_factor)
            .checked_mul(self.bin_step.into())
            .ok_or(anyhow::format_err!("AmountOverflow"))?
            .checked_mul(10)
            .ok_or(anyhow::format_err!("AmountOverflow"))
    }

    fn get_variable_fee(&self) -> anyhow::Result<u64> {
        let variable_fee_control = self.static_fee_parameters.variable_fee_control;

        // (volatilityAccumulator * binStep)^2 * variableFeeControl / VARIABLE_FEE_PRECISION, rounded up
        if variable_fee_control > 0 {
            let prod = u128::from(self.dynamic_fee_parameters.volatility_accumulator)
                .checked_mul(self.bin_step.into())
                .ok_or(anyhow::format_err!("AmountOverflow"))?;

            let variable_fee = (prod
                .checked_mul(prod)
                .ok_or(anyhow::format_err!("AmountUnderflow"))?
                .checked_mul(variable_fee_control.into())
                .ok_or(anyhow::format_err!("AmountOverflow"))?
                .checked_add(VARIABLE_FEE_PRECISION)
                .ok_or(anyhow::format_err!("AmountOverflow"))?
                .checked_sub(1)
                .ok_or(anyhow::format_err!("AmountUnderflow"))?)
                / VARIABLE_FEE_PRECISION;

            Ok(u64::try_from(variable_fee).map_err(|_| anyhow::format_err!("U64ConversionOverflow"))?)
        } else {
            Ok(0)
        }
    }

    pub fn get_protocol_share(&self) -> u64 {
        self.static_fee_parameters.protocol_share as u64
    }

    // pub fn move_active_id(&mut self, swap_for_y: bool) -> anyhow::Result<()> {
    //     if swap_for_y {
    //         self.move_active_id_left()
    //     } else {
    //         self.move_active_id_right()
    //     }
    // }

    // fn move_active_id_left(&mut self) -> anyhow::Result<()> {
    //     self.active_id = self
    //         .active_id
    //         .checked_sub(1)
    //         .ok_or(anyhow::format_err!("ActiveIdUnderflow"))?;
    //     Ok(())
    // }

    // fn move_active_id_right(&mut self) -> anyhow::Result<()> {
    //     // require!(self.active_id < MAX_ACTIVE_ID, ErrorCode::ActiveIdOverflow);
    //     if self.active_id >= MAX_ACTIVE_ID {
    //         Err(anyhow::format_err!("ActiveIdOverflow"))?;
    //     }
    //     self.active_id += 1;
    //     Ok(())
    // }

    pub fn move_active_id_v2(active_id: &mut u32, swap_for_y: bool) -> anyhow::Result<()> {
        if swap_for_y {
            *active_id = active_id
                .checked_sub(1)
                .ok_or(anyhow::format_err!("ActiveIdUnderflow"))?;
        } else {
            if *active_id >= MAX_ACTIVE_ID {
                Err(anyhow::format_err!("ActiveIdOverflow"))?;
            }
            *active_id += 1;
        }
        Ok(())
    }
}

impl Bin {
    pub fn swap_exact_in(
        &self,
        bin_step: u8,
        bin_id: u32,
        amount_in_left: u64,
        fee: u64,
        protocol_share: u64,
        swap_for_y: bool,
    ) -> anyhow::Result<(u64, u64, u64, u64)> {
        let price = get_price_from_id(bin_step, bin_id).ok_or(anyhow::format_err!("ShlDivMathError"))?;

        let bin_reserve_out = if swap_for_y { self.reserve_y } else { self.reserve_x };

        if bin_reserve_out == 0 {
            return Ok((0, 0, 0, 0));
        }

        let mut max_amount_in = if swap_for_y {
            convert_math_result(
                shl_div(u128::from(bin_reserve_out), price, SCALE_OFFSET, Rounding::Up),
                anyhow::format_err!("ShlDivMathError"),
            )?
        } else {
            convert_math_result(
                mul_shr(u128::from(bin_reserve_out), price, SCALE_OFFSET, Rounding::Up),
                anyhow::format_err!("MulShrMathError"),
            )?
        };

        let max_fee_amount = get_fee_for_amount(max_amount_in, fee)?;

        max_amount_in = max_amount_in
            .checked_add(max_fee_amount)
            .ok_or(anyhow::format_err!("AmountOverflow"))?;

        let mut amount_out: u64;
        let amount_in: u64;
        let fee_amount: u64;

        if amount_in_left >= max_amount_in {
            fee_amount = max_fee_amount;

            amount_in = max_amount_in
                .checked_sub(fee_amount)
                .ok_or(anyhow::format_err!("AmountUnderflow"))?;

            amount_out = bin_reserve_out;
        } else {
            fee_amount = get_fee_amount(amount_in_left, fee)?;

            amount_in = amount_in_left
                .checked_sub(fee_amount)
                .ok_or(anyhow::format_err!("AmountUnderflow"))?;

            amount_out = if swap_for_y {
                convert_math_result(
                    mul_shr(u128::from(amount_in), price, SCALE_OFFSET, Rounding::Down),
                    anyhow::format_err!("MulShrMathError"),
                )?
            } else {
                convert_math_result(
                    shl_div(u128::from(amount_in), price, SCALE_OFFSET, Rounding::Down),
                    anyhow::format_err!("ShlDivMathError"),
                )?
            };

            if amount_out > bin_reserve_out {
                amount_out = bin_reserve_out;
            }
        }

        let mut protocol_fee_amount = 0;

        if protocol_share > 0 {
            protocol_fee_amount = get_protocol_fee(fee_amount, protocol_share)?;
        }

        let amount_in_with_fees = amount_in
            .checked_add(fee_amount)
            .ok_or(anyhow::format_err!("AmountOverflow"))?;

        // if swap_for_y {
        //     self.reserve_x = self
        //         .reserve_x
        //         .checked_add(amount_in_with_fees)
        //         .ok_or(anyhow::format_err!("AmountOverflow"))?
        //         .checked_sub(protocol_fee_amount)
        //         .ok_or(anyhow::format_err!("AmountUnderflow"))?;

        //     self.reserve_y = self
        //         .reserve_y
        //         .checked_sub(amount_out)
        //         .ok_or(anyhow::format_err!("AmountUnderflow"))?;
        // } else {
        //     self.reserve_x = self
        //         .reserve_x
        //         .checked_sub(amount_out)
        //         .ok_or(anyhow::format_err!("AmountUnderflow"))?;

        //     self.reserve_y = self
        //         .reserve_y
        //         .checked_add(amount_in_with_fees)
        //         .ok_or(anyhow::format_err!("AmountOverflow"))?
        //         .checked_sub(protocol_fee_amount)
        //         .ok_or(anyhow::format_err!("AmountUnderflow"))?;
        // }

        Ok((amount_in_with_fees, amount_out, fee_amount, protocol_fee_amount))
    }

    pub fn swap_exact_out(
        &self,
        bin_step: u8,
        bin_id: u32,
        amount_out_left: u64,
        fee: u64,
        protocol_share: u64,
        swap_for_y: bool,
    ) -> Result<(u64, u64, u64, u64)> {
        let price = get_price_from_id(bin_step, bin_id).ok_or(anyhow::format_err!("ShlDivMathError"))?;

        let bin_reserve_out = if swap_for_y { self.reserve_y } else { self.reserve_x };

        if bin_reserve_out == 0 {
            return Ok((0, 0, 0, 0));
        }

        let amount_out = if amount_out_left > bin_reserve_out {
            bin_reserve_out
        } else {
            amount_out_left
        };

        let amount_in_without_fee = if swap_for_y {
            convert_math_result(
                shl_div(amount_out as u128, price, SCALE_OFFSET, Rounding::Up),
                anyhow::format_err!("ShlDivMathError"),
            )?
        } else {
            convert_math_result(
                mul_shr(amount_out as u128, price, SCALE_OFFSET, Rounding::Up),
                anyhow::format_err!("MulShrMathError"),
            )?
        };

        let fee_amount = get_fee_for_amount(amount_in_without_fee, fee)?;

        let amount_in = amount_in_without_fee
            .checked_add(fee_amount)
            .ok_or(anyhow::format_err!("AmountOverflow"))?;

        let protocol_fee_amount = get_protocol_fee(fee_amount, protocol_share)?;

        // if swap_for_y {
        //     self.reserve_x = self
        //         .reserve_x
        //         .checked_add(amount_in)
        //         .ok_or(anyhow::format_err!("AmountOverflow"))?
        //         .checked_sub(protocol_fee_amount)
        //         .ok_or(anyhow::format_err!("AmountUnderflow"))?;

        //     self.reserve_y = self
        //         .reserve_y
        //         .checked_sub(amount_out)
        //         .ok_or(anyhow::format_err!("AmountUnderflow"))?;
        // } else {
        //     self.reserve_x = self
        //         .reserve_x
        //         .checked_sub(amount_out)
        //         .ok_or(anyhow::format_err!("AmountUnderflow"))?;

        //     self.reserve_y = self
        //         .reserve_y
        //         .checked_add(amount_in)
        //         .ok_or(anyhow::format_err!("AmountOverflow"))?
        //         .checked_sub(protocol_fee_amount)
        //         .ok_or(anyhow::format_err!("AmountUnderflow"))?;
        // }

        Ok((amount_in, amount_out, fee_amount, protocol_fee_amount))
    }
}
