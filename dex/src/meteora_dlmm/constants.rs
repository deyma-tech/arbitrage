use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use utils::types::MintPair;

// TODO: Macro to compute the constants which changes based on the bit system used ?
// Smallest step between bin is 0.01%, 1 bps
pub const BASIS_POINT_MAX: i32 = 10000;

/// Maximum number of bin a bin array able to contains.
pub const MAX_BIN_PER_ARRAY: usize = 70;

/// Maximum number of bin per position contains.
pub const MAX_BIN_PER_POSITION: usize = 70;

/// Minimum bin ID supported. Computed based on 1 bps.
pub const MIN_BIN_ID: i32 = -443636;

/// Maximum bin ID supported. Computed based on 1 bps.
pub const MAX_BIN_ID: i32 = 443636;

/// Maximum fee rate. 10%
pub const MAX_FEE_RATE: u64 = 100_000_000;

pub const FEE_PRECISION: u64 = 1_000_000_000;

/// Maximum protocol share of the fee. 25%
pub const MAX_PROTOCOL_SHARE: u16 = 2_500;

/// Host fee. 20%
pub const HOST_FEE_BPS: u16 = 2_000;

pub const U24_MAX: u32 = 0xffffff;

// Number of rewards supported by pool
pub const NUM_REWARDS: usize = 2;

// Minimum reward duration
pub const MIN_REWARD_DURATION: u64 = 1;

pub const MAX_REWARD_DURATION: u64 = 31536000; // 1 year = 365 * 24 * 3600

pub const DEFAULT_OBSERVATION_LENGTH: u64 = 100;
pub const SAMPLE_LIFETIME: u64 = 120; // 2
pub const EXTENSION_BINARRAY_BITMAP_SIZE: usize = 12;

pub const BIN_ARRAY_BITMAP_SIZE: i32 = 512;

pub const MAX_BASE_FACTOR_STEP: u16 = 100; // 100 bps, 1%

pub const MAX_FEE_UPDATE_WINDOW: i64 = 0;

pub const MAX_REWARD_BIN_SPLIT: usize = 15;

// ILM token launch protocol fee
pub const ILM_PROTOCOL_SHARE: u16 = 2000; // 20%

/// Maximum bin step
pub const MAX_BIN_STEP: u16 = 400;

/// Maximum base fee, base_fee / 10^9 = fee_in_percentage
pub const MAX_BASE_FEE: u128 = 100_000_000; // 10% (10^9 * 10 / 100)

/// Minimum base fee
pub const MIN_BASE_FEE: u128 = 100_000; // 0.01% (10^9 * 0.01 / 100)

// Supported quote mints
const SOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const QUOTE_MINTS: MintPair = [SOL, USDC];

pub const SCALE_OFFSET: u8 = 64;

// 1.0000... representation of 64x64
pub const ONE: u128 = 1u128 << SCALE_OFFSET;

pub const MAX_EXPONENTIAL: u32 = 0x80000; // 1048576

pub const EVENT_AUTHORITY: Pubkey = pubkey!("D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6");
