use ahash::AHashMap as HashMap;
use anyhow::Context;
use config::CONFIG as cfg;
use dex::orca::{get_oracle_address, OrcaTickArray, OrcaWhirlpool, TICK_ARRAY_DISCRIMINATOR, WHIRLPOOL_DISCRIMINATOR};
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use std::collections::BTreeMap;
use utils::pool::{Pool, PoolType};

use crate::gpa::PoolToCalculator;

#[derive(Default)]
pub struct OrcaGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::orca::OrcaWhirlpool>,
    pub tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::orca::OrcaTickArray)>>,
    pub oracles: HashMap<Pubkey, Pubkey>,
    pub pool_to_oracle: HashMap<Pubkey, dex::orca::Oracle>,
}

#[derive(Debug, Default)]
pub struct OrcaSwapV2GPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub markets: HashMap<Pubkey, dex::orca_swap_v2::accounts::Market>,
    pub authorities: HashMap<Pubkey, Pubkey>,
}

pub fn process_orca_swap_v2(
    pubkey: Pubkey,
    data: &[u8],
    markets: &mut HashMap<Pubkey, dex::orca_swap_v2::accounts::Market>,
    authorities: &mut HashMap<Pubkey, Pubkey>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> Option<PoolToCalculator> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.len() == dex::orca_swap_v2::accounts::MARKET_LENGTH {
        let market = dex::orca_swap_v2::accounts::Market::try_deserialize(data);
        if let Ok(market) = market {
            if cfg.arbitrage.is_c2_only() && !market.has_wsol() {
                return None;
            }
            let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
            pool_to_calculator = Some(PoolToCalculator {
                is_new,
                pubkey,
                keys: if is_new { Some(market.copy_keys()) } else { None },
                pool_type: PoolType::OrcaSwapV2,
            });
            pool_type_and_pubkey.insert(pubkey, PoolType::OrcaSwapV2);
            if !authorities.contains_key(&pubkey) {
                let authority = dex::orca_swap_v2::accounts::get_authority(&pubkey);
                authorities.insert(pubkey, authority);
            }
            markets.insert(pubkey, market);
        }
    }
    pool_to_calculator
}

pub fn process_orca(
    pubkey: Pubkey,
    data: &[u8],
    orca_pools: &mut HashMap<Pubkey, OrcaWhirlpool>,
    // pass None if you don't want to load Orca oracles
    orca_oracles: Option<&mut HashMap<Pubkey, Pubkey>>,
    orca_pool_to_oracle: &mut HashMap<Pubkey, dex::orca::Oracle>,
    orca_tick_arrays: &mut HashMap<Pubkey, BTreeMap<i32, (Pubkey, OrcaTickArray)>>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&WHIRLPOOL_DISCRIMINATOR) {
        let orca_whirlpool = OrcaWhirlpool::deserialize(data);
        let orca_whirlpool =
            orca_whirlpool.map_err(|e| anyhow::format_err!("Failed to deserialize orca whirlpool: {:?}", e))?;
        if cfg.arbitrage.is_c2_only() && !orca_whirlpool.has_wsol() {
            return Ok(None);
        }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new { Some(orca_whirlpool.copy_keys()) } else { None },
            pool_type: PoolType::Orca,
        });
        pool_type_and_pubkey.insert(pubkey, PoolType::Orca);
        orca_pools.insert(pubkey, orca_whirlpool);
        if let Some(orca_oracles) = orca_oracles {
            if !orca_oracles.contains_key(&pubkey) {
                if let Ok(oracle) = get_oracle_address(&pubkey) {
                    orca_oracles.insert(pubkey, oracle.0);
                }
            }
        }
    } else if data.starts_with(&TICK_ARRAY_DISCRIMINATOR) {
        // TODO: ak sú len 2-comb, tu by som nemal vkladať tick_array, ak tick_array.whirlpool nemá WSOL!
        let tick_array = OrcaTickArray::deserialize(data).context("Failed to deserialize orca tick array")?;
        let not_empty = tick_array.ticks.iter().any(|x| x.liquidity_gross > 0);
        let whirlpool = tick_array.whirlpool;
        if let Some(mut tick_arrays_map) = orca_tick_arrays.remove(&whirlpool) {
            //debug!("tick_arrays_map len {:?}", tick_arrays_map.len());
            if not_empty {
                tick_arrays_map.insert(tick_array.start_tick_index, (pubkey, tick_array));
                orca_tick_arrays.insert(whirlpool, tick_arrays_map);
            } else {
                let _ = tick_arrays_map.remove(&tick_array.start_tick_index);
                orca_tick_arrays.insert(whirlpool, tick_arrays_map);
            }
        } else if not_empty {
            let mut tick_arrays_map = BTreeMap::new();
            tick_arrays_map.insert(tick_array.start_tick_index, (pubkey, tick_array));
            orca_tick_arrays.insert(whirlpool, tick_arrays_map);
        }
    } else if data.starts_with(&dex::orca::DYNAMIC_TICK_ARRAY_DISCRIMINATOR) {
        // TODO: process
        // TODO: ak sú len 2-comb, tu by som nemal vkladať tick_array, ak tick_array.whirlpool nemá WSOL!
        let tick_array =
            OrcaTickArray::deserialize_from_dynamic(data).context("Failed to deserialize orca tick array")?;
        let not_empty = tick_array.ticks.iter().any(|x| x.liquidity_gross > 0);
        let whirlpool = tick_array.whirlpool;
        if let Some(mut tick_arrays_map) = orca_tick_arrays.remove(&whirlpool) {
            //debug!("tick_arrays_map len {:?}", tick_arrays_map.len());
            if not_empty {
                tick_arrays_map.insert(tick_array.start_tick_index, (pubkey, tick_array));
                orca_tick_arrays.insert(whirlpool, tick_arrays_map);
            } else {
                let _ = tick_arrays_map.remove(&tick_array.start_tick_index);
                orca_tick_arrays.insert(whirlpool, tick_arrays_map);
            }
        } else if not_empty {
            let mut tick_arrays_map = BTreeMap::new();
            tick_arrays_map.insert(tick_array.start_tick_index, (pubkey, tick_array));
            orca_tick_arrays.insert(whirlpool, tick_arrays_map);
        }
    } else if data.starts_with(&dex::orca::ORACLE_DISCRIMINATOR) {
        let oracle = dex::orca::Oracle::deserialize(data).context("Failed to deserialize orca oracle")?;
        if let Some(orca_oracles) = orca_oracles {
            if !orca_oracles.contains_key(&oracle.whirlpool) {
                orca_oracles.insert(oracle.whirlpool, pubkey);
            }
        }
        orca_pool_to_oracle.insert(oracle.whirlpool, oracle);
    }
    Ok(pool_to_calculator)
}

