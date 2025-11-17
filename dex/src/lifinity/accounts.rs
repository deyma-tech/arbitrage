use crate::lifinity::pc::{Price, PriceStatus};
use crate::lifinity::pda::get_authority;
use crate::lifinity::LifinityAmmV2ProgramIx::Swap;
use crate::lifinity::{AmmConfig, AmmCurve, AmmFees, SwapIxArgs, SwapKeys};
use crate::raydium_amm::CheckedCeilDiv;
use ahash::AHashSet;
use anyhow::{format_err, Context};
//use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::ToPrimitive;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use std::cmp::PartialEq;
use utils::constants::SPL_TOKEN_2022_ID;
use utils::pool::Pool;
use utils::quote::QuoteResult;
use utils::types::MintPair;

impl Price {
    pub fn try_deserialize(data: &[u8]) -> Option<Self> {
        let result = bytemuck::try_from_bytes::<Price>(data);
        match result {
            Ok(price) => Some(*price),
            Err(_) => None,
        }
    }
}

pub const AMM_ACCOUNT_DISCM: [u8; 8] = [143, 245, 200, 17, 74, 214, 196, 135];
#[derive(Clone, Debug, PartialEq)] // BorshDeserialize, BorshSerialize,
pub struct Amm {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receive_token_account: Pubkey,
    pub initializer_amount: u64,
    pub taker_amount: u64,
    pub is_initialized: bool,
    pub bump_seed: u8,
    pub freeze_trade: u8,
    pub freeze_deposit: u8,
    pub freeze_withdraw: u8,
    pub base_decimals: u8,
    pub token_program_id: Pubkey,
    pub token_a_account: Pubkey,
    pub token_b_account: Pubkey,
    pub pool_mint: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub fee_account: Pubkey,
    pub oracle_main_account: Pubkey,
    pub oracle_sub_account: Pubkey,
    pub oracle_pc_account: Pubkey,
    pub fees: AmmFees,
    pub curve: AmmCurve,
    pub config: AmmConfig,
    pub amm_p_temp1: Pubkey,
    pub amm_p_temp2: Pubkey,
    pub amm_p_temp3: Pubkey,
    pub amm_p_temp4: Pubkey,
    pub amm_p_temp5: Pubkey,
}

pub struct QuoteInput<'a> {
    pub coin_a: u64,
    pub coin_b: u64,
    pub slot: u64,
    pub oracle_main: &'a Price,
    pub oracle_sub: &'a Price,
    pub oracle_pc: &'a Price,
}

