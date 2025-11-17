use borsh::{BorshDeserialize, BorshSerialize};
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Observation {
    pub block_timestamp: u64,
    pub cumulative_token0_price_x32: u128,
    pub cumulative_token1_price_x32: u128,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum TradeDirection {
    ZeroForOne,
    OneForZero,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum RoundDirection {
    Floor,
    Ceiling,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum PoolStatusBitIndex {
    Deposit,
    Withdraw,
    Swap,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum PoolStatusBitFlag {
    Enable,
    Disable,
}
