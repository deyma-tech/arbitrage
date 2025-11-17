use ahash::AHashSet;
use arb_core::arbitrage::OpportunityWithCalculators;
use arb_core::calculator::{get_pool_types, get_pubkeys};
use bincode::serialize;
use log::{debug, info, warn};
// use num_traits::ToPrimitive;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use std::{sync::atomic::AtomicU64, time::Instant};
use tokio::sync::broadcast::Receiver;
use utils::transaction::check_transaction_size;

use crate::providers::amount_for_flashloan;
use crate::OptimizeResult;
use arb_core::fee::calculate_max_fee;
use config::providers::NextblockConfig;
use config::CONFIG as cfg;
use solana_sdk::signature::Signer;
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::base64::b64_encode;
use utils::constants::{ALLOWED_TOKEN_2022, WSOL};
use utils::now;

use super::{Provider, SetupResult};

#[derive(Clone, Debug)]
pub struct ProviderNextblock {
    pub counter: Arc<AtomicU64>,
    pub setup: Option<SetupResult>,
}
impl ProviderNextblock {
    pub async fn new() -> ProviderNextblock {
        let mut provider = ProviderNextblock {
            setup: None,
            //start_time: Instant::now(),
            counter: Arc::new(AtomicU64::new(0)),
        };
        provider.setup = Some(provider.setup().await);
        provider
    }
}

impl Provider for ProviderNextblock {
    #[inline(always)]
    fn get_filter(&self) -> u64 {
        cfg.bloxroute.filter
    }

    #[inline(always)]
    fn get_execution_threads(&self) -> u64 {
        cfg.bloxroute.execution_threads
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
        _cfg_option: Option<String>,
    ) -> Option<tokio::runtime::Runtime> {
        //let filter = self.get_filter();
        //let filter_sqwos = self.get_filter_sqwos() as i64;
        let filter = cfg.nextblock.filter;

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
        let url = NextblockConfig::get_regions(&region);

        let exec_runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => panic!("Error creating tokio runtime: {err:?}"),
        };

