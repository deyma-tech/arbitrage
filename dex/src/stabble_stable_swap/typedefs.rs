use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolBalanceUpdatedData {
    pub balances: Vec<u64>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolBalanceUpdatedEvent {
    pub pubkey: Pubkey,
    pub data: PoolBalanceUpdatedData,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolToken {
    pub mint: Pubkey,
    pub decimals: u8,
    pub scaling_up: bool,
    pub scaling_factor: u64,
    pub balance: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolUpdatedData {
    pub is_active: bool,
    pub amp_initial_factor: u16,
    pub amp_target_factor: u16,
    pub ramp_start_ts: i64,
    pub ramp_stop_ts: i64,
    pub swap_fee: u64,
    pub max_supply: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolUpdatedEvent {
    pub pubkey: Pubkey,
    pub data: PoolUpdatedData,
}
