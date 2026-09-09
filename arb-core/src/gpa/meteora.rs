use std::collections::BTreeMap;

use ahash::AHashMap as HashMap;

use crate::gpa::PoolToCalculator;
use anyhow::Context;
use config::CONFIG as cfg;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use utils::pool::{Pool, PoolType};

#[derive(Default)]
pub struct MeteoraDlmmGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::meteora_dlmm::LbPair>,
    pub bin_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::meteora_dlmm::BinArray)>>,
    pub bitmap_extensions: HashMap<Pubkey, (Pubkey, dex::meteora_dlmm::BinArrayBitmapExtension)>,
}

#[derive(Debug, Default)]
pub struct MeteoraDammV2GPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::meteora_damm_v2::PoolState>,
}

pub fn process_meteora_dlmm(
    pubkey: Pubkey,
    data: &[u8],
    meteora_dlmm_pools: &mut HashMap<Pubkey, dex::meteora_dlmm::LbPair>,
    meteora_dlmm_bin_arrays: &mut HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::meteora_dlmm::BinArray)>>,
    meteora_dlmm_bitmap_extensions: &mut HashMap<Pubkey, (Pubkey, dex::meteora_dlmm::BinArrayBitmapExtension)>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&dex::meteora_dlmm::accounts::LB_PAIR_ACCOUNT_DISCM) {
        let lb_pair = dex::meteora_dlmm::LbPairAccount::deserialize(data)
            .context("Failed to deserialize meteora dlmm")?
            .0;
        if cfg.arbitrage.is_c2_only() && !lb_pair.has_wsol() {
            return Ok(None);
        }
        // if lb_pair.is_token_2022() {
        //     return Ok(None);
        // }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new { Some(lb_pair.copy_keys()) } else { None },
            pool_type: PoolType::MeteoraDlmm,
        });
        meteora_dlmm_pools.insert(pubkey, lb_pair);
        pool_type_and_pubkey.insert(pubkey, PoolType::MeteoraDlmm);
        // if let Some(token_mints) = token_mints {
        //     if let Some(new_mint) = lb_pair.update_new_tokens(token_mints) {
        //         return Ok(pool_to_calculator);
        //     }
        // }
    } else if data.starts_with(&dex::meteora_dlmm::accounts::BIN_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM) {
        // TODO: ak sú len 2-comb, tu by som nemal vkladať bitmap_extension, ak bitmap_extension.lb_pair nemá WSOL!
        // Možné riešenie: zaviesť HashMap ignorovaných pool-ov (všeobecne, nie podľa pool_type)
        let bitmap_extension = dex::meteora_dlmm::accounts::BinArrayBitmapExtensionAccount::deserialize(data)
            .context("Failed to deserialize meteora dlmm bitmap extension")?
            .0;
        meteora_dlmm_bitmap_extensions.insert(bitmap_extension.lb_pair, (pubkey, bitmap_extension));
    } else if data.starts_with(&dex::meteora_dlmm::accounts::BIN_ARRAY_ACCOUNT_DISCM) {
        // TODO: ak sú len 2-comb, tu by som nemal vkladať bin_array, ak bin_array.lb_pair nemá WSOL!
        let bin_array = dex::meteora_dlmm::BinArrayAccount::deserialize(data)
            .context("Failed to deserialize meteora dlmm bin array")?
            .0;
        let not_empty = bin_array.bins.iter().any(|x| !x.is_zero_liquidity());
        let lb_pair_pk = bin_array.lb_pair;
        if let Some(mut bin_arrays_map) = meteora_dlmm_bin_arrays.remove(&lb_pair_pk) {
            if not_empty {
                bin_arrays_map.insert(bin_array.index as i32, (pubkey, bin_array));
                meteora_dlmm_bin_arrays.insert(lb_pair_pk, bin_arrays_map);
            } else {
                // TODO: get_mup()_or_insert()?
                let _ = bin_arrays_map.remove(&(bin_array.index as i32));
                meteora_dlmm_bin_arrays.insert(lb_pair_pk, bin_arrays_map);
            }
        } else if not_empty {
            let mut bin_arrays_map = BTreeMap::new();
            bin_arrays_map.insert(bin_array.index as i32, (pubkey, bin_array));
            meteora_dlmm_bin_arrays.insert(lb_pair_pk, bin_arrays_map);
        }
    }
    Ok(pool_to_calculator)
}

pub fn process_meteora_damm_v2(
    pubkey: Pubkey,
    data: &[u8],
    meteora_damm_pools_map: &mut HashMap<Pubkey, dex::meteora_damm_v2::PoolState>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&dex::meteora_damm_v2::POOL_ACCOUNT_DISCM) {
        let pool = dex::meteora_damm_v2::PoolAccount::deserialize(data)?.0;
        if cfg.arbitrage.is_c2_only() && !pool.has_wsol() {
            return Ok(None);
        }
        // if pool.is_token_2022() {
        //     return Ok(None);
        // }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new { Some(pool.copy_keys()) } else { None },
            pool_type: PoolType::MeteoraDammV2,
        });
        meteora_damm_pools_map.insert(pubkey, pool);
        pool_type_and_pubkey.insert(pubkey, PoolType::MeteoraDammV2);
    }
    Ok(pool_to_calculator)
}

pub fn spawn_meteora_dlmm(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<MeteoraDlmmGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::meteora_dlmm::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts_by_discriminators(
                &url,
                &dex::meteora_dlmm::ID,
                &[
                    dex::meteora_dlmm::LB_PAIR_ACCOUNT_DISCM,
                    dex::meteora_dlmm::BIN_ARRAY_ACCOUNT_DISCM,
                    dex::meteora_dlmm::BIN_ARRAY_BITMAP_EXTENSION_ACCOUNT_DISCM,
                ],
            )
            .await
        };
        let mut meteora_dlmm_pools = HashMap::new();
        let mut meteora_dlmm_bin_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::meteora_dlmm::BinArray)>> =
            HashMap::new();
        let mut meteora_dlmm_bitmap_extensions = HashMap::new();
        let mut pool_type_and_pubkey: HashMap<Pubkey, PoolType> = HashMap::new();
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_meteora_dlmm(
                pubkey,
                account.data.as_slice(),
                &mut meteora_dlmm_pools,
                &mut meteora_dlmm_bin_arrays,
                &mut meteora_dlmm_bitmap_extensions,
                &mut pool_type_and_pubkey,
            );
            if let Err(e) = res {
                log::error!("Failed to process meteora dlmm: {:?}", e);
            }
        });
        MeteoraDlmmGPAResult {
            pool_type_and_pubkey,
            pools: meteora_dlmm_pools,
            bin_arrays: meteora_dlmm_bin_arrays,
            bitmap_extensions: meteora_dlmm_bitmap_extensions,
        }
    })
}

pub fn spawn_meteora_damm_v2(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<MeteoraDammV2GPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::meteora_damm_v2::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::meteora_damm_v2::ID).await
        };
        let mut meteora_damm_v2_pools_map: HashMap<Pubkey, dex::meteora_damm_v2::PoolState> = HashMap::new();
        let mut pool_type_and_pubkey: HashMap<Pubkey, PoolType> = HashMap::new();
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_meteora_damm_v2(
                pubkey,
                account.data.as_slice(),
                &mut meteora_damm_v2_pools_map,
                &mut pool_type_and_pubkey,
            );
            if let Err(e) = res {
                log::error!("Failed to process meteora damm v2: {:?}", e);
            }
        });
        MeteoraDammV2GPAResult {
            pool_type_and_pubkey,
            pools: meteora_damm_v2_pools_map,
        }
    })
}
