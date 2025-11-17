use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct AddLiquidity {
    pub lp_token_mint_amount: u64,
    pub x_reserve_deltas: [u64; 10],
    pub y_reserve_deltas: [u64; 10],
    pub inv_l_deltas: [u64; 10],
    pub min_lp_token_mint_amount: u64,
    pub trader: Pubkey,
    pub pool: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct AddLiquidityData {
    pub max_amounts_in: [u64; 10],
    pub min_lp_token_mint_amount: u64,
    pub take_swaps: u8,
    pub swap_paths: [u8; 10],
    pub swap_amounts: [u64; 10],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct CreateStablePoolData {
    pub decimals: u8,
    pub fee_num: u32,
    pub fee_denom: u32,
    pub pool_seed: Pubkey,
    pub weights: [u32; 10],
    pub inv_t: u64,
    pub inv_t_max: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct InitVirtualStablePairData {
    pub decimals: u8,
    pub init_amount: u64,
    pub curve_amp: u128,
    pub curve_a: u128,
    pub curve_b: u128,
    pub curve_alpha: u64,
    pub curve_beta: u64,
    pub pair_seed: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct NumeraireConfig {
    pub owner: Pubkey,
    pub status: u32,
    pub rate_mints: [Pubkey; 10],
    pub rate_nums: [u32; 10],
    pub rate_denoms: [u32; 10],
    pub padding0: [u8; 12],
    pub padding1: [u8; 1024],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Quote {
    pub amount: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RemoveLiquidity {
    pub lp_token_redeem_amount: u64,
    pub x_reserve_deltas: [u64; 10],
    pub y_reserve_deltas: [u64; 10],
    pub inv_l_deltas: [u64; 10],
    pub min_amounts_out: [u64; 10],
    pub trader: Pubkey,
    pub pool: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RemoveLiquidityData {
    pub lp_token_redeem_amount: u64,
    pub min_amounts_out: [u64; 10],
    pub out_index: u8,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SetFeeData {
    pub fee_num: u32,
    pub fee_denom: u32,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SetInvTMaxData {
    pub inv_t_max: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SetMetadataData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SetOwnerData {
    pub owner: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SetRateData {
    pub rate_mint: Pubkey,
    pub rate_num: u32,
    pub rate_denom: u32,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SetStatusData {
    pub status: u32,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SetWhilelistedAddrData {
    pub whitelisted_addr: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct StablePool {
    pub pool_seed: Pubkey,
    pub lp_mint: Pubkey,
    pub whitelisted_adder: Pubkey,
    pub owner: Pubkey,
    pub inv_t: u64,
    pub inv_t_max: u64,
    pub pairs: [VirtualStablePair; 10],
    pub weights: [u32; 10],
    pub total_weight: u64,
    pub status: u32,
    pub fee_num: u32,
    pub fee_denom: u32,
    pub decimals: u8,
    pub num_stables: u8,
    pub padding0: [u8; 2],
    pub padding1: [u8; 128],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapExactIn {
    pub amount_in: u64,
    pub amount_out: u64,
    pub min_amount_out: u64,
    pub trader: Pubkey,
    pub in_index: u8,
    pub out_index: u8,
    pub pool: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapExactInData {
    pub in_index: u8,
    pub out_index: u8,
    pub exact_amount_in: u64,
    pub min_amount_out: u64,
    pub hints: [u64; 10],
    pub path_hints: [u8; 10],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapExactInHintlessData {
    pub in_index: u8,
    pub out_index: u8,
    pub exact_amount_in: u64,
    pub min_amount_out: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapExactOut {
    pub amount_in: u64,
    pub amount_out: u64,
    pub max_amount_in: u64,
    pub trader: Pubkey,
    pub in_index: u8,
    pub out_index: u8,
    pub pool: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapExactOutData {
    pub in_index: u8,
    pub out_index: u8,
    pub exact_amount_out: u64,
    pub max_amount_in: u64,
    pub hints: [u64; 10],
    pub path_hints: [u8; 10],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapExactOutHintlessData {
    pub in_index: u8,
    pub out_index: u8,
    pub exact_amount_out: u64,
    pub max_amount_in: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct VirtualStablePair {
    pub pair_authority: Pubkey,
    pub x_reserve_amount: u64,
    pub y_reserve: u64,
    pub curve_amp: u128,
    pub curve_a: u128,
    pub curve_b: u128,
    pub inv_l: u128,
    pub owner: Pubkey,
    pub x_mint: Pubkey,
    pub x_vault: Pubkey,
    pub curve_alpha: u64,
    pub curve_beta: u64,
    pub newest_rate_num: u32,
    pub newest_rate_denom: u32,
    pub decimals: u8,
    pub pair_index: u8,
    pub x_is_2022: u8,
    pub padding0: [u8; 5],
    pub padding1: [u8; 128],
}
