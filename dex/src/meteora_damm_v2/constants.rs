// pub const SCALE_OFFSET: u32 = 64;
// pub const BASIS_POINT_MAX: u128 = 10_000;
// pub const ONE: u128 = 1_u128.checked_shl(SCALE_OFFSET).unwrap();

use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const AUTHORITY_DAMM_V2: Pubkey = pubkey!("HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC");
pub const EVENT_DAMM_V2: Pubkey = pubkey!("3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet");

pub const MAX_FEE_NUMERATOR: u64 = 500_000_000; // 50%

/// Default fee denominator. DO NOT simply update it as it will break logic that depends on it as default value.
pub const FEE_DENOMINATOR: u64 = 1_000_000_000;

pub const BASIS_POINT_MAX: u64 = 10_000;
pub const ONE_Q64: u128 = 1u128 << 64;

// pub mod activation {
//     pub const SLOT_BUFFER: u64 = 9000; // 1 slot = 400 mls => 1 hour

//     pub const TIME_BUFFER: u64 = 3600; // 1 hour

//     pub const MAX_ACTIVATION_SLOT_DURATION: u64 = SLOT_BUFFER * 24 * 31; // 31 days

//     pub const MAX_ACTIVATION_TIME_DURATION: u64 = TIME_BUFFER * 24 * 31; // 31 days

//     pub const MAX_VESTING_SLOT_DURATION: u64 = SLOT_BUFFER * 24 * 365 * 10; // 10 years
//     pub const MAX_VESTING_TIME_DURATION: u64 = TIME_BUFFER * 24 * 365 * 10; // 10 years

//     pub const FIVE_MINUTES_SLOT_BUFFER: u64 = SLOT_BUFFER / 12; // 5 minutes

//     pub const FIVE_MINUTES_TIME_BUFFER: u64 = TIME_BUFFER / 12; // 5 minutes

//     pub const MAX_FEE_CURVE_TIME_DURATION: u64 = 3600 * 24; // 1 day
//     pub const MAX_FEE_CURVE_SLOT_DURATION: u64 = 9000 * 24; // 1 day

//     pub const MAX_HIGH_TAX_TIME_DURATION: u64 = TIME_BUFFER / 6; // 10 minutes
//     pub const MAX_HIGH_TAX_SLOT_DURATION: u64 = SLOT_BUFFER / 6; // 10 minutes
// }
