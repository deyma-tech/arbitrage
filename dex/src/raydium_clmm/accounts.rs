use std::collections::BTreeMap;

use super::big_num::U1024;
use super::constants::TICK_ARRAY_BITMAP_SIZE;
use super::constants::TICK_ARRAY_SIZE;
use super::constants::TICK_ARRAY_SIZE_USIZE;
use super::instructions::{SwapIxArgs, SwapIxData, SwapV2IxArgs, SwapV2IxData, SwapV2Keys};
use super::quote::swap_internal;
use super::tick_math::MAX_SQRT_PRICE_X64;
use super::tick_math::MIN_SQRT_PRICE_X64;
use super::typedefs::{RayTickState, TickState};
use ahash::AHashSet;
use anyhow::Context;
use anyhow::{format_err, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use log::error;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use utils::pool::Pool;
use utils::quote::QuoteResult;

pub const AMM_CONFIG_ACCOUNT_DISCM: [u8; 8] = [218, 244, 33, 104, 203, 203, 43, 111];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Default)]
pub struct AmmConfig {
    pub bump: u8,
    pub index: u16,
    pub owner: Pubkey,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub tick_spacing: u16,
    pub fund_fee_rate: u32,
    pub padding_u32: u32,
    pub fund_owner: Pubkey,
    pub padding: [u64; 3], // 8 * 3 = 24
}

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Default)]
#[repr(C, packed)]
pub struct RayAmmConfig {
    pub bump: u8,
    pub index: u16,
    pub owner: Pubkey,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub tick_spacing: u16,
    pub fund_fee_rate: u32,
    // padding_u32: u32,       // 4byte
    // pub fund_owner: Pubkey, // 32byte
    // pub padding: [u64; 3],  // 8 * 3 = 24byte
    // 4 + 32 + 24 = 60
}

impl RayAmmConfig {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<AmmConfig> {
        let ray_amm_config = bytemuck::try_from_bytes::<RayAmmConfig>(&buf[8..(buf.len() - 60)]);
        let ray_amm_config = ray_amm_config.map_err(|e| format_err!("failed to deserialize RayAmmConfig. {:?}", e))?;
        let amm_config = AmmConfig {
            bump: ray_amm_config.bump,
            index: ray_amm_config.index,
            owner: ray_amm_config.owner,
            protocol_fee_rate: ray_amm_config.protocol_fee_rate,
            trade_fee_rate: ray_amm_config.trade_fee_rate,
            tick_spacing: ray_amm_config.tick_spacing,
            fund_fee_rate: ray_amm_config.fund_fee_rate,
            padding_u32: 0,
            fund_owner: Default::default(),
            padding: [0; 3],
        };
        Ok(amm_config)
    }
}

// pub const OPERATION_STATE_ACCOUNT_DISCM: [u8; 8] = [19, 236, 58, 237, 81, 222, 183, 252];
//
// #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
// pub struct OperationState {
//     pub bump: u8,
//     pub operation_owners: [Pubkey; 10],
//     pub whitelist_mints: [Pubkey; 100],
// }

