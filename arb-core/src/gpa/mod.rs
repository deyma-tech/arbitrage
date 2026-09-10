use ahash::{AHashMap as HashMap, AHashSet};
use crossbeam_channel::Sender;
use futures::StreamExt;
use log::{debug, error, info, warn};
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use solana_sdk::clock::Slot;
use solana_sdk::commitment_config::CommitmentConfig;
use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::select;
use utils::deserialize::{Message, MessagesV2};
use utils::pool::PoolType;
// use zeromq::{Socket, SocketRecv};

use crate::calculator::CalculatorEnum;

pub mod fusion;
//pub mod goose_gama;
//pub mod lifinity;
pub mod meteora;
pub mod orca;
pub mod pump;
pub mod raydium;
pub mod saros;
pub mod stabble;

#[derive(Debug, Clone)]
pub struct PoolToCalculator {
    pub is_new: bool,
    pub pubkey: Pubkey,
    pub keys: Option<Vec<Pubkey>>,
    pub pool_type: PoolType,
}

pub fn process_token_account(pubkey: Pubkey, data: &[u8], ta_amount_map: &mut HashMap<Pubkey, u64>) {
    let data = if data.len() >= 165 { &data[..165] } else { data };
    let account = spl_token::state::Account::unpack(data);
    if let Ok(account) = account {
        ta_amount_map.insert(pubkey, account.amount);
    }
}

pub fn process_token_2022_account(
    pubkey: Pubkey,
    data: &[u8],
    ta_amount_map: &mut HashMap<Pubkey, u64>,
    token22_set: &mut AHashSet<Pubkey>,
    data_set: &mut AHashSet<Vec<u8>>,
) {
    if data.len() >= 165 {
        let account = spl_token::state::Account::unpack(&data[..165]);
        if let Ok(account) = account {
            //if ALLOWED_TOKEN_2022.contains(&account.mint) {
            ta_amount_map.insert(pubkey, account.amount);
            if !token22_set.contains(&account.mint) {
                data_set.insert(data.to_vec());
            } else {
                token22_set.insert(account.mint);
            }
            //}
        }
    }
}

#[inline]
fn new_rpc_client(url: &str) -> RpcClient {
    RpcClient::new_with_timeout_and_commitment(url.to_string(), Duration::from_secs(600), CommitmentConfig::processed())
}

async fn fetch_program_accounts(url: &str, program_id: &Pubkey) -> Vec<(Pubkey, Account)> {
    let client = new_rpc_client(url);
    match client.get_program_accounts(program_id).await {
        Ok(accounts) => accounts,
        Err(e) => {
            error!("GPA: Failed to fetch program accounts for {:?}: {:?}", program_id, e);
            vec![]
        }
    }
}

/// Visit account data without asking the RPC to materialize one enormous
/// full-data GPA response. The first request returns only matching pubkeys;
/// account data is then hydrated through bounded getMultipleAccounts batches.
/// This is especially important for large DLMM programs.
pub async fn for_each_program_account_by_discriminators<F>(
    url: &str,
    program_id: &Pubkey,
    discriminators: &[(Option<u64>, [u8; 8])],
    mut process: F,
) -> anyhow::Result<()>
where
    F: FnMut(Pubkey, Account) -> anyhow::Result<()> + Send,
{
    let client = new_rpc_client(url);

    for (data_size, discriminator) in discriminators {
        let mut filters = Vec::with_capacity(2);
        if let Some(data_size) = data_size {
            filters.push(RpcFilterType::DataSize(*data_size));
        }
        filters.push(RpcFilterType::Memcmp(Memcmp::new(
            0,
            MemcmpEncodedBytes::Bytes(discriminator.to_vec()),
        )));
        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: Some(solana_account_decoder::UiDataSliceConfig { offset: 0, length: 0 }),
                commitment: Some(CommitmentConfig::processed()),
                min_context_slot: None,
            },
            with_context: None,
            sort_results: None,
        };

        info!(
            "GPA index: fetching pubkeys for program {:?}, discriminator {:?}",
            program_id, discriminator
        );
        let indexed = match client.get_program_accounts_with_config(program_id, config).await {
            Ok(accounts) => accounts,
            Err(err) => {
                error!(
                    "GPA index failed: program {:?}, discriminator {:?}: {:?}",
                    program_id, discriminator, err
                );
                return Err(anyhow::anyhow!("GPA index failed: {:?}", err));
            }
        };
        let pubkeys: Vec<Pubkey> = indexed.into_iter().map(|(pubkey, _)| pubkey).collect();
        info!(
            "GPA index received: program {:?}, discriminator {:?}, accounts={}",
            program_id,
            discriminator,
            pubkeys.len()
        );

        let client_ref = &client;
        let batches: Vec<Vec<Pubkey>> = pubkeys.chunks(100).map(|chunk| chunk.to_vec()).collect();
        let mut hydration = futures::stream::iter(batches.into_iter().map(|batch_pubkeys| {
            let client_ref = client_ref;
            async move {
                let mut last_error = None;
                for attempt in 0..5 {
                    match client_ref.get_multiple_accounts(&batch_pubkeys).await {
                        Ok(accounts) => return (batch_pubkeys, Ok(accounts)),
                        Err(err) => {
                            last_error = Some(err);
                            if attempt < 4 {
                                let delay_ms = 250u64 * (1u64 << attempt);
                                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            }
                        }
                    }
                }
                (
                    batch_pubkeys,
                    Err(last_error.expect("hydration attempts must record an error")),
                )
            }
        }))
        // Helius free-tier rate limits apply across both PumpSwap and DLMM
        // tasks. Keep a small bounded fan-out and retry transient 429s.
        .buffer_unordered(3);
        let mut hydrated = 0usize;

        while let Some((batch_pubkeys, accounts)) = hydration.next().await {
            let accounts = match accounts {
                Ok(accounts) => accounts,
                Err(err) => {
                    error!(
                        "GPA hydrate failed: program {:?}, discriminator {:?}, batch_size={}: {:?}",
                        program_id,
                        discriminator,
                        batch_pubkeys.len(),
                        err
                    );
                    return Err(anyhow::anyhow!("GPA hydrate failed: {:?}", err));
                }
            };
            hydrated += batch_pubkeys.len();
            if hydrated == batch_pubkeys.len() || hydrated / 10_000 != (hydrated - batch_pubkeys.len()) / 10_000 {
                info!(
                    "GPA hydrate progress: program {:?}, discriminator {:?}, accounts={}/{}",
                    program_id,
                    discriminator,
                    hydrated,
                    pubkeys.len()
                );
            }
            for (pubkey, account) in batch_pubkeys.into_iter().zip(accounts) {
                if let Some(account) = account {
                    process(pubkey, account)?;
                }
            }
        }
    }

    Ok(())
}

/// Compatibility wrapper for callers that need an owned account vector.
pub async fn fetch_program_accounts_by_discriminators(
    url: &str,
    program_id: &Pubkey,
    discriminators: &[(Option<u64>, [u8; 8])],
) -> Vec<(Pubkey, Account)> {
    let mut all_accounts = Vec::new();
    let _ = for_each_program_account_by_discriminators::<_>(url, program_id, discriminators, |pubkey, account| {
        all_accounts.push((pubkey, account));
        Ok(())
    })
    .await;
    all_accounts
}

async fn fetch_program_accounts_with_config(
    url: &str,
    program_id: &Pubkey,
    config: RpcProgramAccountsConfig,
) -> Vec<(Pubkey, Account)> {
    let client = new_rpc_client(url);
    match client.get_program_accounts_with_config(program_id, config).await {
        Ok(accounts) => accounts,
        Err(e) => {
            error!(
                "GPA: Failed to fetch program accounts with config for {:?}: {:?}",
                program_id, e
            );
            vec![]
        }
    }
}

#[inline(always)]
async fn sleep() {
    tokio::time::sleep(Duration::from_secs(5)).await;
}

pub async fn get_program_accounts(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
    discriminators: Vec<[u8; 8]>,
) -> anyhow::Result<Vec<(Pubkey, Account)>> {
    let mut all_accounts = Vec::new();

    for discriminator in discriminators {
        let filters = vec![RpcFilterType::Memcmp(Memcmp::new(
            0, // Discriminator is at the start of the account data
            MemcmpEncodedBytes::Bytes(discriminator.to_vec()),
        ))];
        let config = RpcProgramAccountsConfig {
            filters: Some(filters.clone()),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                commitment: Some(CommitmentConfig::processed()),
                min_context_slot: None,
            },
            with_context: None,
            sort_results: None,
        };
        info!(
            "GPA snapshot: fetching full accounts for program {:?}, discriminator {:?}",
            program_id, discriminator
        );
        let accounts = rpc_client
            .get_program_accounts_with_config(program_id, config)
            .await
            .map_err(|err| anyhow::anyhow!("GPA request failed for {:?}: {:?}", program_id, err))?;
        info!(
            "GPA snapshot received: program {:?}, discriminator {:?}, accounts={}",
            program_id,
            discriminator,
            accounts.len()
        );
        all_accounts.extend(accounts);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(all_accounts)
}

type GpaSnapshot = (
    orca::OrcaSwapV2GPAResult,
    saros::SarosAmmGPAResult,
    raydium::RaydiumCpmmGPAResult,
    stabble::StabbleWeightedSwapGPAResult,
    stabble::StabbleStableSwapGPAResult,
    pump::PumpAmmGPAResult,
    orca::OrcaGPAResult,
    raydium::RaydiumAmmGPAResult,
    raydium::RaydiumClmmGPAResult,
    meteora::MeteoraDlmmGPAResult,
    meteora::MeteoraDammV2GPAResult,
    fusion::FusionAmmGPAResult,
    saros::SarosDlmmGPAResult,
);