        exec_runtime.spawn({
            async move {
                let mut grpc_client =
                    nextblock_client::create_grpc_client(&url, &cfg.grpc_pem, cfg.nextblock.grpc_auth_token.clone()).await;

                let simulate_rpc =
                    RpcClient::new_with_commitment(cfg.blockhash_and_simulate_rpc.clone(), CommitmentConfig::processed());

                let alt = alt.clone();

                let mut buffer: Vec<(_, _, _, _)> = vec![];

                let timestamp = self.counter.clone();
                timestamp.store(
                    now::as_micros() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );

                'outer: loop {
                    select! {

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

                        Ok(new_blockhash) = rx_blockhash.recv() => {
                            blockhash = new_blockhash;
                        }

                        _size = rx_arbitrage.recv_many(&mut buffer, 100) => {
                            let now = now::as_millis();
                            buffer.retain(|(opportunity, _, _, _)| {
                                now >= opportunity.timestamp && now - opportunity.timestamp < 50
                            });

                            buffer.sort_by(|( _, optimize, _, _), ( _, optimize1, _, _)| optimize.diff.cmp(&optimize1.diff));
                            if let Some((opportunity, optimize, mut alts , _)) = buffer.pop() {
                            debug!("received opportunity route: {} calcs: {} optimize: {:?}", opportunity.calculators.len(), opportunity.slot, optimize);

                            let calculators = opportunity.calculators;

                            let mint_pair_route = opportunity.mint_pair_route.iter().collect::<Vec<_>>();
                            if optimize.diff < filter {
                                warn!("Optimize diff: {} before: {}, filter: {}", optimize.diff, opportunity.diff, filter);
                                continue 'outer
                            }

                            let mut builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());

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
                            let result_diff = optimize.diff as i64;
                            if result_diff < filter as i64 {
                                continue;
                            }

                            let max_fee = calculate_max_fee(balance);
                            let diff = result_diff;

                            alts.insert(0, alt.clone());

                            let mut compute_unit_limit: u64;

                            //if cfg.arbitrage.compute_cu == 0 || compute_units == 0 {
                                compute_unit_limit = if calculators.len() == 2 { 300_000} else { 450_000};
                                if optimize.diff > 10_000_000 {
                                    compute_unit_limit = 600_000;
                                }
                            // } else {
                            //     compute_unit_limit = compute_units as u64;
                            //     // add `cfg.arbitrage.compute_cu %` as reserve
                            //     match cfg.arbitrage.compute_cu.to_f64() {
                            //         Some(compute_cu) => compute_unit_limit += compute_unit_limit.percent(compute_cu),
                            //         None => warn!("ConfigArbitrageComputeUnitsIsNotF64"),                                        
                            //     };
                            // }

                            let tip_input = arb_core::tip::TipInput{
                                diff,
                                max_tip: max_fee,
                                priority_fee_pct: cfg.nextblock.priority_fee_percent, //  2,
                                min_ratio: cfg.nextblock.tip_min_percent, // 20,
                                max_ratio: cfg.nextblock.tip_max_percent, //50,
                                provider: arb_core::questdb::ExecutionProviderType::Nextblock,
                                compute_unit_limit,
                                max_priority_fee: Some(cfg.nextblock.max_priority_fee), // 50_000),
                            };

                            let mut tip_result = match arb_core::tip::compute_tip(&tip_input) {
                                Ok(tip_result) => tip_result,
                                Err(err) => {
                                    warn!("Got error during building priority tx: {}", err);
                                    continue;
                                }
                            };

                            let use_flash_loan = optimize.amount >= cfg.arbitrage.min_amount_for_flashloan;

                            let tip_result_cu = tip_result.compute_unit_limit as u32;

                            //let simulated_cu = 0;

                            if use_flash_loan {
                                let amount = amount_for_flashloan(optimize.amount);
                                builder.push_ix(preparation.to_floashloan_ix(amount, tip_result.total_tip, pool_ata, pool, token_ata_wsol));
                            } else {
                                builder.push_ix(preparation.to_instruction(tip_result.total_tip));
                            }

                            if cfg.nextblock.simulate {
                                let start = Instant::now();
                                let mut simulation_builder = builder.clone();
                                simulation_builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                                simulation_builder.add_compute_unit_price((tip_result.compute_unit_price).min(1_200_000));
                                //simulation_builder.push_ix(preparation.to_floashloan_ix(amount, tip_result.total_tip, pool_ata, pool, token_ata_wsol));
                                simulation_builder.add_nextblock_tip_ix(tip_result.provider_tip);

                                let txn = simulation_builder.prepare_tx(&keypair, &alts, blockhash);
                                if let Ok(txn) = txn {
                                    let _ = match check_transaction_size(&txn) {
                                        Ok(buffer) => buffer,
                                        Err(err) => {
                                            warn!("{:?}, {:?} {:?}", err, get_pool_types(&calculators), get_pubkeys(&calculators));
                                            continue 'outer;
                                        }
                                    };
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
                                                        let new_cu = cu + 20_000;
                                                        debug!("CCUL: {:?}, SCUL: {:?}, took: {:?}", tip_result_cu, new_cu, elapsed);
                                                        tip_result.compute_unit_limit = new_cu;
                                                    }
                                                } else {
                                                    // warn!("Simulation Error: {:?}", simulation.value.err);
                                                    // extra_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                    continue;
                                        }
                                    }
                                }
                            }

                            builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                            builder.add_compute_unit_price(tip_result.compute_unit_price);
                            //builder.push_ix(preparation.to_floashloan_ix(amount, tip_result.total_tip, pool_ata, pool, token_ata_wsol ));
                            builder.add_nextblock_tip_ix(tip_result.provider_tip);
                            let txn = builder.prepare_tx(&keypair, &alts, blockhash);
                                if let Ok(txn) = txn {
                                    let _ = match check_transaction_size(&txn) {
                                        Ok(buffer) => buffer,
                                        Err(err) => {
                                            warn!("{:?}, {:?} {:?}", err, get_pool_types(&calculators), get_pubkeys(&calculators));
                                            continue 'outer;
                                        }
                                    };
                                    let tx = serialize(&txn).unwrap();
                                    let msg = b64_encode(tx);

                                    let request = nextblock_protos::api::PostSubmitRequest{
                                        transaction: Some(nextblock_protos::api::TransactionMessage { content: msg, is_cleanup: false }),
                                        skip_pre_flight: false,
                                        front_running_protection: Some(false),
                                        experimental_front_running_protection: Some(false),
                                        snipe_transaction: Some(false),
                                        disable_retries: Some(false),
                                        revert_on_fail: Some(cfg.nextblock.revert_on_fail),
                                    };
                                    let now = now::as_micros() as u64;
                                    let last_timestamp = timestamp.load(std::sync::atomic::Ordering::Relaxed);
                                    if last_timestamp > now || now - last_timestamp < 1100 {
                                        continue 'outer;
                                    }
                                    timestamp.store(now::as_micros() as u64, std::sync::atomic::Ordering::Relaxed);
                                    let result = grpc_client.post_submit_v2(request).await;
                                    if let Ok(response) = result {
                                        info!("NB: {:?}", response);
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
