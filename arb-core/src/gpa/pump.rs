use ahash::AHashMap as HashMap;

use config::CONFIG as cfg;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use utils::pool::{Pool, PoolType};

use crate::gpa::PoolToCalculator;

#[derive(Debug)]
pub struct PumpAmmGPAResult {
    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub pools: HashMap<Pubkey, dex::pump_amm::Pool>,
    pub config: HashMap<Pubkey, dex::pump_amm::GlobalConfig>,
    pub fee_config: HashMap<Pubkey, dex::pump_amm::FeeConfig>,
}

pub fn process_pump_amm(
    pubkey: Pubkey,
    data: &[u8],
    pump_amm_map: &mut HashMap<Pubkey, dex::pump_amm::Pool>,
    pump_amm_config_map: &mut HashMap<Pubkey, dex::pump_amm::GlobalConfig>,
    pump_amm_fee_config_map: &mut HashMap<Pubkey, dex::pump_amm::FeeConfig>,
    pool_type_and_pubkey: &mut HashMap<Pubkey, PoolType>,
) -> anyhow::Result<Option<PoolToCalculator>> {
    let mut pool_to_calculator: Option<PoolToCalculator> = None;
    if data.starts_with(&dex::pump_amm::POOL_DISCRIMINATOR) {
        let pool = dex::pump_amm::Pool::deserialize(data);
        let pool = pool.map_err(|e| anyhow::format_err!("process_pump_amm: {:?}", e))?;
        if cfg.arbitrage.is_c2_only() && !pool.has_wsol() {
            return Ok(None);
        }
        let is_new = !pool_type_and_pubkey.contains_key(&pubkey);
        pool_to_calculator = Some(PoolToCalculator {
            is_new,
            pubkey,
            keys: if is_new { Some(pool.copy_keys()) } else { None },
            pool_type: PoolType::PumpAmm,
        });
        pump_amm_map.insert(pubkey, pool);
        pool_type_and_pubkey.insert(pubkey, PoolType::PumpAmm);
    } else if data.starts_with(&dex::pump_amm::GLOBAL_CONFIG_DISCRIMINATOR) {
        let config = dex::pump_amm::GlobalConfig::deserialize(data);
        let config = config.map_err(|e| anyhow::format_err!("process_pump_amm: {:?}", e))?;
        pump_amm_config_map.insert(pubkey, config);
    } else if data.starts_with(&dex::pump_amm::FEE_DISCRIMINATOR) {
        let fee_config = dex::pump_amm::FeeConfig::deserialize(data);
        let fee_config = fee_config.map_err(|e| anyhow::format_err!("process_pump_amm: {:?}", e))?;
        pump_amm_fee_config_map.insert(pubkey, fee_config);
    }
    Ok(pool_to_calculator)
}

pub fn spawn_pump_amm(
    url: String,
    cfg_opt: Option<RpcProgramAccountsConfig>,
) -> tokio::task::JoinHandle<PumpAmmGPAResult> {
    tokio::spawn(async move {
        let result = if let Some(rpc_cfg) = cfg_opt {
            crate::gpa::fetch_program_accounts_with_config(&url, &dex::pump_amm::ID, rpc_cfg).await
        } else {
            crate::gpa::fetch_program_accounts(&url, &dex::pump_amm::ID).await
        };
        let mut pump_amm_result = PumpAmmGPAResult {
            pool_type_and_pubkey: Default::default(),
            pools: Default::default(),
            config: Default::default(),
            fee_config: Default::default(),
        };
        result.into_iter().for_each(|(pubkey, account)| {
            let res = process_pump_amm(
                pubkey,
                account.data.as_slice(),
                &mut pump_amm_result.pools,
                &mut pump_amm_result.config,
                &mut pump_amm_result.fee_config,
                &mut pump_amm_result.pool_type_and_pubkey,
            );
            if let Err(e) = res {
                log::warn!("Failed to process pump amm: {:?}", e);
            }
        });
        pump_amm_result
    })
}
