use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum AccountsType {
    TransferHookA,
    TransferHookB,
    TransferHookReward,
    TransferHookInput,
    TransferHookIntermediate,
    TransferHookOutput,
    SupplementalTickArrays,
    SupplementalTickArraysOne,
    SupplementalTickArraysTwo,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct AmmConfig {
    pub bump: u8,
    pub disable_create_pool: bool,
    pub index: u16,
    pub trade_fee_rate: u64,
    pub protocol_fee_rate: u64,
    pub fund_fee_rate: u64,
    pub create_pool_fee: u64,
    pub protocol_owner: Pubkey,
    pub fund_owner: Pubkey,
    pub referral_project: Pubkey,
    pub max_open_time: u64,
    pub secondary_admin: Pubkey,
    pub padding: [u64; 7],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct BinLiquidityReduction {
    pub bin_id: i32,
    pub bps_to_remove: u16,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct LpChangeEvent {
    pub pool_id: Pubkey,
    pub lp_amount_before: u64,
    pub token_0vault_before: u64,
    pub token_1vault_before: u64,
    pub token_0amount: u64,
    pub token_1amount: u64,
    pub token_0transfer_fee: u64,
    pub token_1transfer_fee: u64,
    pub change_type: u8,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct MigrationEvent {
    pub from_pool: Pubkey,
    pub to_pool: Pubkey,
    pub token_0amount_withdrawn: u64,
    pub token_1amount_withdrawn: u64,
    pub lp_tokens_migrated: u128,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Observation {
    pub block_timestamp: u64,
    pub cumulative_token_0price_x32: u128,
    pub cumulative_token_1price_x32: u128,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct ObservationState {
    pub initialized: bool,
    pub observation_index: u16,
    pub pool_id: Pubkey,
    pub observations: [Observation; 100],
    pub padding: [u64; 4],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PartnerInfo {
    pub partner_id: u64,
    pub lp_token_linked_with_partner: u64,
    pub cumulative_fee_total_times_tvl_share_token_0: u64,
    pub cumulative_fee_total_times_tvl_share_token_1: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum PartnerType {
    AssetDash,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolState {
    pub amm_config: Pubkey,
    pub pool_creator: Pubkey,
    pub token_0vault: Pubkey,
    pub token_1vault: Pubkey,
    pub padding1: [u8; 32],
    pub token_0mint: Pubkey,
    pub token_1mint: Pubkey,
    pub token_0program: Pubkey,
    pub token_1program: Pubkey,
    pub observation_key: Pubkey,
    pub auth_bump: u8,
    pub status: u8,
    pub padding2: u8,
    pub mint_0decimals: u8,
    pub mint_1decimals: u8,
    pub lp_supply: u64,
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,
    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub cumulative_trade_fees_token_0: u128,
    pub cumulative_trade_fees_token_1: u128,
    pub cumulative_volume_token_0: u128,
    pub cumulative_volume_token_1: u128,
    pub latest_dynamic_fee_rate: u64,
    pub max_trade_fee_rate: u64,
    pub volatility_factor: u64,
    pub token_0vault_amount: u64,
    pub token_1vault_amount: u64,
    pub max_shared_token0: u64,
    pub max_shared_token1: u64,
    pub partners: [PartnerInfo; 1],
    pub token_0amount_in_kamino: u64,
    pub token_1amount_in_kamino: u64,
    pub withdrawn_kamino_profit_token_0: u64,
    pub withdrawn_kamino_profit_token_1: u64,
    pub padding: [u64; 8],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RemainingAccountsInfo {
    pub slices: Vec<RemainingAccountsSlice>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RewardInfo {
    pub pool: Pubkey,
    pub start_at: u64,
    pub end_rewards_at: u64,
    pub mint: Pubkey,
    pub total_to_disburse: u64,
    pub rewarded_by: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapEvent {
    pub pool_id: Pubkey,
    pub input_vault_before: u64,
    pub output_vault_before: u64,
    pub input_amount: u64,
    pub output_amount: u64,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_transfer_fee: u64,
    pub output_transfer_fee: u64,
    pub base_input: bool,
    pub dynamic_fee: u128,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct UserPoolLiquidity {
    pub user: Pubkey,
    pub pool_state: Pubkey,
    pub token_0deposited: u128,
    pub token_1deposited: u128,
    pub token_0withdrawn: u128,
    pub token_1withdrawn: u128,
    pub lp_tokens_owned: u128,
    pub partner: PartnerType,
    pub padding: [u8; 23],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct UserRewardInfo {
    pub total_claimed: u64,
    pub total_rewards: u64,
    pub rewards_last_calculated_at: u64,
}
