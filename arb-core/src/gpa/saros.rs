use ahash::AHashMap as HashMap;
use anyhow::Context;
use config::CONFIG as cfg;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use std::collections::BTreeMap;
use utils::pool::{Pool, PoolType};

use crate::gpa::PoolToCalculator;

#[derive(Debug)]
pub struct SarosAmmGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub markets: HashMap<Pubkey, dex::saros_amm::accounts::Market>,
    pub authorities: HashMap<Pubkey, Pubkey>,
}

pub struct SarosDlmmGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::saros_dlmm::Pair>,
    pub bin_arrays: HashMap<Pubkey, BTreeMap<u32, (Pubkey, dex::saros_dlmm::BinArray)>>,
}

pub fn process_saros_amm(
    pubkey: Pubkey,
    data: &[u8],
    markets: &mut HashMap<Pubkey, dex::saros_amm::accounts::Market>,
    authorities: &mut HashMap<Pubkey, Pubkey>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> Option<PoolToCalculator> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.len() == dex::saros_amm::accounts::MARKET_LENGTH {
        let market = dex::saros_amm::accounts::Market::try_deserialize(data);
        if let Ok(market) = market {
            if cfg.arbitrage.is_c2_only() && !market.has_wsol() {
                return None;
            }
            let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
            pool_to_calculator = Some(PoolToCalculator {
                is_new,
                pubkey,
                keys: if is_new { Some(market.copy_keys()) } else { None },
                pool_type: PoolType::SarosAmm,
            });
            pool_type_and_pubkey.insert(pubkey, PoolType::SarosAmm);
            if !authorities.contains_key(&pubkey) {
                let authority = dex::saros_amm::accounts::get_authority(&pubkey);
                authorities.insert(pubkey, authority);
            }
            markets.insert(pubkey, market);
        }
    }
    pool_to_calculator
}

pub fn process_saros_dlmm(
    pubkey: Pubkey,
    data: &[u8],
    saros_pools: &mut HashMap<Pubkey, dex::saros_dlmm::Pair>,
    saros_bin_arrays: &mut HashMap<Pubkey, BTreeMap<u32, (Pubkey, dex::saros_dlmm::BinArray)>>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&dex::saros_dlmm::PAIR_ACCOUNT_DISCM) {
        let pool = dex::saros_dlmm::PairAccount::deserialize(data);
        let pool = pool
            .map_err(|e| anyhow::format_err!("Failed to deserialize saros: {:?}", e))?
            .0;
        if cfg.arbitrage.is_c2_only() && !pool.has_wsol() {
            return Ok(None);
        }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new { Some(pool.copy_keys()) } else { None },
            pool_type: PoolType::SarosDlmm,
        });
        pool_type_and_pubkey.insert(pubkey, PoolType::SarosDlmm);
        saros_pools.insert(pubkey, pool);
    } else if data.starts_with(&dex::saros_dlmm::BIN_ARRAY_ACCOUNT_DISCM) {
        let bin_array = dex::saros_dlmm::BinArrayAccount::deserialize(data)
            .context("Failed to deserialize saros bin array")?
            .0;
        let not_empty = bin_array.bins.iter().any(|x| x.total_supply > 0);
        let pool_pubkey = bin_array.pair;
        if let Some(mut bin_arrays_map) = saros_bin_arrays.remove(&pool_pubkey) {
            if not_empty {
                bin_arrays_map.insert(bin_array.index, (pubkey, bin_array));
                saros_bin_arrays.insert(pool_pubkey, bin_arrays_map);
            } else {
                let _ = bin_arrays_map.remove(&bin_array.index);
                saros_bin_arrays.insert(pool_pubkey, bin_arrays_map);
            }
        } else if not_empty {
            let mut bin_arrays_map = BTreeMap::new();
            bin_arrays_map.insert(bin_array.index, (pubkey, bin_array));
            saros_bin_arrays.insert(pool_pubkey, bin_arrays_map);
        }
    }
    Ok(pool_to_calculator)
}

pub fn spawn_saros_amm(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<SarosAmmGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::saros_amm::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::saros_amm::ID).await
        };
        let mut saros_amm_result = SarosAmmGPAResult {
            pool_type_and_pubkey: Default::default(),
            markets: Default::default(),
            authorities: Default::default(),
        };
        result.into_iter().for_each(|(pubkey, account)| {
            let _ = process_saros_amm(
                pubkey,
                account.data.as_slice(),
                &mut saros_amm_result.markets,
                &mut saros_amm_result.authorities,
                &mut saros_amm_result.pool_type_and_pubkey,
            );
        });
        saros_amm_result
    })
}

pub fn spawn_saros_dlmm(
    url: String,
    cfg_op: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<SarosDlmmGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_op {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::saros_dlmm::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::saros_dlmm::ID).await
        };
        let mut pools: HashMap<Pubkey, dex::saros_dlmm::Pair> = HashMap::new();
        let mut bin_arrays: HashMap<Pubkey, BTreeMap<u32, (Pubkey, dex::saros_dlmm::BinArray)>> = HashMap::new();
        let mut pool_type_and_pubkey: HashMap<Pubkey, PoolType> = HashMap::new();
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_saros_dlmm(
                pubkey,
                account.data.as_slice(),
                &mut pools,
                &mut bin_arrays,
                &mut pool_type_and_pubkey,
            );
            if let Err(e) = res {
                log::error!("Failed to process saros: {:?}", e);
            }
        });
        SarosDlmmGPAResult {
            pool_type_and_pubkey,
            pools,
            bin_arrays,
        }
    })
}
