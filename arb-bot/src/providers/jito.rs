use ahash::AHashSet;
use jito_searcher_client::{get_searcher_client_without_interceptor, send_bundle_no_wait};
use log::{debug, info, warn};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast::Receiver;
use utils::constants::ALLOWED_TOKEN_2022;
use utils::safe::ResultExt;
use utils::transaction::check_transaction_size;

use crate::OptimizeResult;
use arb_core::{
    arbitrage::OpportunityWithCalculators,
    calculator::{get_pool_types, get_pubkeys},
    fee::calculate_max_fee,
};
use config::{JitoConfig, CONFIG as cfg};
use solana_sdk::signature::Signer;
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::{constants::WSOL, now, queue::Queue};

// use crate::process::process_jito;
use super::{Provider, SetupResult};
use crate::providers::amount_for_flashloan;
//use crate::watch_dog::{ThreadWatchDogType, WATCHDOG as watch_dog};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ProviderJito {
    pub counter: Arc<AtomicU64>,
    pub setup: Option<SetupResult>,
    // pub channels: Vec<tokio::sync::broadcast::Sender<VersionedTransaction>>,
}

impl ProviderJito {
    pub async fn new() -> ProviderJito {
        let mut provider = ProviderJito {
            setup: None,
            //start_time: Instant::now(),
            counter: Arc::new(AtomicU64::new(0)),
            // channels,
        };
        provider.setup = Some(provider.setup().await);
        provider
    }

    pub async fn get_channels() {}
}

impl Provider for ProviderJito {
    #[inline(always)]
    fn get_filter(&self) -> u64 {
        cfg.jito.filter
    }

    #[inline(always)]
    fn get_execution_threads(&self) -> u64 {
        cfg.jito.execution_threads
    }

    fn run(
        self,
        mut rx_arbitrage: UnboundedReceiver<(
            OpportunityWithCalculators,
            OptimizeResult,
            Vec<AddressLookupTableAccount>,
            Vec<AddressLookupTableAccount>,
        )>,
        mut rx_token2022_bc: Receiver<AHashSet<Pubkey>>,
        cfg_option: Option<String>,
    ) -> Option<tokio::runtime::Runtime> {
        let filter = cfg.jito.filter;

        let setup = match self.setup {
            Some(setup) => setup,
            None => panic!("Failed to load setup"),
        };
        let keypair = setup.keypair;
        let token_ata_wsol = setup.token_ata_wsol;
        //let regions = setup.regions;

        let mut mint_to_ata = setup.mint_to_ata;

        let mut balance = setup.balance;
        let mut blockhash = setup.blockhash;
        let mut rx_balance = setup.tx_balance.subscribe();
        let mut rx_blockhash = setup.tx_blockhash.subscribe();

        let flashloan_keys = setup.flashloan_keys;
        let (pool, pool_ata) = match flashloan_keys.get(&WSOL) {
            Some((pool, pool_ata)) => (*pool, *pool_ata),
            None => panic!("No flashloan keys found for WSOL"),
        };

        let alt = setup.alt;

        let mut allowed_token2022 = AHashSet::from_iter(ALLOWED_TOKEN_2022.iter().cloned());

        let region = cfg.get_region();

        let primary_jito = if let Some(cfg_region) = cfg_option {
            cfg_region
        } else {
            let (primary_jito, _secondary_jito) = JitoConfig::get_regions(&region);
            primary_jito
        };

        let exec_runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => {
                panic!("Error creating tokio runtime: {err:?}");
            }
        };