pub fn spawn_orca(url: String, cfg_opt: Option<RpcProgramAccountsConfig>) -> tokio::task::JoinHandle<OrcaGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::orca::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::orca::ID).await
        };
        let mut orca_pools: HashMap<Pubkey, OrcaWhirlpool> = HashMap::new();
        let mut orca_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, OrcaTickArray)>> = HashMap::new();
        let mut pool_type_and_pubkey: HashMap<Pubkey, PoolType> = HashMap::new();
        let mut orca_oracles: HashMap<Pubkey, Pubkey> = HashMap::new();
        let mut orca_pool_to_oracle: HashMap<Pubkey, dex::orca::Oracle> = HashMap::new();
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_orca(
                pubkey,
                account.data.as_slice(),
                &mut orca_pools,
                Some(&mut orca_oracles),
                &mut orca_pool_to_oracle,
                &mut orca_tick_arrays,
                &mut pool_type_and_pubkey,
            );
            if let Err(e) = res {
                log::error!("Failed to process orca: {:?}", e);
            }
        });
        orca_pools.iter().for_each(|(pubkey, _)| {
            if let Ok(oracle) = get_oracle_address(pubkey) {
                orca_oracles.insert(*pubkey, oracle.0);
            }
        });
        OrcaGPAResult {
            pool_type_and_pubkey,
            pools: orca_pools,
            tick_arrays: orca_tick_arrays,
            oracles: orca_oracles,
            pool_to_oracle: orca_pool_to_oracle,
        }
    })
}

pub fn spawn_orca_swap_v2(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<OrcaSwapV2GPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::orca_swap_v2::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::orca_swap_v2::ID).await
        };
        let mut orca_swap_v2_result = OrcaSwapV2GPAResult {
            pool_type_and_pubkey: Default::default(),
            markets: Default::default(),
            authorities: Default::default(),
        };
        result.into_iter().for_each(|(pubkey, account)| {
            let _ = process_orca_swap_v2(
                pubkey,
                account.data.as_slice(),
                &mut orca_swap_v2_result.markets,
                &mut orca_swap_v2_result.authorities,
                &mut orca_swap_v2_result.pool_type_and_pubkey,
            );
        });
        orca_swap_v2_result
    })
}
