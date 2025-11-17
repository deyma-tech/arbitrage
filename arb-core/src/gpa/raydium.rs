use std::collections::BTreeMap;

use ahash::AHashMap as HashMap;
use anyhow::Context;
use config::CONFIG as cfg;
use dex::raydium_clmm::TickArrayStateAccount;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use utils::pool::{Pool, PoolType};

use crate::gpa::PoolToCalculator;

const SIZE_AMM_INFO: usize = size_of::<dex::raydium_amm::AmmInfo>();

pub struct RaydiumAmmGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::raydium_amm::AmmInfo>,
}

pub struct RaydiumClmmGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::raydium_clmm::PoolState>,
    pub tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::raydium_clmm::TickArrayState)>>,
    pub config: HashMap<Pubkey, dex::raydium_clmm::AmmConfig>,
    pub bitmap_extensions: HashMap<Pubkey, (Pubkey, dex::raydium_clmm::TickArrayBitmapExtension)>,
}

#[derive(Debug)]
pub struct RaydiumCpmmGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::raydium_cpmm::PoolState>,
    pub configs: HashMap<Pubkey, dex::raydium_cpmm::AmmConfig>,
}

pub fn process_raydium_amm(
    pubkey: Pubkey,
    data: &[u8],
    raydium_amm_v4_map: &mut HashMap<Pubkey, dex::raydium_amm::AmmInfo>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.len() == SIZE_AMM_INFO {
        let amm_info = bytemuck::try_from_bytes::<dex::raydium_amm::AmmInfo>(data);
        if let Ok(amm_info) = amm_info {
            if cfg.arbitrage.is_c2_only() && !amm_info.has_wsol() {
                return Ok(None);
            }
            let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
            pool_to_calculator = Some(PoolToCalculator {
                is_new,
                pubkey,
                keys: if is_new { Some(amm_info.copy_keys()) } else { None },
                pool_type: PoolType::RaydiumAmmV4,
            });
            raydium_amm_v4_map.insert(pubkey, *amm_info);
            pool_type_and_pubkey.insert(pubkey, PoolType::RaydiumAmmV4);
        } else {
            return Err(anyhow::format_err!(
                "Failed to deserialize raydium amm v4: {:?}",
                amm_info
            ));
        }
    }
    Ok(pool_to_calculator)
}

pub fn process_raydium_cpmm(
    pubkey: Pubkey,
    data: &[u8],
    raydium_cpmm_configs: &mut HashMap<Pubkey, dex::raydium_cpmm::AmmConfig>,
    raydium_cpmm_pool_maps: &mut HashMap<Pubkey, dex::raydium_cpmm::PoolState>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator = None;
    if data.starts_with(&dex::raydium_cpmm::AMM_CONFIG_ACCOUNT_DISCM) {
        let amm_info = dex::raydium_cpmm::AmmConfigAccount::deserialize(data);
        if let Ok(amm_info) = amm_info {
            raydium_cpmm_configs.insert(pubkey, amm_info);
        }
    } else if data.starts_with(&dex::raydium_cpmm::POOL_STATE_ACCOUNT_DISCM) {
        let pool_state = dex::raydium_cpmm::PoolStateAccount::deserialize(data);
        if let Ok(pool_state) = pool_state {
            if cfg.arbitrage.is_c2_only() && !pool_state.has_wsol() {
                return Ok(None);
            }
            // if pool_state.is_token_2022() {
            //     return Ok(None);
            // }
            let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
            pool_to_calculator = Some(PoolToCalculator {
                is_new,
                pubkey,
                keys: if is_new { Some(pool_state.copy_keys()) } else { None },
                pool_type: PoolType::RaydiumCpmm,
            });
            raydium_cpmm_pool_maps.insert(pubkey, pool_state);
            pool_type_and_pubkey.insert(pubkey, PoolType::RaydiumCpmm);
        }
    }
    Ok(pool_to_calculator)
}