impl Pool<QuoteInput<'_>> for Amm {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![
            &self.token_a_account,
            &self.token_b_account,
            &self.token_a_mint,
            &self.token_b_mint,
        ]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![
            self.token_a_account,
            self.token_b_account,
            self.token_a_mint,
            self.token_b_mint,
        ]
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount_in: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
        if exact_in {
            if a_to_b {
                if amount_in > input.coin_a {
                    return Err(format_err!("LifinityPoolOutOfLiquidity"));
                }
            } else if amount_in > input.coin_b {
                return Err(format_err!("LifinityPoolOutOfLiquidity"));
            }

            let result = self.get_curve_amount(
                amount_in,
                input.slot,
                a_to_b,
                input.coin_a,
                input.coin_b,
                input.oracle_main,
                input.oracle_sub,
                input.oracle_pc,
            );
            match result {
                Ok(amount_out) => {
                    let mut quote_result = QuoteResult {
                        amount_a_in: 0,
                        amount_b_in: 0,
                        amount_a_out: 0,
                        amount_b_out: 0,
                        indices: None,
                        compute_units: 67_500,
                    };
                    if a_to_b {
                        quote_result.amount_a_in = amount_in;
                        quote_result.amount_b_out = amount_out;
                    } else {
                        quote_result.amount_b_in = amount_in;
                        quote_result.amount_a_out = amount_out;
                    }
                    return Ok(quote_result);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Err(format_err!("NotAllowed (Lifinity)"))
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
        if exact_in {
            Swap(SwapIxArgs {
                amount_in: amount,
                minimum_amount_out: amount_threshold,
            })
            .try_to_vec()
            .context("Failed to serialize swap instruction")
        } else {
            Ok(vec![])
        }
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        _optional_account: Option<Pubkey>,
        _optional_accounts: Option<Vec<Pubkey>>,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        let keys = SwapKeys {
            authority: get_authority(&pool_pubkey),
            amm: pool_pubkey,
            user_transfer_authority: signer,
            source_info: source_token_account,
            destination_info: destination_token_account,
            swap_source: if a_to_b {
                self.token_a_account
            } else {
                self.token_b_account
            },
            swap_destination: if a_to_b {
                self.token_b_account
            } else {
                self.token_a_account
            },
            pool_mint: self.pool_mint,
            fee_account: self.fee_account,
            token_program: self.token_program_id,
            oracle_main_account: self.oracle_main_account,
            oracle_sub_account: self.oracle_sub_account,
            oracle_pc_account: self.oracle_pc_account,
        };
        Ok(vec![
            AccountMeta {
                pubkey: keys.authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.amm,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_transfer_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.source_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.destination_info,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.swap_source,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.swap_destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.fee_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.oracle_main_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.oracle_sub_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.oracle_pc_account,
                is_signer: false,
                is_writable: false,
            },
        ])
    }

    fn get_count_mint_pairs(&self) -> u64 {
        1
    }

    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        vec![self.get_sorted_mints_as_array()]
    }
}

