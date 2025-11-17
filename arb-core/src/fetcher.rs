use crate::calculator::{CalculatorEnum, MeteoraDlmmData, OrcaData, RaydiumClmmV3Data};
use anyhow::Context;
use dex::orca::{OrcaTickArray, OrcaWhirlpool};
use dex::raydium_clmm::RayAmmConfig;
//use dex::raydium_clmm::AmmConfigAccount;
use log::debug;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_program::pubkey::Pubkey;
use solana_sdk::program_pack::Pack;
use std::collections::BTreeMap;
use utils::pool::PoolType;
use utils::rpc::{get_account_data, get_token_account_amount};

pub async fn fetch_calculator(
    pubkey: &Pubkey,
    pool_type: PoolType,
    rpc_client: &RpcClient,
) -> anyhow::Result<Option<CalculatorEnum>> {
    let data = rpc_client.get_account_data(pubkey).await?;
    match pool_type {
        PoolType::RaydiumClmmV3 => {
            let pool_state = dex::raydium_clmm::RayPoolState::deserialize(data.as_slice())?;
            let amm_config_pubkey = pool_state.amm_config;
            let data = rpc_client.get_account_data(&amm_config_pubkey).await?;
            let amm_config = RayAmmConfig::deserialize(data.as_slice())?;
            let memcmp_discr = RpcFilterType::Memcmp(Memcmp::new(
                0,
                MemcmpEncodedBytes::Bytes(dex::raydium_clmm::TICK_ARRAY_STATE_ACCOUNT_DISCM.to_vec()),
            ));
            let memcmp_pk = RpcFilterType::Memcmp(Memcmp::new(
                8, // discriminator
                MemcmpEncodedBytes::Bytes(pubkey.to_bytes().to_vec()),
            ));
            let config = RpcProgramAccountsConfig {
                filters: Some(vec![
                    RpcFilterType::DataSize(10232 + 8), //tick_array_state_size as u64),
                    memcmp_discr,
                    memcmp_pk,
                ]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: None,
                    min_context_slot: None,
                },
                with_context: Some(false),
                sort_results: Some(true),
            };

            let tick_arrays_raw = rpc_client
                .get_program_accounts_with_config(&dex::raydium_clmm::ID, config)
                .await?;
            let mut tick_arrays_map = BTreeMap::new();
            for (key, account) in tick_arrays_raw.into_iter() {
                let tick_array = dex::raydium_clmm::TickArrayStateAccount::deserialize(account.data.as_slice())?.0;
                tick_arrays_map.insert(tick_array.start_tick_index, (key, tick_array));
            }

            let extension_pubkey = Pubkey::find_program_address(
                &[
                    "pool_tick_array_bitmap_extension".as_bytes(),
                    pubkey.to_bytes().as_ref(),
                ],
                &dex::raydium_clmm::ID,
            )
            .0;

            debug!("bitmap_extension_pubkey: {:?}", extension_pubkey);

            let (bitmap_extension, bitmap_extension_data) = match rpc_client.get_account_data(&extension_pubkey).await {
                Ok(data) => {
                    if !data.is_empty() && !data.iter().all(|byte| *byte == 0) {
                        let data = dex::raydium_clmm::TickArrayBitmapExtensionAccount::deserialize(data.as_slice())?.0;
                        (Some(extension_pubkey), Some(data))
                    } else {
                        debug!("bitmap_extension is empty");
                        (None, None)
                    }
                }
                Err(e) => {
                    debug!("Error fetching bitmap extension: {:?}", e);
                    (None, None)
                }
            };

            Ok(Some(CalculatorEnum::RaydiumClmmV3(RaydiumClmmV3Data {
                amm_config,
                pool_state,
                pool_state_pubkey: *pubkey,
                tick_arrays_map,
                bitmap_extension,
                bitmap_extension_data,
            })))
        }

        PoolType::RaydiumAmmV4 => {
            let data = get_account_data(rpc_client, pubkey).await?;
            let amm_info = bytemuck::try_from_bytes::<dex::raydium_amm::AmmInfo>(data.as_slice());
            let amm_info = match amm_info {
                Ok(amm_info) => amm_info,
                Err(_) => {
                    return Err(anyhow::format_err!("Failed to deserialize AmmInfo"));
                }
            };
            let coin_a = get_token_account_amount(rpc_client, &amm_info.coin_vault).await?;
            let coin_b = get_token_account_amount(rpc_client, &amm_info.pc_vault).await?;
            Ok(Some(crate::calculator::CalculatorEnum::RaydiumAmmV4(
                crate::calculator::RaydiumAmmV4Data {
                    amm_info_pubkey: *pubkey,
                    amm_info: *amm_info,
                    total_pc: coin_b,
                    total_coin: coin_a,
                },
            )))
        }

        PoolType::Orca => {
            let whirlpool = OrcaWhirlpool::deserialize(data.as_slice())?;
            // Fixed arrays
            let memcmp = RpcFilterType::Memcmp(Memcmp::new(
                9988 - 32,                                             // offset
                MemcmpEncodedBytes::Bytes(pubkey.to_bytes().to_vec()), // encoded bytes
            ));
            let config = RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::DataSize(9988), memcmp]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: None,
                    min_context_slot: None,
                },
                with_context: Some(false),
                sort_results: Some(true),
            };
            let tick_arrays_raw = rpc_client
                .get_program_accounts_with_config(&dex::orca::ID, config)
                .await?;
            let mut tick_arrays_map = BTreeMap::new();
            for (key, account) in tick_arrays_raw.into_iter() {
                let tick_array = OrcaTickArray::deserialize(account.data.as_slice())?;
                tick_arrays_map.insert(tick_array.start_tick_index, (key, tick_array));
            }
            // Dynamic arrays
            let memcmp = RpcFilterType::Memcmp(Memcmp::new(12, MemcmpEncodedBytes::Bytes(pubkey.to_bytes().to_vec())));
            let config = RpcProgramAccountsConfig {
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: None,
                    min_context_slot: None,
                },
                with_context: Some(false),
                sort_results: Some(true),
                filters: Some(vec![memcmp]),
            };
            let tick_arrays_raw = rpc_client
                .get_program_accounts_with_config(&dex::orca::ID, config)
                .await?;
            for (key, account) in tick_arrays_raw.into_iter() {
                if let Ok(tick_array) = OrcaTickArray::deserialize_from_dynamic(account.data.as_slice()) {
                    tick_arrays_map.insert(tick_array.start_tick_index, (key, tick_array));
                }
            }
            // Result
            Ok(Some(crate::calculator::CalculatorEnum::Orca(OrcaData {
                whirlpool,
                whirlpool_pubkey: *pubkey,
                tick_arrays_map,
                // TODO: fetch oracle
                oracle: None,
            })))
        }

        PoolType::MeteoraDlmm => {
            let data = get_account_data(rpc_client, pubkey).await?;
            let lb_pair = dex::meteora_dlmm::LbPairAccount::deserialize(&data).unwrap().0;

            let memcmp = RpcFilterType::Memcmp(Memcmp::new(24, MemcmpEncodedBytes::Bytes(pubkey.to_bytes().to_vec())));
            let config = RpcProgramAccountsConfig {
                filters: Some(vec![memcmp, RpcFilterType::DataSize(10136)]), // RpcFilterType::DataSize(10136),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: None,
                    min_context_slot: None,
                },
                with_context: Some(false),
                sort_results: Some(true),
            };
            let bin_arrays_raw = rpc_client
                .get_program_accounts_with_config(&dex::meteora_dlmm::ID, config)
                .await?;

            let mut bin_arrays_map = BTreeMap::new();
            for (key, account) in bin_arrays_raw.into_iter() {
                let bin_array = dex::meteora_dlmm::BinArrayAccount::deserialize(account.data.as_slice())?;
                bin_arrays_map.insert(bin_array.0.index as i32, (key, bin_array.0));
            }

            let bimap_extension_pubkey = Pubkey::find_program_address(
                &[dex::meteora_dlmm::pda::BIN_ARRAY_BITMAP_SEED, pubkey.as_ref()],
                &dex::meteora_dlmm::ID,
            )
            .0;

            let bitmap_extension = match rpc_client.get_account_data(&bimap_extension_pubkey).await {
                Ok(data) => {
                    if !data.is_empty() {
                        Some(dex::meteora_dlmm::BinArrayBitmapExtensionAccount::deserialize(data.as_slice())?.0)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            };

            let slot = rpc_client.get_slot().await.context("GetSlotFailed")?;

            Ok(Some(crate::calculator::CalculatorEnum::MeteoraDlmm(MeteoraDlmmData {
                lb_pair_pubkey: *pubkey,
                lb_pair,
                bin_arrays_map,
                bitmap_extension_pubkey: if bitmap_extension.is_some() {
                    Some(bimap_extension_pubkey)
                } else {
                    None
                },
                bitmap_extension,
                slot,
            })))
        }

        PoolType::PumpAmm => {
            let config_data = get_account_data(rpc_client, &dex::pump_amm::GLOBAL_CONFIG_ID).await?;
            let config = dex::pump_amm::GlobalConfig::deserialize(config_data.as_slice())?;
            let pool = dex::pump_amm::Pool::deserialize(data.as_slice())?;
            let base = get_token_account_amount(rpc_client, &pool.pool_base_token_account).await?;
            let quote = get_token_account_amount(rpc_client, &pool.pool_quote_token_account).await?;
            let fee_config = rpc_client
                .get_account_data(&dex::pump_amm::FEE_CONFIG)
                .await
                .context("GetFeeConfigFailed")?;
            let fee_config = dex::pump_amm::FeeConfig::deserialize(&fee_config).context("DeserFailed")?;
            let mint_data = rpc_client.get_account_data(&pool.base_mint).await.unwrap();
            let mint = spl_token::state::Mint::unpack(&mint_data).unwrap();
            let is_pump = dex::pump_amm::is_pump_pool(&pool.base_mint, &pool.creator);
            let mc = dex::pump_amm::pool_market_cap(mint.supply as u128, base as u128, quote as u128).unwrap();
            let fees = fee_config.get_fees(is_pump, mc);
            let fees = fees.protocol_fee_bps + fees.lp_fee_bps + fees.creator_fee_bps;
            Ok(Some(crate::calculator::CalculatorEnum::PumpAmm(
                crate::calculator::PumpAmmData {
                    pool_pubkey: *pubkey,
                    pool,
                    config_pubkey: dex::pump_amm::GLOBAL_CONFIG_ID,
                    config,
                    base,
                    quote,
                    fee: fees,
                },
            )))
        }

        PoolType::StabbleStableSwap => {
            let pool_swap = dex::stabble_stable_swap::PoolAccount::deserialize(&data)?.0;
            Ok(Some(crate::calculator::CalculatorEnum::StabbleStableSwap(
                crate::calculator::StabbleStableSwapData {
                    pool_swap_pubkey: *pubkey,
                    pool_swap,
                },
            )))
        }

        PoolType::StabbleWeightedSwap => {
            let pool_swap = dex::stabble_weighted_swap::accounts::PoolWeightedSwap::try_deserialize(data.as_slice())
                .context("DeserFailed")?;
            Ok(Some(crate::calculator::CalculatorEnum::StabbleWeightedSwap(
                crate::calculator::StabbleWeightedSwapData {
                    pool_swap_pubkey: *pubkey,
                    pool_swap,
                },
            )))
        }

        // PoolType::Lifinity => {
        //     //let amm = dex::lifinity::accounts::AmmAccount::deserialize(data.as_slice())?.0;
        //     let amm = dex::lifinity::accounts::Amm::deserialize(data.as_slice())?;
        //     let total_coin = get_token_account_amount(rpc_client, &amm.token_a_account).await?;
        //     let total_pc = get_token_account_amount(rpc_client, &amm.token_b_account).await?;
        //     let slot = rpc_client.get_slot().await.context("GetSlotFailed")?;

        //     let oracle_main: dex::lifinity::pc::Price = *bytemuck::try_from_bytes(
        //         rpc_client
        //             .get_account_data(&amm.oracle_main_account)
        //             .await
        //             .unwrap()
        //             .as_slice(),
        //     )
        //     .unwrap();

        //     let oracle_sub: dex::lifinity::pc::Price = *bytemuck::try_from_bytes(
        //         rpc_client
        //             .get_account_data(&amm.oracle_sub_account)
        //             .await
        //             .unwrap()
        //             .as_slice(),
        //     )
        //     .unwrap();

        //     let oracle_pc: dex::lifinity::pc::Price = *bytemuck::try_from_bytes(
        //         rpc_client
        //             .get_account_data(&amm.oracle_pc_account)
        //             .await
        //             .unwrap()
        //             .as_slice(),
        //     )
        //     .unwrap();

        //     Ok(Some(CalculatorEnum::Lifinity(crate::calculator::LifinityData {
        //         amm_pubkey: *pubkey,
        //         amm,
        //         slot,
        //         total_pc,
        //         total_coin,
        //         oracle_main,
        //         oracle_sub,
        //         oracle_pc,
        //     })))
        // }
        PoolType::RaydiumCpmm => {
            let pool_state = dex::raydium_cpmm::PoolStateAccount::deserialize(data.as_slice())?;
            let total_coin = get_token_account_amount(rpc_client, &pool_state.token0_vault).await?;
            let total_pc = get_token_account_amount(rpc_client, &pool_state.token1_vault).await?;
            let amm_config_data = get_account_data(rpc_client, &pool_state.amm_config).await?;
            let amm_config = dex::raydium_cpmm::AmmConfigAccount::deserialize(amm_config_data.as_slice())?;
            Ok(Some(CalculatorEnum::RaydiumCpmm(crate::calculator::RaydiumCpmmData {
                pool_state_pubkey: *pubkey,
                pool_state,
                amm_config,
                total_pc,
                total_coin,
            })))
        }

        PoolType::OrcaSwapV2 => {
            let pool = dex::orca_swap_v2::accounts::Market::try_deserialize(data.as_slice())?;
            let total_coin = get_token_account_amount(rpc_client, &pool.token_a).await?;
            let total_pc = get_token_account_amount(rpc_client, &pool.token_b).await?;
            Ok(Some(CalculatorEnum::OrcaSwapV2(crate::calculator::OrcaSwapV2Data {
                market_pubkey: *pubkey,
                market: pool,
                total_coin,
                total_pc,
            })))
        }

        PoolType::SarosAmm => {
            let pool = dex::saros_amm::accounts::Market::try_deserialize(data.as_slice())?;
            let total_coin = get_token_account_amount(rpc_client, &pool.token_a).await?;
            let total_pc = get_token_account_amount(rpc_client, &pool.token_b).await?;
            Ok(Some(CalculatorEnum::SarosAmm(crate::calculator::SarosAmmData {
                market_pubkey: *pubkey,
                market: pool,
                total_coin,
                total_pc,
            })))
        }

        // PoolType::GooseGamma => {
        //     let pool = dex::goose_gamma::accounts::PoolStateAccount::deserialize(data.as_slice())?.0;
        //     let coin_a = get_token_account_amount(rpc_client, &pool.token_0vault).await?;
        //     let coin_b = get_token_account_amount(rpc_client, &pool.token_1vault).await?;

        //     let config_data = rpc_client.get_account_data(&pool.amm_config).await.unwrap();
        //     let config = dex::goose_gamma::accounts::AmmConfigAccount::deserialize(config_data.as_slice())
        //         .unwrap()
        //         .0;

        //     let observation_data = rpc_client.get_account_data(&pool.observation_key).await.unwrap();
        //     let observation_state =
        //         dex::goose_gamma::accounts::ObservationStateAccount::deserialize(observation_data.as_slice())
        //             .unwrap()
        //             .0;

        //     Ok(Some(CalculatorEnum::GooseGamma(crate::calculator::GooseGammaData {
        //         pool_pubkey: *pubkey,
        //         pool,
        //         config,
        //         observation_state,
        //         coin_a,
        //         coin_b,
        //     })))
        // }
        PoolType::MeteoraDammV2 => {
            let pool = dex::meteora_damm_v2::PoolAccount::deserialize(data.as_slice())?.0;
            let slot = rpc_client.get_slot().await.context("GetSlotFailed")?;
            Ok(Some(CalculatorEnum::MeteoraDammV2(
                crate::calculator::MeteoraDammV2Data {
                    pool_pubkey: *pubkey,
                    pool,
                    slot,
                },
            )))
        }

        PoolType::FusionAmm => {
            let pool =
                dex::fusion_amm::accounts::FusionPoolWhirlpool::deserialize(data.as_slice()).context("DeserFailed")?;
            let memcmp = RpcFilterType::Memcmp(Memcmp::new(
                9988 - 32,                                             // offset
                MemcmpEncodedBytes::Bytes(pubkey.to_bytes().to_vec()), // encoded bytes
            ));
            let config = RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::DataSize(9988), memcmp]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: None,
                    min_context_slot: None,
                },
                with_context: Some(false),
                sort_results: Some(true),
            };
            let tick_arrays_raw = rpc_client
                .get_program_accounts_with_config(&dex::fusion_amm::ID, config)
                .await?;
            let mut tick_arrays_map = BTreeMap::new();
            for (key, account) in tick_arrays_raw.into_iter() {
                // Note: using Orca stuff
                let tick_array = OrcaTickArray::deserialize(account.data.as_slice())?;
                tick_arrays_map.insert(tick_array.start_tick_index, (key, tick_array));
            }
            Ok(Some(crate::calculator::CalculatorEnum::FusionAmm(
                crate::calculator::FusionAmmData {
                    pool_pubkey: *pubkey,
                    pool,
                    tick_arrays_map,
                },
            )))
        }

        PoolType::SarosDlmm => {
            let pair = dex::saros_dlmm::PairAccount::deserialize(data.as_slice())?.0;
            let memcmp_discr = RpcFilterType::Memcmp(Memcmp::new(
                0,
                MemcmpEncodedBytes::Bytes(dex::saros_dlmm::BIN_ARRAY_ACCOUNT_DISCM.to_vec()),
            ));
            let memcmp_pk = RpcFilterType::Memcmp(Memcmp::new(
                8, // discriminator
                MemcmpEncodedBytes::Bytes(pubkey.to_bytes().to_vec()),
            ));
            let config = RpcProgramAccountsConfig {
                filters: Some(vec![/*RpcFilterType::DataSize(8248),*/ memcmp_discr, memcmp_pk]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: None,
                    min_context_slot: None,
                },
                with_context: Some(false),
                sort_results: Some(true),
            };
            let bin_arrays_raw = rpc_client
                .get_program_accounts_with_config(&dex::saros_dlmm::ID, config)
                .await?;
            let mut bin_arrays_map = BTreeMap::new();
            for (key, account) in bin_arrays_raw.into_iter() {
                let bin_array = dex::saros_dlmm::BinArrayAccount::deserialize(account.data.as_slice())?.0;
                bin_arrays_map.insert(bin_array.index, (key, bin_array));
            }
            Ok(Some(CalculatorEnum::SarosDlmm(crate::calculator::SarosDlmmData {
                pool_pubkey: *pubkey,
                pair,
                bin_arrays_map,
            })))
        }
    }
}