#[allow(clippy::too_many_arguments)]
pub fn process_raydium_clmm(
    pubkey: Pubkey,
    data: &[u8],
    raydium_clmm_tick_arrays: &mut HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::raydium_clmm::TickArrayState)>>,
    raydium_clmm_pools: &mut HashMap<Pubkey, dex::raydium_clmm::PoolState>,
    raydium_clmm_config: &mut HashMap<Pubkey, dex::raydium_clmm::AmmConfig>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
    bitmap_extensions: &mut HashMap<Pubkey, (Pubkey, dex::raydium_clmm::TickArrayBitmapExtension)>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&dex::raydium_clmm::POOL_STATE_ACCOUNT_DISCM) {
        //let pool_state_account = PoolStateAccount::deserialize(data);
        let pool_state_account = dex::raydium_clmm::RayPoolState::deserialize(data);
        let pool_state_account =
            pool_state_account.map_err(|e| anyhow::format_err!("process_raydium_clmm: {:?}", e))?;
        if cfg.arbitrage.is_c2_only() && !pool_state_account.has_wsol() {
            return Ok(None);
        }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new {
                Some(pool_state_account.copy_keys())
            } else {
                None
            },
            pool_type: PoolType::RaydiumClmmV3,
        });
        pool_type_and_pubkey.insert(pubkey, PoolType::RaydiumClmmV3);
        raydium_clmm_pools.insert(pubkey, pool_state_account);
    } else if data.starts_with(&dex::raydium_clmm::AMM_CONFIG_ACCOUNT_DISCM) {
        let amm_config =
            dex::raydium_clmm::RayAmmConfig::deserialize(data).context("FailedToDeser:RaydiumClmmConfig")?;
        raydium_clmm_config.insert(pubkey, amm_config);
    } else if data.starts_with(&dex::raydium_clmm::TICK_ARRAY_STATE_ACCOUNT_DISCM) {
        let tick_array = TickArrayStateAccount::deserialize(data).context("FailedToDeser:RaydiumClmmTickArray")?;
        let pool_id = tick_array.0.pool_id;
        let not_empty = tick_array.0.ticks.iter().any(|x| x.liquidity_gross > 0);
        if let Some(mut tick_arrays_map) = raydium_clmm_tick_arrays.remove(&pool_id) {
            if not_empty {
                tick_arrays_map.insert(tick_array.0.start_tick_index, (pubkey, tick_array.0));
                raydium_clmm_tick_arrays.insert(pool_id, tick_arrays_map);
            } else {
                let _ = tick_arrays_map.remove(&tick_array.0.start_tick_index);
                raydium_clmm_tick_arrays.insert(pool_id, tick_arrays_map);
            }
        } else if not_empty {
            let mut tick_arrays_map = BTreeMap::new();
            tick_arrays_map.insert(tick_array.0.start_tick_index, (pubkey, tick_array.0));
            raydium_clmm_tick_arrays.insert(pool_id, tick_arrays_map);
        }
    } else if data.starts_with(&dex::raydium_clmm::TICK_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM) {
        let bitmap_extension = dex::raydium_clmm::TickArrayBitmapExtensionAccount::deserialize(data)
            .context("FailedToDeser:RaydiumClmmBitmapExtension")?
            .0;
        bitmap_extensions.insert(bitmap_extension.pool_id, (pubkey, bitmap_extension));
    }
    Ok(pool_to_calculator)
}

pub fn spawn_raydium_amm(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<RaydiumAmmGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::raydium_amm::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::raydium_amm::ID).await
        };
        let mut raydium_amm_v4_map = HashMap::new();
        let mut pool_type_and_pubkey: HashMap<Pubkey, PoolType> = HashMap::new();
        result.into_iter().for_each(|(pubkey, account)| {
            let _ = process_raydium_amm(
                pubkey,
                account.data.as_slice(),
                &mut raydium_amm_v4_map,
                &mut pool_type_and_pubkey,
            );
        });
        RaydiumAmmGPAResult {
            pool_type_and_pubkey,
            pools: raydium_amm_v4_map,
        }
    })
}

pub fn spawn_raydium_cpmm(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<RaydiumCpmmGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::raydium_cpmm::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::raydium_cpmm::ID).await
        };
        let mut raydium_cpmm_result = RaydiumCpmmGPAResult {
            pool_type_and_pubkey: Default::default(),
            pools: Default::default(),
            configs: Default::default(),
        };
        result.into_iter().for_each(|(pubkey, account)| {
            let _res = process_raydium_cpmm(
                pubkey,
                account.data.as_slice(),
                &mut raydium_cpmm_result.configs,
                &mut raydium_cpmm_result.pools,
                &mut raydium_cpmm_result.pool_type_and_pubkey,
            );
        });
        raydium_cpmm_result
    })
}

pub fn spawn_raydium_clmm(url: String) -> tokio::task::JoinHandle<RaydiumClmmGPAResult> {
    let discriminators = vec![
        dex::raydium_clmm::TICK_ARRAY_STATE_ACCOUNT_DISCM,
        dex::raydium_clmm::POOL_STATE_ACCOUNT_DISCM,
        dex::raydium_clmm::AMM_CONFIG_ACCOUNT_DISCM,
        dex::raydium_clmm::TICK_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM,
    ];

    tokio::spawn(async move {
        let rpc_client = crate::gpa::new_rpc_client(&url);
        let result = match crate::gpa::get_program_accounts(&rpc_client, &dex::raydium_clmm::ID, discriminators).await {
            Ok(accounts) => accounts,
            Err(err) => {
                log::error!("FailedToGetProgramAccounts: {:?}", err);
                vec![]
            }
        };

        let mut raydium_clmm_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::raydium_clmm::TickArrayState)>> =
            HashMap::new();
        let mut raydium_clmm_pools = HashMap::new();
        let mut raydium_clmm_config = HashMap::new();
        let mut raydium_bitmap_extensions = HashMap::new();
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_raydium_clmm(
                pubkey,
                account.data.as_slice(),
                &mut raydium_clmm_tick_arrays,
                &mut raydium_clmm_pools,
                &mut raydium_clmm_config,
                &mut HashMap::default(),
                &mut raydium_bitmap_extensions,
            );
            if let Err(e) = res {
                log::error!("Failed to process raydium clmm: {:?}", e);
            }
        });
        RaydiumClmmGPAResult {
            pool_type_and_pubkey: Default::default(),
            pools: raydium_clmm_pools,
            tick_arrays: raydium_clmm_tick_arrays,
            config: raydium_clmm_config,
            bitmap_extensions: raydium_bitmap_extensions,
        }
    })
}
