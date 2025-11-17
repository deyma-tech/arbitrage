use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use solana_program::pubkey::Pubkey;
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct InitPresetParametersIx {
    pub bin_step: u16,
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub min_bin_id: i32,
    pub max_bin_id: i32,
    pub protocol_share: u16,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct FeeParameter {
    pub protocol_share: u16,
    pub base_factor: u16,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct LiquidityParameterByStrategyOneSide {
    pub amount: u64,
    pub active_id: i32,
    pub max_active_bin_slippage: i32,
    pub strategy_parameters: StrategyParameters,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct LiquidityParameterByStrategy {
    pub amount_x: u64,
    pub amount_y: u64,
    pub active_id: i32,
    pub max_active_bin_slippage: i32,
    pub strategy_parameters: StrategyParameters,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct StrategyParameters {
    pub min_bin_id: i32,
    pub max_bin_id: i32,
    pub strategy_type: StrategyType,
    pub parameteres: [u8; 64],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct LiquidityOneSideParameter {
    pub amount: u64,
    pub active_id: i32,
    pub max_active_bin_slippage: i32,
    pub bin_liquidity_dist: Vec<BinLiquidityDistributionByWeight>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct BinLiquidityDistributionByWeight {
    pub bin_id: i32,
    pub weight: u16,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct LiquidityParameterByWeight {
    pub amount_x: u64,
    pub amount_y: u64,
    pub active_id: i32,
    pub max_active_bin_slippage: i32,
    pub bin_liquidity_dist: Vec<BinLiquidityDistributionByWeight>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct AddLiquiditySingleSidePreciseParameter {
    pub bins: Vec<CompressedBinDepositAmount>,
    pub decompress_multiplier: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct CompressedBinDepositAmount {
    pub bin_id: i32,
    pub amount: u32,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct BinLiquidityDistribution {
    pub bin_id: i32,
    pub distribution_x: u16,
    pub distribution_y: u16,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct LiquidityParameter {
    pub amount_x: u64,
    pub amount_y: u64,
    pub bin_liquidity_dist: Vec<BinLiquidityDistribution>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct CustomizableParams {
    pub active_id: i32,
    pub bin_step: u16,
    pub base_factor: u16,
    pub activation_type: u8,
    pub has_alpha_vault: bool,
    pub activation_point: Option<u64>,
    pub padding: [u8; 64],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct InitPermissionPairIx {
    pub active_id: i32,
    pub bin_step: u16,
    pub base_factor: u16,
    pub min_bin_id: i32,
    pub max_bin_id: i32,
    pub activation_type: u8,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct BinLiquidityReduction {
    pub bin_id: i32,
    pub bps_to_remove: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Bin {
    pub amount_x: u64, // 8
    pub amount_y: u64, // 8
    pub price: u128,   // 16
    pub liquidity_supply: u128, // 16
                       // 8 + 8 + 16 + 16 = 48
                       /*
                       pub reward_per_token_stored: [u128; 2], // u128 in bytes = 16
                       //pub padding0: [u8; 32],
                       pub fee_amount_x_per_token_stored: u128,
                       //pub padding1: [u8; 16],
                       pub fee_amount_y_per_token_stored: u128,
                       //pub padding2: [u8; 16],
                       pub amount_x_in: u128,
                       //pub padding3: [u8; 16],
                       pub amount_y_in: u128,
                       //pub padding4: [u8; 16],
                        */
                       //pub padding0: [u8; 64],
                       //pub padding1: [u8; 32],
}
// 8 + 8 + 16 + 16 + 64 + 32 = 144
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Pod, Zeroable, Copy)]
#[repr(C)]
pub struct ProtocolFee {
    pub amount_x: u64,
    pub amount_y: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RewardInfo {
    pub mint: Pubkey,                                        // 32
    pub vault: Pubkey,                                       // 64
    pub funder: Pubkey,                                      // 96
    pub reward_duration: u64,                                // 104
    pub reward_duration_end: u64,                            // 112
    pub reward_rate: u128,                                   // 128
    pub last_update_time: u64,                               // 136
    pub cumulative_seconds_with_empty_liquidity_reward: u64, // 144
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Observation {
    pub cumulative_active_bin_id: i128,
    pub created_at: i64,
    pub last_updated_at: i64,
}
#[derive(Clone, Debug, PartialEq, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct StaticParameters {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub min_bin_id: i32,
    pub max_bin_id: i32,
    pub protocol_share: u16,
    pub padding: [u8; 6],
}
#[derive(Clone, Debug, PartialEq, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct VariableParameters {
    pub volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub index_reference: i32,
    pub padding: [u8; 4],
    pub last_update_timestamp: i64,
    pub padding1: [u8; 8],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct FeeInfo {
    pub fee_x_per_token_complete: u128,
    pub fee_y_per_token_complete: u128,
    pub fee_x_pending: u64,
    pub fee_y_pending: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct UserRewardInfo {
    pub reward_per_token_completes: [u128; 2],
    pub reward_pendings: [u64; 2],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum StrategyType {
    SpotOneSide,
    CurveOneSide,
    BidAskOneSide,
    SpotBalanced,
    CurveBalanced,
    BidAskBalanced,
    SpotImBalanced,
    CurveImBalanced,
    BidAskImBalanced,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum Rounding {
    Up,
    Down,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ActivationType {
    Slot,
    Timestamp,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum LayoutVersion {
    V0,
    V1,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PairType {
    Permissionless,
    Permission,
    CustomizablePermissionless,
    PermissionlessV2,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PairStatus {
    Enabled,
    Disabled,
}