/// Chainstack-compatible initial snapshot. The two mandatory DEXs are the
/// only ones hydrated and subscribed; all other DEX result sets stay empty.
async fn get_chainstack_gpa(url: &str) -> anyhow::Result<GpaSnapshot> {
    // Keep the original warmup behavior: the two mandatory DEX snapshots run
    // concurrently, while the live streams are already active. This is the
    // path that previously reached the opportunity loop successfully.
    let pump_task = pump::spawn_pump_amm(url.to_string(), None);
    let dlmm_task = meteora::spawn_meteora_dlmm(url.to_string(), None);
    let (pump, dlmm) = tokio::join!(pump_task, dlmm_task);
    let pump = pump.map_err(|err| anyhow::anyhow!("PumpSwap GPA task failed: {:?}", err))?;
    let dlmm = dlmm.map_err(|err| anyhow::anyhow!("Meteora DLMM GPA task failed: {:?}", err))?;

    if pump.pools.is_empty() {
        anyhow::bail!("Chainstack initial snapshot returned no PumpSwap pools");
    }
    if dlmm.pools.is_empty() {
        anyhow::bail!("Chainstack initial snapshot returned no Meteora DLMM pools");
    }
    info!(
        "Chainstack initial snapshot: PumpSwap pools={}, Meteora DLMM pools={}, bin-array groups={}",
        pump.pools.len(),
        dlmm.pools.len(),
        dlmm.bin_arrays.len()
    );

    Ok((
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        pump,
        Default::default(),
        Default::default(),
        Default::default(),
        dlmm,
        Default::default(),
        Default::default(),
        Default::default(),
    ))
}

pub async fn get_all_gpa(
    url: &str,
) -> Result<
    (
        //goose_gama::GooseGammaGPAResult,
        orca::OrcaSwapV2GPAResult,
        saros::SarosAmmGPAResult,
        raydium::RaydiumCpmmGPAResult,
        //lifinity::LifinityGPAResult,
        stabble::StabbleWeightedSwapGPAResult,
        stabble::StabbleStableSwapGPAResult,
        pump::PumpAmmGPAResult,
        orca::OrcaGPAResult,
        raydium::RaydiumAmmGPAResult,
        raydium::RaydiumClmmGPAResult,
        meteora::MeteoraDlmmGPAResult,
        meteora::MeteoraDammV2GPAResult,
        fusion::FusionAmmGPAResult,
        saros::SarosDlmmGPAResult,
    ),
    tokio::task::JoinError,
> {
    // let handle_goose_gamma = goose_gama::spawn_goose_gamma(url.to_string(), None);
    // sleep().await;

    let handle_orca_swap_v2 = orca::spawn_orca_swap_v2(url.to_string(), None);
    sleep().await;

    let handle_saros_amm = saros::spawn_saros_amm(url.to_string(), None);
    sleep().await;

    let handle_raydium_cpmm = raydium::spawn_raydium_cpmm(url.to_string(), None);
    sleep().await;

    // let handle_lifinity = lifinity::spawn_lifinity(url.to_string(), None);
    //sleep().await;

    let handle_stabble_weighted_swap = stabble::spawn_stabble_weighted_swap(url.to_string(), None);
    sleep().await;

    let handle_stabble_stable_swap = stabble::spawn_stabble_stable_swap(url.to_string(), None);
    sleep().await;

    let handle_pump_amm: tokio::task::JoinHandle<pump::PumpAmmGPAResult> = pump::spawn_pump_amm(url.to_string(), None);
    sleep().await;

    let handle_orca: tokio::task::JoinHandle<orca::OrcaGPAResult> = orca::spawn_orca(url.to_string(), None);
    sleep().await;

    let handle_raydium_amm = raydium::spawn_raydium_amm(url.to_string(), None);
    sleep().await;

    let handle_raydium_clmm: tokio::task::JoinHandle<raydium::RaydiumClmmGPAResult> =
        raydium::spawn_raydium_clmm(url.to_string());
    sleep().await;

    let handle_meteora_dlmm: tokio::task::JoinHandle<meteora::MeteoraDlmmGPAResult> =
        meteora::spawn_meteora_dlmm(url.to_string(), None);
    sleep().await;

    let handle_meteora_damm_v2: tokio::task::JoinHandle<meteora::MeteoraDammV2GPAResult> =
        meteora::spawn_meteora_damm_v2(url.to_string(), None);
    sleep().await;

    let handle_fusion: tokio::task::JoinHandle<fusion::FusionAmmGPAResult> =
        fusion::spawn_fusion(url.to_string(), None);
    sleep().await;

    let handle_saros: tokio::task::JoinHandle<saros::SarosDlmmGPAResult> =
        saros::spawn_saros_dlmm(url.to_string(), None);

    sleep().await;

    let result = tokio::join!(
        //handle_goose_gamma,
        handle_orca_swap_v2,
        handle_saros_amm,
        handle_raydium_cpmm,
        //handle_lifinity,
        handle_stabble_weighted_swap,
        handle_stabble_stable_swap,
        handle_pump_amm,
        handle_orca,
        handle_raydium_amm,
        handle_raydium_clmm,
        handle_meteora_dlmm,
        handle_meteora_damm_v2,
        handle_fusion,
        handle_saros,
    );

    Ok((
        result.0?, result.1?, result.2?, result.3?, result.4?, result.5?, result.6?, result.7?, result.8?, result.9?,
        result.10?, result.11?, result.12?,
    ))
}

#[derive(Debug, Clone, Default)]
pub struct GPAResult {
    pub pump_amm_map: HashMap<Pubkey, dex::pump_amm::Pool>,
    pub pump_amm_config: HashMap<Pubkey, dex::pump_amm::GlobalConfig>,
    pub pump_amm_fee_config: HashMap<Pubkey, dex::pump_amm::FeeConfig>,
    pub pump_amm_pool_to_mint_supply: HashMap<Pubkey, u128>,

    pub orca_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::orca::OrcaTickArray)>>,
    pub orca_pools: HashMap<Pubkey, dex::orca::OrcaWhirlpool>,
    pub orca_oracles: HashMap<Pubkey, Pubkey>,
    pub orca_pool_to_oracle: HashMap<Pubkey, dex::orca::Oracle>,

    pub raydium_clmm_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::raydium_clmm::TickArrayState)>>,
    pub raydium_clmm_pools: HashMap<Pubkey, dex::raydium_clmm::PoolState>,
    pub raydium_clmm_config: HashMap<Pubkey, dex::raydium_clmm::AmmConfig>,
    pub raydium_bitmap_extensions: HashMap<Pubkey, (Pubkey, dex::raydium_clmm::TickArrayBitmapExtension)>,

    pub meteora_dlmm_pools: HashMap<Pubkey, dex::meteora_dlmm::LbPair>,
    pub meteora_dlmm_bin_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::meteora_dlmm::BinArray)>>,
    pub meteora_dlmm_bitmap_extensions: HashMap<Pubkey, (Pubkey, dex::meteora_dlmm::BinArrayBitmapExtension)>,

    pub raydium_amm_v4_map: HashMap<Pubkey, dex::raydium_amm::AmmInfo>,
    pub ta_amount_map: HashMap<Pubkey, u64>,

    pub pool_type_and_pubkey: HashMap<Pubkey, PoolType>,
    pub stabble_stable_swap_map: HashMap<Pubkey, dex::stabble_stable_swap::PoolSwap>,
    pub stabble_weighted_swap_map: HashMap<Pubkey, dex::stabble_weighted_swap::accounts::PoolWeightedSwap>,

    // pub lifinity_amm_map: HashMap<Pubkey, dex::lifinity::Amm>,
    // pub lifinity_authorities: HashMap<Pubkey, Pubkey>,
    // pub pyth_map: HashMap<Pubkey, dex::lifinity::pc::Price>,
    pub raydium_cpmm_pools: HashMap<Pubkey, dex::raydium_cpmm::PoolState>,
    pub raydium_cpmm_configs: HashMap<Pubkey, dex::raydium_cpmm::AmmConfig>,

    pub orca_swap_v2_markets: HashMap<Pubkey, dex::orca_swap_v2::accounts::Market>,
    pub orca_swap_v2_authorities: HashMap<Pubkey, Pubkey>,

    pub saros_amm_markets: HashMap<Pubkey, dex::saros_amm::accounts::Market>,
    pub saros_amm_authorities: HashMap<Pubkey, Pubkey>,

    // pub goose_gamma_pools: HashMap<Pubkey, dex::goose_gamma::accounts::PoolState>,
    // pub goose_gamma_configs: HashMap<Pubkey, dex::goose_gamma::accounts::AmmConfig>,
    // pub goose_gamma_observation_states: HashMap<Pubkey, dex::goose_gamma::accounts::ObservationState>,

    //pub obric_v2_pools: HashMap<Pubkey, dex::obric_v2::SSTradingPair>,
    pub meteora_damm_v2_pools: HashMap<Pubkey, dex::meteora_damm_v2::PoolState>,

    pub fusion_amm_pools: HashMap<Pubkey, dex::fusion_amm::FusionPoolWhirlpool>,
    pub fusion_amm_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::orca::OrcaTickArray)>>,

    pub saros_dlmm_pools: HashMap<Pubkey, dex::saros_dlmm::Pair>,
    pub saros_dlmm_bin_arrays: HashMap<Pubkey, BTreeMap<u32, (Pubkey, dex::saros_dlmm::BinArray)>>,

    pub token_mints: HashSet<Pubkey>,
}

// impl GPAResult {
//     pub fn to_calculators(&self, slot: u64) -> Vec<CalculatorEnum> {
//         let mut calculators = Vec::with_capacity(self.pool_type_and_pubkey.len());
//         for (pool_pubkey, pool_type) in self.pool_type_and_pubkey.iter() {
//             let calculator = self.get_partial_data(pool_type, pool_pubkey, slot);
//             if let Some(calculator) = calculator {
//                 calculators.push(calculator);
//             }
//         }
//         calculators
//     }
// }

#[derive(Debug)]
struct ChainstackAccountUpdate {
    pubkey: Pubkey,
    owner: Pubkey,
    data: Vec<u8>,
}

