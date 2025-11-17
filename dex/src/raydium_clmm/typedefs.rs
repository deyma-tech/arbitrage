use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct InitializeRewardParam {
    pub open_time: u64,
    pub end_time: u64,
    pub emissions_per_second_x64: u128,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Observation {
    pub block_timestamp: u32,
    pub tick_cumulative: i64,
    pub padding: [u64; 4],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PositionRewardInfo {
    pub growth_inside_last_x64: u128,
    pub reward_amount_owed: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Default, Copy)]
pub struct RewardInfo {
    pub reward_state: u8,               // 1
    pub open_time: u64,                 // 8
    pub end_time: u64,                  // 8
    pub last_update_time: u64,          // 8
    pub emissions_per_second_x64: u128, // 16
    pub reward_total_emissioned: u64,   // 8
    pub reward_claimed: u64,            // 8
    pub token_mint: Pubkey,             // 32
    pub token_vault: Pubkey,            // 32
    pub authority: Pubkey,              // 32
    pub reward_growth_global_x64: u128, // 16
                                        // total size: 1 + 8 + 8 + 8 + 16 + 8 + 8 + 32 + 32 + 32 + 16 = 161
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Copy, Default)]
pub struct TickState {
    pub tick: i32,                             // 4
    pub liquidity_net: i128,                   // 16
    pub liquidity_gross: u128,                 // 16
    pub fee_growth_outside0_x64: u128,         // 16
    pub fee_growth_outside1_x64: u128,         // 16
    pub reward_growths_outside_x64: [u128; 3], // 48
    pub padding: [u32; 13],                    // 52
                                               // total: 4 + 16 + 16 + 16 + 16 + 48 + 52 = 168
}

#[derive(Clone, Debug, PartialEq, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct RayTickState {
    pub liquidity_net: i128,           // 16
    pub liquidity_gross: u128,         // 16
    pub fee_growth_outside0_x64: u128, // 16
    pub fee_growth_outside1_x64: u128, // 16
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum PoolStatusBitIndex {
    OpenPositionOrIncreaseLiquidity,
    DecreaseLiquidity,
    CollectFee,
    CollectReward,
    Swap,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum PoolStatusBitFlag {
    Enable,
    Disable,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum RewardState {
    Uninitialized,
    Initialized,
    Opening,
    Ended,
}
/*
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct TickArryBitmap {
    pub tick_arry_bitmap: [u64; 8],
}
*/
