mod accounts;
mod events;
mod instructions;
pub mod typedefs;

//use accounts::*;
//pub use errors::*;
//use events::*;
//use instructions::*;

use typedefs::*;

pub use accounts::*;
pub mod big_num;
pub mod constants;
pub mod error;
pub mod fixed_point_64;
pub mod full_math;
pub mod liquidity_math;
pub mod quote;
pub mod sqrt_price_math;
pub mod swap;
pub mod swap_math;
pub mod tick_array_bit_map;
pub mod tick_math;

solana_program::declare_id!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");
use big_num::{U1024, U512};
use constants::{POOL_TICK_ARRAY_BITMAP_SEED, TICK_ARRAY_BITMAP_SIZE, TICK_ARRAY_SIZE};
use solana_program::pubkey::Pubkey;

use tick_array_bit_map::{check_current_tick_array_is_initialized, get_bitmap_tick_boundary, TickArryBitmap};

//use core as core_;

use anyhow::{Context, Result};

pub fn max_tick_in_tickarray_bitmap(tick_spacing: u16) -> i32 {
    i32::from(tick_spacing) * TICK_ARRAY_SIZE * TICK_ARRAY_BITMAP_SIZE
}

impl TickState {
    pub fn check_is_out_of_boundary(tick: i32) -> bool {
        !(tick_math::MIN_TICK..=tick_math::MAX_TICK).contains(&tick)
    }
}

