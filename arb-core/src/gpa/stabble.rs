use ahash::AHashMap as HashMap;

use anyhow::Context;
use config::CONFIG as cfg;
use dex::{stabble_stable_swap::PoolAccount, stabble_weighted_swap::accounts::PoolWeightedSwap};
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use utils::pool::{Pool, PoolType};

use crate::gpa::PoolToCalculator;

#[derive(Debug)]
pub struct StabbleStableSwapGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::stabble_stable_swap::PoolSwap>,
}

#[derive(Debug)]
pub struct StabbleWeightedSwapGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::stabble_weighted_swap::accounts::PoolWeightedSwap>,
}

pub fn process_stabble_stable_swap(
    pubkey: Pubkey,
    data: &[u8],
    pools: &mut HashMap<Pubkey, dex::stabble_stable_swap::PoolSwap>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&dex::stabble_stable_swap::POOL_ACCOUNT_DISCM) {
        let pool = PoolAccount::deserialize(data)
            .context("Failed to deserialize stabble stable swap")?
            .0;
        if cfg.arbitrage.is_c2_only() && !pool.has_wsol() {
            return Ok(None);
        }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new { Some(pool.copy_keys()) } else { None },
            pool_type: PoolType::StabbleStableSwap,
        });
        pool_type_and_pubkey.insert(pubkey, PoolType::StabbleStableSwap);
        pools.insert(pubkey, pool);
    }
    Ok(pool_to_calculator)
}

pub fn process_stabble_weighted_swap(
    pubkey: Pubkey,
    data: &[u8],
    pools: &mut HashMap<Pubkey, dex::stabble_weighted_swap::accounts::PoolWeightedSwap>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&dex::stabble_weighted_swap::accounts::PoolWeightedSwap::DISCRIMINATOR) {
        let pool = PoolWeightedSwap::try_deserialize(data).context("Failed to deserialize stabble weighted swap")?;
        if cfg.arbitrage.is_c2_only() && !pool.has_wsol() {
            return Ok(None);
        }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new { Some(pool.copy_keys()) } else { None },
            pool_type: PoolType::StabbleWeightedSwap,
        });
        pool_type_and_pubkey.insert(pubkey, PoolType::StabbleWeightedSwap);
        pools.insert(pubkey, pool);
    }
    Ok(pool_to_calculator)
}

pub fn spawn_stabble_weighted_swap(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<StabbleWeightedSwapGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::stabble_weighted_swap::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::stabble_weighted_swap::ID).await
        };
        let mut stabble_weighted_swap_result = StabbleWeightedSwapGPAResult {
            pool_type_and_pubkey: Default::default(),
            pools: Default::default(),
        };
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_stabble_weighted_swap(
                pubkey,
                account.data.as_slice(),
                &mut stabble_weighted_swap_result.pools,
                &mut stabble_weighted_swap_result.pool_type_and_pubkey,
            );
            if let Err(e) = res {
                log::error!("Failed to process stabble weighted swap: {:?}", e);
            }
        });
        stabble_weighted_swap_result
    })
}

pub fn spawn_stabble_stable_swap(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<StabbleStableSwapGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::stabble_stable_swap::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::stabble_stable_swap::ID).await
        };
        let mut stabble_stable_swap_result = StabbleStableSwapGPAResult {
            pool_type_and_pubkey: Default::default(),
            pools: Default::default(),
        };
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_stabble_stable_swap(
                pubkey,
                account.data.as_slice(),
                &mut stabble_stable_swap_result.pools,
                &mut stabble_stable_swap_result.pool_type_and_pubkey,
            );
            if let Err(e) = res {
                log::error!("Failed to process stabble stable swap: {:?}", e);
            }
        });
        stabble_stable_swap_result
    })
}
