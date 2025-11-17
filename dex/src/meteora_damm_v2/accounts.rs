use ahash::AHashSet;
use anyhow::Context;
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;
use utils::{pool::Pool, quote::QuoteResult};

use crate::meteora_damm_v2::{BaseFeeStruct, DynamicFeeStruct};

use super::{
    activation_dandler::get_current_point,
    constants::{AUTHORITY_DAMM_V2, EVENT_DAMM_V2},
    curve::{get_delta_amount_a_unsigned, get_delta_amount_b_unsigned, get_next_sqrt_price_from_input},
    math::u128x128_math::Rounding,
    quote::{FeeMode, FeeOnAmountResult, TradeDirection},
    PoolFeesStruct, SwapAmount, SwapIxArgs, SwapIxData, SwapKeys, SwapParameters, SwapResult,
};

pub const POOL_ACCOUNT_DISCM: [u8; 8] = [241, 154, 109, 4, 17, 177, 109, 188];
#[derive(Clone, Copy, Debug, PartialEq)] // BorshDeserialize, BorshSerialize,
pub struct PoolState {
    pub pool_fees: PoolFeesStruct,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub whitelisted_vault: Pubkey,
    pub partner: Pubkey,
    pub liquidity: u128,
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub protocol_a_fee: u64,
    pub protocol_b_fee: u64,
    pub partner_a_fee: u64,
    pub partner_b_fee: u64,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub sqrt_price: u128,
    pub activation_point: u64,
    pub activation_type: u8,
    pub pool_status: u8,
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    pub collect_fee_mode: u8,
    pub pool_type: u8,
    pub padding_0: [u8; 2],
    pub fee_a_per_liquidity: [u8; 32],
    pub fee_b_per_liquidity: [u8; 32],
    // pub permanent_lock_liquidity: u128,
    // pub metrics: PoolMetrics,
    // pub padding_1: [u64; 10],
    // pub reward_infos: [RewardInfo; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolAccount(pub PoolState);
impl PoolAccount {
    // pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
    //     use std::io::Read;
    //     let mut reader = buf;
    //     let mut maybe_discm = [0u8; 8];
    //     reader.read_exact(&mut maybe_discm)?;
    //     if maybe_discm != POOL_ACCOUNT_DISCM {
    //         return Err(std::io::Error::other(format!(
    //             "discm does not match. Expected: {POOL_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
    //         )));
    //     }
    //     Ok(Self(PoolState::deserialize(&mut reader)?))
    // }
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        // let pool = bytemuck::try_from_bytes::<PoolState>(&buf[8..])
        //     .map_err(|e| anyhow::anyhow!("FailedToDeserializePoolAccountMeteoraDAMMV2: {}", e))?;
        let pool = PoolState {
            pool_fees: //*bytemuck::try_from_bytes::<PoolFeesStruct>(&buf[8..168])
            //     .map_err(|e| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:PoolFees: {}", e))?,
            PoolFeesStruct {
                base_fee: *bytemuck::try_from_bytes::<BaseFeeStruct>(&buf[8..48])
                    .map_err(|e| anyhow::anyhow!("MeteoraDAMMv2::FailedToDeser::BaseFee: {}", e))?,
                protocol_fee_percent: buf[48],
                partner_fee_percent: buf[49],
                referral_fee_percent: buf[50],
                padding_0: [0u8; 5], // buf[51..56], --- IGNORE ---
                // dynamic_fee: *bytemuck::try_from_bytes::<DynamicFeeStruct>(&buf[56..152])
                //     .map_err(|e| anyhow::anyhow!("MeteoraDAMMv2::FailedToDeser::DynamicFee: {}", e))?,
                dynamic_fee: DynamicFeeStruct {
                    initialized: buf[56],
                    // padding: [u8; 7],
                    max_volatility_accumulator: u32::from_le_bytes(
                        buf[64..68]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::MaxVolatilityAccumulator")?,
                    ),
                    variable_fee_control: u32::from_le_bytes(
                        buf[68..72]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::VariableFeeControl")?,
                    ),
                    bin_step: u16::from_le_bytes(
                        buf[72..74]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::BinStep")?,
                    ),
                    filter_period: u16::from_le_bytes(
                        buf[74..76]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::FilterPeriod")?,
                    ),
                    decay_period: u16::from_le_bytes(
                        buf[76..78]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::DecayPeriod")?,
                    ),
                    reduction_factor: u16::from_le_bytes(
                        buf[78..80]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::ReductionFactor")?,
                    ),
                    last_update_timestamp: u64::from_le_bytes(
                        buf[80..88]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::LastUpdateTimestamp")?,
                    ),
                    bin_step_u128: u128::from_le_bytes(
                        buf[88..104]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::BinStepU128")?,
                    ),
                    sqrt_price_reference: u128::from_le_bytes(
                        buf[104..120]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::SqrtPriceReference")?,
                    ),
                    volatility_accumulator: u128::from_le_bytes(
                        buf[120..136]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::VolatilityAccumulator")?,
                    ),
                    volatility_reference: u128::from_le_bytes(
                        buf[136..152]
                            .try_into()
                            .context("MeteoraDAMMv2::FailedToDeser::VolatilityReference")?,
                    ),
                },
                padding_1: [0u64; 2], // buf[152..168], --- IGNORE ---
            },
            token_a_mint: *bytemuck::try_from_bytes::<Pubkey>(&buf[168..200])
                .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:TokenAMint"))?,
            token_b_mint: *bytemuck::try_from_bytes::<Pubkey>(&buf[200..232])
                .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:TokenBMint"))?,
            token_a_vault: *bytemuck::try_from_bytes::<Pubkey>(&buf[232..264])
                .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:TokenAVault"))?,
            token_b_vault: *bytemuck::try_from_bytes::<Pubkey>(&buf[264..296])
                .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:TokenBVault"))?,
            whitelisted_vault: *bytemuck::try_from_bytes::<Pubkey>(&buf[296..328])
                .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:WhitelistedVault"))?,
            partner: *bytemuck::try_from_bytes::<Pubkey>(&buf[328..360])
                .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:Partner"))?,
            liquidity: bytemuck::cast::<[u8; 16], u128>(
                buf[360..376]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::Liquidity")?,
            ),
            token_a_reserve: u64::from_le_bytes(
                buf[376..384]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::TokenAReserve")?,
            ),
            token_b_reserve: u64::from_le_bytes(
                buf[384..392]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::TokenBReserve")?,
            ),
            protocol_a_fee: u64::from_le_bytes(
                buf[392..400]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::ProtocolAFee")?,
            ),
            protocol_b_fee: u64::from_le_bytes(
                buf[400..408]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::ProtocolBFee")?,
            ),
            partner_a_fee: u64::from_le_bytes(
                buf[408..416]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::PartnerAFee")?,
            ),
            partner_b_fee: u64::from_le_bytes(
                buf[416..424]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::PartnerBFee")?,
            ),
            sqrt_min_price: u128::from_le_bytes(
                buf[424..440]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::SqrtMinPrice")?,
            ),
            sqrt_max_price: u128::from_le_bytes(
                buf[440..456]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::SqrtMaxPrice")?,
            ),
            sqrt_price: u128::from_le_bytes(
                buf[456..472]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::SqrtPrice")?,
            ),
            activation_point: u64::from_le_bytes(
                buf[472..480]
                    .try_into()
                    .context("MeteoraDAMMv2::FailedToDeser::ActivationPoint")?,
            ),
            activation_type: buf[480],
            pool_status: buf[481],
            token_a_flag: buf[482],
            token_b_flag: buf[483],
            collect_fee_mode: buf[484],
            pool_type: buf[485],
            padding_0: [0u8; 2], // buf[486..488], --- IGNORE ---
            fee_a_per_liquidity: buf[488..520]
                .try_into()
                .context("MeteoraDAMMv2::FailedToDeser::FeeAPerLiquidity")?,
            fee_b_per_liquidity: buf[520..552]
                .try_into()
                .context("MeteoraDAMMv2::FailedToDeser::FeeBPerLiquidity")?,
            // permanent_lock_liquidity: bytemuck::cast::<[u8; 16], u128>(
            //     buf[552..568]
            //         .try_into()
            //         .context("MeteoraDAMMv2::FailedToDeser::PermanentLockLiquidity")?,
            // ),
            // metrics: *bytemuck::try_from_bytes::<PoolMetrics>(&buf[568..600])
            //     .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:PoolMetrics"))?,
            // padding_1: [0u64; 10], // buf[600..680], --- IGNORE ---
            // reward_infos: [
            //     {
            //         let start = 680;
            //         let end = start + 80;
            //         *bytemuck::try_from_bytes::<RewardInfo>(&buf[start..end])
            //             .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:RewardInfo0"))?
            //     },
            //     {
            //         let start = 760;
            //         let end = start + 80;
            //         *bytemuck::try_from_bytes::<RewardInfo>(&buf[start..end])
            //             .map_err(|_| anyhow::anyhow!("MeteoraDAMMV2:FailedToDeser:RewardInfo1"))?
            //     },
            // ],
        };
        Ok(Self(pool))
    }
    // pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
    //     writer.write_all(&POOL_ACCOUNT_DISCM)?;
    //     self.0.serialize(&mut writer)
    // }
    // pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
    //     let mut data = Vec::new();
    //     self.serialize(&mut data)?;
    //     Ok(data)
    // }
}