// #[derive(Clone, Debug, PartialEq)]
// pub struct OperationStateAccount(pub OperationState);
// impl OperationStateAccount {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         use std::io::Read;
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != OPERATION_STATE_ACCOUNT_DISCM {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!(
//                     "discm does not match. Expected: {:?}. Received: {:?}",
//                     OPERATION_STATE_ACCOUNT_DISCM, maybe_discm
//                 ),
//             ));
//         }
//         Ok(Self(OperationState::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&OPERATION_STATE_ACCOUNT_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }

// pub const OBSERVATION_STATE_ACCOUNT_DISCM: [u8; 8] = [122, 174, 197, 53, 129, 9, 165, 132];
//
// #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
// pub struct ObservationState {
//     pub initialized: bool,
//     pub recent_epoch: u64,
//     pub observation_index: u16,
//     pub pool_id: Pubkey,
//     pub observations: [Observation; 100],
//     pub padding: [u64; 4],
// }

// #[derive(Clone, Debug, PartialEq)]
// pub struct ObservationStateAccount(pub ObservationState);
// impl ObservationStateAccount {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         use std::io::Read;
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != OBSERVATION_STATE_ACCOUNT_DISCM {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!(
//                     "discm does not match. Expected: {:?}. Received: {:?}",
//                     OBSERVATION_STATE_ACCOUNT_DISCM, maybe_discm
//                 ),
//             ));
//         }
//         Ok(Self(ObservationState::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&OBSERVATION_STATE_ACCOUNT_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }

// pub const PERSONAL_POSITION_STATE_ACCOUNT_DISCM: [u8; 8] = [70, 111, 150, 126, 230, 15, 25, 117];

// #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
// pub struct PersonalPositionState {
//     pub bump: [u8; 1],
//     pub nft_mint: Pubkey,
//     pub pool_id: Pubkey,
//     pub tick_lower_index: i32,
//     pub tick_upper_index: i32,
//     pub liquidity: u128,
//     pub fee_growth_inside0_last_x64: u128,
//     pub fee_growth_inside1_last_x64: u128,
//     pub token_fees_owed0: u64,
//     pub token_fees_owed1: u64,
//     pub reward_infos: [PositionRewardInfo; 3],
//     pub recent_epoch: u64,
//     pub padding: [u64; 7],
// }

// #[derive(Clone, Debug, PartialEq)]
// pub struct PersonalPositionStateAccount(pub PersonalPositionState);
// impl PersonalPositionStateAccount {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         use std::io::Read;
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != PERSONAL_POSITION_STATE_ACCOUNT_DISCM {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!(
//                     "discm does not match. Expected: {:?}. Received: {:?}",
//                     PERSONAL_POSITION_STATE_ACCOUNT_DISCM, maybe_discm
//                 ),
//             ));
//         }
//         Ok(Self(PersonalPositionState::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&PERSONAL_POSITION_STATE_ACCOUNT_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }

pub const POOL_STATE_ACCOUNT_DISCM: [u8; 8] = [247, 237, 227, 245, 215, 195, 222, 70];
#[derive(Clone, Debug, PartialEq)] // BorshDeserialize, BorshSerialize,
pub struct PoolState {
    pub bump: [u8; 1],                    // 8..9
    pub amm_config: Pubkey,               // 9..41
    pub owner: Pubkey,                    // 41..73
    pub token_mint0: Pubkey,              // 73..105
    pub token_mint1: Pubkey,              // 105..137
    pub token_vault0: Pubkey,             // 137..169
    pub token_vault1: Pubkey,             // 169..201
    pub observation_key: Pubkey,          // 201..233
    pub mint_decimals0: u8,               // 233
    pub mint_decimals1: u8,               // 234
    pub tick_spacing: u16,                // 235..237
    pub liquidity: u128,                  // 237..253
    pub sqrt_price_x64: u128,             // 253..269
    pub tick_current: i32,                // 269..273
    pub padding3: u16,                    // 273..275
    pub padding4: u16,                    // 275..277
    pub fee_growth_global0_x64: u128,     // 277..293
    pub fee_growth_global1_x64: u128,     // 293..309
    pub protocol_fees_token0: u64,        // 309..317
    pub protocol_fees_token1: u64,        // 317..325
    pub swap_in_amount_token0: u128,      // 325..341
    pub swap_out_amount_token1: u128,     // 341..357
    pub swap_in_amount_token1: u128,      // 357..373
    pub swap_out_amount_token0: u128,     // 373..389
    pub status: u8,                       // 389
    pub padding: [u8; 7],                 // 390..397
    pub reward_infos: [RayRewardInfo; 3], // 397..880
    pub tick_array_bitmap: [u64; 16],     // 880..1008
    pub total_fees_token0: u64,           // 1008..1016
    pub total_fees_claimed_token0: u64,   // 1016..1024
    pub total_fees_token1: u64,           // 1024..1032
    pub total_fees_claimed_token1: u64,   // 1032..1040
    pub fund_fees_token0: u64,            // 1040..1048
    pub fund_fees_token1: u64,            // 1048..1056
    pub open_time: u64,                   // 1056..1064
    pub recent_epoch: u64,                // 1064..1072
    pub padding1: [u64; 24],              // 1072..1184
    pub padding2: [u64; 32],              // 1184..1312
}

// impl Default for PoolState {
//     fn default() -> Self {
//         PoolState {
//             bump: [0],
//             amm_config: Pubkey::default(),
//             owner: Pubkey::default(),
//             token_mint0: Pubkey::default(),
//             token_mint1: Pubkey::default(),
//             token_vault0: Pubkey::default(),
//             token_vault1: Pubkey::default(),
//             observation_key: Pubkey::default(),
//             mint_decimals0: 0,
//             mint_decimals1: 0,
//             tick_spacing: 0,
//             liquidity: 0,
//             sqrt_price_x64: 0,
//             tick_current: 0,
//             padding3: 0,
//             padding4: 0,
//             fee_growth_global0_x64: 0,
//             fee_growth_global1_x64: 0,
//             protocol_fees_token0: 0,
//             protocol_fees_token1: 0,
//             swap_in_amount_token0: 0,
//             swap_out_amount_token1: 0,
//             swap_in_amount_token1: 0,
//             swap_out_amount_token0: 0,
//             status: 0,
//             padding: [0; 7],
//             reward_infos: [RayRewardInfo::default(); 3],
//             tick_array_bitmap: [0; 16],
//             total_fees_token0: 0,
//             total_fees_claimed_token0: 0,
//             total_fees_token1: 0,
//             total_fees_claimed_token1: 0,
//             fund_fees_token0: 0,
//             fund_fees_token1: 0,
//             open_time: 0,
//             recent_epoch: 0,
//             padding1: [0; 24],
//             padding2: [0; 32],
//         }
//     }
// }

impl PoolState {
    // TODO: Need to test
    // pub fn remove_tick_arrays(&self, tick_arrays: &mut BTreeMap<i32, (Pubkey, TickArrayState)>) {
    //     let tick_array_current_index =
    //         TickArrayState::get_array_start_index(self.tick_current, self.tick_spacing);
    //     let mut tick_array_down =
    //         // down
    //             tick_arrays
    //             .iter()
    //             .rev()
    //             .filter(|(_id, (_, tick_array))| {
    //                 tick_array.start_tick_index <= tick_array_current_index
    //                     && tick_array.ticks.iter().any(|tick| tick.liquidity_gross > 0)
    //             })
    //             .take(5)
    //                 .map(|(tick, _)| tick.clone() )
    //             .collect::<Vec<_>>();
    //     let tick_array_up = tick_arrays
    //         .iter()
    //         .filter(|(_id, (_, tick_array))| {
    //             tick_array.start_tick_index >= tick_array_current_index
    //                 && tick_array.ticks.iter().any(|tick| tick.liquidity_gross > 0)
    //         })
    //         .take(5)
    //         .map(|(tick, _)| tick.clone())
    //         .collect::<Vec<_>>();
    //     tick_array_down.extend(tick_array_up);
    //     let keys: Vec<i32> = tick_arrays.keys().cloned().collect();
    //     for key in keys.iter() {
    //         if !tick_array_down.contains(key) {
    //             tick_arrays.remove(key);
    //         }
    //     }
    // }

    #[allow(clippy::too_many_arguments)]
    pub fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        _optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> Vec<AccountMeta> {
        if allowed_token2022.contains(&self.token_mint1) || allowed_token2022.contains(&self.token_mint0) {
            let keys = SwapV2Keys {
                payer: signer,
                amm_config: self.amm_config,
                pool_state: pool_pubkey,
                input_token_account: source_token_account,
                output_token_account: destination_token_account,
                input_vault: if a_to_b { self.token_vault0 } else { self.token_vault1 },
                output_vault: if a_to_b { self.token_vault1 } else { self.token_vault0 },
                observation_state: self.observation_key,
                token_program: spl_token::ID,
                token_program2022: spl_token_2022::ID,
                memo_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                input_vault_mint: if a_to_b { self.token_mint0 } else { self.token_mint1 },
                output_vault_mint: if a_to_b { self.token_mint1 } else { self.token_mint0 },
            };
            let mut accounts = vec![
                AccountMeta {
                    pubkey: keys.payer,
                    is_signer: true,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.amm_config,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.pool_state,
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
                    pubkey: keys.input_vault,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.output_vault,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.observation_state,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.token_program,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.token_program2022,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.memo_program,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.input_vault_mint,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.output_vault_mint,
                    is_signer: false,
                    is_writable: false,
                },
            ];
            if let Some(optional_accounts) = optional_accounts {
                let meta = optional_accounts
                    .into_iter()
                    .map(|account| AccountMeta::new(account, false))
                    .collect::<Vec<AccountMeta>>();
                accounts.extend(meta);
            }
            accounts
        } else {
            let mut accounts = vec![
                AccountMeta {
                    pubkey: signer,
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: self.amm_config,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: pool_pubkey,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: source_token_account,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: destination_token_account,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: if a_to_b { self.token_vault0 } else { self.token_vault1 },
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: if a_to_b { self.token_vault1 } else { self.token_vault0 },
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: self.observation_key,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: spl_token::ID,
                    is_signer: false,
                    is_writable: false,
                },
            ];
            if let Some(optional_accounts) = optional_accounts {
                let meta = optional_accounts
                    .into_iter()
                    .map(|account| AccountMeta::new(account, false))
                    .collect::<Vec<AccountMeta>>();
                //println!("RaydiumCLMM:Meta: {:?}", meta);
                accounts.extend(meta);
            }
            // for account in accounts.iter() {
            //     println!(
            //         "RaydiumCLMM: {:?}, w:{}, s:
            //         {}",
            //         account.pubkey, account.is_writable, account.is_signer
            //     );
            // }
            accounts
        }
    }
}

// Just for testing

impl U1024 {
    fn bitxor(self, other: U1024) -> U1024 {
        const N_WORDS: usize = 16;
        let U1024(ref arr1) = self;
        let U1024(ref arr2) = other;
        let mut ret = [0u64; N_WORDS];
        for i in 0..N_WORDS {
            ret[i] = arr1[i] ^ arr2[i];
        }
        U1024(ret)
    }
}

impl PoolState {
    pub fn get_tick_array_offset(&self, tick_array_start_index: i32) -> Result<usize> {
        let tick_array_offset_in_bitmap =
            tick_array_start_index / TickArrayState::tick_count(self.tick_spacing) + TICK_ARRAY_BITMAP_SIZE;
        Ok(tick_array_offset_in_bitmap as usize)
    }

    fn flip_tick_array_bit_internal(&mut self, tick_array_start_index: i32) -> Result<()> {
        let tick_array_offset_in_bitmap = self.get_tick_array_offset(tick_array_start_index)?;

        let tick_array_bitmap = U1024(self.tick_array_bitmap);
        let mask = U1024::one() << tick_array_offset_in_bitmap; //.try_into().un_wrap();
        self.tick_array_bitmap = tick_array_bitmap.bitxor(mask).0;
        Ok(())
    }

    pub fn flip_tick_array_bit<'c: 'info, 'info>(
        &mut self,
        //tickarray_bitmap_extension: Option<&'c AccountInfo<'info>>,
        tick_array_start_index: i32,
    ) -> Result<()> {
        if !self.is_overflow_default_tickarray_bitmap(vec![tick_array_start_index]) {
            let _ = self.flip_tick_array_bit_internal(tick_array_start_index);
        }
        Ok(())
    }
}

// end testing ^

impl PoolState {
    // Calculates the next global reward growth variables based on the given timestamp.
    // The provided timestamp must be greater than or equal to the last updated timestamp.
    pub fn check_reward_infos(&self, curr_timestamp: u64) -> Result<()> {
        let mut next_reward_infos = self.reward_infos;

        for reward_info in &mut next_reward_infos {
            // !initialized
            if !reward_info.token_mint.ne(&Pubkey::default()) {
                continue;
            }
            if curr_timestamp <= reward_info.open_time {
                continue;
            }
            let latest_update_timestamp = curr_timestamp.min(reward_info.end_time);

            if self.liquidity != 0 {
                //require_gte!(latest_update_timestamp, reward_info.last_update_time);
                if latest_update_timestamp < reward_info.last_update_time {
                    return Err(format_err!("RequireGteViolated"));
                }
            }
        }
        Ok(())
    }
}

pub struct QuoteInput<'a> {
    pub amm_config: &'a AmmConfig,
    pub tick_arrays: &'a BTreeMap<i32, (Pubkey, TickArrayState)>,
    pub bitmap_extension: Option<&'a TickArrayBitmapExtension>,
}

impl<'a> Pool<QuoteInput<'a>> for PoolState {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.token_vault0, &self.token_vault1, &self.observation_key]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.token_vault0, self.token_vault1, self.observation_key]
    }

    fn get_a_mint(&self) -> &Pubkey {
        &self.token_mint0
    }

    fn get_b_mint(&self) -> &Pubkey {
        &self.token_mint1
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount: u64, input: &QuoteInput<'a>) -> anyhow::Result<QuoteResult> {
        let tick_array_current_index = TickArrayState::get_array_start_index(self.tick_current, self.tick_spacing);

        let tick_array = if a_to_b {
            // down
            input
                .tick_arrays
                .iter()
                .rev()
                .filter(|(_id, (_, tick_array))| {
                    tick_array.start_tick_index <= tick_array_current_index
                        && tick_array.ticks.iter().any(|tick| tick.liquidity_gross > 0)
                })
                .take(6)
                .map(|(_, (_, tick_array))| tick_array)
                .collect::<Vec<_>>()
        } else {
            input
                .tick_arrays
                .iter()
                .filter(|(_id, (_, tick_array))| {
                    tick_array.start_tick_index >= tick_array_current_index
                        && tick_array.ticks.iter().any(|tick| tick.liquidity_gross > 0)
                })
                .take(6)
                .map(|(_, (_, tick_array))| tick_array)
                .collect::<Vec<_>>()
        };

        //tick_array.dedup();

        swap_internal(
            input.amm_config,
            self,
            &tick_array,
            //input.bitmap_extension,
            amount,
            a_to_b,
            exact_in,
        )
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        a_to_b: bool,
        exact_in: bool,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        if allowed_token2022.contains(&self.token_mint1) || allowed_token2022.contains(&self.token_mint0) {
            SwapV2IxData(SwapV2IxArgs {
                amount,
                other_amount_threshold: amount_threshold,
                sqrt_price_limit_x64: if a_to_b {
                    MIN_SQRT_PRICE_X64 + 1
                } else {
                    MAX_SQRT_PRICE_X64 - 1
                },
                is_base_input: exact_in,
            })
            .try_to_vec()
            .context("Raydium CLMM: SwapIxData")
        } else {
            SwapIxData(SwapIxArgs {
                amount,
                other_amount_threshold: amount_threshold,
                sqrt_price_limit_x64: if a_to_b {
                    MIN_SQRT_PRICE_X64 + 1
                } else {
                    MAX_SQRT_PRICE_X64 - 1
                },
                is_base_input: exact_in,
            })
            .try_to_vec()
            .context("Raydium CLMM: SwapIxData")
        }
    }

    //     // TODO support token 2022
    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        Ok(self.get_swap_keys_ix(
            pool_pubkey,
            signer,
            source_token_account,
            destination_token_account,
            a_to_b,
            optional_account,
            optional_accounts,
            allowed_token2022,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C, packed)]
pub struct RayRewardInfo {
    pub reward_state: u8, // 1
    pub open_time: u64,   // 8
    pub end_time: u64,    // 8

    pub last_update_time: u64, // 8
    // pub emissions_per_second_x64: u128, // 16
    // pub reward_total_emissioned: u64,   // 8
    // pub reward_claimed: u64,            // 8
    pub padding1: [u8; 32],

    pub token_mint: Pubkey, // 32

    // pub token_vault: Pubkey,            // 32
    // pub authority: Pubkey,              // 32
    // pub reward_growth_global_x64: u128, // 16
    pub padding2: [u8; 64],
    pub padding3: [u8; 8],
}

impl Default for RayRewardInfo {
    fn default() -> Self {
        RayRewardInfo {
            reward_state: 0,
            open_time: 0,
            end_time: 0,
            last_update_time: 0,
            padding1: [0; 32],
            token_mint: Pubkey::default(),
            padding2: [0; 64],
            padding3: [0; 8],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C, packed)]
pub struct RayPoolState {
    pub bump: [u8; 1],
    //pub bump: u8,
    pub amm_config: Pubkey,
    pub owner: Pubkey,
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub observation_key: Pubkey,
    pub mint_decimals0: u8,
    pub mint_decimals1: u8,
    pub tick_spacing: u16,
    pub liquidity: u128,
    pub sqrt_price_x64: u128,
    //pub tick_current: i32, // i32
    pub tick_current: [u8; 4],
    //pub padding3: u16,
    //pub padding4: u16,
    pub padding0: [u8; 4],
    pub fee_growth_global0_x64: u128,
    pub fee_growth_global1_x64: u128,
    // pub protocol_fees_token0: u64,    // 8
    // pub protocol_fees_token1: u64,    // 8
    // pub swap_in_amount_token0: u128,  // 16
    // pub swap_out_amount_token1: u128, // 16
    // pub swap_in_amount_token1: u128,  // 16
    // pub swap_out_amount_token0: u128, // 16
    // 8 + 8 + 16 + 16 + 16 + 16 = 80; decompose 80: 64 + 16
    pub padding1: [u8; 64],
    pub padding2: [u8; 16],
    //pub status: u8,
    pub status: [u8; 1],
    pub padding3: [u8; 7], // 7 * 8 = 56
    //pub reward_infos: [RewardInfo; 3], // 161 * 3 = 483
    pub reward_infos: [RayRewardInfo; 3],
    // 56 + 483 = 539; decompose 539: 512 + 16 + 8 + 2 + 1
    // pub padding3: [u8; 512],
    pub padding4: [u8; 32],
    pub padding5: [u8; 16],
    pub padding6: [u8; 1],
    pub tick_array_bitmap: [u64; 16],
    // pub total_fees_token0: u64,         // 8
    // pub total_fees_claimed_token0: u64, // 8
    // pub total_fees_token1: u64,         // 8
    // pub total_fees_claimed_token1: u64, // 8
    // pub fund_fees_token0: u64,          // 8
    // pub fund_fees_token1: u64,          // 8
    // pub open_time: u64,                 // 8
    // pub recent_epoch: u64,              // 8
    // pub padding1_: [u64; 24],           // 24 * 8 = 192
    // pub padding2_: [u64; 32],           // 32 * 8 = 256
    // (8 * 8) + 192 + 256 = 512
    // pub padding3: [u8; 512],
}

impl RayPoolState {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<PoolState> {
        //println!("RayPool buf len: {:?}", buf.len());
        //println!("RewardInfo size: {:?}", std::mem::size_of::<RewardInfo>());

        let offset = 24;
        let pool_state = PoolState {
            bump: buf[8..9]
                .try_into()
                .map_err(|e| anyhow::format_err!("Raydium CLMM deser failed: bump. {:?}", e))?,
            amm_config: *bytemuck::try_from_bytes::<Pubkey>(&buf[9..41])
                .map_err(|e| anyhow::format_err!("Raydium CLMM deser failed: aam_config. {:?}", e))?,
            owner: *bytemuck::try_from_bytes::<Pubkey>(&buf[41..73])
                .map_err(|e| anyhow::format_err!("Raydium CLMM deser failed: owner. {:?}", e))?,
            token_mint0: *bytemuck::try_from_bytes::<Pubkey>(&buf[73..105])
                .map_err(|e| anyhow::format_err!("Raydium CLMM deser failed: token_min0. {:?}", e))?,
            token_mint1: *bytemuck::try_from_bytes::<Pubkey>(&buf[105..137])
                .map_err(|e| anyhow::format_err!("Raydium CLMM deser failed: token_mint1. {:?}", e))?,
            token_vault0: *bytemuck::try_from_bytes::<Pubkey>(&buf[137..169])
                .map_err(|e| anyhow::format_err!("Raydium CLMM deser failed: token_vault0. {:?}", e))?,
            token_vault1: *bytemuck::try_from_bytes::<Pubkey>(&buf[169..201])
                .map_err(|e| anyhow::format_err!("Raydium CLMM deser failed: token_vault1. {:?}", e))?,
            observation_key: *bytemuck::try_from_bytes::<Pubkey>(&buf[201..233])
                .map_err(|e| anyhow::format_err!("Raydium CLMM deser failed: observation_key. {:?}", e))?,
            mint_decimals0: buf[233],
            mint_decimals1: buf[234],
            tick_spacing: u16::from_le_bytes(
                buf[235..237]
                    .try_into()
                    .context("Raydium CLMM deser failed: tick_spacing")?,
            ),
            liquidity: u128::from_le_bytes(
                buf[237..253]
                    .try_into()
                    .context("Raydium CLMM deser failed: liquidity")?,
            ),
            sqrt_price_x64: u128::from_le_bytes(
                buf[253..269]
                    .try_into()
                    .context("Raydium CLMM deser failed: sqrt_price_x64")?,
            ),
            tick_current: i32::from_le_bytes(
                buf[269..273]
                    .try_into()
                    .context("Raydium CLMM deser failed: tick_current")?,
            ),
            padding3: 0, // u16::from_le_bytes(buf[273..275].try_into().context(),
            padding4: 0, // u16::from_le_bytes(buf[275..277].try_into().context(),
            fee_growth_global0_x64: u128::from_le_bytes(
                buf[277..293]
                    .try_into()
                    .context("Raydium CLMM deser failed: fee_growth_global0_x64")?,
            ),
            fee_growth_global1_x64: u128::from_le_bytes(
                buf[293..309]
                    .try_into()
                    .context("Raydium CLMM deser failed: fee_growth_global1_x64")?,
            ),
            protocol_fees_token0: 0,   //u64::from_le_bytes(buf[309..317].try_into().context(),
            protocol_fees_token1: 0,   //u64::from_le_bytes(buf[317..325].try_into().context(),
            swap_in_amount_token0: 0,  //u128::from_le_bytes(buf[325..341].try_into().context(),
            swap_out_amount_token1: 0, //u128::from_le_bytes(buf[341..357].try_into().context(),
            swap_in_amount_token1: 0,  //u128::from_le_bytes(buf[357..373].try_into().context(),
            swap_out_amount_token0: 0, //u128::from_le_bytes(buf[373..389].try_into().context(),
            status: buf[389],
            padding: [0; 7], //buf[390..397].try_into().context(,
            //reward_infos: [Default::default(); 3], // 397..880
            reward_infos: *bytemuck::try_from_bytes::<[RayRewardInfo; 3]>(&buf[397..880])
                .map_err(|e| format_err!("Raydium CLMM deser failed:  reward_infos. {:?}", e))?,
            tick_array_bitmap: *bytemuck::try_from_bytes::<[u64; 16]>(&buf[880 + offset..1008 + offset])
                .map_err(|e| format_err!("Raydium CLMM deser failed:  tick_array_bitmap. {:?}", e))?,
            total_fees_token0: 0,         //u64::from_le_bytes(buf[1008..1016].try_into().context(),
            total_fees_claimed_token0: 0, //u64::from_le_bytes(buf[1016..1024].try_into().context(),
            total_fees_token1: 0,         //u64::from_le_bytes(buf[1024..1032].try_into().context(),
            total_fees_claimed_token1: 0, //u64::from_le_bytes(buf[1032..1040].try_into().context(),
            fund_fees_token0: 0,          //u64::from_le_bytes(buf[1040..1048].try_into().context(),
            fund_fees_token1: 0,          // u64::from_le_bytes(buf[1048..1056].try_into().context(),
            open_time: 0,                 //u64::from_le_bytes(buf[1056..1064].try_into().context(),
            // just for testing
            recent_epoch: u64::from_le_bytes(
                buf[1064 + offset..1072 + offset]
                    .try_into()
                    .context("Raydium CLMM deser failed: recent_epoch")?,
            ),
            padding1: [0; 24], // buf[1072..1184].try_into().context(,
            padding2: [0; 32], //buf[1184..1312].try_into().context(,
        };
        //println!("N: {:?}", pool_state);
        //println!();
        Ok(pool_state)
    }
}

impl RayPoolState {
    // Calculates the next global reward growth variables based on the given timestamp.
    // The provided timestamp must be greater than or equal to the last updated timestamp.
    pub fn update_reward_infos(&mut self, curr_timestamp: u64) -> anyhow::Result<()> {
        let mut next_reward_infos = self.reward_infos;

        for reward_info in &mut next_reward_infos {
            if reward_info.token_mint == Pubkey::default() {
                continue;
            }
            if curr_timestamp <= reward_info.open_time {
                continue;
            }
            let latest_update_timestamp = curr_timestamp.min(reward_info.end_time);

            if self.liquidity != 0 {
                //require_gte!(latest_update_timestamp, reward_info.last_update_time);
                if latest_update_timestamp < reward_info.last_update_time {
                    return Err(anyhow::anyhow!(
                        "latest_update_timestamp < reward_info.last_update_time"
                    ));
                }
            }
        }
        Ok(())
    }
}

// #[derive(Clone, Debug, PartialEq)]
// pub struct PoolStateAccount(pub PoolState);
// impl PoolStateAccount {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         //println!("PoolStateAccount buf len: {:?}", buf.len());
//         //println!("PoolState size: {:?}", std::mem::size_of::<PoolState>());
//         use std::io::Read;
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != POOL_STATE_ACCOUNT_DISCM {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!(
//                     "discm does not match. Expected: {:?}. Received: {:?}",
//                     POOL_STATE_ACCOUNT_DISCM, maybe_discm
//                 ),
//             ));
//         }
//         let pool_state = PoolState::deserialize(&mut reader)?;
//         //println!("S: {:?}", pool_state);
//         //println!();
//         Ok(Self(pool_state))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&POOL_STATE_ACCOUNT_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }

pub const PROTOCOL_POSITION_STATE_ACCOUNT_DISCM: [u8; 8] = [100, 226, 145, 99, 146, 218, 160, 106];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct ProtocolPositionState {
    pub bump: u8,
    pub pool_id: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub liquidity: u128,
    pub fee_growth_inside0_last_x64: u128,
    pub fee_growth_inside1_last_x64: u128,
    pub token_fees_owed0: u64,
    pub token_fees_owed1: u64,
    pub reward_growth_inside: [u128; 3],
    pub recent_epoch: u64,
    pub padding: [u64; 7],
}
#[derive(Clone, Debug, PartialEq)]
pub struct ProtocolPositionStateAccount(pub ProtocolPositionState);
impl ProtocolPositionStateAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != PROTOCOL_POSITION_STATE_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {PROTOCOL_POSITION_STATE_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(ProtocolPositionState::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&PROTOCOL_POSITION_STATE_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const TICK_ARRAY_STATE_ACCOUNT_DISCM: [u8; 8] = [192, 155, 85, 205, 49, 249, 129, 42];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct TickArrayState {
    pub pool_id: Pubkey,            // 32
    pub start_tick_index: i32,      // 4
    pub ticks: [TickState; 60],     // 168 * 60 = 10080
    pub initialized_tick_count: u8, // 1
    pub recent_epoch: u64,          // 8
    pub padding: [u8; 107],         // 107
                                    // 32 + 4 + 10080 + 1 + 8 + 107 = 10232
}

impl Default for TickArrayState {
    fn default() -> Self {
        Self {
            pool_id: Pubkey::default(),
            start_tick_index: 0,
            ticks: [TickState::default(); 60],
            initialized_tick_count: 0,
            recent_epoch: 0,
            padding: [0; 107],
        }
    }
}

impl TickArrayState {
    /**
     * Initialize only can be called when first created
     */
    pub fn initialize(&mut self, start_index: i32, _tick_spacing: u16, pool_key: Pubkey) -> Result<()> {
        //TickArrayState::check_is_valid_start_index(start_index, tick_spacing);
        self.start_tick_index = start_index;
        self.pool_id = pool_key;
        self.recent_epoch = 0; //get_recent_epoch()?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TickArrayStateAccount(pub TickArrayState);
impl TickArrayStateAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        let bm_pool_id = *bytemuck::try_from_bytes::<Pubkey>(&buf[8..40])
            .map_err(|e| format_err!("Raydium CLMM deser failed: pool_id. {:?}", e))?;
        let bm_start_tick_index = *bytemuck::try_from_bytes::<i32>(&buf[40..44])
            .map_err(|e| format_err!("Raydium CLMM deser failed: start_tick_index. {:?}", e))?;
        let tick_array_tuple = buf[44..44 + 60 * 168].chunks(168).filter_map(|chunk| {
            let tick = bytemuck::try_pod_read_unaligned::<RayTickState>(&chunk[4..68]);
            if let Ok(tick) = tick {
                let index_bytes = TryInto::<[u8; 4]>::try_into(chunk[..4].to_vec());
                if let Ok(index_bytes) = index_bytes {
                    return Some((i32::from_le_bytes(index_bytes), tick));
                } else {
                    error!(
                        "Raydium CLMM. Error deserializing TickArrayStateAccount. Converting index bytes: {:?}",
                        index_bytes
                    );
                }
            } else {
                error!(
                    "Raydium CLMM. Error deserializing TickArrayStateAccount. Converting tick: {:?}",
                    tick
                );
            }
            None
        });
        let bm_initialized_tick_count = *bytemuck::try_from_bytes::<u8>(&buf[44 + 60 * 115..45 + 60 * 115])
            .map_err(|e| format_err!("Raydium CLMM deser failed: initialized_tick_count. {:?}", e))?;
        let tick_array = TickArrayState {
            pool_id: bm_pool_id,
            start_tick_index: bm_start_tick_index,
            ticks: tick_array_tuple
                .map(|(tick_index, tick)| TickState {
                    tick: tick_index,
                    liquidity_net: tick.liquidity_net,
                    liquidity_gross: tick.liquidity_gross,
                    fee_growth_outside0_x64: tick.fee_growth_outside0_x64,
                    fee_growth_outside1_x64: tick.fee_growth_outside1_x64,
                    reward_growths_outside_x64: [0, 0, 0],
                    padding: [0; 13],
                })
                .collect::<Vec<TickState>>()
                .try_into()
                .map_err(|_| format_err!("Raydium CLMM deser failed: Ticks. {:?}", bm_pool_id))?,
            initialized_tick_count: bm_initialized_tick_count,
            recent_epoch: 0,
            padding: [0; 107],
        };
        Ok(Self(tick_array))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&TICK_ARRAY_STATE_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const TICK_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM: [u8; 8] = [60, 150, 36, 219, 97, 128, 139, 153];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct TickArrayBitmapExtension {
    pub pool_id: Pubkey,
    pub positive_tick_array_bitmap: [[u64; 8]; 14],
    pub negative_tick_array_bitmap: [[u64; 8]; 14],
}
#[derive(Clone, Debug, PartialEq)]
pub struct TickArrayBitmapExtensionAccount(pub TickArrayBitmapExtension);
impl TickArrayBitmapExtensionAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != TICK_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM {
            return Err(std::io::Error::other(
                format!(
                    "discm does not match. Expected: {TICK_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
                ),
            ));
        }
        Ok(Self(TickArrayBitmapExtension::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&TICK_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

impl TickState {
    #[inline(always)]
    pub fn is_initialized(self) -> bool {
        self.liquidity_gross != 0
    }
}

impl TickArrayState {
    #[inline(always)]
    pub fn tick_count(tick_spacing: u16) -> i32 {
        TICK_ARRAY_SIZE * i32::from(tick_spacing)
    }

    #[inline(always)]
    pub fn get_array_start_index(tick_index: i32, tick_spacing: u16) -> i32 {
        let ticks_in_array = TickArrayState::tick_count(tick_spacing);
        let mut start = tick_index / ticks_in_array;
        if tick_index < 0 && tick_index % ticks_in_array != 0 {
            start -= 1
        }
        start * ticks_in_array
    }

    pub fn get_tick_state_mut(&mut self, tick_index: i32, tick_spacing: u16) -> Result<&mut TickState> {
        let offset_in_array = self.get_tick_offset_in_array(tick_index, tick_spacing)?;
        Ok(&mut self.ticks[offset_in_array])
    }

    /// Get tick's offset in current tick array, tick must be include in tick array， otherwise throw an error
    pub fn get_tick_offset_in_array(&mut self, tick_index: i32, tick_spacing: u16) -> Result<usize> {
        let offset_in_array = ((tick_index - self.start_tick_index) / i32::from(tick_spacing)) as usize;
        Ok(offset_in_array)
    }

    pub fn first_initialized_tick(&self, zero_for_one: bool) -> anyhow::Result<&TickState> {
        if zero_for_one {
            let mut i = TICK_ARRAY_SIZE - 1;
            while i >= 0 {
                if self.ticks[i as usize].is_initialized() {
                    //return Ok(self.ticks.get(i as usize).un_wrap());
                    if let Some(tick) = self.ticks.get(i as usize) {
                        return Ok(tick);
                    }
                }
                i -= 1;
            }
        } else {
            let mut i = 0;
            while i < TICK_ARRAY_SIZE_USIZE {
                if self.ticks[i].is_initialized() {
                    //return Ok(self.ticks.get(i).un_wrap());
                    if let Some(tick) = self.ticks.get(i) {
                        return Ok(tick);
                    }
                }
                i += 1;
            }
        }
        Err(format_err!("InvalidTickArray"))
    }

    /// Get next initialized tick in tick array, `current_tick_index` can be any tick index, in other words, `current_tick_index` not exactly a point in the tickarray,
    /// and current_tick_index % tick_spacing maybe not equal zero.
    /// If price move to left tick <= current_tick_index, or to right tick > current_tick_index
    pub fn next_initialized_tick(
        &self,
        current_tick_index: i32,
        tick_spacing: u16,
        zero_for_one: bool,
    ) -> Result<Option<&TickState>> {
        let current_tick_array_start_index = TickArrayState::get_array_start_index(current_tick_index, tick_spacing);
        if current_tick_array_start_index != self.start_tick_index {
            return Ok(None);
        }
        let mut offset_in_array = (current_tick_index - self.start_tick_index) / i32::from(tick_spacing);

        if zero_for_one {
            while offset_in_array >= 0 {
                if self.ticks[offset_in_array as usize].is_initialized() {
                    return Ok(self.ticks.get(offset_in_array as usize));
                }
                offset_in_array -= 1;
            }
        } else {
            offset_in_array += 1;
            while offset_in_array < TICK_ARRAY_SIZE {
                if self.ticks[offset_in_array as usize].is_initialized() {
                    return Ok(self.ticks.get(offset_in_array as usize));
                }
                offset_in_array += 1;
            }
        }
        Ok(None)
    }
}
