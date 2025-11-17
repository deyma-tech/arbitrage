use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use solana_program::pubkey::Pubkey;

use super::{
    constants::{BASIS_POINT_MAX, FEE_DENOMINATOR, MAX_FEE_NUMERATOR},
    math::{
        fee_math::get_fee_in_period, safe_math::SafeMath, u128x128_math::Rounding, utils_math::safe_mul_div_cast_u64,
    },
    quote::FeeOnAmountResult,
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct AddLiquidityParameters {
    pub liquidity_delta: u128,
    pub token_a_amount_threshold: u64,
    pub token_b_amount_threshold: u64,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct BaseFeeConfig {
    pub cliff_fee_numerator: u64,
    pub fee_scheduler_mode: u8,
    pub padding: [u8; 5],
    pub number_of_period: u16,
    pub period_frequency: u64,
    pub reduction_factor: u64,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct BaseFeeParameters {
    pub cliff_fee_numerator: u64,
    pub number_of_period: u16,
    pub period_frequency: u64,
    pub reduction_factor: u64,
    pub fee_scheduler_mode: u8,
}

/// collect fee mode
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
// https://www.desmos.com/calculator/oxdndn2xdx
pub enum FeeSchedulerMode {
    // fee = cliff_fee_numerator - passed_period * reduction_factor
    Linear,
    // fee = cliff_fee_numerator * (1-reduction_factor/10_000)^passed_period
    Exponential,
}

#[derive(Clone, Copy, Debug, Zeroable, Pod, PartialEq)] // BorshDeserialize, BorshSerialize
#[repr(C)]
pub struct BaseFeeStruct {
    pub cliff_fee_numerator: u64, // 8
    pub fee_scheduler_mode: u8,   // 1
    pub padding_0: [u8; 5],       // 5
    pub number_of_period: u16,    // 2
    pub period_frequency: u64,    // 8
    pub reduction_factor: u64,    // 8
    pub padding_1: u64,           // 8
                                  // Total: 40 bytes
}

impl BaseFeeStruct {
    pub fn get_max_base_fee_numerator(&self) -> u64 {
        self.cliff_fee_numerator
    }
    pub fn get_min_base_fee_numerator(&self) -> anyhow::Result<u64> {
        // trick to force current_point < activation_point
        self.get_current_base_fee_numerator(0, 1)
    }
    pub fn get_current_base_fee_numerator(&self, current_point: u64, activation_point: u64) -> anyhow::Result<u64> {
        if self.period_frequency == 0 {
            return Ok(self.cliff_fee_numerator);
        }
        // can trade before activation point, so it is alpha-vault, we use min fee
        let period = if current_point < activation_point {
            self.number_of_period.into()
        } else {
            let period = current_point
                .safe_sub(activation_point)?
                .safe_div(self.period_frequency)?;
            period.min(self.number_of_period.into())
        };
        let fee_scheduler_mode =
            FeeSchedulerMode::try_from(self.fee_scheduler_mode).map_err(|_| anyhow::format_err!("TypeCastFailed"))?;

        match fee_scheduler_mode {
            FeeSchedulerMode::Linear => {
                let fee_numerator = self
                    .cliff_fee_numerator
                    .safe_sub(period.safe_mul(self.reduction_factor)?)?;
                Ok(fee_numerator)
            }
            FeeSchedulerMode::Exponential => {
                let period = u16::try_from(period).map_err(|_| anyhow::format_err!("MathOverflow"))?;
                let fee_numerator = get_fee_in_period(self.cliff_fee_numerator, self.reduction_factor, period)?;
                Ok(fee_numerator)
            }
        }
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct ClaimFeeOperator {
    pub operator: Pubkey,
    pub padding: [u8; 128],
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Config {
    pub vault_config_key: Pubkey,
    pub pool_creator_authority: Pubkey,
    pub pool_fees: PoolFeesConfig,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
    pub padding_0: [u8; 6],
    pub index: u64,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub padding_1: [u64; 10],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct ConfigParameters {
    pub pool_fees: PoolFeeParameters,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub vault_config_key: Pubkey,
    pub pool_creator_authority: Pubkey,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
    pub index: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct DynamicFeeConfig {
    pub initialized: u8,
    pub padding: [u8; 7],
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
    pub bin_step: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub padding_1: [u8; 8],
    pub bin_step_u128: u128,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct DynamicFeeParameters {
    pub bin_step: u16,
    pub bin_step_u128: u128,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
//#[repr(C)]
pub struct DynamicFeeStruct {
    pub initialized: u8, // 1
    // pub padding: [u8; 7],                // 7
    pub max_volatility_accumulator: u32, // 4
    pub variable_fee_control: u32,       // 4
    pub bin_step: u16,                   // 2
    pub filter_period: u16,              // 2
    pub decay_period: u16,               // 2
    pub reduction_factor: u16,           // 2
    pub last_update_timestamp: u64,      // 8
    pub bin_step_u128: u128,             // 16
    pub sqrt_price_reference: u128,      // 16
    pub volatility_accumulator: u128,    // 16
    pub volatility_reference: u128,      // 16
                                         // Total: 96 bytes
}

impl DynamicFeeStruct {
    pub fn is_dynamic_fee_enable(&self) -> bool {
        self.initialized != 0
    }

    pub fn update_references(&mut self, sqrt_price_current: u128, current_timestamp: u64) -> Result<()> {
        let elapsed = current_timestamp.safe_sub(self.last_update_timestamp)?;
        // Not high frequency trade
        if elapsed >= self.filter_period as u64 {
            // Update sqrt of last transaction
            self.sqrt_price_reference = sqrt_price_current;
            // filter period < t < decay_period. Decay time window.
            if elapsed < self.decay_period as u64 {
                let volatility_reference = self
                    .volatility_accumulator
                    .safe_mul(self.reduction_factor.into())?
                    .safe_div(BASIS_POINT_MAX.into())?;

                self.volatility_reference = volatility_reference;
            }
            // Out of decay time window
            else {
                self.volatility_reference = 0;
            }
        }
        Ok(())
    }

    pub fn get_variable_fee(&self) -> Result<u128> {
        if self.is_dynamic_fee_enable() {
            let square_vfa_bin: u128 = self
                .volatility_accumulator
                .safe_mul(self.bin_step.into())?
                .checked_pow(2)
                .unwrap();
            // Variable fee control, volatility accumulator, bin step are in basis point unit (10_000)
            // This is 1e20. Which > 1e9. Scale down it to 1e9 unit and ceiling the remaining.
            let v_fee = square_vfa_bin.safe_mul(self.variable_fee_control.into())?;

            let scaled_v_fee = v_fee.safe_add(99_999_999_999)?.safe_div(100_000_000_000)?;

            Ok(scaled_v_fee)
        } else {
            Ok(0)
        }
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtAddLiquidity {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub params: AddLiquidityParameters,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub total_amount_a: u64,
    pub total_amount_b: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtClaimPartnerFee {
    pub pool: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtClaimPositionFee {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub fee_a_claimed: u64,
    pub fee_b_claimed: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtClaimProtocolFee {
    pub pool: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtClaimReward {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub mint_reward: Pubkey,
    pub reward_index: u8,
    pub total_reward: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtCloseClaimFeeOperator {
    pub claim_fee_operator: Pubkey,
    pub operator: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtCloseConfig {
    pub config: Pubkey,
    pub admin: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtClosePosition {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub position: Pubkey,
    pub position_nft_mint: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtCreateClaimFeeOperator {
    pub operator: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtCreateConfig {
    pub pool_fees: PoolFeeParameters,
    pub vault_config_key: Pubkey,
    pub pool_creator_authority: Pubkey,
    pub activation_type: u8,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub collect_fee_mode: u8,
    pub index: u64,
    pub config: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtCreatePosition {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub position: Pubkey,
    pub position_nft_mint: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtCreateTokenBadge {
    pub token_mint: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtFundReward {
    pub pool: Pubkey,
    pub funder: Pubkey,
    pub mint_reward: Pubkey,
    pub reward_index: u8,
    pub amount: u64,
    pub transfer_fee_excluded_amount_in: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtInitializePool {
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub creator: Pubkey,
    pub payer: Pubkey,
    pub alpha_vault: Pubkey,
    pub pool_fees: PoolFeeParameters,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub activation_point: u64,
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub total_amount_a: u64,
    pub total_amount_b: u64,
    pub pool_type: u8,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtInitializeReward {
    pub pool: Pubkey,
    pub reward_mint: Pubkey,
    pub funder: Pubkey,
    pub reward_index: u8,
    pub reward_duration: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtLockPosition {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub vesting: Pubkey,
    pub cliff_point: u64,
    pub period_frequency: u64,
    pub cliff_unlock_liquidity: u128,
    pub liquidity_per_period: u128,
    pub number_of_period: u16,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtPermanentLockPosition {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub lock_liquidity_amount: u128,
    pub total_permanent_locked_liquidity: u128,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtRemoveLiquidity {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub params: RemoveLiquidityParameters,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtSetPoolStatus {
    pub pool: Pubkey,
    pub status: u8,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtSwap {
    pub pool: Pubkey,
    pub trade_direction: u8,
    pub has_referral: bool,
    pub params: SwapParameters,
    pub swap_result: SwapResult,
    pub actual_amount_in: u64,
    pub current_timestamp: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtUpdateRewardDuration {
    pub pool: Pubkey,
    pub reward_index: u8,
    pub old_reward_duration: u64,
    pub new_reward_duration: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtUpdateRewardFunder {
    pub pool: Pubkey,
    pub reward_index: u8,
    pub old_funder: Pubkey,
    pub new_funder: Pubkey,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct EvtWithdrawIneligibleReward {
    pub pool: Pubkey,
    pub reward_mint: Pubkey,
    pub amount: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct InitializeCustomizablePoolParameters {
    pub pool_fees: PoolFeeParameters,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub has_alpha_vault: bool,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
    pub activation_point: Option<u64>,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct InitializePoolParameters {
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub activation_point: Option<u64>,
}

// #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
// pub struct Pool {
//     pub pool_fees: PoolFeesStruct,
//     pub token_a_mint: Pubkey,
//     pub token_b_mint: Pubkey,
//     pub token_a_vault: Pubkey,
//     pub token_b_vault: Pubkey,
//     pub whitelisted_vault: Pubkey,
//     pub partner: Pubkey,
//     pub liquidity: u128,
//     pub token_a_reserve: u64,
//     pub token_b_reserve: u64,
//     pub protocol_a_fee: u64,
//     pub protocol_b_fee: u64,
//     pub partner_a_fee: u64,
//     pub partner_b_fee: u64,
//     pub sqrt_min_price: u128,
//     pub sqrt_max_price: u128,
//     pub sqrt_price: u128,
//     pub activation_point: u64,
//     pub activation_type: u8,
//     pub pool_status: u8,
//     pub token_a_flag: u8,
//     pub token_b_flag: u8,
//     pub collect_fee_mode: u8,
//     pub pool_type: u8,
//     pub padding_0: [u8; 2],
//     pub fee_a_per_liquidity: [u8; 32],
//     pub fee_b_per_liquidity: [u8; 32],
//     pub permanent_lock_liquidity: u128,
//     pub metrics: PoolMetrics,
//     pub padding_1: [u64; 10],
//     pub reward_infos: [RewardInfo; 2],
// }

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolFeeParameters {
    pub base_fee: BaseFeeParameters,
    pub protocol_fee_percent: u8,
    pub partner_fee_percent: u8,
    pub referral_fee_percent: u8,
    pub dynamic_fee: DynamicFeeParameters,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolFeesConfig {
    pub base_fee: BaseFeeConfig,
    pub dynamic_fee: DynamicFeeConfig,
    pub protocol_fee_percent: u8,
    pub partner_fee_percent: u8,
    pub referral_fee_percent: u8,
    pub padding_0: [u8; 5],
    pub padding_1: [u64; 5],
}

#[derive(Clone, Copy, Debug, PartialEq)] // BorshDeserialize, BorshSerialize,
pub struct PoolFeesStruct {
    pub base_fee: BaseFeeStruct,       // 40
    pub protocol_fee_percent: u8,      // 1
    pub partner_fee_percent: u8,       // 1
    pub referral_fee_percent: u8,      // 1
    pub padding_0: [u8; 5],            // 5
    pub dynamic_fee: DynamicFeeStruct, // 96
    pub padding_1: [u64; 2],           // 16
                                       // Total: 160 bytes
}

impl PoolFeesStruct {
    // in numerator
    pub fn get_total_trading_fee(&self, current_point: u64, activation_point: u64) -> anyhow::Result<u128> {
        let base_fee_numerator = self
            .base_fee
            .get_current_base_fee_numerator(current_point, activation_point)?;
        let total_fee_numerator = self
            .dynamic_fee
            .get_variable_fee()?
            .safe_add(base_fee_numerator.into())?;
        Ok(total_fee_numerator)
    }

    pub fn get_fee_on_amount(
        &self,
        amount: u64,
        has_referral: bool,
        current_point: u64,
        activation_point: u64,
    ) -> anyhow::Result<FeeOnAmountResult> {
        let trade_fee_numerator = self.get_total_trading_fee(current_point, activation_point)?;
        let trade_fee_numerator = if trade_fee_numerator > (MAX_FEE_NUMERATOR as u128) {
            MAX_FEE_NUMERATOR
        } else {
            trade_fee_numerator.try_into().unwrap()
        };
        let lp_fee: u64 = safe_mul_div_cast_u64(amount, trade_fee_numerator, FEE_DENOMINATOR, Rounding::Up)?;
        // update amount
        let amount = amount.safe_sub(lp_fee)?;

        let protocol_fee = safe_mul_div_cast_u64(lp_fee, self.protocol_fee_percent.into(), 100, Rounding::Down)?;
        // update lp fee
        let lp_fee = lp_fee.safe_sub(protocol_fee)?;

        let referral_fee = if has_referral {
            safe_mul_div_cast_u64(protocol_fee, self.referral_fee_percent.into(), 100, Rounding::Down)?
        } else {
            0
        };

        let protocol_fee_after_referral_fee = protocol_fee.safe_sub(referral_fee)?;
        let partner_fee = safe_mul_div_cast_u64(
            protocol_fee_after_referral_fee,
            self.partner_fee_percent.into(),
            100,
            Rounding::Down,
        )?;

        let protocol_fee = protocol_fee_after_referral_fee.safe_sub(partner_fee)?;

        Ok(FeeOnAmountResult {
            amount,
            lp_fee,
            protocol_fee,
            partner_fee,
            referral_fee,
        })
    }
}

#[derive(Clone, Copy, Debug, Zeroable, Pod, PartialEq)]
#[repr(C)]
pub struct PoolMetrics {
    pub total_lp_a_fee: u128,
    pub total_lp_b_fee: u128,
    pub total_protocol_a_fee: u64,
    pub total_protocol_b_fee: u64,
    pub total_partner_a_fee: u64,
    pub total_partner_b_fee: u64,
    pub total_position: u64,
    pub padding: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Position {
    pub pool: Pubkey,
    pub nft_mint: Pubkey,
    pub fee_a_per_token_checkpoint: [u8; 32],
    pub fee_b_per_token_checkpoint: [u8; 32],
    pub fee_a_pending: u64,
    pub fee_b_pending: u64,
    pub unlocked_liquidity: u128,
    pub vested_liquidity: u128,
    pub permanent_locked_liquidity: u128,
    pub metrics: PositionMetrics,
    pub reward_infos: [UserRewardInfo; 2],
    pub padding: [u128; 6],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PositionMetrics {
    pub total_claimed_a_fee: u64,
    pub total_claimed_b_fee: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct RemoveLiquidityParameters {
    pub liquidity_delta: u128,
    pub token_a_amount_threshold: u64,
    pub token_b_amount_threshold: u64,
}
#[derive(Clone, Copy, Debug, Zeroable, Pod, PartialEq)]
#[repr(C)]
pub struct RewardInfo {
    pub initialized: u8,
    pub reward_token_flag: u8,
    pub padding_0: [u8; 6],
    pub padding_1: [u8; 8],
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub reward_duration: u64,
    pub reward_duration_end: u64,
    pub reward_rate: u128,
    pub reward_per_token_stored: [u8; 32],
    pub last_update_time: u64,
    pub cumulative_seconds_with_empty_liquidity_reward: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapParameters {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct SwapResult {
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

pub struct SwapAmount {
    pub output_amount: u64,
    pub next_sqrt_price: u128,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct TokenBadge {
    pub token_mint: Pubkey,
    pub padding: [u8; 128],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct UserRewardInfo {
    pub reward_per_token_checkpoint: [u8; 32],
    pub reward_pendings: u64,
    pub total_claimed_rewards: u64,
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Vesting {
    pub position: Pubkey,
    pub cliff_point: u64,
    pub period_frequency: u64,
    pub cliff_unlock_liquidity: u128,
    pub liquidity_per_period: u128,
    pub total_released_liquidity: u128,
    pub number_of_period: u16,
    pub padding: [u8; 14],
    pub padding2: [u128; 4],
}
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct VestingParameters {
    pub cliff_point: Option<u64>,
    pub period_frequency: u64,
    pub cliff_unlock_liquidity: u128,
    pub liquidity_per_period: u128,
    pub number_of_period: u16,
}
