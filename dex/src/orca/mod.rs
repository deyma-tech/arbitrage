mod accounts;
mod constants;
mod quote;
mod swap;

pub use accounts::{OrcaTick, OrcaTickArray, OrcaWhirlpool, QuoteInput};
pub use constants::*;
pub use orca_whirlpools_client::*;
pub use orca_whirlpools_core::TICK_ARRAY_SIZE;
pub use quote::*;
pub use swap::*;

#[inline(always)]
pub fn get_tick_array_start_tick_index(tick_index: i32, tick_spacing: u16) -> i32 {
    orca_whirlpools_core::get_tick_array_start_tick_index(tick_index, tick_spacing)
}
