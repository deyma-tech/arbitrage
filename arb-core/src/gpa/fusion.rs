use ahash::AHashMap as HashMap;
use anyhow::Context;
use config::CONFIG as cfg;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use std::collections::BTreeMap;
use utils::pool::{Pool, PoolType};

use crate::gpa::PoolToCalculator;

pub struct FusionAmmGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::fusion_amm::FusionPoolWhirlpool>,
    pub tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::orca::OrcaTickArray)>>,
}

pub fn process_fusion(
    pubkey: Pubkey,
    data: &[u8],
    fusion_pools: &mut HashMap<Pubkey, dex::fusion_amm::FusionPoolWhirlpool>,
    fusion_tick_arrays: &mut HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::orca::OrcaTickArray)>>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&dex::fusion_amm::FUSION_POOL_ACCOUNT_DISCM) {
        let pool = dex::fusion_amm::FusionPoolWhirlpool::deserialize(data);
        let pool = pool.map_err(|e| anyhow::format_err!("Failed to deserialize fusion amm: {:?}", e))?;
        if cfg.arbitrage.is_c2_only() && !pool.has_wsol() {
            return Ok(None);
        }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new { Some(pool.copy_keys()) } else { None },
            pool_type: PoolType::FusionAmm,
        });
        pool_type_and_pubkey.insert(pubkey, PoolType::FusionAmm);
        fusion_pools.insert(pubkey, pool);
    } else if data.starts_with(&dex::fusion_amm::TICK_ARRAY_ACCOUNT_DISCM) {
        let tick_array =
            dex::orca::OrcaTickArray::deserialize(data).context("Failed to deserialize fusion tick array")?;
        let not_empty = tick_array.ticks.iter().any(|x| x.liquidity_gross > 0);
        let pool_pubkey = tick_array.whirlpool;
        if let Some(mut tick_arrays_map) = fusion_tick_arrays.remove(&pool_pubkey) {
            //debug!("tick_arrays_map len {:?}", tick_arrays_map.len());
            if not_empty {
                tick_arrays_map.insert(tick_array.start_tick_index, (pubkey, tick_array));
                fusion_tick_arrays.insert(pool_pubkey, tick_arrays_map);
            } else {
                let _ = tick_arrays_map.remove(&tick_array.start_tick_index);
                fusion_tick_arrays.insert(pool_pubkey, tick_arrays_map);
            }
        } else if not_empty {
            let mut tick_arrays_map = BTreeMap::new();
            tick_arrays_map.insert(tick_array.start_tick_index, (pubkey, tick_array));
            fusion_tick_arrays.insert(pool_pubkey, tick_arrays_map);
        }
    }
    Ok(pool_to_calculator)
}

pub fn spawn_fusion(
    url: String,
    cfg_op: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<FusionAmmGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_op {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::fusion_amm::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::fusion_amm::ID).await
        };
        let mut fusion_pools: HashMap<Pubkey, dex::fusion_amm::FusionPoolWhirlpool> = HashMap::new();
        let mut fusion_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::orca::OrcaTickArray)>> = HashMap::new();
        let mut pool_type_and_pubkey: HashMap<Pubkey, PoolType> = HashMap::new();
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_fusion(
                pubkey,
                account.data.as_slice(),
                &mut fusion_pools,
                &mut fusion_tick_arrays,
                &mut pool_type_and_pubkey,
            );
            if let Err(e) = res {
                log::error!("Failed to process fusion: {:?}", e);
            }
        });
        FusionAmmGPAResult {
            pool_type_and_pubkey,
            pools: fusion_pools,
            tick_arrays: fusion_tick_arrays,
        }
    })
}