impl PoolState {
    pub fn is_token_2022(&self) -> bool {
        self.token_a_flag != 0 || self.token_b_flag != 0
    }
}

impl PoolState {
    pub fn update_pre_swap(&mut self, current_timestamp: u64) -> anyhow::Result<()> {
        if self.pool_fees.dynamic_fee.is_dynamic_fee_enable() {
            self.pool_fees
                .dynamic_fee
                .update_references(self.sqrt_price, current_timestamp)?;
        }
        Ok(())
    }

    pub fn get_swap_result(
        &self,
        amount_in: u64,
        fee_mode: &FeeMode,
        trade_direction: TradeDirection,
        current_point: u64,
    ) -> anyhow::Result<SwapResult> {
        let mut actual_protocol_fee = 0;
        let mut actual_lp_fee = 0;
        let mut actual_referral_fee = 0;
        let mut actual_partner_fee = 0;

        let actual_amount_in = if fee_mode.fees_on_input {
            let FeeOnAmountResult {
                amount,
                lp_fee,
                protocol_fee,
                partner_fee,
                referral_fee,
            } = self.pool_fees.get_fee_on_amount(
                amount_in,
                fee_mode.has_referral,
                current_point,
                self.activation_point,
            )?;

            actual_protocol_fee = protocol_fee;
            actual_lp_fee = lp_fee;
            actual_referral_fee = referral_fee;
            actual_partner_fee = partner_fee;

            amount
        } else {
            amount_in
        };

        let SwapAmount {
            output_amount,
            next_sqrt_price,
        } = match trade_direction {
            TradeDirection::AtoB => self.get_swap_result_from_a_to_b(actual_amount_in),
            TradeDirection::BtoA => self.get_swap_result_from_b_to_a(actual_amount_in),
        }?;

        let actual_amount_out = if fee_mode.fees_on_input {
            output_amount
        } else {
            let FeeOnAmountResult {
                amount,
                lp_fee,
                protocol_fee,
                partner_fee,
                referral_fee,
            } = self.pool_fees.get_fee_on_amount(
                output_amount,
                fee_mode.has_referral,
                current_point,
                self.activation_point,
            )?;
            actual_protocol_fee = protocol_fee;
            actual_lp_fee = lp_fee;
            actual_referral_fee = referral_fee;
            actual_partner_fee = partner_fee;
            amount
        };

        Ok(SwapResult {
            output_amount: actual_amount_out,
            next_sqrt_price,
            lp_fee: actual_lp_fee,
            protocol_fee: actual_protocol_fee,
            partner_fee: actual_partner_fee,
            referral_fee: actual_referral_fee,
        })
    }

