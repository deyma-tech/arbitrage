use ahash::{AHashMap as HashMap, AHashSet};
use arb_bot::optimizer::{spawn_filter_arbitrages_v8, spawn_optimization, UpdateAlts};
use arb_bot::price::PriceMap;
use arb_bot::setup::{get_tables, get_tables_v2};
use arb_bot::OptimizeResult;
use arb_core::arbitrage::OpportunityWithCalculators;
use arb_core::calculator::CalculatorEnum;
use arb_core::executor_v2::EXECUTOR_V2_PROGRAM_ID;

use arb_core::gpa::{sync_gpa, PoolToCalculator};

use clap::{crate_name, crate_version, Parser};
use crossbeam_channel::unbounded;
use log::{debug, info, warn, LevelFilter};
use num_traits::{ToPrimitive, Zero};
use rand::seq::IteratorRandom;
use solana_client::nonblocking::rpc_client;
use solana_sdk::message::AddressLookupTableAccount;
use spl_token_2022::extension::transfer_fee::{TransferFeeAmount, TransferFeeConfig};
use spl_token_2022::extension::{BaseStateWithExtensions, PodStateWithExtensions};
use spl_token_2022::pod::{PodAccount, PodMint};
use std::str::FromStr;
use std::time::Duration;
use tokio::select;
use utils::constants::{ALLOWED_TOKEN_2022, WSOL};
use utils::safe::ResultExt;
use utils::telegram;

use solana_program::pubkey::Pubkey;
use solana_sdk::clock::Slot;
use std::collections::BinaryHeap;

use chrono::Local;
use std::io::Write;
use utils::deserialize::{Message, MessagesV2};

use arb_bot::process::{add_to_calculator, process_message};
use arb_bot::providers::{get_providers, Provider, ProviderType};
use config::{dump_config, Config, JitoConfig, CONFIG as cfg};
use utils::queue::Queue;

use tikv_jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Default, Debug)]
struct Cli {
    #[arg(short('c'), long, default_value = "/home/ubuntu/.arb/config.toml")]
    pub config: String,

    #[arg(short = 'v', long, default_value_t = false, help = "Print version")]
    pub version: bool,

    #[arg(long, default_value_t = false, help = "Dump default config")]
    pub dump_default_config: bool,
}

impl Cli {
    pub fn print_version(&self) {
        println!("{} {}", crate_name!(), crate_version!());
    }
}