        exec_runtime.spawn({
            async move {

                let mut primary_searcher_client = get_searcher_client_without_interceptor(&primary_jito, &cfg.grpc_pem)
                    .await
                    .or_panic("FailedToLoadPrimarySearcherClient");

                info!("Jito client connected to: {}", primary_jito);

                let timestamp = self.counter.clone();
                timestamp.store(now::as_micros() as u64, std::sync::atomic::Ordering::Relaxed);

                let _queue = Queue::new(20);

                let simulate_rpc =
                    RpcClient::new_with_commitment(cfg.blockhash_and_simulate_rpc.clone(), CommitmentConfig::processed());

                let alt = alt.clone();
                let mut buffer: Vec<(_, _, _, _)> = vec![];
                'outer: loop {
                    select! {

                        //
                        Ok(new_balance) = rx_balance.recv() => {
                            if balance != new_balance {
                                debug!("New balance: {} -> {}", balance, new_balance);
                                balance = new_balance;
                            }
                        }

                        Ok(token_set) = rx_token2022_bc.recv() => {
                            allowed_token2022 = token_set;
                            debug!("Updated allowed_token2022: {:?}", allowed_token2022);
                        }

                        // blockhash
                        Ok(new_blockhash) = rx_blockhash.recv() => {
                            blockhash = new_blockhash;
                        }

                        // opportunities
                        _size = rx_arbitrage.recv_many(&mut buffer, 100) => {

                            let now = now::as_millis();
                            buffer.retain(|(opportunity, _, _, _)| {
                                now >= opportunity.timestamp && now - opportunity.timestamp < 50
                            });
                            if let Some((opportunity, optimize, mut alts , _optional_alts))  = buffer.pop() {
                            debug!("received opportunity route: {} slot: {} optimize: {:?}", opportunity.calculators.len(), opportunity.slot, optimize);

                            let calculators = opportunity.calculators;
                            let mint_pair_route = opportunity.mint_pair_route.iter().collect::<Vec<_>>();

                            if optimize.diff < filter {
                                warn!("Optimize diff: {} before: {}, filter: {}", optimize.diff, opportunity.diff, filter);
                                continue 'outer
                            }

                            let mut builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());
                            let max_fee = calculate_max_fee(balance);

                            let preparation = match arb_core::arbitrage::process_arbitrage_v6(&calculators, &WSOL, &mut builder, &mut mint_to_ata, &mint_pair_route, &allowed_token2022, optimize.amounts, optimize.remaining_accounts) {
                                Ok(preparation) => preparation,
                                Err(err) => {
                                    warn!("Error processing arbitrage: {:?} pools: {:?}, types: {:?}, volume: {:?}", err, 
                                        calculators.iter().map(|c| c.get_pubkey()).collect::<Vec<_>>(),
                                        calculators.iter().map(|c| c.get_pool_type()).collect::<Vec<_>>(),
                                        optimize.amount,
                                    );
                                    continue 'outer;
                                }
                            };

                            let diff = optimize.diff as i64;
                            
                            alts.insert(0, alt.clone());

                            let use_flash_loan = true; // optimize.amount >= cfg.arbitrage.min_amount_for_flashloan;
                            let mut compute_unit_limit = if calculators.len() == 2 { 300_000} else { 450_000};
                            if optimize.diff > 10_000_000 {
                                compute_unit_limit = 600_000;
                            }

                            let tip_input = arb_core::tip::TipInput{
                                diff,
                                max_tip: max_fee,
                                priority_fee_pct: cfg.jito.priority_fee_percent, // 2,
                                min_ratio: cfg.jito.tip_min_percent, // 20,
                                max_ratio: cfg.jito.tip_max_percent, //  60,
                                provider: arb_core::questdb::ExecutionProviderType::Jito,
                                compute_unit_limit,
                                max_priority_fee: None,
                            };

                            let tip_result= arb_core::tip::compute_tip(&tip_input);

                            if let Ok(mut tip_result) = tip_result {
                                if use_flash_loan {
                                    let amount = amount_for_flashloan(optimize.amount);
                                    builder.push_ix(preparation.to_floashloan_ix(amount, tip_result.total_tip, pool_ata, pool, token_ata_wsol));
                                } else {
                                    builder.push_ix(preparation.to_instruction(tip_result.total_tip));
                                }
                                if cfg.jito.simulate {
                                    let start = Instant::now();
                                    let mut simulation_builder = builder.clone();
                                    simulation_builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                                    simulation_builder.add_compute_unit_price((tip_result.compute_unit_price).min(1_200_000));
                                    simulation_builder.add_jito_tip_ix(tip_result.provider_tip);

                                    let txn = simulation_builder.prepare_tx(&keypair, &alts, blockhash);
                                    if let Ok(txn) = txn {
                                        let simulation = simulate_rpc.simulate_transaction(&txn).await;
                                        if let Ok(simulation) = simulation {
                                            let mut improve_cu = false;
                                            if simulation.value.err.is_none() {
                                                improve_cu = true;
                                            }
                                            if let Some(err) = simulation.value.err {
                                                if err.to_string().contains("InvalidAccountData") || err.to_string().contains("invalid account data") {
                                                    improve_cu = true;
                                                }
                                            }
                                            if improve_cu {
                                                let cu = simulation.value.units_consumed;
                                                if let Some(cu) = cu {
                                                    if cu < 50_000 {
                                                        // extra_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                        continue;
                                                    }
                                                    let elapsed = start.elapsed();
                                                    let new_cu = cu + 25_000;
                                                    debug!("CCUL: {:?}, SCUL: {:?}, took: {:?}", tip_result.compute_unit_limit, new_cu, elapsed);
                                                    tip_result.compute_unit_limit = new_cu;
                                                }
                                            } else {
                                                continue;
                                            }
                                        }
                                    }
                                }

                                builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                                builder.add_compute_unit_price(tip_result.compute_unit_price);

                                builder.add_jito_tip_ix(tip_result.provider_tip);

                                let now = now::as_micros() as u64;
                                let last_timestamp = timestamp.load(std::sync::atomic::Ordering::Relaxed);
                                if last_timestamp > now || now - last_timestamp < 1100 {
                                    continue 'outer;
                                }
                                timestamp.store(now::as_micros() as u64, std::sync::atomic::Ordering::Relaxed);

                                let txn = builder.prepare_tx(&keypair, &alts, blockhash);

                                if let Ok(txn) = txn {

                                    let _ = match check_transaction_size(&txn) {
                                        Ok(buffer) => buffer,
                                        Err(err) => {
                                            warn!("{:?}, {:?} {:?}", err, get_pool_types(&calculators), get_pubkeys(&calculators));
                                            continue 'outer;
                                        }
                                    };
                                    let bundle = [txn];
                                    let result = send_bundle_no_wait(&bundle, &mut primary_searcher_client).await;
                                    info!("Primary Tx result: {:?}", result);
                                }
                            }
                                buffer.clear();
                                }
                        }
                    }
                }
            }
        });

        Some(exec_runtime)
    }
}
