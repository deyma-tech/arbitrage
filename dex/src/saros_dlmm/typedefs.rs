use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bin {
    pub total_supply: u128,
    pub reserve_x: u64,
    pub reserve_y: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinArray {
    pub pair: Pubkey,
    pub bins: [Bin; 256],
    pub index: u32,
    pub space: [u8; 12],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinArrayInitializationEvent {
    pub pair: Pubkey,
    pub index: u32,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinLiquidityDistribution {
    pub relative_bin_id: i32,
    pub distribution_x: u16,
    pub distribution_y: u16,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinStepConfig {
    pub bump: u8,
    pub liquidity_book_config: Pubkey,
    pub bin_step: u8,
    pub status: ConfigStatus,
    pub availability: ConfigAvailability,
    pub fee_parameters: StaticFeeParameters,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinStepConfigInitializationEvent {
    pub liquidity_book_config: Pubkey,
    pub bin_step_config: Pubkey,
    pub bin_step: u8,
    pub fee_parameters: StaticFeeParameters,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinStepConfigUpdateEvent {
    pub bin_step_config: Pubkey,
    pub status: ConfigStatus,
    pub availability: ConfigAvailability,
    pub fee_parameters: StaticFeeParameters,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinSwapEvent {
    pub pair: Pubkey,
    pub swap_for_y: bool,
    pub protocol_fee: u64,
    pub bin_id: u32,
    pub amount_in: u64,
    pub amount_out: u64,
    pub volatility_accumulator: u32,
    pub fee: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompositionFeesEvent {
    pub pair: Pubkey,
    pub active_id: u32,
    pub composition_fees_x: u64,
    pub composition_fees_y: u64,
    pub protocol_fees_x: u64,
    pub protocol_fees_y: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConfigAvailability {
    Closed,
    Open,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConfigStatus {
    Inactive,
    Active,
}
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DynamicFeeParameters {
    pub time_last_updated: u64,
    pub volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub id_reference: u32,
    pub space: [u8; 4],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiquidityBookConfig {
    pub preset_authority: Pubkey,
    pub pending_preset_authority: Option<Pubkey>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiquidityBookConfigInitializationEvent {
    pub config: Pubkey,
    pub preset_authority: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiquidityBookConfigTransferOwnershipEvent {
    pub new_authority: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiquidityBookConfigTransferOwnershipInitEvent {
    pub new_pending_authority: Option<Pubkey>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pair {
    pub bump: [u8; 1],
    pub liquidity_book_config: Pubkey,
    pub bin_step: u8,
    pub bin_step_seed: [u8; 1],
    pub token_mint_x: Pubkey,
    pub token_mint_y: Pubkey,
    pub static_fee_parameters: StaticFeeParameters,
    pub active_id: u32,
    pub dynamic_fee_parameters: DynamicFeeParameters,
    pub protocol_fees_x: u64,
    pub protocol_fees_y: u64,
    pub hook: Option<Pubkey>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PairInitializationEvent {
    pub pair: Pubkey,
    pub token_mint_x: Pubkey,
    pub token_mint_y: Pubkey,
    pub bin_step_config: Pubkey,
    pub active_id: u32,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PairStaticFeeParametersUpdateEvent {
    pub pair: Pubkey,
    pub fee_parameters: StaticFeeParameters,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    pub pair: Pubkey,
    pub position_mint: Pubkey,
    pub liquidity_shares: [u128; 64],
    pub lower_bin_id: u32,
    pub upper_bin_id: u32,
    pub space: [u8; 8],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PositionCreationEvent {
    pub pair: Pubkey,
    pub position: Pubkey,
    pub position_mint: Pubkey,
    pub lower_bin_id: u32,
    pub upper_bin_id: u32,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PositionDecreaseEvent {
    pub pair: Pubkey,
    pub position: Pubkey,
    pub bin_ids: Vec<u32>,
    pub amounts_x: Vec<u64>,
    pub amounts_y: Vec<u64>,
    pub liquidity_burned: Vec<u128>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PositionIncreaseEvent {
    pub pair: Pubkey,
    pub position: Pubkey,
    pub bin_ids: Vec<u32>,
    pub amounts_x: Vec<u64>,
    pub amounts_y: Vec<u64>,
    pub liquidity_minted: Vec<u128>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProtocolFeesCollectionEvent {
    pub pair: Pubkey,
    pub protocol_fees_x: u64,
    pub protocol_fees_y: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuoteAssetBadge {
    pub bump: u8,
    pub status: QuoteAssetBadgeStatus,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuoteAssetBadgeInitializationEvent {
    pub liquidity_book_config: Pubkey,
    pub quote_asset_badge: Pubkey,
    pub token_mint: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QuoteAssetBadgeStatus {
    Disabled,
    Enabled,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuoteAssetBadgeUpdateEvent {
    pub quote_asset_badge: Pubkey,
    pub status: QuoteAssetBadgeStatus,
}
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StaticFeeParameters {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub protocol_share: u16,
    pub space: [u8; 2],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SwapType {
    ExactInput,
    ExactOutput,
}