impl TickArrayBitmapExtension {
    pub fn key(pool_id: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(), pool_id.as_ref()],
            &ID, //&super::id(),
        )
        .0
    }

    /// Check if the tick in tick array bitmap extension
    pub fn check_extension_boundary(tick_index: i32, tick_spacing: u16) -> Result<()> {
        let positive_tick_boundary = max_tick_in_tickarray_bitmap(tick_spacing);
        let negative_tick_boundary = -positive_tick_boundary;
        if tick_index >= negative_tick_boundary && tick_index < positive_tick_boundary {
            return Err(anyhow::format_err!("ErrorCode::InvalidTickArrayBoundary"));
        }
        Ok(())
    }

    fn get_bitmap_offset(tick_index: i32, tick_spacing: u16) -> Result<usize> {
        Self::check_extension_boundary(tick_index, tick_spacing)?;
        let ticks_in_one_bitmap = max_tick_in_tickarray_bitmap(tick_spacing);
        let mut offset = tick_index.abs() / ticks_in_one_bitmap - 1;
        if tick_index < 0 && tick_index.abs() % ticks_in_one_bitmap == 0 {
            offset -= 1;
        }
        Ok(offset as usize)
    }

    /// According to the given tick, calculate its corresponding tickarray and then find the bitmap it belongs to.
    fn get_bitmap(&self, tick_index: i32, tick_spacing: u16) -> Result<(usize, TickArryBitmap)> {
        let offset = Self::get_bitmap_offset(tick_index, tick_spacing)?;
        if tick_index < 0 {
            Ok((offset, self.negative_tick_array_bitmap[offset]))
        } else {
            Ok((offset, self.positive_tick_array_bitmap[offset]))
        }
    }

    pub fn tick_array_offset_in_bitmap(tick_array_start_index: i32, tick_spacing: u16) -> i32 {
        let m = tick_array_start_index.abs() % max_tick_in_tickarray_bitmap(tick_spacing);
        let mut tick_array_offset_in_bitmap = m / TickArrayState::tick_count(tick_spacing);
        if tick_array_start_index < 0 && m != 0 {
            tick_array_offset_in_bitmap = TICK_ARRAY_BITMAP_SIZE - tick_array_offset_in_bitmap;
        }
        tick_array_offset_in_bitmap
    }

    /// Check if the tick array is initialized
    pub fn check_tick_array_is_initialized(
        &self,
        tick_array_start_index: i32,
        tick_spacing: u16,
    ) -> Result<(bool, i32)> {
        let (_, tickarray_bitmap) = self.get_bitmap(tick_array_start_index, tick_spacing)?;

        let tick_array_offset_in_bitmap = Self::tick_array_offset_in_bitmap(tick_array_start_index, tick_spacing);

        if U512(tickarray_bitmap).bit(tick_array_offset_in_bitmap as usize) {
            return Ok((true, tick_array_start_index));
        }
        Ok((false, tick_array_start_index))
    }

    pub fn next_initialized_tick_array_in_bitmap(
        tickarray_bitmap: TickArryBitmap,
        next_tick_array_start_index: i32,
        tick_spacing: u16,
        zero_for_one: bool,
    ) -> anyhow::Result<(bool, i32)> {
        let (bitmap_min_tick_boundary, bitmap_max_tick_boundary) =
            get_bitmap_tick_boundary(next_tick_array_start_index, tick_spacing);

        let tick_array_offset_in_bitmap = Self::tick_array_offset_in_bitmap(next_tick_array_start_index, tick_spacing);
        if zero_for_one {
            // tick from upper to lower
            // find from highter bits to lower bits
            let offset_bit_map = U512(tickarray_bitmap) << (TICK_ARRAY_BITMAP_SIZE - 1 - tick_array_offset_in_bitmap);

            let next_bit = if offset_bit_map.is_zero() {
                None
            } else {
                Some(u16::try_from(offset_bit_map.leading_zeros()).map_err(|e| anyhow::format_err!(e))?)
            };

            if let Some(next_bit) = next_bit {
                let next_array_start_index =
                    next_tick_array_start_index - i32::from(next_bit) * TickArrayState::tick_count(tick_spacing);
                Ok((true, next_array_start_index))
            } else {
                // not found til to the end
                Ok((false, bitmap_min_tick_boundary))
            }
        } else {
            // tick from lower to upper
            // find from lower bits to highter bits
            let offset_bit_map = U512(tickarray_bitmap) >> tick_array_offset_in_bitmap;

            let next_bit = if offset_bit_map.is_zero() {
                None
            } else {
                Some(u16::try_from(offset_bit_map.trailing_zeros()).map_err(|e| anyhow::format_err!(e))?)
            };
            if let Some(next_bit) = next_bit {
                let next_array_start_index =
                    next_tick_array_start_index + i32::from(next_bit) * TickArrayState::tick_count(tick_spacing);
                Ok((true, next_array_start_index))
            } else {
                // not found til to the end
                Ok((
                    false,
                    bitmap_max_tick_boundary - TickArrayState::tick_count(tick_spacing),
                ))
            }
        }
    }

    /// Search for the first initialized bit in bitmap according to the direction, if found return ture and the tick array start index,
    /// if not, return false and tick boundary index
    pub fn next_initialized_tick_array_from_one_bitmap(
        &self,
        last_tick_array_start_index: i32,
        tick_spacing: u16,
        zero_for_one: bool,
    ) -> Result<(bool, i32)> {
        let multiplier = TickArrayState::tick_count(tick_spacing);
        let next_tick_array_start_index = if zero_for_one {
            last_tick_array_start_index - multiplier
        } else {
            last_tick_array_start_index + multiplier
        };
        let min_tick_array_start_index = TickArrayState::get_array_start_index(tick_math::MIN_TICK, tick_spacing);
        let max_tick_array_start_index = TickArrayState::get_array_start_index(tick_math::MAX_TICK, tick_spacing);

        if next_tick_array_start_index < min_tick_array_start_index
            || next_tick_array_start_index > max_tick_array_start_index
        {
            return Ok((false, next_tick_array_start_index));
        }

        let (_, tickarray_bitmap) = self.get_bitmap(next_tick_array_start_index, tick_spacing)?;

        Self::next_initialized_tick_array_in_bitmap(
            tickarray_bitmap,
            next_tick_array_start_index,
            tick_spacing,
            zero_for_one,
        )
    }
}

