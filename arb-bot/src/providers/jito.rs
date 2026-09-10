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
use utils::{now, queue::Queue};

// use crate::process::process_jito;
use super::{fetch_nonce_context, prepare_transaction, require_min_net_profit, NonceLease, Provider, SetupResult};
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
        //let regions = setup.regions;

        let mut balance = setup.balance;
        let mut blockhash = setup.blockhash;
        let mut rx_balance = setup.tx_balance.subscribe();
        let mut rx_blockhash = setup.tx_blockhash.subscribe();

        let alt = setup.alt;
        let nonce_manager = setup.nonce_manager.clone();

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
                            if optimize.diff < filter {
                                warn!("Optimize diff: {} before: {}, filter: {}", optimize.diff, opportunity.diff, filter);
                                continue 'outer
                            }

                            let mut builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());
                            let max_fee = calculate_max_fee(balance);

                            let preparation = match super::prepare_executor_v2(
                                &calculators,
                                keypair.pubkey(),
                                optimize.diff,
                                &optimize.amounts,
                                &optimize.remaining_accounts,
                                &allowed_token2022,
                            ) {
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
                                let net_profit = match require_min_net_profit(optimize.diff, tip_result.total_tip) {
                                    Ok(net_profit) => net_profit,
                                    Err(err) => {
                                        debug!("Jito net-profit reject: {}", err);
                                        continue 'outer;
                                    }
                                };
                                debug!(
                                    "Jito economics: gross={}, cost={}, net={}",
                                    optimize.diff,
                                    tip_result.total_tip,
                                    net_profit
                                );
                                let mut nonce_lease = match fetch_nonce_context(&simulate_rpc, &nonce_manager).await {
                                    Ok(context) => context,
                                    Err(err) => {
                                        warn!("Jito nonce read failed: {:?}, route={:?}", err, get_pubkeys(&calculators));
                                        continue 'outer;
                                    }
                                };
                                let nonce_context = nonce_lease.as_ref().map(NonceLease::context);
                                if cfg.jito.simulate {
                                    let start = Instant::now();
                                    let mut simulation_builder = builder.clone();
                                    simulation_builder.push_ix(preparation.to_instruction(tip_result.total_tip));
                                    simulation_builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                                    simulation_builder.add_compute_unit_price((tip_result.compute_unit_price).min(1_200_000));
                                    simulation_builder.add_jito_tip_ix(tip_result.provider_tip);

                                    let txn = prepare_transaction(&mut simulation_builder, &keypair, &alts, blockhash, nonce_context);
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
                                                    tip_result.set_final_compute_unit_limit(new_cu);
                                                }
                                            } else {
                                                continue;
                                            }
                                        }
                                    }
                                }

                                builder.push_ix(preparation.to_instruction(tip_result.total_tip));
                                builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                                builder.add_compute_unit_price(tip_result.compute_unit_price);

                                builder.add_jito_tip_ix(tip_result.provider_tip);

                                let now = now::as_micros() as u64;
                                let last_timestamp = timestamp.load(std::sync::atomic::Ordering::Relaxed);
                                if last_timestamp > now || now - last_timestamp < 1100 {
                                    continue 'outer;
                                }
                                timestamp.store(now::as_micros() as u64, std::sync::atomic::Ordering::Relaxed);

                                let txn = prepare_transaction(&mut builder, &keypair, &alts, blockhash, nonce_context);

                                if let Ok(txn) = txn {

                                    let _ = match check_transaction_size(&txn) {
                                        Ok(buffer) => buffer,
                                        Err(err) => {
                                            warn!("{:?}, {:?} {:?}", err, get_pool_types(&calculators), get_pubkeys(&calculators));
                                            continue 'outer;
                                        }
                                    };
                                    let signature = txn.signatures.first().copied();
                                    let bundle = [txn];
                                    let result = send_bundle_no_wait(&bundle, &mut primary_searcher_client).await;
                                    match result {
                                        Ok(result) => {
                                            info!("Jito bundle accepted: {:?}", result);
                                            if let Some(signature) = signature {
                                                if let Some(lease) = nonce_lease.as_mut() {
                                                    match lease
                                                        .wait_for_finalized(
                                                            &simulate_rpc,
                                                            &signature,
                                                            std::time::Duration::from_millis(
                                                                cfg.nonce_confirmation_timeout_ms,
                                                            ),
                                                        )
                                                        .await
                                                    {
                                                        Ok(()) => info!("Jito transaction finalized: signature={signature}"),
                                                        Err(err) => warn!("Jito transaction finalization failed: signature={signature}, error={err}"),
                                                    }
                                                }
                                            } else if let Some(lease) = nonce_lease.as_mut() {
                                                lease.quarantine();
                                            }
                                        }
                                        Err(err) => {
                                            if let Some(lease) = nonce_lease.as_mut() {
                                                lease.quarantine();
                                            }
                                            warn!("Jito bundle submission failed: {:?}", err);
                                        }
                                    }
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