async fn stream_chainstack_accounts(
    ws_url: String,
    tx: tokio::sync::mpsc::UnboundedSender<ChainstackAccountUpdate>,
    tx_updates: Sender<MessagesV2>,
) {
    loop {
        let client = match solana_client::nonblocking::pubsub_client::PubsubClient::new(&ws_url).await {
            Ok(client) => client,
            Err(err) => {
                error!("Chainstack WebSocket connection failed: {:?}", err);
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let config = || RpcProgramAccountsConfig {
            // One broad subscription per mandatory DEX avoids consuming the
            // provider's filter quota. The discriminator is parsed locally.
            filters: None,
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                commitment: Some(CommitmentConfig::processed()),
                min_context_slot: None,
            },
            with_context: Some(true),
            sort_results: None,
        };

        let (mut pump_stream, _pump_unsubscribe) =
            match client.program_subscribe(&dex::pump_amm::ID, Some(config())).await {
                Ok(subscription) => subscription,
                Err(err) => {
                    error!("Chainstack PumpSwap subscription failed: {:?}", err);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };
        let (mut dlmm_stream, _dlmm_unsubscribe) =
            match client.program_subscribe(&dex::meteora_dlmm::ID, Some(config())).await {
                Ok(subscription) => subscription,
                Err(err) => {
                    error!("Chainstack Meteora DLMM subscription failed: {:?}", err);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

        info!("Chainstack account streams active: PumpSwap + Meteora DLMM");
        loop {
            tokio::select! {
                Some(response) = pump_stream.next() => {
                    if let Some(data) = response.value.account.data.decode() {
                        let pubkey = match Pubkey::from_str(&response.value.pubkey) {
                            Ok(pubkey) => pubkey,
                            Err(err) => { warn!("Invalid PumpSwap account pubkey: {:?}", err); continue; }
                        };
                        let slot = response.context.slot;
                        if tx_updates.send(MessagesV2 { message: vec![Message { pubkey, owner: dex::pump_amm::ID, data: data.clone() }], slot }).is_err() { return; }
                        let _ = tx.send(ChainstackAccountUpdate { pubkey, owner: dex::pump_amm::ID, data });
                    }
                }
                Some(response) = dlmm_stream.next() => {
                    if let Some(data) = response.value.account.data.decode() {
                        let pubkey = match Pubkey::from_str(&response.value.pubkey) {
                            Ok(pubkey) => pubkey,
                            Err(err) => { warn!("Invalid Meteora DLMM account pubkey: {:?}", err); continue; }
                        };
                        let slot = response.context.slot;
                        if tx_updates.send(MessagesV2 { message: vec![Message { pubkey, owner: dex::meteora_dlmm::ID, data: data.clone() }], slot }).is_err() { return; }
                        let _ = tx.send(ChainstackAccountUpdate { pubkey, owner: dex::meteora_dlmm::ID, data });
                    }
                }
                else => {
                    warn!("Chainstack account stream closed; reconnecting");
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub async fn sync_gpa(url: &str, ws_url: &str, tx_updates: Sender<MessagesV2>) -> anyhow::Result<GPAResult> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<(
        //goose_gama::GooseGammaGPAResult,
        orca::OrcaSwapV2GPAResult,
        saros::SarosAmmGPAResult,
        raydium::RaydiumCpmmGPAResult,
        //lifinity::LifinityGPAResult,
        stabble::StabbleWeightedSwapGPAResult,
        stabble::StabbleStableSwapGPAResult,
        pump::PumpAmmGPAResult,
        orca::OrcaGPAResult,
        raydium::RaydiumAmmGPAResult,
        raydium::RaydiumClmmGPAResult,
        meteora::MeteoraDlmmGPAResult,
        meteora::MeteoraDammV2GPAResult,
        fusion::FusionAmmGPAResult,
        saros::SarosDlmmGPAResult,
    )>(1);

    tokio::spawn({
        let url = url.to_string();
        async move {
            info!("GPA start");
            loop {
                match get_chainstack_gpa(&url).await {
                    Ok((
                        //result_goose_gamma,
                        result_orca_swap_v2,
                        result_saros_amm,
                        result_raydium_cpmm,
                        //result_lifinity,
                        result_stabble_weighted_swap,
                        result_stabble_stable_swap,
                        result_pump_amm,
                        result_orca,
                        result_raydium_amm,
                        result_raydium_clmm,
                        result_meteroa_dlmm,
                        result_meteora_damm_v2,
                        result_fusion,
                        result_saros,
                    )) => {
                        info!("GPA Ok");
                        if let Err(e) = tx
                            .send((
                                //result_goose_gamma,
                                result_orca_swap_v2,
                                result_saros_amm,
                                result_raydium_cpmm,
                                //result_lifinity,
                                result_stabble_weighted_swap,
                                result_stabble_stable_swap,
                                result_pump_amm,
                                result_orca,
                                result_raydium_amm,
                                result_raydium_clmm,
                                result_meteroa_dlmm,
                                result_meteora_damm_v2,
                                result_fusion,
                                result_saros,
                            ))
                            .await
                        {
                            error!("Failed to send GPA result: {:?}", e);
                        }
                        break;
                    }
                    Err(err) => {
                        error!("GPA initial snapshot unavailable: {:?}; retrying in 10s", err);
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                }
            }
        }
    });
    // info!("connecting to zeromq");
    // let mut socket = zeromq::SubSocket::new();
    // socket
    //     .connect("ipc:///tmp/accounts_zmq_v2.sock")
    //     .await
    //     .or_panic("FailedToConnect");
    // socket.subscribe("").await.or_panic("FailedToSubscribe");
    // info!("connected to zeromq");

    // orca
    let mut orca_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::orca::OrcaTickArray)>> = HashMap::new();
    let mut orca_pools: HashMap<Pubkey, dex::orca::OrcaWhirlpool> = HashMap::new();
    let mut orca_oracles: HashMap<Pubkey, Pubkey> = HashMap::new();
    let mut orca_pool_to_oracle: HashMap<Pubkey, dex::orca::Oracle> = HashMap::new();

    // raydium clmm
    let mut raydium_clmm_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::raydium_clmm::TickArrayState)>> =
        HashMap::new();
    let mut raydium_clmm_pools = HashMap::new();
    let mut raydium_clmm_config = HashMap::new();
    let mut raydium_bitmap_extensions = HashMap::new();

    // meteora dlmm
    let mut meteora_dlmm_pools = HashMap::new();
    let mut meteora_dlmm_bin_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::meteora_dlmm::BinArray)>> =
        HashMap::new();
    let mut meteora_dlmm_bitmap_extensions = HashMap::new();

    // raydium amm info
    let mut raydium_amm_v4_map = HashMap::new();
    let mut ta_amount_map = HashMap::new();

    let mut pool_type_and_pubkey: HashMap<Pubkey, PoolType> = HashMap::new();

    let mut pump_amm_map: HashMap<Pubkey, dex::pump_amm::Pool> = HashMap::new();
    let mut pump_amm_config: HashMap<Pubkey, dex::pump_amm::GlobalConfig> = HashMap::new();
    let mut pump_amm_fee_config: HashMap<Pubkey, dex::pump_amm::FeeConfig> = HashMap::new();

    let mut stabble_stable_swap_map: HashMap<Pubkey, dex::stabble_stable_swap::PoolSwap> = HashMap::new();

    let mut stabble_weighted_swap_map: HashMap<Pubkey, dex::stabble_weighted_swap::accounts::PoolWeightedSwap> =
        HashMap::new();

    // let mut lifinity_amm_map: HashMap<Pubkey, dex::lifinity::Amm> = HashMap::new();
    // let mut lifinity_authorities: HashMap<Pubkey, Pubkey> = HashMap::new();
    // let pyth_map: HashMap<Pubkey, dex::lifinity::pc::Price> = HashMap::new();

    let mut raydium_cpmm_pools: HashMap<Pubkey, dex::raydium_cpmm::PoolState> = HashMap::new();
    let mut raydium_cpmm_configs: HashMap<Pubkey, dex::raydium_cpmm::AmmConfig> = HashMap::new();

    let mut orca_swap_v2_markets: HashMap<Pubkey, dex::orca_swap_v2::accounts::Market> = HashMap::new();
    let mut orca_swap_v2_authorities: HashMap<Pubkey, Pubkey> = HashMap::new();

    let mut saros_amm_markets: HashMap<Pubkey, dex::saros_amm::accounts::Market> = HashMap::new();
    let mut saros_amm_authorities: HashMap<Pubkey, Pubkey> = HashMap::new();

    // let mut goose_gamma_pools: HashMap<Pubkey, dex::goose_gamma::accounts::PoolState> = HashMap::new();
    // let mut goose_gamma_configs: HashMap<Pubkey, dex::goose_gamma::accounts::AmmConfig> = HashMap::new();
    // let mut goose_gamma_observation_states: HashMap<Pubkey, dex::goose_gamma::accounts::ObservationState> =
    //     HashMap::new();

    //let mut obric_v2_pools: HashMap<Pubkey, dex::obric_v2::SSTradingPair> = HashMap::new();
    let mut meteora_damm_v2_pools: HashMap<Pubkey, dex::meteora_damm_v2::PoolState> = HashMap::new();

    let mut fusion_amm_pools: HashMap<Pubkey, dex::fusion_amm::FusionPoolWhirlpool> = HashMap::new();
    let mut fusion_amm_tick_arrays: HashMap<Pubkey, BTreeMap<i32, (Pubkey, dex::orca::OrcaTickArray)>> = HashMap::new();

    let mut saros_dlmm_pools: HashMap<Pubkey, dex::saros_dlmm::Pair> = HashMap::new();
    let mut saros_dlmm_bin_arrays: HashMap<Pubkey, BTreeMap<u32, (Pubkey, dex::saros_dlmm::BinArray)>> = HashMap::new();

    let token_mints: HashSet<Pubkey> = HashSet::new();

    let (tx_socket, mut rx_socket) = tokio::sync::mpsc::unbounded_channel::<ChainstackAccountUpdate>();

    // Start the live streams while the initial GPA is loading. This preserves
    // the previously working warmup behavior: account updates can arrive and
    // queue while the HTTP snapshot is slow or retrying.
    tokio::spawn(stream_chainstack_accounts(
        ws_url.to_string(),
        tx_socket.clone(),
        tx_updates.clone(),
    ));

    loop {
        select! {
            // GPA
            Some((gpa_orca_swap_v2, gpa_saros_amm, mut gpa_raydium_cpmm, gpa_stabble_weighted_swap, gpa_stabble_stable_swap, mut gpa_pump_amm, mut gpa_orca, mut gpa_raydium_amm, mut gpa_raydium_clmm, mut gpa_meteroa_dlmm, gpa_meteora_damm_v2, mut gpa_fusion, mut gpa_saros)) = rx.recv() => {
                info!("GPA received");

                //gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_goose_gamma.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_orca_swap_v2.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_saros_amm.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_orca.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_raydium_clmm.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_meteroa_dlmm.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_raydium_cpmm.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_stabble_weighted_swap.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_stabble_stable_swap.pool_type_and_pubkey);
                //gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_lifinity.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_pump_amm.pool_type_and_pubkey);
                //gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_obric_v2.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_meteora_damm_v2.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_fusion.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(gpa_saros.pool_type_and_pubkey);
                gpa_raydium_amm.pool_type_and_pubkey.extend(pool_type_and_pubkey);
                pool_type_and_pubkey = gpa_raydium_amm.pool_type_and_pubkey;

                debug!("Ticks");
                let start = Instant::now();

                for (pubkey, map) in orca_tick_arrays.into_iter() {
                    for (index, tick_array) in map.into_iter() {
                        gpa_orca.tick_arrays.entry(pubkey).and_modify(|e| {
                            e.insert(index, tick_array);
                        }).or_insert_with(|| {
                            let mut map_to_update = BTreeMap::new();
                            map_to_update.insert(index, tick_array);
                            map_to_update
                        });
                    }
                }
                orca_tick_arrays = gpa_orca.tick_arrays;
                orca_oracles = gpa_orca.oracles;

                // TODO: check, bacause oracle could be updated
                orca_pool_to_oracle.extend(gpa_orca.pool_to_oracle);

                for (pubkey, map) in raydium_clmm_tick_arrays.into_iter() {
                    for (index, tick_array) in map.into_iter() {
                        gpa_raydium_clmm.tick_arrays.entry(pubkey).and_modify(|e| {
                            e.insert(index, tick_array.clone());
                        }).or_insert_with(|| {
                            let mut map_to_update = BTreeMap::new();
                            map_to_update.insert(index, tick_array);
                            map_to_update
                        });
                    }
                }
                raydium_clmm_tick_arrays = gpa_raydium_clmm.tick_arrays;

                for (pubkey, map) in meteora_dlmm_bin_arrays.into_iter() {
                    for (index, bin_array) in map.into_iter() {
                        gpa_meteroa_dlmm.bin_arrays.entry(pubkey).and_modify(|e| {
                            e.insert(index, bin_array);
                        }).or_insert_with(|| {
                            let mut map_to_update = BTreeMap::new();
                            map_to_update.insert(index, bin_array);
                            map_to_update
                        });
                    }
                }
                meteora_dlmm_bin_arrays = gpa_meteroa_dlmm.bin_arrays;

                stabble_stable_swap_map.extend(gpa_stabble_stable_swap.pools);

                stabble_weighted_swap_map.extend(gpa_stabble_weighted_swap.pools);

                // lifinity_amm_map.extend(gpa_lifinity.amms);
                // lifinity_authorities.extend(gpa_lifinity.authorities);

                orca_swap_v2_markets.extend(gpa_orca_swap_v2.markets);
                orca_swap_v2_authorities.extend(gpa_orca_swap_v2.authorities);

                saros_amm_markets.extend(gpa_saros_amm.markets);
                saros_amm_authorities.extend(gpa_saros_amm.authorities);

                meteora_damm_v2_pools.extend(gpa_meteora_damm_v2.pools);

                for (pubkey, map) in fusion_amm_tick_arrays.into_iter() {
                    for (index, tick_array) in map.into_iter() {
                        gpa_fusion.tick_arrays.entry(pubkey).and_modify(|e| {
                            e.insert(index, tick_array);
                        }).or_insert_with(|| {
                            let mut map_to_update = BTreeMap::new();
                            map_to_update.insert(index, tick_array);
                            map_to_update
                        });
                    }
                }
                fusion_amm_tick_arrays = gpa_fusion.tick_arrays;

                for (pubkey, map) in saros_dlmm_bin_arrays.into_iter() {
                    for (index, bin_array) in map.into_iter() {
                        gpa_saros.bin_arrays.entry(pubkey).and_modify(|e| {
                            e.insert(index, bin_array);
                        }).or_insert_with(|| {
                            let mut map_to_update = BTreeMap::new();
                            map_to_update.insert(index, bin_array);
                            map_to_update
                        });
                    }
                }
                saros_dlmm_bin_arrays = gpa_saros.bin_arrays;


                let duration = start.elapsed();
                debug!("Ticks duration {:?}", duration);

                //

                debug!("Data");
                let start = Instant::now();
                gpa_pump_amm.pools.extend(pump_amm_map);
                pump_amm_map = gpa_pump_amm.pools;

                gpa_pump_amm.config.extend(pump_amm_config);
                pump_amm_config = gpa_pump_amm.config;

                pump_amm_fee_config.extend(gpa_pump_amm.fee_config);

                gpa_orca.pools.extend(orca_pools);
                orca_pools = gpa_orca.pools;

                gpa_raydium_clmm.pools.extend(raydium_clmm_pools);
                raydium_clmm_pools = gpa_raydium_clmm.pools;

                gpa_raydium_clmm.config.extend(raydium_clmm_config);
                raydium_clmm_config = gpa_raydium_clmm.config;

                gpa_meteroa_dlmm.pools.extend(meteora_dlmm_pools);
                meteora_dlmm_pools = gpa_meteroa_dlmm.pools;

                gpa_meteroa_dlmm.bitmap_extensions.extend(meteora_dlmm_bitmap_extensions);
                meteora_dlmm_bitmap_extensions = gpa_meteroa_dlmm.bitmap_extensions;

                gpa_raydium_amm.pools.extend(raydium_amm_v4_map);
                raydium_amm_v4_map = gpa_raydium_amm.pools;

                gpa_raydium_cpmm.pools.extend(raydium_cpmm_pools);
                raydium_cpmm_pools = gpa_raydium_cpmm.pools;

                raydium_cpmm_configs.extend(gpa_raydium_cpmm.configs);

                // goose_gamma_pools.extend(gpa_goose_gamma.pools);
                // goose_gamma_configs.extend(gpa_goose_gamma.configs);
                // goose_gamma_observation_states.extend(gpa_goose_gamma.observation_states);

                gpa_fusion.pools.extend(fusion_amm_pools);
                fusion_amm_pools = gpa_fusion.pools;

                gpa_saros.pools.extend(saros_dlmm_pools);
                saros_dlmm_pools = gpa_saros.pools;

                let duration = start.elapsed();
                debug!("GPA duration {:?}", duration);


                break;
            }
            // Chainstack programSubscribe
            Some(msg) = rx_socket.recv() => {
                    {
                        let msg = msg;
                        if msg.owner == dex::pump_amm::ID {
                            let res = pump::process_pump_amm(msg.pubkey, msg.data.as_slice(),
                                &mut pump_amm_map,
                                &mut pump_amm_config,
                                &mut pump_amm_fee_config,
                                &mut pool_type_and_pubkey,
                            );
                            if let Err(e) = res {
                                error!("Failed to process pump amm: {:?}", e);
                            }
                        } else if msg.owner == dex::raydium_amm::ID {
                            let res = raydium::process_raydium_amm(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut raydium_amm_v4_map,
                                &mut pool_type_and_pubkey,
                            );
                            if let Err(e) = res {
                                error!("Failed to process raydium amm: {:?}", e);
                            }
                        } else if msg.owner == spl_token::ID {
                            process_token_account(msg.pubkey,
                                msg.data.as_slice(),
                                &mut ta_amount_map,
                            );
                        } else if msg.owner == dex::raydium_clmm::ID {
                            let res = raydium::process_raydium_clmm(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut raydium_clmm_tick_arrays,
                                &mut raydium_clmm_pools,
                                &mut raydium_clmm_config,
                                &mut pool_type_and_pubkey,
                                &mut raydium_bitmap_extensions,
                            );
                            if let Err(e) = res {
                                error!("Failed to process raydium clmm: {:?}", e);
                            }
                        } else if msg.owner == dex::meteora_dlmm::ID {
                            let res = meteora::process_meteora_dlmm(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut meteora_dlmm_pools,
                                &mut meteora_dlmm_bin_arrays,
                                &mut meteora_dlmm_bitmap_extensions,
                                &mut pool_type_and_pubkey,
                            );
                            if let Err(e) = res {
                                error!("Failed to process meteora dlmm: {:?}", e);
                            }
                        } else if msg.owner == dex::orca::ID {
                            let res = orca::process_orca(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut orca_pools,
                                Some(&mut orca_oracles), // NATS: derive oracle addresses, or pass None to skip derivation
                                &mut orca_pool_to_oracle,
                                &mut orca_tick_arrays,
                                &mut pool_type_and_pubkey,
                            );
                            if let Err(e) = res {
                                error!("Failed to process orca: {:?}", e);
                            }
                        } else if msg.owner == dex::stabble_stable_swap::ID {
                            let res = stabble::process_stabble_stable_swap(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut stabble_stable_swap_map,
                                &mut pool_type_and_pubkey,
                            );
                            if let Err(e) = res {
                                error!("Failed to process stabble stable swap: {:?}", e);
                            }
                        } else if msg.owner == dex::stabble_weighted_swap::ID {
                            let res = stabble::process_stabble_weighted_swap(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut stabble_weighted_swap_map,
                                &mut pool_type_and_pubkey,
                            );
                            if let Err(e) = res {
                                error!("Failed to process stabble weighted swap: {:?}", e);
                            }
                        }
                        // else if msg.owner == dex::lifinity::ID {
                        //     let res = lifinity::process_lifinity(
                        //         msg.pubkey,
                        //         msg.data.as_slice(),
                        //         &mut lifinity_amm_map,
                        //         &mut lifinity_authorities,
                        //         &mut pool_type_and_pubkey,
                        //     );
                        //     if let Err(e) = res {
                        //         error!("Error processing lifinity: {:?}", e);
                        //     }
                        // }
                        else if msg.owner == dex::raydium_cpmm::ID {
                            let _res = raydium::process_raydium_cpmm(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut raydium_cpmm_configs,
                                &mut raydium_cpmm_pools,
                                &mut pool_type_and_pubkey,
                            );
                        } else if msg.owner == dex::orca_swap_v2::ID {
                            let _res = orca::process_orca_swap_v2(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut orca_swap_v2_markets,
                                &mut orca_swap_v2_authorities,
                                &mut pool_type_and_pubkey,
                            );
                        } else if msg.owner == dex::saros_amm::ID {
                            let _res = saros::process_saros_amm(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut saros_amm_markets,
                                &mut saros_amm_authorities,
                                &mut pool_type_and_pubkey,
                            );
                        }
                        // else if msg.owner == dex::goose_gamma::ID {
                        //     let _res = goose_gama::process_goose_gamma(
                        //         msg.pubkey,
                        //         msg.data.as_slice(),
                        //         &mut goose_gamma_pools,
                        //         &mut goose_gamma_configs,
                        //         &mut goose_gamma_observation_states,
                        //         &mut pool_type_and_pubkey,
                        //     );
                        // }
                        else if msg.owner == dex::meteora_damm_v2::ID {
                            let _res = meteora::process_meteora_damm_v2(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut meteora_damm_v2_pools,
                                &mut pool_type_and_pubkey,
                            );
                        } else if msg.owner == dex::fusion_amm::ID {
                            let res = fusion::process_fusion(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut fusion_amm_pools,
                                &mut fusion_amm_tick_arrays,
                                &mut pool_type_and_pubkey,
                            );
                            if let Err(e) = res {
                                error!("Failed to process fusion: {:?}", e);
                            }
                        }
                        else if msg.owner == dex::saros_dlmm::ID {
                            let res = saros::process_saros_dlmm(
                                msg.pubkey,
                                msg.data.as_slice(),
                                &mut saros_dlmm_pools,
                                &mut saros_dlmm_bin_arrays,
                                &mut pool_type_and_pubkey,
                            );
                            if let Err(e) = res {
                                error!("Failed to process saros dlmm: {:?}", e);
                            }
                        }
                    }
            }
        }
    }

    let pump_amm_pool_to_mint_supply = HashMap::default();

    Ok(GPAResult {
        pump_amm_map,
        pump_amm_config,
        pump_amm_fee_config,
        pump_amm_pool_to_mint_supply,
        orca_tick_arrays,
        orca_pools,
        orca_oracles,
        orca_pool_to_oracle,
        raydium_clmm_tick_arrays,
        raydium_clmm_pools,
        raydium_clmm_config,
        raydium_bitmap_extensions,
        meteora_dlmm_pools,
        meteora_dlmm_bin_arrays,
        meteora_dlmm_bitmap_extensions,
        raydium_amm_v4_map,
        ta_amount_map,
        pool_type_and_pubkey,
        stabble_stable_swap_map,
        stabble_weighted_swap_map,
        //lifinity_amm_map,
        //lifinity_authorities,
        //pyth_map,
        raydium_cpmm_pools,
        raydium_cpmm_configs,
        orca_swap_v2_markets,
        orca_swap_v2_authorities,
        saros_amm_markets,
        saros_amm_authorities,
        //goose_gamma_pools,
        //goose_gamma_configs,
        //goose_gamma_observation_states,
        meteora_damm_v2_pools,
        fusion_amm_pools,
        fusion_amm_tick_arrays,
        saros_dlmm_pools,
        saros_dlmm_bin_arrays,
        token_mints,
    })
}

impl GPAResult {
    pub fn get_partial_data(
        &mut self,
        pool_type: &PoolType,
        pubkey: &Pubkey,
        slot: Slot,
        rpc_client: &solana_client::rpc_client::RpcClient,
    ) -> Option<CalculatorEnum> {
        match pool_type {
            PoolType::RaydiumCpmm => {
                let pool_state = self.raydium_cpmm_pools.get(pubkey);
                if let Some(pool_state) = pool_state {
                    let pc = self.ta_amount_map.get(&pool_state.token1_vault);
                    let coin = self.ta_amount_map.get(&pool_state.token0_vault);
                    let config = self.raydium_cpmm_configs.get(&pool_state.amm_config);
                    if let (Some(pc), Some(coin), Some(config)) = (pc, coin, config) {
                        // if pool_state.token0_mint == WSOL && coin < &cfg.arbitrage.min_wsol {
                        //     return None;
                        // }
                        // if pool_state.token1_mint == WSOL && pc < &cfg.arbitrage.min_wsol {
                        //     return None;
                        // }
                        // if STABLECOINS.contains(&pool_state.token0_mint) && coin < &cfg.arbitrage.min_stable {
                        //     return None;
                        // }
                        // if STABLECOINS.contains(&pool_state.token1_mint) && pc < &cfg.arbitrage.min_stable {
                        //     return None;
                        // }
                        Some(CalculatorEnum::RaydiumCpmm(crate::calculator::RaydiumCpmmData {
                            pool_state_pubkey: *pubkey,
                            pool_state: pool_state.clone(),
                            amm_config: config.clone(),
                            total_pc: *pc,
                            total_coin: *coin,
                        }))
                    } else {
                        debug!(
                            "Raydium CPMM token data not found: {} {} {}",
                            pc.is_some(),
                            coin.is_some(),
                            config.is_some()
                        );
                        None
                    }
                } else {
                    debug!("Raydium CPMM data not found");
                    None
                }
            }

            /*
            PoolType::Lifinity => {
                let amm = self.lifinity_amm_map.get(pubkey);
                if let Some(amm) = amm {
                    let pc = self.ta_amount_map.get(&amm.token_b_account);
                    let coin = self.ta_amount_map.get(&amm.token_a_account);
                    let oracle_main = self.pyth_map.get(&amm.oracle_main_account);
                    let oracle_sub = self.pyth_map.get(&amm.oracle_sub_account);
                    let oracle_pc = self.pyth_map.get(&amm.oracle_pc_account);
                    if let (Some(pc), Some(coin), Some(oracle_main), Some(oracle_sub), Some(oracle_pc)) =
                        (pc, coin, oracle_main, oracle_sub, oracle_pc)
                    {
                        Some(CalculatorEnum::Lifinity(crate::calculator::LifinityData {
                            amm_pubkey: *pubkey,
                            amm: amm.clone(),
                            slot,
                            total_pc: *pc,
                            total_coin: *coin,
                            oracle_main: *oracle_main,
                            oracle_sub: *oracle_sub,
                            oracle_pc: *oracle_pc,
                        }))
                    } else {
                        debug!("Lifinity AMM data not found");
                        None
                    }
                } else {
                    debug!("Lifinity AMM data not found");
                    None
                }
            }
            */
            PoolType::PumpAmm => {
                let pool = self.pump_amm_map.get(pubkey);
                if let Some(pool) = pool {
                    if let Some((config_key, config)) = self.pump_amm_config.iter().last() {
                        let quote = self.ta_amount_map.get(&pool.pool_quote_token_account);
                        let base = self.ta_amount_map.get(&pool.pool_base_token_account);
                        if let (Some(base), Some(quote)) = (base, quote) {
                            // if pool.base_mint == WSOL && base < &cfg.arbitrage.min_wsol {
                            //     return None;
                            // }
                            // if pool.quote_mint == WSOL && quote < &cfg.arbitrage.min_wsol {
                            //     return None;
                            // }
                            // if STABLECOINS.contains(&pool.base_mint) && base < &cfg.arbitrage.min_stable {
                            //     return None;
                            // }
                            // if STABLECOINS.contains(&pool.quote_mint) && quote < &cfg.arbitrage.min_stable {
                            //     return None;
                            // }
                            let is_pump = dex::pump_amm::is_pump_pool(&pool.base_mint, &pool.creator);
                            let fee_config = self.pump_amm_fee_config.get(&dex::pump_amm::FEE_CONFIG)?;
                            let fee = if is_pump {
                                fee_config.flat_fees.lp_fee_bps
                                    + fee_config.flat_fees.protocol_fee_bps
                                    + fee_config.flat_fees.creator_fee_bps
                            } else if let Some(mint_supply) = self.pump_amm_pool_to_mint_supply.get(pubkey) {
                                //*mint_supply
                                let mc = dex::pump_amm::pool_market_cap(*mint_supply, *base as u128, *quote as u128)
                                    .unwrap_or(0);
                                let fees = fee_config.get_fees(is_pump, mc);
                                fees.lp_fee_bps + fees.creator_fee_bps + fees.protocol_fee_bps
                            } else {
                                let account_data = match rpc_client.get_account_data(&pool.base_mint) {
                                    Ok(data) => data,
                                    Err(_) => return None,
                                };
                                let mint = match spl_token::state::Mint::unpack(&account_data) {
                                    Ok(mint) => mint,
                                    Err(_) => return None,
                                };

                                self.pump_amm_pool_to_mint_supply.insert(*pubkey, mint.supply as u128);

                                let mc =
                                    dex::pump_amm::pool_market_cap(mint.supply as u128, *base as u128, *quote as u128)
                                        .unwrap_or(0);

                                let fees = fee_config.get_fees(is_pump, mc);
                                fees.lp_fee_bps + fees.creator_fee_bps + fees.protocol_fee_bps

                                // fee_config
                                //     .fee_tiers
                                //     .iter()
                                //     .next()
                                //     .map(|tier| {
                                //         tier.fees.lp_fee_bps
                                //             + tier.fees.creator_fee_bps
                                //             + tier.fees.protocol_fee_bps
                                //     })
                                //     .unwrap_or(125)
                            };
                            Some(CalculatorEnum::PumpAmm(crate::calculator::PumpAmmData {
                                pool_pubkey: *pubkey,
                                pool: *pool,
                                config_pubkey: *config_key,
                                config: config.clone(),
                                base: *base,
                                quote: *quote,
                                fee,
                            }))
                        } else {
                            debug!("Pump AMM data not found");
                            None
                        }
                    } else {
                        debug!("Pump AMM config not found");
                        None
                    }
                } else {
                    debug!("Pump AMM data not found");
                    None
                }
            }
            PoolType::RaydiumClmmV3 => {
                let pool_state = self.raydium_clmm_pools.get(pubkey);
                if let Some(pool_state) = pool_state {
                    let amm_config = self.raydium_clmm_config.get(&pool_state.amm_config);
                    if let (Some(amm_config), Some(tick_arrays)) =
                        (amm_config, self.raydium_clmm_tick_arrays.get(pubkey))
                    {
                        let (bitmap_extension, bitmap_extension_data) = self
                            .raydium_bitmap_extensions
                            .get(pubkey)
                            .map(|(pubkey, data)| (Some(*pubkey), Some(data.clone())))
                            .unwrap_or((None, None));
                        Some(CalculatorEnum::RaydiumClmmV3(crate::calculator::RaydiumClmmV3Data {
                            amm_config: amm_config.clone(),
                            pool_state_pubkey: *pubkey,
                            pool_state: pool_state.clone(),
                            tick_arrays_map: tick_arrays.clone(),
                            bitmap_extension,
                            bitmap_extension_data,
                        }))
                    } else {
                        debug!("Raydium clmm data not found");
                        None
                    }
                } else {
                    debug!("Raydium clmm data not found");
                    None
                }
            }
            PoolType::RaydiumAmmV4 => {
                let amm_info = self.raydium_amm_v4_map.get(pubkey);
                if let Some(amm_info) = amm_info {
                    debug!(
                        "amm_info coin_vault={}, pc_vault={}",
                        amm_info.coin_vault, amm_info.pc_vault
                    );
                    let coin = self.ta_amount_map.get(&amm_info.coin_vault);
                    let pc = self.ta_amount_map.get(&amm_info.pc_vault);
                    if let (Some(total_pc), Some(total_coin)) = (pc, coin) {
                        // if amm_info.coin_vault_mint == WSOL && total_coin < &cfg.arbitrage.min_wsol {
                        //     return None;
                        // }
                        // if amm_info.pc_vault_mint == WSOL && total_pc < &cfg.arbitrage.min_wsol {
                        //     return None;
                        // }
                        // if STABLECOINS.contains(&amm_info.coin_vault_mint) && total_coin < &cfg.arbitrage.min_stable {
                        //     return None;
                        // }
                        // if STABLECOINS.contains(&amm_info.pc_vault_mint) && total_pc < &cfg.arbitrage.min_stable {
                        //     return None;
                        // }
                        Some(CalculatorEnum::RaydiumAmmV4(crate::calculator::RaydiumAmmV4Data {
                            amm_info_pubkey: *pubkey,
                            amm_info: *amm_info,
                            total_coin: *total_coin,
                            total_pc: *total_pc,
                        }))
                    } else {
                        debug!("Raydium amm data not found");
                        None
                    }
                } else {
                    debug!("Raydium amm data not found");
                    None
                }
            }
            PoolType::Orca => {
                if let Some(tick_arrays) = self.orca_tick_arrays.get(pubkey) {
                    if let Some(whirlpool) = self.orca_pools.get(pubkey) {
                        Some(CalculatorEnum::Orca(crate::calculator::OrcaData {
                            whirlpool: *whirlpool,
                            whirlpool_pubkey: *pubkey,
                            tick_arrays_map: tick_arrays.clone(),
                            oracle: self.orca_pool_to_oracle.get(pubkey).cloned(),
                        }))
                    } else {
                        debug!("Orca whirlpool missing for {}", pubkey);
                        None
                    }
                } else {
                    debug!("Orca data not found");
                    None
                }
            }
            PoolType::MeteoraDlmm => {
                if let Some(lb_pair) = self.meteora_dlmm_pools.get(pubkey) {
                    if let Some(bin_arrays_map) = self.meteora_dlmm_bin_arrays.get(pubkey) {
                        if let Some((bitmap_extension_pubkey, bitmap_extension)) =
                            self.meteora_dlmm_bitmap_extensions.get(pubkey)
                        {
                            return Some(CalculatorEnum::MeteoraDlmm(crate::calculator::MeteoraDlmmData {
                                lb_pair_pubkey: *pubkey,
                                lb_pair: *lb_pair,
                                bin_arrays_map: bin_arrays_map.clone(),
                                bitmap_extension_pubkey: Some(*bitmap_extension_pubkey),
                                bitmap_extension: Some(bitmap_extension.clone()),
                                slot,
                            }));
                        } else {
                            return Some(CalculatorEnum::MeteoraDlmm(crate::calculator::MeteoraDlmmData {
                                lb_pair_pubkey: *pubkey,
                                lb_pair: *lb_pair,
                                bin_arrays_map: bin_arrays_map.clone(),
                                bitmap_extension_pubkey: None,
                                bitmap_extension: None,
                                slot,
                            }));
                        }
                    }
                }
                debug!("Meteora data not found");
                None
            }
            PoolType::StabbleStableSwap => {
                let pool_swap = self.stabble_stable_swap_map.get(pubkey);
                if let Some(pool_swap) = pool_swap {
                    Some(CalculatorEnum::StabbleStableSwap(
                        crate::calculator::StabbleStableSwapData {
                            pool_swap_pubkey: *pubkey,
                            pool_swap: pool_swap.clone(),
                        },
                    ))
                } else {
                    debug!("StabbleStableSwap data not found");
                    None
                }
            }
            PoolType::StabbleWeightedSwap => {
                let pool_swap = self.stabble_weighted_swap_map.get(pubkey);
                if let Some(pool_swap) = pool_swap {
                    Some(CalculatorEnum::StabbleWeightedSwap(
                        crate::calculator::StabbleWeightedSwapData {
                            pool_swap_pubkey: *pubkey,
                            pool_swap: pool_swap.clone(),
                        },
                    ))
                } else {
                    debug!("StabbleWeightedSwap data not found");
                    None
                }
            }
            PoolType::OrcaSwapV2 => {
                let market = self.orca_swap_v2_markets.get(pubkey);
                if let Some(market) = market {
                    let coin = self.ta_amount_map.get(&market.token_a);
                    let pc = self.ta_amount_map.get(&market.token_b);
                    if let (Some(coin), Some(pc)) = (coin, pc) {
                        // if market.token_a_mint == WSOL && coin < &cfg.arbitrage.min_wsol {
                        //     return None;
                        // }
                        // if market.token_b_mint == WSOL && pc < &cfg.arbitrage.min_wsol {
                        //     return None;
                        // }
                        // if STABLECOINS.contains(&market.token_a_mint) && coin < &cfg.arbitrage.min_stable {
                        //     return None;
                        // }
                        // if STABLECOINS.contains(&market.token_b_mint) && pc < &cfg.arbitrage.min_stable {
                        //     return None;
                        // }
                        Some(CalculatorEnum::OrcaSwapV2(crate::calculator::OrcaSwapV2Data {
                            market_pubkey: *pubkey,
                            market: market.clone(),
                            total_coin: *coin,
                            total_pc: *pc,
                        }))
                    } else {
                        debug!("Orca Swap V2 data not found");
                        None
                    }
                } else {
                    debug!("Orca Swap V2 data not found");
                    None
                }
            }
            PoolType::SarosAmm => {
                let market = self.saros_amm_markets.get(pubkey);
                if let Some(market) = market {
                    let coin = self.ta_amount_map.get(&market.token_a);
                    let pc = self.ta_amount_map.get(&market.token_b);
                    if let (Some(coin), Some(pc)) = (coin, pc) {
                        Some(CalculatorEnum::SarosAmm(crate::calculator::SarosAmmData {
                            market_pubkey: *pubkey,
                            market: market.clone(),
                            total_coin: *coin,
                            total_pc: *pc,
                        }))
                    } else {
                        debug!("Saros Amm data not found");
                        None
                    }
                } else {
                    debug!("Saros Amm data not found");
                    None
                }
            }
            /*
            PoolType::GooseGamma => {
                let pool = self.goose_gamma_pools.get(pubkey);
                if let Some(pool) = pool {
                    let a = self.ta_amount_map.get(&pool.token_0vault);
                    let b = self.ta_amount_map.get(&pool.token_1vault);
                    let config = self.goose_gamma_configs.get(&pool.amm_config);
                    let observation_state = self.goose_gamma_observation_states.get(pubkey);
                    if let (Some(a), Some(b), Some(config), Some(observation_state)) = (a, b, config, observation_state)
                    {
                        Some(CalculatorEnum::GooseGamma(crate::calculator::GooseGammaData {
                            pool_pubkey: *pubkey,
                            pool: pool.clone(),
                            config: config.clone(),
                            observation_state: observation_state.clone(),
                            coin_a: *a,
                            coin_b: *b,
                        }))
                    } else {
                        debug!("Goose Gamma data not found");
                        None
                    }
                } else {
                    debug!("Goose Gamma data not found");
                    None
                }
            }
            */
            PoolType::MeteoraDammV2 => {
                let pool = self.meteora_damm_v2_pools.get(pubkey);
                if let Some(pool) = pool {
                    // if pool.token_a_mint == WSOL && pool.token_a_reserve < cfg.arbitrage.min_wsol {
                    //     return None;
                    // }
                    // if pool.token_b_mint == WSOL && pool.token_b_reserve < cfg.arbitrage.min_wsol {
                    //     return None;
                    // }
                    // if STABLECOINS.contains(&pool.token_a_mint) && pool.token_a_reserve < cfg.arbitrage.min_stable {
                    //     return None;
                    // }
                    // if STABLECOINS.contains(&pool.token_b_mint) && pool.token_b_reserve < cfg.arbitrage.min_stable {
                    //     return None;
                    // }
                    Some(CalculatorEnum::MeteoraDammV2(crate::calculator::MeteoraDammV2Data {
                        pool_pubkey: *pubkey,
                        pool: *pool,
                        slot,
                    }))
                } else {
                    debug!("MeteoraDammV2 data not found");
                    None
                }
            }
            PoolType::FusionAmm => {
                if let Some(tick_arrays) = self.fusion_amm_tick_arrays.get(pubkey) {
                    if let Some(pool) = self.fusion_amm_pools.get(pubkey) {
                        Some(CalculatorEnum::FusionAmm(crate::calculator::FusionAmmData {
                            pool: *pool,
                            pool_pubkey: *pubkey,
                            tick_arrays_map: tick_arrays.clone(),
                        }))
                    } else {
                        debug!("Fusion AMM pool missing for {}", pubkey);
                        None
                    }
                } else {
                    debug!("Fusion AMM data not found");
                    None
                }
            }

            PoolType::SarosDlmm => {
                if let Some(pair) = self.saros_dlmm_pools.get(pubkey) {
                    if let Some(bin_arrays_map) = self.saros_dlmm_bin_arrays.get(pubkey) {
                        return Some(CalculatorEnum::SarosDlmm(crate::calculator::SarosDlmmData {
                            pool_pubkey: *pubkey,
                            pair: *pair,
                            bin_arrays_map: bin_arrays_map.clone(),
                        }));
                    } else {
                        debug!("Saros DLMM bin arrays not found for {}", pubkey);
                        return None;
                    }
                }
                debug!("Meteora data not found");
                None
            }
        }
    }

    // This version is for ALT
    pub fn get_partial_data_v2(
        &self,
        pool_type: &PoolType,
        pubkey: &Pubkey,
        slot: Slot,
        default: bool,
    ) -> Option<CalculatorEnum> {
        match pool_type {
            PoolType::RaydiumCpmm => {
                let pool_state = self.raydium_cpmm_pools.get(pubkey);
                if let Some(pool_state) = pool_state {
                    let pc = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&pool_state.token1_vault).copied()
                    };
                    let coin = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&pool_state.token0_vault).copied()
                    };
                    let config = self.raydium_cpmm_configs.get(&pool_state.amm_config);
                    if let (Some(pc), Some(coin), Some(config)) = (pc, coin, config) {
                        Some(CalculatorEnum::RaydiumCpmm(crate::calculator::RaydiumCpmmData {
                            pool_state_pubkey: *pubkey,
                            pool_state: pool_state.clone(),
                            amm_config: config.clone(),
                            total_pc: pc,
                            total_coin: coin,
                        }))
                    } else {
                        debug!(
                            "Raydium CPMM token data not found: {} {} {}",
                            pc.is_some(),
                            coin.is_some(),
                            config.is_some()
                        );
                        None
                    }
                } else {
                    debug!("Raydium CPMM data not found");
                    None
                }
            }
            /*
            PoolType::Lifinity => {
                let amm = self.lifinity_amm_map.get(pubkey);
                if let Some(amm) = amm {
                    let pc = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&amm.token_b_account).copied()
                    };
                    let coin = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&amm.token_a_account).copied()
                    };
                    let oracle_main = self.pyth_map.get(&amm.oracle_main_account);
                    let oracle_sub = self.pyth_map.get(&amm.oracle_sub_account);
                    let oracle_pc = self.pyth_map.get(&amm.oracle_pc_account);
                    if let (Some(pc), Some(coin), Some(oracle_main), Some(oracle_sub), Some(oracle_pc)) =
                        (pc, coin, oracle_main, oracle_sub, oracle_pc)
                    {
                        Some(CalculatorEnum::Lifinity(crate::calculator::LifinityData {
                            amm_pubkey: *pubkey,
                            amm: amm.clone(),
                            slot,
                            total_pc: pc,
                            total_coin: coin,
                            oracle_main: *oracle_main,
                            oracle_sub: *oracle_sub,
                            oracle_pc: *oracle_pc,
                        }))
                    } else {
                        debug!("Lifinity AMM data not found");
                        None
                    }
                } else {
                    debug!("Lifinity AMM data not found");
                    None
                }
            }
             */
            PoolType::PumpAmm => {
                let pool = self.pump_amm_map.get(pubkey);
                if let Some(pool) = pool {
                    let (config_key, config) = self.pump_amm_config.iter().last().unwrap();
                    let quote = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&pool.pool_quote_token_account).copied()
                    };
                    let base = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&pool.pool_base_token_account).copied()
                    };
                    if let (Some(base), Some(quote)) = (base, quote) {
                        let is_pump = dex::pump_amm::is_pump_pool(&pool.base_mint, &pool.creator);
                        let fee_config = match self.pump_amm_fee_config.iter().last() {
                            Some((_, config)) => config,
                            None => return None,
                        };
                        let fee = if is_pump {
                            fee_config.flat_fees.lp_fee_bps
                                + fee_config.flat_fees.protocol_fee_bps
                                + fee_config.flat_fees.creator_fee_bps
                        } else if let Some(mint_supply) = self.pump_amm_pool_to_mint_supply.get(pubkey) {
                            //*mint_supply
                            let mc =
                                dex::pump_amm::pool_market_cap(*mint_supply, base as u128, quote as u128).unwrap_or(0);
                            let fees = fee_config.get_fees(is_pump, mc);
                            fees.lp_fee_bps + fees.creator_fee_bps + fees.protocol_fee_bps
                        } else {
                            fee_config
                                .fee_tiers
                                .first()
                                .map(|tier| {
                                    tier.fees.lp_fee_bps + tier.fees.creator_fee_bps + tier.fees.protocol_fee_bps
                                })
                                .unwrap_or(125)
                        };
                        Some(CalculatorEnum::PumpAmm(crate::calculator::PumpAmmData {
                            pool_pubkey: *pubkey,
                            pool: *pool,
                            config_pubkey: *config_key,
                            config: config.clone(),
                            base,
                            quote,
                            fee,
                        }))
                    } else {
                        debug!("Pump AMM data not found");
                        None
                    }
                } else {
                    debug!("Pump AMM data not found");
                    None
                }
            }
            PoolType::RaydiumClmmV3 => {
                let pool_state = self.raydium_clmm_pools.get(pubkey);
                if let Some(pool_state) = pool_state {
                    let amm_config = self.raydium_clmm_config.get(&pool_state.amm_config);
                    if let (Some(amm_config), Some(tick_arrays)) =
                        (amm_config, self.raydium_clmm_tick_arrays.get(pubkey))
                    {
                        // let bitmap_extension = self
                        //     .raydium_bitmap_extensions
                        //     .get(pubkey)
                        //     .map(|(pubkey, _)| *pubkey);
                        let (bitmap_extension, bitmap_extension_data) = self
                            .raydium_bitmap_extensions
                            .get(pubkey)
                            .map(|(pubkey, data)| (Some(*pubkey), Some(data.clone())))
                            .unwrap_or((None, None));
                        Some(CalculatorEnum::RaydiumClmmV3(crate::calculator::RaydiumClmmV3Data {
                            amm_config: amm_config.clone(),
                            pool_state_pubkey: *pubkey,
                            pool_state: pool_state.clone(),
                            tick_arrays_map: tick_arrays.clone(),
                            bitmap_extension,
                            bitmap_extension_data,
                        }))
                    } else {
                        debug!("Raydium clmm data not found");
                        None
                    }
                } else {
                    debug!("Raydium clmm data not found");
                    None
                }
            }
            PoolType::RaydiumAmmV4 => {
                let amm_info = self.raydium_amm_v4_map.get(pubkey);
                if let Some(amm_info) = amm_info {
                    debug!(
                        "amm_info coin_vault={}, pc_vault={}",
                        amm_info.coin_vault, amm_info.pc_vault
                    );
                    let coin = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&amm_info.coin_vault).copied()
                    };
                    let pc = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&amm_info.pc_vault).copied()
                    };
                    if let (Some(total_pc), Some(total_coin)) = (pc, coin) {
                        Some(CalculatorEnum::RaydiumAmmV4(crate::calculator::RaydiumAmmV4Data {
                            amm_info_pubkey: *pubkey,
                            amm_info: *amm_info,
                            total_coin,
                            total_pc,
                        }))
                    } else {
                        debug!("Raydium amm data not found");
                        None
                    }
                } else {
                    debug!("Raydium amm data not found");
                    None
                }
            }
            PoolType::Orca => {
                if let Some(tick_arrays) = self.orca_tick_arrays.get(pubkey) {
                    Some(CalculatorEnum::Orca(crate::calculator::OrcaData {
                        whirlpool: *self.orca_pools.get(pubkey).unwrap(),
                        whirlpool_pubkey: *pubkey,
                        tick_arrays_map: tick_arrays.clone(),
                        oracle: self.orca_pool_to_oracle.get(pubkey).cloned(),
                    }))
                } else {
                    debug!("Orca data not found");
                    None
                }
            }
            PoolType::MeteoraDlmm => {
                if let Some(lb_pair) = self.meteora_dlmm_pools.get(pubkey) {
                    if let Some(bin_arrays_map) = self.meteora_dlmm_bin_arrays.get(pubkey) {
                        if let Some((bitmap_extension_pubkey, bitmap_extension)) =
                            self.meteora_dlmm_bitmap_extensions.get(pubkey)
                        {
                            return Some(CalculatorEnum::MeteoraDlmm(crate::calculator::MeteoraDlmmData {
                                lb_pair_pubkey: *pubkey,
                                lb_pair: *lb_pair,
                                bin_arrays_map: bin_arrays_map.clone(),
                                bitmap_extension_pubkey: Some(*bitmap_extension_pubkey),
                                bitmap_extension: Some(bitmap_extension.clone()),
                                slot,
                            }));
                        } else {
                            return Some(CalculatorEnum::MeteoraDlmm(crate::calculator::MeteoraDlmmData {
                                lb_pair_pubkey: *pubkey,
                                lb_pair: *lb_pair,
                                bin_arrays_map: bin_arrays_map.clone(),
                                bitmap_extension_pubkey: None,
                                bitmap_extension: None,
                                slot,
                            }));
                        }
                    }
                }
                debug!("Meteora data not found");
                None
            }
            PoolType::StabbleStableSwap => {
                let pool_swap = self.stabble_stable_swap_map.get(pubkey);
                if let Some(pool_swap) = pool_swap {
                    Some(CalculatorEnum::StabbleStableSwap(
                        crate::calculator::StabbleStableSwapData {
                            pool_swap_pubkey: *pubkey,
                            pool_swap: pool_swap.clone(),
                        },
                    ))
                } else {
                    debug!("StabbleStableSwap data not found");
                    None
                }
            }
            PoolType::StabbleWeightedSwap => {
                let pool_swap = self.stabble_weighted_swap_map.get(pubkey);
                if let Some(pool_swap) = pool_swap {
                    Some(CalculatorEnum::StabbleWeightedSwap(
                        crate::calculator::StabbleWeightedSwapData {
                            pool_swap_pubkey: *pubkey,
                            pool_swap: pool_swap.clone(),
                        },
                    ))
                } else {
                    debug!("StabbleWeightedSwap data not found");
                    None
                }
            }
            PoolType::OrcaSwapV2 => {
                let market = self.orca_swap_v2_markets.get(pubkey);
                if let Some(market) = market {
                    let coin = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&market.token_a).copied()
                    };
                    let pc = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&market.token_b).copied()
                    };
                    if let (Some(coin), Some(pc)) = (coin, pc) {
                        Some(CalculatorEnum::OrcaSwapV2(crate::calculator::OrcaSwapV2Data {
                            market_pubkey: *pubkey,
                            market: market.clone(),
                            total_coin: coin,
                            total_pc: pc,
                        }))
                    } else {
                        debug!("Orca Swap V2 data not found");
                        None
                    }
                } else {
                    debug!("Orca Swap V2 data not found");
                    None
                }
            }
            PoolType::SarosAmm => {
                let market = self.saros_amm_markets.get(pubkey);
                if let Some(market) = market {
                    let coin = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&market.token_a).copied()
                    };
                    let pc = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&market.token_b).copied()
                    };
                    if let (Some(coin), Some(pc)) = (coin, pc) {
                        Some(CalculatorEnum::SarosAmm(crate::calculator::SarosAmmData {
                            market_pubkey: *pubkey,
                            market: market.clone(),
                            total_coin: coin,
                            total_pc: pc,
                        }))
                    } else {
                        debug!("Saros AMM data not found");
                        None
                    }
                } else {
                    debug!("Saros AMM data not found");
                    None
                }
            }
            /*
            PoolType::GooseGamma => {
                let pool = self.goose_gamma_pools.get(pubkey);
                if let Some(pool) = pool {
                    let a = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&pool.token_0vault).copied()
                    };
                    let b = if default {
                        Some(0_u64)
                    } else {
                        self.ta_amount_map.get(&pool.token_1vault).copied()
                    };
                    let config = self.goose_gamma_configs.get(&pool.amm_config);
                    let observation_state = self.goose_gamma_observation_states.get(pubkey);
                    if let (Some(a), Some(b), Some(config), Some(observation_state)) = (a, b, config, observation_state)
                    {
                        Some(CalculatorEnum::GooseGamma(crate::calculator::GooseGammaData {
                            pool_pubkey: *pubkey,
                            pool: pool.clone(),
                            config: config.clone(),
                            observation_state: observation_state.clone(),
                            coin_a: a,
                            coin_b: b,
                        }))
                    } else {
                        debug!("Goose Gamma data not found");
                        None
                    }
                } else {
                    debug!("Goose Gamma data not found");
                    None
                }
            }
            */
            PoolType::MeteoraDammV2 => {
                let pool = self.meteora_damm_v2_pools.get(pubkey);
                if let Some(pool) = pool {
                    Some(CalculatorEnum::MeteoraDammV2(crate::calculator::MeteoraDammV2Data {
                        pool_pubkey: *pubkey,
                        pool: *pool,
                        slot,
                    }))
                } else {
                    debug!("MeteoraDammV2 data not found");
                    None
                }
            }

            PoolType::FusionAmm => {
                if let Some(tick_arrays) = self.fusion_amm_tick_arrays.get(pubkey) {
                    Some(CalculatorEnum::FusionAmm(crate::calculator::FusionAmmData {
                        pool: *self.fusion_amm_pools.get(pubkey).unwrap(),
                        pool_pubkey: *pubkey,
                        tick_arrays_map: tick_arrays.clone(),
                    }))
                } else {
                    debug!("Fusion data not found");
                    None
                }
            }

            PoolType::SarosDlmm => {
                if let Some(pair) = self.saros_dlmm_pools.get(pubkey) {
                    if let Some(bin_arrays_map) = self.saros_dlmm_bin_arrays.get(pubkey) {
                        return Some(CalculatorEnum::SarosDlmm(crate::calculator::SarosDlmmData {
                            pool_pubkey: *pubkey,
                            pair: *pair,
                            bin_arrays_map: bin_arrays_map.clone(),
                        }));
                    } else {
                        debug!("Saros DLMM bin arrays not found for {}", pubkey);
                        return None;
                    }
                }
                debug!("Meteora data not found");
                None
            }
        }
    }
}