//#[tokio::main]
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.version {
        cli.print_version();
        return Ok(());
    }

    if cli.dump_default_config {
        println!("\n");
        match dump_config(&Config::default()) {
            Ok(config) => println!("{config}"),
            Err(err) => println!("Error dumping default config: {err}"),
        }
        println!("\n");
        return Ok(());
    }

    config::load_dotenv(".env");
    println!("Config: {:?}", cli.config);
    config::init(&cli.config);
    if cfg.enable_execution {
        let executor_program = Pubkey::from_str(&cfg.arb_executor_v2_program_id)
            .map_err(|err| anyhow::anyhow!("ARB_EXECUTOR_V2_PROGRAM_ID inválido: {err}"))?;
        if executor_program != EXECUTOR_V2_PROGRAM_ID {
            anyhow::bail!("Execution refused: el executor configurado no coincide con el ABI V2 verificado");
        }
        if cfg.arbitrage.enable_flashloan {
            anyhow::bail!("Execution refused: flashloan todavía no está soportado por executor V2");
        }
    }
    telegram::init(
        &cfg.telegram.teloxide_token,
        if cfg.telegram.notification_channel_id != 0 {
            Some(cfg.telegram.notification_channel_id)
        } else {
            None
        },
    );
    //watch_dog::init();

    setup_logger(cfg.log_level.0);

    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(err) => panic!("Error creating tokio runtime: {err:?}"),
    };

    runtime.block_on(async {
        telegram::notify(&format!("Bot '{}' started", cfg.name)).await;
    });

    let (tx_pool, _rx_pool) = tokio::sync::mpsc::unbounded_channel::<PoolToCalculator>();
    let (tx_token2022_bc, mut _rx_token2022_bc) = tokio::sync::broadcast::channel::<AHashSet<Pubkey>>(100);

    let providers = runtime.block_on(async { get_providers().await });

    let (tx_messages, rx_messages) = unbounded::<MessagesV2>();

    // Optimizer

    let total_execution_threads = providers
        .iter()
        .fold(0, |acc, provider| acc + provider.get_execution_threads());

    let mut filter = u64::MAX;
    let mut runtimes = vec![];
    let mut tx_bloxroute_execution_arbitrages = vec![];
    let mut tx_jito_quicknode_execution_arbitrages = vec![];
    let mut tx_jito_execution_arbitrages = vec![];
    let mut tx_nextblock_execution_arbitrages = vec![];
    for provider in providers.iter() {
        if provider.get_filter() < filter {
            filter = provider.get_filter();
        }
        match provider {
            ProviderType::Bloxroute(_) => {
                for _ in 0..provider.get_execution_threads() {
                    let provider = provider.clone();
                    let (tx_arbitrage, rx_arbitrage) = tokio::sync::mpsc::unbounded_channel::<(
                        OpportunityWithCalculators,
                        OptimizeResult,
                        Vec<AddressLookupTableAccount>,
                        Vec<AddressLookupTableAccount>,
                    )>();
                    tx_bloxroute_execution_arbitrages.push(tx_arbitrage);
                    let runtime = provider.run(rx_arbitrage, tx_token2022_bc.subscribe(), None);
                    runtimes.push(runtime);
                }
            }
            ProviderType::Nextblock(_) => {
                for _ in 0..provider.get_execution_threads() {
                    let provider = provider.clone();
                    let (tx_arbitrage, rx_arbitrage) = tokio::sync::mpsc::unbounded_channel::<(
                        OpportunityWithCalculators,
                        OptimizeResult,
                        Vec<AddressLookupTableAccount>,
                        Vec<AddressLookupTableAccount>,
                    )>();
                    tx_nextblock_execution_arbitrages.push(tx_arbitrage);
                    let runtime = provider.run(rx_arbitrage, tx_token2022_bc.subscribe(), None);
                    runtimes.push(runtime);
                }
            }
            ProviderType::JitoQuicknode(_) => {
                for _ in 0..provider.get_execution_threads() {
                    let provider = provider.clone();
                    let (tx_arbitrage, rx_arbitrage) = tokio::sync::mpsc::unbounded_channel::<(
                        OpportunityWithCalculators,
                        OptimizeResult,
                        Vec<AddressLookupTableAccount>,
                        Vec<AddressLookupTableAccount>,
                    )>();
                    tx_jito_quicknode_execution_arbitrages.push(tx_arbitrage);
                    let runtime = provider.run(rx_arbitrage, tx_token2022_bc.subscribe(), None);
                    runtimes.push(runtime);
                }
            }
            ProviderType::Jito(_) => {
                let (primary_jito, secondary_jito) = JitoConfig::get_regions(&cfg.get_region());
                let mut engines = secondary_jito;
                engines.insert(0, primary_jito);
                for engine in engines {
                    let provider = provider.clone();
                    let (tx_arbitrage, rx_arbitrage) = tokio::sync::mpsc::unbounded_channel::<(
                        OpportunityWithCalculators,
                        OptimizeResult,
                        Vec<AddressLookupTableAccount>,
                        Vec<AddressLookupTableAccount>,
                    )>();
                    tx_jito_execution_arbitrages.push(tx_arbitrage);
                    let runtime = provider.run(rx_arbitrage, tx_token2022_bc.subscribe(), Some(engine));
                    runtimes.push(runtime);
                }
            }
            // TODO: temp solution
            ProviderType::Log(_) => {
                for _ in 0..provider.get_execution_threads() {
                    let provider = provider.clone();
                    let (tx_arbitrage, rx_arbitrage) = tokio::sync::mpsc::unbounded_channel::<(
                        OpportunityWithCalculators,
                        OptimizeResult,
                        Vec<AddressLookupTableAccount>,
                        Vec<AddressLookupTableAccount>,
                    )>();
                    tx_jito_quicknode_execution_arbitrages.push(tx_arbitrage);
                    let runtime = provider.run(rx_arbitrage, tx_token2022_bc.subscribe(), None);
                    runtimes.push(runtime);
                }
            }
        }
    }

    let (tx_arbitrages, rx_optimized_arbitrage) = spawn_optimization(filter).or_panic("FailedToSpawnOptimization");

    let (pool_table, table_table, optionals) = get_tables();
    info!(
        "Found table keys for {} tables, pools: {}, optionals: {}",
        table_table.len(),
        pool_table.len(),
        optionals.len()
    );

    let (tx_update_alts, rx_update_alts) = tokio::sync::mpsc::channel::<UpdateAlts>(1);

    std::thread::spawn({
        let alt_path = cfg.atl_manager.path.clone();
        async move || {
            let checksum_file_path = format!("{}/.checksum", alt_path.clone());
            let mut last_checksum = utils::directory_hash::DirectoryHash::read_from_file(checksum_file_path.clone())
                .unwrap_or_else(|err| {
                    warn!("ALT: Failed to read checksum file: {err:?}");
                    "".to_string()
                });
            loop {
                std::thread::sleep(Duration::from_secs(60 * 10)); // 10 minutes
                let new_checksum = utils::directory_hash::DirectoryHash::read_from_file(checksum_file_path.clone())
                    .unwrap_or_else(|err| {
                        warn!("ALT: Failed to read checksum file: {err:?}");
                        "".to_string()
                    });
                if !new_checksum.is_empty() && new_checksum != last_checksum {
                    last_checksum = new_checksum;
                    for _ in 0..4 {
                        let res = get_tables_v2();
                        if let Ok((pool_to_table_key, table_key_to_table, pool_pubkey_to_optionals)) = res {
                            let _ = tx_update_alts
                                .send((pool_to_table_key, table_key_to_table, pool_pubkey_to_optionals))
                                .await;
                            info!("ALT: sent updated tables {:?}", last_checksum);
                            break;
                        } else {
                            std::thread::sleep(Duration::from_secs(120));
                        }
                    }
                }
            }
        }
    });

    spawn_filter_arbitrages_v8(
        total_execution_threads, // bloxroute is configurable separately // if is_bloxroute { 100 } else { execution_threads },
        rx_optimized_arbitrage,
        tx_jito_quicknode_execution_arbitrages,
        tx_jito_execution_arbitrages,
        tx_nextblock_execution_arbitrages,
        tx_bloxroute_execution_arbitrages,
        pool_table,
        table_table,
        optionals,
        rx_update_alts,
    );

    let (tx_calculators, rx_calculators) = unbounded::<(Slot, Vec<Box<CalculatorEnum>>)>();

    std::thread::spawn(move || {
        let mut cursor = 0_u64;
        let _queue = Queue::new(2500);
        let mut pubkey_to_calculator = HashMap::new();

        let mut price = PriceMap::new(cfg.arbitrage.price_map_size);

        let mut buffer = Vec::with_capacity(100);

        let mut top_opportunities = BinaryHeap::new();
        let mut already_done = AHashSet::new();
        //let mut opp_already_inserted = AHashSet::new();

        let filter_factor = cfg.arbitrage.opportunity_filter_factor.max(1);
        // experiment reinitialize map after some time ...
        // start
        // let mut loop_start = now::as_micros();

        let filter: f64 = (filter / filter_factor).to_f64().unwrap();

        loop {
            let Ok((slot, calculators)) = rx_calculators.recv() else {
                break;
            };
            buffer.push((slot, calculators));
            while let Ok((slot, calculators)) = rx_calculators.try_recv() {
                buffer.push((slot, calculators));
            }

            let mut slot = 0;

            buffer.reverse();
            let mut sol_calculators: Vec<Box<CalculatorEnum>> = Vec::new();
            let mut calculators: Vec<Box<CalculatorEnum>> = Vec::new();
            while let Some((new_slot, new_calculators)) = buffer.pop() {
                slot = new_slot;
                for calculator in new_calculators {
                    if !already_done.contains(calculator.get_pubkey()) {
                        already_done.insert(*calculator.get_pubkey());
                        if calculator.get_sorted_mints().contains(&&WSOL) {
                            sol_calculators.push(calculator);
                        } else {
                            calculators.push(calculator);
                        }
                    }
                }
            }
            buffer.clear();
            already_done.clear();
            let mut mints_mints = vec![];

            // SOL creates corresponding volumes
            let mut processed_pubkeys = vec![];
            for calculator in sol_calculators.into_iter() {
                let update = price.iterate_volumes_v2(&calculator, slot);
                if update {
                    mints_mints.extend(calculator.get_all_sorted_mints_as_array());
                }
                let pubkey = *calculator.get_pubkey();
                processed_pubkeys.push(pubkey);
                pubkey_to_calculator.insert(pubkey, calculator);
            }

            for calculator in calculators.into_iter() {
                let update = price.iterate_volumes_v2(&calculator, slot);
                if update {
                    mints_mints.extend(calculator.get_all_sorted_mints_as_array());
                }
                let pubkey = *calculator.get_pubkey();
                processed_pubkeys.push(pubkey);
                pubkey_to_calculator.insert(pubkey, calculator);
            }

            mints_mints.sort();
            mints_mints.dedup();

            for mints in mints_mints.iter() {
                for index in 0..price.size {
                    let opportunities = price.find_arbitrage(mints, slot, filter, index);
                    if index == 0 && opportunities.is_empty() {
                        break;
                    }
                    for (opp, volume) in opportunities {
                        //if !opp_already_inserted.contains(&pubkeys) {
                        // BinaryHeap is a max-heap by volume
                        top_opportunities.push((volume, opp));
                        //    opp_already_inserted.insert(pubkeys);
                        //}
                    }

                    while let Some((volume, opportunity)) = top_opportunities.pop() {
                        //if !proceed_opportunities.contains(&opportunity) {
                        debug!("Opportunity: {:?}", opportunity.route);
                        let opportunity_with_calculator =
                            match opportunity.to_opportunity_with_calculators(&pubkey_to_calculator) {
                                Ok(opportunity) => opportunity,
                                Err(err) => {
                                    warn!("Got error during opportunity conversion {:?}", err);
                                    continue;
                                }
                            };

                        let current_cursor: usize = match cursor.to_usize() {
                            Some(cursor) => cursor,
                            None => {
                                warn!("CursorIsNaN");
                                continue;
                            }
                        };

                        let result = tx_arbitrages[current_cursor].send((opportunity_with_calculator, volume));

                        match result {
                            Ok(_) => {
                                cursor += 1;
                                let tx_arbitrages_len = match tx_arbitrages.len().to_u64() {
                                    Some(tx_arbitrages_len) => tx_arbitrages_len,
                                    None => {
                                        warn!("tx_arbitrages_lenIsNaN");
                                        continue;
                                    }
                                };
                                if cursor >= tx_arbitrages_len {
                                    cursor = 0;
                                }
                            }
                            Err(err) => warn!("Got error during sending {:?}", err),
                        }
                    }
                }
            }

            let pool_pubkeys = top_opportunities
                .iter()
                .map(|(_, opp)| opp.route.iter().map(|(pubkey, _, _)| *pubkey).collect::<Vec<Pubkey>>())
                .collect::<Vec<_>>();
            let mut pool_pubkeys = pool_pubkeys.into_iter().flatten().collect::<Vec<Pubkey>>();
            pool_pubkeys.sort();
            pool_pubkeys.dedup();
            price.remove_from_all_volume(&mints_mints, &pool_pubkeys);

            top_opportunities.clear();
        }
    });

    info!("Opportunity calculation thread started...");

    // Use Helius only for the expensive initial GPA snapshot. The live
    // account streams continue to use the configured Chainstack WebSocket.
    let (initial_gpa_rpc, initial_gpa_source) = match std::env::var("HELIUS_RPC_URL") {
        Ok(url) if !url.trim().is_empty() => (url, "HELIUS_RPC_URL"),
        _ => (cfg.rpc.clone(), "configured RPC"),
    };
    info!(
        "Initial GPA source: {}; live account streams remain on configured WebSocket",
        initial_gpa_source
    );
    let mut result = runtime.block_on(async { sync_gpa(&initial_gpa_rpc, &cfg.ws, tx_messages.clone()).await })?;

    let client = solana_client::rpc_client::RpcClient::new(cfg.blockhash_and_simulate_rpc.clone());
    let account_data = client
        .get_account_data(&dex::pump_amm::FEE_CONFIG)
        .or_panic("FailedToFetchPumpAmmFeeConfig");
    let fee_account =
        dex::pump_amm::FeeConfig::deserialize(&account_data).or_panic("FailedToDeserializePumpAmmFeeConfig");
    result
        .pump_amm_fee_config
        .insert(dex::pump_amm::FEE_CONFIG, fee_account);

    info!("Runtimes created: {}", runtimes.len());

    let (tx_token_tick, mut rx_token_tick) = tokio::sync::mpsc::unbounded_channel::<bool>();
    runtime.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            let _ = tx_token_tick.send(true);
        }
    });

    let (tx_no_allowed_token22, mut rx_no_allowed_token22) = tokio::sync::mpsc::unbounded_channel::<AHashSet<Pubkey>>();
    let (tx_dataset, mut rx_dataset) = tokio::sync::mpsc::unbounded_channel::<AHashSet<Vec<u8>>>();
    runtime.spawn(async move {
        let mut token2022_with_transfers = AHashSet::new();
        let mut allowed_token2022 = AHashSet::from_iter(ALLOWED_TOKEN_2022.iter().cloned());
        let mut no_allowed_token22 = AHashSet::new();
        let rpc_client = rpc_client::RpcClient::new(cfg.blockhash_and_simulate_rpc.clone());
        loop {
            select! {
                Some(data) = rx_dataset.recv() => {
                    let mut changed_no_allowed = false;
                    let len = allowed_token2022.len();
                    for item in data {
                        if let Ok(state) = PodStateWithExtensions::<PodAccount>::unpack(item.as_slice()) {
                            if let Ok(_extension) = state.get_extension::<TransferFeeAmount>() {
                                token2022_with_transfers.insert(state.base.mint);
                                if let Ok(account_data) = rpc_client.get_account_data(&state.base.mint).await {
                                    if let Ok(state_mint) = PodStateWithExtensions::<PodMint>::unpack(account_data.as_slice()) {
                                        if let Ok(config) = state_mint.get_extension::<TransferFeeConfig>() {
                                            if config.newer_transfer_fee.transfer_fee_basis_points.0.iter().all(|f| f.is_zero())  {
                                                allowed_token2022.insert(state.base.mint);
                                                if no_allowed_token22.contains(&state.base.mint) {
                                                    no_allowed_token22.remove(&state.base.mint);
                                                    changed_no_allowed = true;
                                                }
                                            } else if !no_allowed_token22.contains(&state.base.mint) {
                                                no_allowed_token22.insert(state.base.mint);
                                                changed_no_allowed = true;
                                            }
                                        }
                                    }
                                }
                            } else {
                                allowed_token2022.insert(state.base.mint);
                            }
                        }
                    }
                    if len < allowed_token2022.len() {
                       let _ = tx_token2022_bc.send(allowed_token2022.clone());
                    }
                    if changed_no_allowed {
                        let _ = tx_no_allowed_token22.send(no_allowed_token22.clone());
                    }
                }
                Some(_signal) = rx_token_tick.recv() => {
                    let tokens = if token2022_with_transfers.len() <= 50 {
                        token2022_with_transfers.iter().cloned().collect::<Vec<_>>()
                    } else {
                        // select random 50 members
                        token2022_with_transfers.iter().choose_multiple(&mut rand::thread_rng(), 50).into_iter().cloned().collect::<Vec<_>>()
                    };
                    info!("Token2022: len {}, transfers {}, allowed {}", tokens.len(), token2022_with_transfers.len(), allowed_token2022.len());
                    let mut changed_allowed = false;
                    let mut changed_no_allowed = false;
                    if let Ok(accounts) = rpc_client.get_multiple_accounts(&tokens).await {
                        for (pubkey, account) in tokens.iter().zip(accounts.into_iter()) {
                            if let Some(account) = account {
                                if let Ok(state_mint) = PodStateWithExtensions::<PodMint>::unpack(account.data.as_slice()) {
                                    if let Ok(config) = state_mint.get_extension::<TransferFeeConfig>() {
                                        if config.newer_transfer_fee.transfer_fee_basis_points.0.iter().all(|f| f.is_zero())  {
                                            if !allowed_token2022.contains(pubkey) {
                                                allowed_token2022.insert(*pubkey);
                                                changed_allowed = true;

                                            }
                                            if no_allowed_token22.contains(pubkey) {
                                                no_allowed_token22.remove(pubkey);
                                                changed_no_allowed = true;
                                            }
                                        } else {
                                            if allowed_token2022.contains(pubkey) {
                                                allowed_token2022.remove(pubkey);
                                                changed_allowed = true;
                                            }
                                            if !no_allowed_token22.contains(pubkey) {
                                                no_allowed_token22.insert(*pubkey);
                                                changed_no_allowed = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    };
                    if changed_allowed {
                        let _ = tx_token2022_bc.send(allowed_token2022.clone());
                    }
                    if changed_no_allowed {
                        let _ = tx_no_allowed_token22.send(no_allowed_token22.clone());
                    }
                }
            }
        }
    });

    // Process arbitrage opportunities

    std::thread::spawn(move || {
        let mut token22_set = AHashSet::new();
        let mut no_allowed_token22 = AHashSet::new();
        let mut data_set = AHashSet::new();
        let rpc_client = solana_client::rpc_client::RpcClient::new(cfg.blockhash_and_simulate_rpc.clone());

        loop {
            let msgs = match rx_messages.recv_timeout(Duration::from_millis(50)) {
                Ok(msgs) => msgs,
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    if let Ok(not_allowed_token) = rx_no_allowed_token22.try_recv() {
                        no_allowed_token22 = not_allowed_token;
                    }
                    continue;
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            };
            let slot = msgs.slot;

            let mut calculators = vec![];
            let mut pools = vec![];

            let messages: Vec<Message> = msgs.message;
            for msg in messages.into_iter() {
                let pool = process_message(&msg, &mut result, &mut token22_set, &mut data_set);
                if let Some(pool) = pool {
                    pools.push(pool);
                }
            }

            for pool in pools.into_iter() {
                add_to_calculator(
                    &pool.pool_type,
                    &pool.pubkey,
                    &mut result,
                    &mut calculators,
                    msgs.slot,
                    &rpc_client,
                );
                if pool.is_new {
                    let _ = tx_pool.send(pool);
                }
            }

            if calculators.is_empty() {
                continue;
            }

            let mut calculators_to_send = vec![];

            for calculator in calculators.iter() {
                let sorted_mints = calculator.get_sorted_mints();
                if sorted_mints.iter().any(|mint| no_allowed_token22.contains(*mint)) {
                    continue;
                }
                // let pubkey = calculator.get_pubkey();
                if !cfg.arbitrage.c3 {
                    if calculator.get_sorted_mints().contains(&&WSOL) {
                        calculators_to_send.push(Box::new(calculator.clone()));
                    }
                } else {
                    calculators_to_send.push(Box::new(calculator.clone()));
                }
                // This should be faster if pubkey is already in the map - need to benchmark
                //pubkey_to_calculators.entry(*pubkey).or_insert(calculator.clone());
            }

            if !calculators_to_send.is_empty() {
                debug!("Sent {:?}", calculators_to_send.len());
                let _ = tx_calculators.send((slot, calculators_to_send));
            }

            if !data_set.is_empty() {
                let _ = tx_dataset.send(data_set.clone());
                data_set.clear();
            }
        }
    });

    // sync_gpa owns the two Chainstack program subscriptions and forwards the
    // same account updates to the calculator pipeline. Keep the process alive.
    runtime.block_on(std::future::pending::<()>());

    Ok(())
}

fn setup_logger(level: LevelFilter) {
    env_logger::Builder::new()
        .filter_level(level)
        .filter_module("reqwest", LevelFilter::Off)
        .format(|buf, record| {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S%.6f");
            writeln!(buf, "{} [{}] - {}", timestamp, record.level(), record.args())
        })
        .init();
}