impl Amm {
    pub fn is_token_2022(&self) -> bool {
        self.token_program_id == SPL_TOKEN_2022_ID
    }

    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Amm> {
        let amm =
            Amm {
                initializer_key: *bytemuck::try_from_bytes::<Pubkey>(&buf[8..40])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: initializer_key. {:?}", e))?,
                initializer_deposit_token_account: *bytemuck::try_from_bytes::<Pubkey>(&buf[40..72]).map_err(|e| {
                    anyhow::format_err!("LifinityAmmDeserFailed: initializer_deposit_token_account. {:?}", e)
                })?,
                initializer_receive_token_account: *bytemuck::try_from_bytes::<Pubkey>(&buf[72..104]).map_err(|e| {
                    anyhow::format_err!("LifinityAmmDeserFailed: initializer_receive_token_account. {:?}", e)
                })?,
                initializer_amount: u64::from_le_bytes(
                    buf[104..112]
                        .try_into()
                        .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: initializer_amount. {:?}", e))?,
                ),
                taker_amount: u64::from_le_bytes(
                    buf[112..120]
                        .try_into()
                        .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: taker_amount. {:?}", e))?,
                ),
                is_initialized: buf[120] != 0,
                bump_seed: buf[121],
                freeze_trade: buf[122],
                freeze_deposit: buf[123],
                freeze_withdraw: buf[124],
                base_decimals: buf[125],
                token_program_id: *bytemuck::try_from_bytes::<Pubkey>(&buf[126..158])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: token_program_id. {:?}", e))?,
                token_a_account: *bytemuck::try_from_bytes::<Pubkey>(&buf[158..190])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: token_a_account. {:?}", e))?,
                token_b_account: *bytemuck::try_from_bytes::<Pubkey>(&buf[190..222])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: token_b_account. {:?}", e))?,
                pool_mint: *bytemuck::try_from_bytes::<Pubkey>(&buf[222..254])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: pool_mint. {:?}", e))?,
                token_a_mint: *bytemuck::try_from_bytes::<Pubkey>(&buf[254..286])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: token_a_mint. {:?}", e))?,
                token_b_mint: *bytemuck::try_from_bytes::<Pubkey>(&buf[286..318])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: token_b_mint. {:?}", e))?,
                fee_account: *bytemuck::try_from_bytes::<Pubkey>(&buf[318..350])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: fee_account. {:?}", e))?,
                oracle_main_account: *bytemuck::try_from_bytes::<Pubkey>(&buf[350..382])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: oracle_main_account. {:?}", e))?,
                oracle_sub_account: *bytemuck::try_from_bytes::<Pubkey>(&buf[382..414])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: oracle_sub_account. {:?}", e))?,
                oracle_pc_account: *bytemuck::try_from_bytes::<Pubkey>(&buf[414..446])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: oracle_pc_account. {:?}", e))?,
                fees: AmmFees {
                    trade_fee_numerator: u64::from_le_bytes(
                        buf[446..454]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: trade_fee_numerator. {:?}", e))?,
                    ),
                    trade_fee_denominator: u64::from_le_bytes(
                        buf[454..462].try_into().map_err(|e| {
                            anyhow::format_err!("LifinityAmmDeserFailed: trade_fee_denominator. {:?}", e)
                        })?,
                    ),
                    owner_trade_fee_numerator: u64::from_le_bytes(buf[462..470].try_into().map_err(|e| {
                        anyhow::format_err!("LifinityAmmDeserFailed: owner_trade_fee_numerator. {:?}", e)
                    })?),
                    owner_trade_fee_denominator: u64::from_le_bytes(buf[470..478].try_into().map_err(|e| {
                        anyhow::format_err!("LifinityAmmDeserFailed: owner_trade_fee_denominator. {:?}", e)
                    })?),
                    owner_withdraw_fee_numerator: u64::from_le_bytes(buf[478..486].try_into().map_err(|e| {
                        anyhow::format_err!("LifinityAmmDeserFailed: owner_withdraw_fee_numerator. {:?}", e)
                    })?),
                    owner_withdraw_fee_denominator: u64::from_le_bytes(buf[486..494].try_into().map_err(|e| {
                        anyhow::format_err!("LifinityAmmDeserFailed: owner_withdraw_fee_denominator. {:?}", e)
                    })?),
                    host_fee_numerator: u64::from_le_bytes(
                        buf[494..502]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: host_fee_numerator. {:?}", e))?,
                    ),
                    host_fee_denominator: u64::from_le_bytes(
                        buf[502..510].try_into().map_err(|e| {
                            anyhow::format_err!("LifinityAmmDeserFailed: host_fee_denominator. {:?}", e)
                        })?,
                    ),
                },
                curve: AmmCurve {
                    curve_type: buf[510],
                    curve_parameters: u64::from_le_bytes(
                        buf[511..519]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: curve_parameters. {:?}", e))?,
                    ),
                },
                config: AmmConfig {
                    last_price: u64::from_le_bytes(
                        buf[519..527]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: last_price. {:?}", e))?,
                    ),
                    last_balanced_price: u64::from_le_bytes(
                        buf[527..535]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: last_balanced_price. {:?}", e))?,
                    ),
                    config_denominator: u64::from_le_bytes(
                        buf[535..543]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: config_denominator. {:?}", e))?,
                    ),
                    volume_x: u64::from_le_bytes(
                        buf[543..551]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: volume_x. {:?}", e))?,
                    ),
                    volume_y: u64::from_le_bytes(
                        buf[551..559]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: volume_y. {:?}", e))?,
                    ),
                    volume_x_in_y: u64::from_le_bytes(
                        buf[559..567]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: volume_x_in_y. {:?}", e))?,
                    ),
                    deposit_cap: u64::from_le_bytes(
                        buf[567..575]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: deposit_cap. {:?}", e))?,
                    ),
                    regression_target: u64::from_le_bytes(
                        buf[575..583]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: regression_target. {:?}", e))?,
                    ),
                    oracle_type: u64::from_le_bytes(
                        buf[583..591]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: oracle_type. {:?}", e))?,
                    ),
                    oracle_status: u64::from_le_bytes(
                        buf[591..599]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: oracle_status. {:?}", e))?,
                    ),
                    oracle_main_slot_limit: u64::from_le_bytes(
                        buf[599..607].try_into().map_err(|e| {
                            anyhow::format_err!("LifinityAmmDeserFailed: oracle_main_slot_limit. {:?}", e)
                        })?,
                    ),
                    oracle_sub_confidence_limit: u64::from_le_bytes(buf[607..615].try_into().map_err(|e| {
                        anyhow::format_err!("LifinityAmmDeserFailed: oracle_sub_confidence_limit. {:?}", e)
                    })?),
                    oracle_sub_slot_limit: u64::from_le_bytes(
                        buf[615..623].try_into().map_err(|e| {
                            anyhow::format_err!("LifinityAmmDeserFailed: oracle_sub_slot_limit. {:?}", e)
                        })?,
                    ),
                    oracle_pc_confidence_limit: u64::from_le_bytes(buf[623..631].try_into().map_err(|e| {
                        anyhow::format_err!("LifinityAmmDeserFailed: oracle_pc_confidence_limit. {:?}", e)
                    })?),
                    oracle_pc_slot_limit: u64::from_le_bytes(
                        buf[631..639].try_into().map_err(|e| {
                            anyhow::format_err!("LifinityAmmDeserFailed: oracle_pc_slot_limit. {:?}", e)
                        })?,
                    ),
                    std_spread: u64::from_le_bytes(
                        buf[639..647]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: std_spread. {:?}", e))?,
                    ),
                    std_spread_buffer: u64::from_le_bytes(
                        buf[647..655]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: std_spread_buffer. {:?}", e))?,
                    ),
                    spread_coefficient: u64::from_le_bytes(
                        buf[655..663]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: spread_coefficient. {:?}", e))?,
                    ),
                    price_buffer_coin: i64::from_le_bytes(
                        buf[663..671]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: price_buffer_coin. {:?}", e))?,
                    ),
                    price_buffer_pc: i64::from_le_bytes(
                        buf[671..679]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: price_buffer_pc. {:?}", e))?,
                    ),
                    rebalance_ratio: u64::from_le_bytes(
                        buf[679..687]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: rebalance_ratio. {:?}", e))?,
                    ),
                    fee_trade: u64::from_le_bytes(
                        buf[687..695]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: fee_trade. {:?}", e))?,
                    ),
                    fee_platform: u64::from_le_bytes(
                        buf[695..703]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: fee_platform. {:?}", e))?,
                    ),
                    oracle_main_slot_buffer: u64::from_le_bytes(buf[703..711].try_into().map_err(|e| {
                        anyhow::format_err!("LifinityAmmDeserFailed: oracle_main_slot_buffer. {:?}", e)
                    })?),
                    config_temp4: u64::from_le_bytes(
                        buf[711..719]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: config_temp4. {:?}", e))?,
                    ),
                    config_temp5: u64::from_le_bytes(
                        buf[719..727]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: config_temp5. {:?}", e))?,
                    ),
                    config_temp6: u64::from_le_bytes(
                        buf[727..735]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: config_temp6. {:?}", e))?,
                    ),
                    config_temp7: u64::from_le_bytes(
                        buf[735..743]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: config_temp7. {:?}", e))?,
                    ),
                    config_temp8: u64::from_le_bytes(
                        buf[743..751]
                            .try_into()
                            .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: config_temp8. {:?}", e))?,
                    ),
                },
                amm_p_temp1: *bytemuck::try_from_bytes::<Pubkey>(&buf[751..783])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: amm_p_temp1. {:?}", e))?,
                amm_p_temp2: *bytemuck::try_from_bytes::<Pubkey>(&buf[783..815])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: amm_p_temp2. {:?}", e))?,
                amm_p_temp3: *bytemuck::try_from_bytes::<Pubkey>(&buf[815..847])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: amm_p_temp3. {:?}", e))?,
                amm_p_temp4: *bytemuck::try_from_bytes::<Pubkey>(&buf[847..879])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: amm_p_temp4. {:?}", e))?,
                amm_p_temp5: *bytemuck::try_from_bytes::<Pubkey>(&buf[879..911])
                    .map_err(|e| anyhow::format_err!("LifinityAmmDeserFailed: amm_p_temp5. {:?}", e))?,
            };
        Ok(amm)
    }
}