pub async fn get_pools_gpa(
    url: &str,
) -> Result<
    (
        //goose_gama::GooseGammaGPAResult,
        orca::OrcaSwapV2GPAResult,
        raydium::RaydiumCpmmGPAResult,
        //lifinity::LifinityGPAResult,
        stabble::StabbleWeightedSwapGPAResult,
        stabble::StabbleStableSwapGPAResult,
        pump::PumpAmmGPAResult,
        orca::OrcaGPAResult,
        raydium::RaydiumAmmGPAResult,
        raydium::RaydiumClmmGPAResult,
        meteora::MeteoraDlmmGPAResult,
        meteora::MeteoraDammV2GPAResult,
        fusion::FusionAmmGPAResult,
        saros::SarosDlmmGPAResult,
    ),
    tokio::task::JoinError,
> {
    // let handle_goose_gamma: tokio::task::JoinHandle<goose_gama::GooseGammaGPAResult> =
    //     goose_gama::spawn_goose_gamma(url.to_string(), None);
    // sleep().await;

    let handle_orca_swap_v2: tokio::task::JoinHandle<orca::OrcaSwapV2GPAResult> =
        orca::spawn_orca_swap_v2(url.to_string(), None);
    sleep().await;

    let handle_raydium_cpmm: tokio::task::JoinHandle<raydium::RaydiumCpmmGPAResult> =
        raydium::spawn_raydium_cpmm(url.to_string(), None);
    sleep().await;

    // let handle_lifinity: tokio::task::JoinHandle<lifinity::LifinityGPAResult> =
    //     lifinity::spawn_lifinity(url.to_string(), None);
    // sleep().await;

    let handle_stabble_weighted_swap: tokio::task::JoinHandle<stabble::StabbleWeightedSwapGPAResult> =
        stabble::spawn_stabble_weighted_swap(url.to_string(), None);
    sleep().await;

    let handle_stabble_stable_swap: tokio::task::JoinHandle<stabble::StabbleStableSwapGPAResult> =
        stabble::spawn_stabble_stable_swap(url.to_string(), None);
    sleep().await;

    let handle_pump_amm: tokio::task::JoinHandle<pump::PumpAmmGPAResult> = pump::spawn_pump_amm(url.to_string(), None);
    sleep().await;

    let memcmp = RpcFilterType::Memcmp(Memcmp::new(
        0,                                                                      // offset
        MemcmpEncodedBytes::Bytes(dex::orca::WHIRLPOOL_DISCRIMINATOR.to_vec()), // encoded bytes
    ));
    let config_orca = RpcProgramAccountsConfig {
        filters: Some(vec![memcmp]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig::processed()),
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };
    let handle_orca = orca::spawn_orca(url.to_string(), Some(config_orca));
    sleep().await;

    let handle_raydium_amm: tokio::task::JoinHandle<raydium::RaydiumAmmGPAResult> =
        raydium::spawn_raydium_amm(url.to_string(), None);
    sleep().await;

    let handle_raydium_clmm = raydium::spawn_raydium_clmm(url.to_string());
    sleep().await;

    let memcmp = RpcFilterType::Memcmp(Memcmp::new(
        0,                                                                            // offset
        MemcmpEncodedBytes::Bytes(dex::meteora_dlmm::LB_PAIR_ACCOUNT_DISCM.to_vec()), // encoded bytes
    ));
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![memcmp]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig::processed()),
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };
    let handle_meteora_dlmm = meteora::spawn_meteora_dlmm(url.to_string(), Some(config));
    sleep().await;

    let handle_meteora_damm_v2 = meteora::spawn_meteora_damm_v2(url.to_string(), None);
    sleep().await;

    let memcmp = RpcFilterType::Memcmp(Memcmp::new(
        0,                                                                              // offset
        MemcmpEncodedBytes::Bytes(dex::fusion_amm::FUSION_POOL_ACCOUNT_DISCM.to_vec()), // encoded bytes
    ));
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![memcmp]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig::processed()),
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };
    let handle_fusion = fusion::spawn_fusion(url.to_string(), Some(config));
    sleep().await;

    let memcmp = RpcFilterType::Memcmp(Memcmp::new(
        0,                                                                       // offset
        MemcmpEncodedBytes::Bytes(dex::saros_dlmm::PAIR_ACCOUNT_DISCM.to_vec()), // encoded bytes
    ));
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![memcmp]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig::processed()),
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };
    let handle_saros = saros::spawn_saros_dlmm(url.to_string(), Some(config));

    sleep().await;

    let result = tokio::join!(
        //handle_goose_gamma,
        handle_orca_swap_v2,
        handle_raydium_cpmm,
        //handle_lifinity,
        handle_stabble_weighted_swap,
        handle_stabble_stable_swap,
        handle_pump_amm,
        handle_orca,
        handle_raydium_amm,
        handle_raydium_clmm,
        handle_meteora_dlmm,
        handle_meteora_damm_v2,
        handle_fusion,
        handle_saros,
    );

    Ok((
        result.0?, result.1?, result.2?, result.3?, result.4?, result.5?, result.6?, result.7?, result.8?, result.9?,
        result.10?, result.11?,
    ))
}

// eof
