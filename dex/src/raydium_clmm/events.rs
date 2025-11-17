#![allow(dead_code)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
pub const CONFIG_CHANGE_EVENT_EVENT_DISCM: [u8; 8] = [247, 189, 7, 119, 106, 112, 95, 151];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct ConfigChangeEvent {
    index: u16,
    owner: Pubkey,
    protocol_fee_rate: u32,
    trade_fee_rate: u32,
    tick_spacing: u16,
    fund_fee_rate: u32,
    fund_owner: Pubkey,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigChangeEventEvent(pub ConfigChangeEvent);
impl BorshSerialize for ConfigChangeEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        CONFIG_CHANGE_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl ConfigChangeEventEvent {}
pub const CREATE_PERSONAL_POSITION_EVENT_EVENT_DISCM: [u8; 8] = [100, 30, 87, 249, 196, 223, 154, 206];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct CreatePersonalPositionEvent {
    pool_state: Pubkey,
    minter: Pubkey,
    nft_owner: Pubkey,
    tick_lower_index: i32,
    tick_upper_index: i32,
    liquidity: u128,
    deposit_amount0: u64,
    deposit_amount1: u64,
    deposit_amount0_transfer_fee: u64,
    deposit_amount1_transfer_fee: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreatePersonalPositionEventEvent(pub CreatePersonalPositionEvent);
impl BorshSerialize for CreatePersonalPositionEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        CREATE_PERSONAL_POSITION_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl CreatePersonalPositionEventEvent {}
pub const INCREASE_LIQUIDITY_EVENT_EVENT_DISCM: [u8; 8] = [49, 79, 105, 212, 32, 34, 30, 84];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct IncreaseLiquidityEvent {
    position_nft_mint: Pubkey,
    liquidity: u128,
    amount0: u64,
    amount1: u64,
    amount0_transfer_fee: u64,
    amount1_transfer_fee: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IncreaseLiquidityEventEvent(pub IncreaseLiquidityEvent);
impl BorshSerialize for IncreaseLiquidityEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        INCREASE_LIQUIDITY_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl IncreaseLiquidityEventEvent {}
pub const DECREASE_LIQUIDITY_EVENT_EVENT_DISCM: [u8; 8] = [58, 222, 86, 58, 68, 50, 85, 56];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct DecreaseLiquidityEvent {
    position_nft_mint: Pubkey,
    liquidity: u128,
    decrease_amount0: u64,
    decrease_amount1: u64,
    fee_amount0: u64,
    fee_amount1: u64,
    reward_amounts: [u64; 3],
    transfer_fee0: u64,
    transfer_fee1: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DecreaseLiquidityEventEvent(pub DecreaseLiquidityEvent);
impl BorshSerialize for DecreaseLiquidityEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DECREASE_LIQUIDITY_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl DecreaseLiquidityEventEvent {}
pub const LIQUIDITY_CALCULATE_EVENT_EVENT_DISCM: [u8; 8] = [237, 112, 148, 230, 57, 84, 180, 162];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct LiquidityCalculateEvent {
    pool_liquidity: u128,
    pool_sqrt_price_x64: u128,
    pool_tick: i32,
    calc_amount0: u64,
    calc_amount1: u64,
    trade_fee_owed0: u64,
    trade_fee_owed1: u64,
    transfer_fee0: u64,
    transfer_fee1: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LiquidityCalculateEventEvent(pub LiquidityCalculateEvent);
impl BorshSerialize for LiquidityCalculateEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        LIQUIDITY_CALCULATE_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl LiquidityCalculateEventEvent {}
pub const COLLECT_PERSONAL_FEE_EVENT_EVENT_DISCM: [u8; 8] = [166, 174, 105, 192, 81, 161, 83, 105];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct CollectPersonalFeeEvent {
    position_nft_mint: Pubkey,
    recipient_token_account0: Pubkey,
    recipient_token_account1: Pubkey,
    amount0: u64,
    amount1: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollectPersonalFeeEventEvent(pub CollectPersonalFeeEvent);
impl BorshSerialize for CollectPersonalFeeEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        COLLECT_PERSONAL_FEE_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl CollectPersonalFeeEventEvent {}
pub const UPDATE_REWARD_INFOS_EVENT_EVENT_DISCM: [u8; 8] = [109, 127, 186, 78, 114, 65, 37, 236];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct UpdateRewardInfosEvent {
    reward_growth_global_x64: [u128; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateRewardInfosEventEvent(pub UpdateRewardInfosEvent);
impl BorshSerialize for UpdateRewardInfosEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        UPDATE_REWARD_INFOS_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl UpdateRewardInfosEventEvent {}
pub const POOL_CREATED_EVENT_EVENT_DISCM: [u8; 8] = [25, 94, 75, 47, 112, 99, 53, 63];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct PoolCreatedEvent {
    token_mint0: Pubkey,
    token_mint1: Pubkey,
    tick_spacing: u16,
    pool_state: Pubkey,
    sqrt_price_x64: u128,
    tick: i32,
    token_vault0: Pubkey,
    token_vault1: Pubkey,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PoolCreatedEventEvent(pub PoolCreatedEvent);
impl BorshSerialize for PoolCreatedEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        POOL_CREATED_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl PoolCreatedEventEvent {}
pub const COLLECT_PROTOCOL_FEE_EVENT_EVENT_DISCM: [u8; 8] = [206, 87, 17, 79, 45, 41, 213, 61];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct CollectProtocolFeeEvent {
    pool_state: Pubkey,
    recipient_token_account0: Pubkey,
    recipient_token_account1: Pubkey,
    amount0: u64,
    amount1: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollectProtocolFeeEventEvent(pub CollectProtocolFeeEvent);
impl BorshSerialize for CollectProtocolFeeEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        COLLECT_PROTOCOL_FEE_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl CollectProtocolFeeEventEvent {}
pub const SWAP_EVENT_EVENT_DISCM: [u8; 8] = [64, 198, 205, 232, 38, 8, 113, 226];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct SwapEvent {
    pool_state: Pubkey,
    sender: Pubkey,
    token_account0: Pubkey,
    token_account1: Pubkey,
    amount0: u64,
    transfer_fee0: u64,
    amount1: u64,
    transfer_fee1: u64,
    zero_for_one: bool,
    sqrt_price_x64: u128,
    liquidity: u128,
    tick: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SwapEventEvent(pub SwapEvent);
impl BorshSerialize for SwapEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        SWAP_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl SwapEventEvent {}
pub const LIQUIDITY_CHANGE_EVENT_EVENT_DISCM: [u8; 8] = [126, 240, 175, 206, 158, 88, 153, 107];
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct LiquidityChangeEvent {
    pool_state: Pubkey,
    tick: i32,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_before: u128,
    liquidity_after: u128,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LiquidityChangeEventEvent(pub LiquidityChangeEvent);
impl BorshSerialize for LiquidityChangeEventEvent {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        LIQUIDITY_CHANGE_EVENT_EVENT_DISCM.serialize(writer)?;
        self.0.serialize(writer)
    }
}
impl LiquidityChangeEventEvent {}