    fn get_swap_result_from_a_to_b(&self, amount_in: u64) -> anyhow::Result<SwapAmount> {
        // finding new target price
        let next_sqrt_price = get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount_in, true)?;

        if next_sqrt_price < self.sqrt_min_price {
            return Err(anyhow::format_err!("PriceRangeViolation"));
        }

        // finding output amount
        let output_amount =
            get_delta_amount_b_unsigned(next_sqrt_price, self.sqrt_price, self.liquidity, Rounding::Down)?;

        Ok(SwapAmount {
            output_amount,
            next_sqrt_price,
        })
    }

    fn get_swap_result_from_b_to_a(&self, amount_in: u64) -> anyhow::Result<SwapAmount> {
        // finding new target price
        let next_sqrt_price = get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount_in, false)?;

        if next_sqrt_price > self.sqrt_max_price {
            return Err(anyhow::format_err!("PriceRangeViolation"));
        }
        // finding output amount
        let output_amount =
            get_delta_amount_a_unsigned(self.sqrt_price, next_sqrt_price, self.liquidity, Rounding::Down)?;

        Ok(SwapAmount {
            output_amount,
            next_sqrt_price,
        })
    }
}

#[derive(Debug)]
pub struct QuoteInput {
    pub slot: u64,
}

impl Pool<QuoteInput> for PoolState {
    fn quote(&self, a_to_b: bool, exact_in: bool, amount_in: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
        if exact_in {
            let fee_mode = &FeeMode::get_fee_mode(
                self.collect_fee_mode,
                if a_to_b {
                    TradeDirection::AtoB
                } else {
                    TradeDirection::BtoA
                },
                false,
            )?;
            let current_point = get_current_point(self.activation_type, input.slot)?;
            if a_to_b {
                let res = self.get_swap_result(amount_in, fee_mode, TradeDirection::AtoB, current_point)?;
                Ok(QuoteResult {
                    amount_a_in: amount_in,
                    amount_b_in: 0,
                    amount_a_out: 0,
                    amount_b_out: res.output_amount,
                    indices: None,
                    compute_units: 48_000,
                })
            } else {
                let res = self.get_swap_result(amount_in, fee_mode, TradeDirection::BtoA, current_point)?;
                Ok(QuoteResult {
                    amount_a_in: 0,
                    amount_b_in: amount_in,
                    amount_a_out: res.output_amount,
                    amount_b_out: 0,
                    indices: None,
                    compute_units: 48_000,
                })
            }
        } else {
            Err(anyhow::anyhow!("ExactOutNotSupportedMeteoraDAMMV2"))?
        }
    }

    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.token_a_vault, &self.token_b_vault]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.token_a_vault, self.token_b_vault]
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        &self.token_a_mint
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        &self.token_b_mint
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        _a_to_b: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        if !exact_in {
            return Err(anyhow::anyhow!("ExactOutNotSupportedMeteoraDAMMV2"))?;
        }
        SwapIxData(SwapIxArgs {
            params: SwapParameters {
                amount_in: amount,
                minimum_amount_out: amount_threshold,
            },
        })
        .try_to_vec()
        .map_err(|e| anyhow::anyhow!("FailedToSerializeSwapDataMeteoraDAMMV2: {}", e))
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        _a_to_b: bool,
        _optional_account: Option<Pubkey>,
        _optional_accounts: Option<Vec<Pubkey>>,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<solana_sdk::instruction::AccountMeta>> {
        let keys = SwapKeys {
            pool_authority: AUTHORITY_DAMM_V2,
            pool: pool_pubkey,
            input_token_account: source_token_account,
            output_token_account: destination_token_account,
            token_a_vault: self.token_a_vault,
            token_b_vault: self.token_b_vault,
            token_a_mint: self.token_a_mint,
            token_b_mint: self.token_b_mint,
            payer: signer,
            token_a_program: if self.token_a_flag == 0 {
                spl_token::ID
            } else {
                spl_token_2022::ID
            },
            token_b_program: if self.token_b_flag == 0 {
                spl_token::ID
            } else {
                spl_token_2022::ID
            },
            referral_token_account: super::ID,
            event_authority: EVENT_DAMM_V2,
            program: super::ID,
        };
        Ok(vec![
            AccountMeta {
                pubkey: keys.pool_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.input_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.output_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_a_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_b_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_a_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_b_mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_a_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_b_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.referral_token_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.event_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.program,
                is_signer: false,
                is_writable: false,
            },
        ])
    }
}