/*
#[derive(Clone, Debug, PartialEq)]
pub struct AmmAccount(pub Amm);
impl AmmAccount {
    // Borsh deserialization
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        // use std::io::Read;
        // let mut reader = buf;
        // let mut maybe_discm = [0u8; 8];
        // reader.read_exact(&mut maybe_discm)?;
        // if maybe_discm != AMM_ACCOUNT_DISCM {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!(
        //             "discm does not match. Expected: {:?}. Received: {:?}",
        //             AMM_ACCOUNT_DISCM, maybe_discm
        //         ),
        //     ));
        // }
        let mut reader = &buf[8..];
        Ok(Self(Amm::deserialize(&mut reader)?))
    }

    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&AMM_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
*/

impl Amm {
    pub fn check_oracle_slot(slot: u64, oracle: &Price, limit: u64) -> anyhow::Result<()> {
        if slot < oracle.agg.pub_slot {
            if oracle.agg.pub_slot - slot > limit {
                return Err(format_err!("NotValidSlot"));
            }
        } else if slot - oracle.agg.pub_slot > limit {
            return Err(format_err!("NotValidSlot"));
        }
        Ok(())
    }

    pub fn check_confidence(&self, oracle: &Price, confidence_limit: u64) -> anyhow::Result<()> {
        // if confidence is high than price/confidence is small
        if oracle.agg.conf != 0
            && ((oracle.agg.price as u64 / oracle.agg.conf) < (self.config.config_denominator / confidence_limit))
        {
            return Err(format_err!("ConfidenceOutOfLimit"));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_curve_amount(
        &self,
        amount: u64,
        slot: u64,
        a_to_b: bool,
        coin_a: u64,
        coin_b: u64,
        oracle: &Price,
        oracle_sub: &Price,
        oracle_pc: &Price,
    ) -> anyhow::Result<u64> {
        if self.freeze_trade == 1 {
            return Err(format_err!("FreezeTrade"));
        };
        let trade_fee = amount
            .checked_mul(self.fees.trade_fee_numerator)
            .context("Overflow")?
            .checked_div(self.fees.trade_fee_denominator)
            .context("Overflow")?;
        let owner_trade_fee = amount
            .checked_mul(self.fees.owner_trade_fee_numerator)
            .context("Overflow")?
            .checked_div(self.fees.owner_trade_fee_denominator)
            .context("Overflow")?;
        let fee = trade_fee.max(1) + owner_trade_fee.max(1);
        let amount_minus_fee = amount - fee;
        let mut price: f64;
        let mut confidence = 0;
        let base_decimals = 10_u64.pow(self.base_decimals as u32);

        // oracle valid slot
        Self::check_oracle_slot(slot, oracle, self.config.oracle_main_slot_limit)?;
        Self::check_oracle_slot(slot, oracle_sub, self.config.oracle_sub_slot_limit)?;
        Self::check_oracle_slot(slot, oracle_pc, self.config.oracle_pc_slot_limit)?;
        self.check_confidence(oracle_pc, self.config.oracle_pc_confidence_limit)?;
        self.check_confidence(oracle_sub, self.config.oracle_sub_confidence_limit)?;
        if self.config.oracle_type == 0 {
            match oracle.agg.status {
                PriceStatus::Trading => {
                    price = oracle.agg.price as f64;

                    confidence = oracle.agg.conf;
                }
                _ => {
                    return Err(format_err!("InvalidPythMainStatus"));
                }
            }
        } else {
            match oracle_sub.agg.status {
                PriceStatus::Trading => {
                    price = oracle_sub.agg.price as f64;
                }
                _ => {
                    return Err(format_err!("InvalidPythMainStatus"));
                }
            }
        };
        match oracle_pc.agg.status {
            PriceStatus::Trading => {
                let expo = oracle_pc.expo as f64;
                let exponent: f64 = f64::powf(10.0, expo.abs());
                let price_pc = oracle_pc.agg.price as f64;
                price = price * exponent / price_pc;
            }
            _ => {
                return Err(format_err!("InvalidPythPcStatus"));
            }
        }
        match self.curve.curve_type {
            0 => {
                return self.standard_curve(amount_minus_fee, price, base_decimals, confidence, coin_a, a_to_b);
            }
            1 => {
                return self.constant_curve(amount_minus_fee, coin_a, coin_b, a_to_b);
            }
            _ => {}
        }
        Err(format_err!("NotAllowed (Lifinity)"))
    }

    pub fn standard_curve(
        &self,
        amount: u64,
        price: f64,
        base_decimals: u64,
        confidence: u64,
        coin_balance: u64,
        a_to_b: bool,
    ) -> anyhow::Result<u64> {
        let mut adjusted_balance = 0.0;
        if a_to_b {
            let adjusted_price =
                price * ((self.config.price_buffer_coin as f64 / self.config.config_denominator as f64) + 1.0);
            let coin_balance = (coin_balance + amount) / (self.config.regression_target * 2);
            if coin_balance >= 1 {
                adjusted_balance = ((coin_balance as f64)
                    .powf(self.config.spread_coefficient as f64 / self.config.config_denominator as f64)
                    * (self.config.std_spread as f64 / self.config.config_denominator as f64))
                    + (self.config.std_spread_buffer as f64 / self.config.config_denominator as f64);
            }

            let new_price = (adjusted_price * (1.0 - adjusted_balance)) - confidence as f64;
            /*
                      destinationAmountSwapped = amount.times(newPrice.div(baseDecimals)).floor()
            */
            let destination_amount_swapped = (amount as f64 * new_price) / base_decimals as f64;
            Ok(destination_amount_swapped as u64)
        } else {
            let adjusted_price =
                price * ((self.config.price_buffer_pc as f64 / self.config.config_denominator as f64) + 1.0);

            let new_coin_balance = if (self.config.regression_target * 2) < coin_balance {
                0.0
            } else {
                (((self.config.regression_target * 2) as f64 - coin_balance as f64) * adjusted_price)
                    / base_decimals as f64
            };
            let temporal_balance = (new_coin_balance + amount as f64)
                / (self.config.regression_target as f64 * (adjusted_price / base_decimals as f64));
            if temporal_balance >= 1.0 {
                adjusted_balance = (temporal_balance
                    .powf(self.config.spread_coefficient as f64 / self.config.config_denominator as f64)
                    * (self.config.std_spread as f64 / self.config.config_denominator as f64))
                    + (self.config.std_spread_buffer as f64 / self.config.config_denominator as f64);
            }
            let new_price = (adjusted_price * (1.0 + adjusted_balance)) + confidence as f64;
            /*
                      destinationAmountSwapped = amount.times(newPrice.div(baseDecimals)).floor()
            */
            let destination_amount_swapped = amount as f64 / (new_price / base_decimals as f64);
            Ok(destination_amount_swapped as u64)
        }
    }

    pub fn constant_curve(&self, amount: u64, coin_balance: u64, pc_balance: u64, a_to_b: bool) -> anyhow::Result<u64> {
        let lp_balance = coin_balance as u128 * pc_balance as u128;
        if a_to_b {
            let new_coin_balance = coin_balance + amount;
            let (new_pc_balance, _) = lp_balance
                .checked_ceil_div(new_coin_balance as u128)
                .context("Overflow")?;
            Ok(pc_balance
                .checked_sub(new_pc_balance.to_u64().context("Overflow")?)
                .context("Overflow")?)
        } else {
            let new_pc_balance = pc_balance + amount;
            let (new_coin_balance, _) = lp_balance
                .checked_ceil_div(new_pc_balance as u128)
                .context("Overflow")?;
            Ok(coin_balance
                .checked_sub(new_coin_balance.to_u64().context("Overflow")?)
                .context("Overflow")?)
        }
    }
}
