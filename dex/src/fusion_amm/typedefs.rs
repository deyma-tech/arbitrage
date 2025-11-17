use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum AccountsType {
    TransferHookA,
    TransferHookB,
    TransferHookInput,
    TransferHookIntermediate,
    TransferHookOutput,
    SupplementalTickArrays,
    SupplementalTickArraysOne,
    SupplementalTickArraysTwo,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct FusionPool {
    pub bump: [u8; 1],
    pub version: u16,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub tick_spacing: u16,
    pub tick_spacing_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub clp_reward_rate: u16,
    pub order_protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub fee_growth_global_a: u128,
    pub fee_growth_global_b: u128,
    pub orders_total_amount_a: u64,
    pub orders_total_amount_b: u64,
    pub orders_filled_amount_a: u64,
    pub orders_filled_amount_b: u64,
    pub olp_fee_owed_a: u64,
    pub olp_fee_owed_b: u64,
    pub reserved: [u8; 140],
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct FusionPoolsConfig {
    pub version: u16,
    pub fee_authority: Pubkey,
    pub collect_protocol_fees_authority: Pubkey,
    pub token_badge_authority: Pubkey,
    pub default_protocol_fee_rate: u16,
    pub default_order_protocol_fee_rate: u16,
    pub default_clp_reward_rate: u16,
    pub reserved: [u8; 170],
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct LimitOrder {
    pub version: u16,
    pub fusion_pool: Pubkey,
    pub limit_order_mint: Pubkey,
    pub tick_index: i32,
    pub amount: u64,
    pub a_to_b: bool,
    pub age: u64,
    pub reserved: [u8; 128],
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Position {
    pub version: u16,
    pub fusion_pool: Pubkey,
    pub position_mint: Pubkey,
    pub liquidity: u128,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub fee_growth_checkpoint_a: u128,
    pub fee_owed_a: u64,
    pub fee_growth_checkpoint_b: u128,
    pub fee_owed_b: u64,
    pub reserved: [u8; 128],
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PositionBundle {
    pub position_bundle_mint: Pubkey,
    pub position_bitmap: [u8; 32],
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RemainingAccountsInfo {
    pub slices: [RemainingAccountsSlice; 10],
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Tick {
    pub initialized: bool,                       // 1
    pub liquidity_net: i128,                     // 17
    pub liquidity_gross: u128,                   // 33
    pub fee_growth_outside_a: u128,              // 49
    pub fee_growth_outside_b: u128,              // 65
    pub age: u64,                                // 73
    pub open_orders_input: u64,                  // 81
    pub part_filled_orders_input: u64,           // 89
    pub part_filled_orders_remaining_input: u64, // 97
    pub fulfilled_a_to_b_orders_input: u64,      // 105
    pub fulfilled_b_to_a_orders_input: u64,      // 113
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct TickArray {
    pub start_tick_index: i32,
    pub ticks: [Tick; 88],
    pub fusion_pool: Pubkey,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct TokenBadge {
    pub token_mint: Pubkey,
    pub reserved: [u8; 128],
}
