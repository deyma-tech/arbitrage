use ahash::AHashSet;
use arb_core::arbitrage::OpportunityWithCalculators;
use arb_core::calculator::{get_pool_types, get_pubkeys};
use bloxroute_client::instruction::versioned_tx_to_string;
use log::{debug, info, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::message::AddressLookupTableAccount;
use std::sync::Arc;
use std::{sync::atomic::AtomicU64, time::Instant};
use utils::transaction::check_transaction_size;

use crate::OptimizeResult;
use arb_core::fee::calculate_max_fee;
use config::CONFIG as cfg;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::VersionedTransaction;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::constants::{ALLOWED_TOKEN_2022, WSOL};
use utils::now;

use super::{require_min_net_profit, Provider, SetupResult};

#[derive(Clone, Debug)]
pub struct ProviderBloxroute {
    pub counter: Arc<AtomicU64>,
    pub setup: Option<SetupResult>,
}
impl ProviderBloxroute {
    pub async fn new() -> ProviderBloxroute {
        let mut provider = ProviderBloxroute {
            setup: None,
            //start_time: Instant::now(),
            counter: Arc::new(AtomicU64::new(0)),
        };
        provider.setup = Some(provider.setup().await);
        provider
    }
}

impl Provider for ProviderBloxroute {
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
        let filter = cfg.bloxroute.filter;
        //let filter_jito_secondary = cfg.bloxroute.filter_jito_secondary;
        // let filter_paladin_secondary = cfg.bloxroute.filter_paladin_secondary;
        // let filter_paladin = cfg.bloxroute.filter_paladin;
        // let filter_swqos = cfg.bloxroute.filter_swqos;

        let setup = match self.setup {
            Some(setup) => setup,
            None => panic!("Failed to load setup"),
        };

        let keypair = setup.keypair;
        let token_ata_wsol = setup.token_ata_wsol;
        //let regions = setup.regions;

        let mut balance = setup.balance;
        let mut blockhash = setup.blockhash;
        let mut rx_balance = setup.tx_balance.subscribe();
        let mut rx_blockhash = setup.tx_blockhash.subscribe();

        let mut allowed_token2022 = AHashSet::from_iter(ALLOWED_TOKEN_2022.iter().cloned());

        let alt = setup.alt;

        let simulate_rpc =
            RpcClient::new_with_commitment(cfg.blockhash_and_simulate_rpc.clone(), CommitmentConfig::processed());

        let mut timestamp = now::as_millis();

        std::thread::spawn(move || {
            let auth_token = cfg.bloxroute.grpc_auth_token.clone();
            let mut headers = HeaderMap::new();
            headers.insert("Authorization", HeaderValue::from_str(&auth_token).unwrap());
            let client = reqwest::blocking::Client::builder()
                .default_headers(headers)
                .pool_idle_timeout(None)
                .pool_max_idle_per_host(200)
                .tcp_keepalive(Some(std::time::Duration::from_secs(15)))
                .build()
                .unwrap();

            let alt = alt.clone();

            'outer: loop {
                if !rx_arbitrage.is_empty() {
                    if let Some((opportunity, optimize, mut alts, _optional_alts)) = rx_arbitrage.blocking_recv() {
                        let now = now::as_millis();
                        if now >= opportunity.timestamp && now - opportunity.timestamp > 50 {
                            continue;
                        }
                        if now > timestamp && now - timestamp < 1000 {
                            continue;
                        }
                        timestamp = now;

                        debug!(
                            "received opportunity route: {} calcs: {} optimize: {:?}",
                            opportunity.calculators.len(),
                            opportunity.slot,
                            optimize
                        );

                        let calculators = opportunity.calculators;

                        if optimize.diff < filter {
                            warn!(
                                "Optimize diff: {} before: {}, filter: {}",
                                optimize.diff, opportunity.diff, filter
                            );
                            continue 'outer;
                        }

                        let mut builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());

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
                                warn!("Error processing arbitrage: {:?}", err);
                                // extra_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                continue 'outer;
                            }
                        };

                        let jito_txn: Option<VersionedTransaction>;
                        let max_fee = calculate_max_fee(balance);

                        let diff = optimize.diff as i64;

                        alts.insert(0, alt.clone());

                        let mut compute_unit_limit = if calculators.len() == 2 { 300_000 } else { 450_000 };
                        if optimize.diff > 10_000_000 {
                            compute_unit_limit = 600_000;
                        }

                        let tip_input = arb_core::tip::TipInput {
                            diff,
                            max_tip: max_fee,
                            priority_fee_pct: cfg.bloxroute.priority_fee_percent, //  2,
                            min_ratio: cfg.bloxroute.tip_min_percent,             // 20,
                            max_ratio: cfg.bloxroute.tip_max_percent,             //50,
                            provider: arb_core::questdb::ExecutionProviderType::BloxrouteJito,
                            compute_unit_limit,
                            max_priority_fee: Some(cfg.bloxroute.max_priority_fee), // 50_000),
                        };

                        let mut tip_result = match arb_core::tip::compute_tip(&tip_input) {
                            Ok(tip_result) => tip_result,
                            Err(err) => {
                                warn!("Got error during building priority tx: {}", err);
                                continue;
                            }
                        };

                        let net_profit = match require_min_net_profit(optimize.diff, tip_result.total_tip) {
                            Ok(net_profit) => net_profit,
                            Err(err) => {
                                debug!("Bloxroute net-profit reject: {}", err);
                                continue 'outer;
                            }
                        };
                        debug!(
                            "Bloxroute economics: gross={}, cost={}, net={}",
                            optimize.diff, tip_result.total_tip, net_profit
                        );

                        let bloxroute_tip = tip_result.provider_tip;

                        builder.push_ix(preparation.to_instruction(tip_result.total_tip));

                        if cfg.bloxroute.simulate {
                            let start = Instant::now();
                            let mut simulation_builder = builder.clone();
                            simulation_builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                            simulation_builder.add_compute_unit_price((tip_result.compute_unit_price).min(1_200_000));
                            simulation_builder.add_bloxroute_tip_ix(/*tip_result.provider_tip*/ 1);

                            let txn = simulation_builder.prepare_tx(&keypair, &alts, blockhash);
                            if let Ok(txn) = txn {
                                let _ = match check_transaction_size(&txn) {
                                    Ok(buffer) => buffer,
                                    Err(err) => {
                                        warn!(
                                            "{:?}, {:?} {:?}",
                                            err,
                                            get_pool_types(&calculators),
                                            get_pubkeys(&calculators)
                                        );
                                        continue 'outer;
                                    }
                                };
                                let simulation = simulate_rpc.simulate_transaction(&txn);
                                if let Ok(simulation) = simulation {
                                    let mut improve_cu = false;
                                    if simulation.value.err.is_none() {
                                        improve_cu = true;
                                    }
                                    if let Some(err) = simulation.value.err {
                                        if err.to_string().contains("InvalidAccountData")
                                            || err.to_string().contains("invalid account data")
                                        {
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
                                            debug!(
                                                "CCUL: {:?}, SCUL: {:?}, took: {:?}",
                                                tip_result.compute_unit_limit, new_cu, elapsed
                                            );
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
                        if bloxroute_tip > 1_000_000 {
                            if balance > tip_result.provider_tip + 1_000_000 {
                                builder.add_bloxroute_tip_ix(tip_result.provider_tip);
                            } else {
                                let mut tip_builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());
                                tip_builder.add_compute_unit_limit(70_000);
                                tip_builder.add_compute_unit_price(1);
                                tip_builder.add_compute_accounts_data_size_limit(1_024_000);
                                let _ = tip_builder.close_ata_ix(&token_ata_wsol);
                                tip_builder.add_create_ata_if_not_exists(&WSOL);
                                tip_builder.add_system_transfer_ix(cfg.wallet_manager.max_wsol, &token_ata_wsol);
                                tip_builder.add_sync_native(&token_ata_wsol);
                                tip_builder.add_bloxroute_tip_ix(tip_result.provider_tip);
                                let _tip_txn = tip_builder.prepare_tx(&keypair, &[], blockhash);
                                // if let Ok(txn) = tip_txn {
                                //     ttxn = Some(txn);
                                // }
                            }
                            let txn = builder.prepare_tx(&keypair, &alts, blockhash);
                            match txn {
                                Ok(txn) => {
                                    let _ = match check_transaction_size(&txn) {
                                        Ok(buffer) => buffer,
                                        Err(err) => {
                                            warn!(
                                                "{:?}, {:?} {:?}",
                                                err,
                                                get_pool_types(&calculators),
                                                get_pubkeys(&calculators)
                                            );
                                            // extra_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                            continue 'outer;
                                        }
                                    };
                                    jito_txn = Some(txn)
                                }
                                Err(err) => {
                                    warn!("got error during building priority tx: {}", err);
                                    // extra_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    continue;
                                }
                            }
                        } else {
                            warn!("Bx tip too low: {} {} {:?}", bloxroute_tip, diff, tip_result);
                            continue;
                        }
                        if let Some(jito_txn) = jito_txn {
                            let tx_message = versioned_tx_to_string(&jito_txn);
                            let request_json = json!({
                                "transaction": { "content": tx_message, "isCleanup": false },
                                "skipPreFlight": false,
                                "frontRunningProtection": true,
                                "useStakedRPCs": false,
                                "fastBestEffort": false,
                                "revertProtection": true,
                                "submitProtection": "SP_LOW"
                            });
                            match client
                                .post(format!("{}/submit", cfg.bloxroute.ofr_http_server))
                                .json(&request_json)
                                .send()
                            {
                                Ok(res) => info!("Bx-jito {:?}", res),
                                Err(err) => warn!("Bx-jito error {:?}", err),
                            };
                        }
                    } // opp
                } else if !rx_token2022_bc.is_empty() {
                    if let Ok(token_set) = rx_token2022_bc.blocking_recv() {
                        allowed_token2022 = token_set;
                        debug!("Updated allowed_token2022: {:?}", allowed_token2022);
                    }
                } else if !rx_balance.is_empty() {
                    if let Ok(new_balance) = rx_balance.blocking_recv() {
                        if balance != new_balance {
                            debug!("New balance: {} -> {}", balance, new_balance);
                            balance = new_balance;
                        }
                    }
                } else if !rx_blockhash.is_empty() {
                    if let Ok(new_blockhash) = rx_blockhash.blocking_recv() {
                        blockhash = new_blockhash;
                    }
                }
            }
        });

        None
    }
}