impl PoolState {
    // the range of tick array start index that default tickarray bitmap can represent
    // if tick_spacing = 1, the result range is [-30720, 30720)
    pub fn tick_array_start_index_range(&self) -> (i32, i32) {
        // the range of ticks that default tickarrary can represent
        let mut max_tick_boundary = max_tick_in_tickarray_bitmap(self.tick_spacing);
        let mut min_tick_boundary = -max_tick_boundary;
        if max_tick_boundary > tick_math::MAX_TICK {
            max_tick_boundary = TickArrayState::get_array_start_index(tick_math::MAX_TICK, self.tick_spacing);
            // find the next tick array start index
            max_tick_boundary += TickArrayState::tick_count(self.tick_spacing);
        }
        if min_tick_boundary < tick_math::MIN_TICK {
            min_tick_boundary = TickArrayState::get_array_start_index(tick_math::MIN_TICK, self.tick_spacing);
        }
        (min_tick_boundary, max_tick_boundary)
    }

    pub fn is_overflow_default_tickarray_bitmap(&self, tick_indexs: Vec<i32>) -> bool {
        let (min_tick_array_start_index_boundary, max_tick_array_index_boundary) = self.tick_array_start_index_range();
        for tick_index in tick_indexs {
            let tick_array_start_index = TickArrayState::get_array_start_index(tick_index, self.tick_spacing);
            if tick_array_start_index >= max_tick_array_index_boundary
                || tick_array_start_index < min_tick_array_start_index_boundary
            {
                return true;
            }
        }
        false
    }

    pub fn next_initialized_tick_array_start_index(
        &self,
        tickarray_bitmap_extension: Option<&TickArrayBitmapExtension>,
        mut last_tick_array_start_index: i32,
        zero_for_one: bool,
    ) -> Result<Option<i32>> {
        last_tick_array_start_index =
            TickArrayState::get_array_start_index(last_tick_array_start_index, self.tick_spacing);

        loop {
            let (is_found, start_index) = tick_array_bit_map::next_initialized_tick_array_start_index(
                U1024(self.tick_array_bitmap),
                last_tick_array_start_index,
                self.tick_spacing,
                zero_for_one,
            )?;
            if is_found {
                return Ok(Some(start_index));
            }
            last_tick_array_start_index = start_index;

            if tickarray_bitmap_extension.is_none() {
                return Err(anyhow::format_err!("ErrorCode::MissingTickArrayBitmapExtensionAccount"));
            }

            let (is_found, start_index) = tickarray_bitmap_extension
                .as_ref()
                .context("RaydiumCLMMNextInititalizedTickArray")?
                .next_initialized_tick_array_from_one_bitmap(
                    last_tick_array_start_index,
                    self.tick_spacing,
                    zero_for_one,
                )?;
            if is_found {
                return Ok(Some(start_index));
            }
            last_tick_array_start_index = start_index;

            if !(tick_math::MIN_TICK..=tick_math::MAX_TICK).contains(&last_tick_array_start_index) {
                return Ok(None);
            }
        }
    }

    pub fn get_first_initialized_tick_array(
        &self,
        tickarray_bitmap_extension: Option<&TickArrayBitmapExtension>,
        zero_for_one: bool,
    ) -> Result<(bool, i32)> {
        let (is_initialized, start_index) = if self.is_overflow_default_tickarray_bitmap(vec![self.tick_current])
            && tickarray_bitmap_extension.is_some()
        {
            tickarray_bitmap_extension
                .as_ref()
                .context("RaydiumCLMMGetFirstInitializedTickArray")?
                .check_tick_array_is_initialized(
                    TickArrayState::get_array_start_index(self.tick_current, self.tick_spacing),
                    self.tick_spacing,
                )?
        } else {
            check_current_tick_array_is_initialized(
                U1024(self.tick_array_bitmap),
                self.tick_current,
                self.tick_spacing,
            )?
        };
        if is_initialized {
            return Ok((true, start_index));
        }
        let next_start_index = self.next_initialized_tick_array_start_index(
            tickarray_bitmap_extension,
            TickArrayState::get_array_start_index(self.tick_current, self.tick_spacing),
            zero_for_one,
        )?;
        Ok((
            false,
            next_start_index.context("RaydiumCLMMGetFirstInitializedTickArray")?,
        ))
    }
}
