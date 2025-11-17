use super::{Provider, SetupResult};
use crate::providers::amount_for_flashloan;
use crate::OptimizeResult;
use ahash::AHashSet;
use arb_core::arbitrage::OpportunityWithCalculators;
use arb_core::calculator::{get_pool_types, get_pubkeys};
use arb_core::fee::calculate_max_fee;
use config::{JitoQuicknodeConfig, CONFIG as cfg};
use log::{debug, info, warn};
use reqwest::header;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::VersionedTransaction;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::constants::{ALLOWED_TOKEN_2022, WSOL};
use utils::now;
// use utils::math::Percent;
use utils::transaction::{
    check_transaction_size, generate_body_for_jito_from_data, generate_body_for_jito_from_data_v2,
};

#[derive(Clone, Debug)]
pub struct ProviderJitoQuicknode {
    pub counter: Arc<AtomicU64>,
    pub setup: Option<SetupResult>,
}

impl ProviderJitoQuicknode {
    pub async fn new() -> ProviderJitoQuicknode {
        let mut provider = ProviderJitoQuicknode {
            setup: None,
            //start_time: Instant::now(),
            counter: Arc::new(AtomicU64::new(0)),
        };
        provider.setup = Some(provider.setup().await);
        provider
    }
}

impl Provider for ProviderJitoQuicknode {
    #[inline(always)]
    fn get_filter(&self) -> u64 {
        cfg.jito_quicknode.filter
    }

    #[inline(always)]
    fn get_execution_threads(&self) -> u64 {
        cfg.jito_quicknode.execution_threads
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
        let filter = cfg.jito_quicknode.filter;

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

        let simulate_rpc =
            RpcClient::new_with_commitment(cfg.blockhash_and_simulate_rpc.clone(), CommitmentConfig::processed());

        let mut client_builder = reqwest::blocking::ClientBuilder::new()
            .tcp_keepalive(Duration::from_secs(120))
            .default_headers(header::HeaderMap::from_iter(vec![(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/json"),
            )]));

        if !cfg.jito_quicknode.quicknode_override_ip.is_empty() {
            let domain = cfg.jito_quicknode.quicknode_rpc.split('/').nth(2).unwrap_or_else(|| {
                panic!(
                    "Failed to extract domain from quicknode_rpc: {}",
                    cfg.jito_quicknode.quicknode_rpc
                );
            });
            client_builder = client_builder.resolve(
                domain,
                cfg.jito_quicknode
                    .quicknode_override_ip
                    .parse::<SocketAddr>()
                    .unwrap_or_else(|_| {
                        panic!(
                            "Invalid quicknode override IP: {}",
                            cfg.jito_quicknode.quicknode_override_ip
                        )
                    }),
            );
        }

        let client = match client_builder.build() {
            Ok(client) => client,
            Err(err) => panic!("Error creating reqwest client: {err:?}"),
        };

        std::thread::spawn(move || {
            let region = cfg.get_region();
            let (jito_quicknode_region, _secondary_regions) = JitoQuicknodeConfig::get_regions(&region);
            let mut timestamp = now::as_millis();

            let alt = alt.clone();
            let mut buffer: Vec<(_, _, _, _)> = vec![];

            'outer: loop {
                if !rx_arbitrage.is_empty() {
                    let _size = rx_arbitrage.blocking_recv_many(&mut buffer, 100);

                    let now = now::as_millis();
                    if now > timestamp && now - timestamp < 1000 {
                        continue;
                    }

                    buffer.retain(|(opportunity, _, _, _)| {
                        now >= opportunity.timestamp && now - opportunity.timestamp < 50
                    });

                    timestamp = now;

                    buffer.sort_by(|(_, optimize, _, _), (_, optimize1, _, _)| optimize.diff.cmp(&optimize1.diff));
                    if let Some((opportunity, optimize, mut alts, mut _optional_alts)) = buffer.pop() {
                        buffer.clear();

                        debug!(
                            "Received opportunity: route: {} at slot: {} optimize: {:?}",
                            opportunity.calculators.len(),
                            opportunity.slot,
                            optimize
                        );

                        let calculators = opportunity.calculators;
                        // calculators.reverse();
                        let mint_pair_route = opportunity.mint_pair_route.iter().collect::<Vec<_>>();

                        if optimize.diff < filter {
                            warn!(
                                "Optimize diff: {} before: {}, filter: {}",
                                optimize.diff, opportunity.diff, filter
                            );
                            // extra_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            continue 'outer;
                        }

                        if optimize.diff < balance * 2 {
                            let mut builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());
                            let max_fee = calculate_max_fee(balance);

                            let preparation = match arb_core::arbitrage::process_arbitrage_v6(
                                &calculators,
                                &WSOL,
                                &mut builder,
                                &mut mint_to_ata,
                                &mint_pair_route,
                                &allowed_token2022,
                                optimize.amounts,
                                optimize.remaining_accounts,
                            ) {
                                Ok(preparation) => preparation,
                                Err(err) => {
                                    warn!("Error processing arbitrage: {:?}", err);
                                    continue 'outer;
                                }
                            };

                            alts.insert(0, alt.clone());

                            let mut compute_unit_limit: u64;
                            compute_unit_limit = if calculators.len() == 2 { 240_000 } else { 390_000 };
                            if optimize.diff > 100_000_000 {
                                compute_unit_limit = 490_000;
                            }

                            let tip_input = arb_core::tip::TipInput {
                                diff: optimize.diff as i64,
                                max_tip: max_fee,
                                priority_fee_pct: cfg.jito_quicknode.priority_fee_percent, // 2,
                                min_ratio: cfg.jito_quicknode.tip_min_percent,             // 20,
                                max_ratio: cfg.jito_quicknode.tip_max_percent,             //  60,
                                provider: arb_core::questdb::ExecutionProviderType::JitoQuicknode,
                                compute_unit_limit,
                                max_priority_fee: Some(cfg.jito_quicknode.max_priority_fee), // 50_000),
                            };
                            let tip_result = arb_core::tip::compute_tip(&tip_input);

                            let use_flash_loan = optimize.amount >= cfg.arbitrage.min_amount_for_flashloan;

                            if let Ok(mut tip_result) = tip_result {
                                if use_flash_loan {
                                    let amount = amount_for_flashloan(optimize.amount);
                                    builder.push_ix(preparation.to_floashloan_ix(
                                        amount,
                                        tip_result.total_tip,
                                        pool_ata,
                                        pool,
                                        token_ata_wsol,
                                    ));
                                } else {
                                    builder.push_ix(preparation.to_instruction(tip_result.total_tip));
                                }
                                if cfg.jito_quicknode.simulate {
                                    let start = Instant::now();
                                    let mut simulation_builder = builder.clone();
                                    simulation_builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                                    simulation_builder
                                        .add_compute_unit_price((tip_result.compute_unit_price).min(1_200_000));
                                    simulation_builder.add_jito_tip_ix(/*tip_result.provider_tip*/ 1);
                                    let txn = simulation_builder.prepare_tx(&keypair, &alts, blockhash);
                                    if let Ok(txn) = txn {
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
                                                        continue;
                                                    }
                                                    let elapsed = start.elapsed();
                                                    let mut new_cu = cu + 6_000;
                                                    if optimize.diff > 10_000_000 {
                                                        new_cu += 4000;
                                                    }
                                                    if optimize.diff > 1_000_000_000 {
                                                        new_cu += 10_000;
                                                    }
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
                                builder.add_compute_unit_price((tip_result.compute_unit_price).min(1_200_000));
                                builder.add_compute_accounts_data_size_limit(32_000_000);
                                builder.add_jito_tip_ix(tip_result.provider_tip);

                                let vtxn = builder.prepare_tx(&keypair, &alts, blockhash);
                                if let Ok(vtxn) = vtxn {
                                    let buff: Vec<u8> = match check_transaction_size(&vtxn) {
                                        Ok(buffer) => buffer,
                                        Err(err) => {
                                            warn!(
                                                "{:?}, {:?}, {:?}",
                                                err,
                                                get_pool_types(&calculators),
                                                get_pubkeys(&calculators)
                                            );
                                            continue 'outer;
                                        }
                                    };

                                    let msg = match generate_body_for_jito_from_data(&buff, &jito_quicknode_region) {
                                        Ok(msg) => msg,
                                        Err(e) => {
                                            warn!("Error generating body for jito: {:?}", e);
                                            continue;
                                        }
                                    };

                                    let res = client.post(cfg.jito_quicknode.quicknode_rpc.clone()).body(msg).send();
                                    match res {
                                        Ok(response) => {
                                            info!("R: {:?}, S: {:?}", response.status(), vtxn.signatures[0])
                                        }
                                        Err(err) => warn!("Error sending tx: {:?}", err),
                                    }

                                    if optimize.diff > 1_000_000_000_000 {
                                        warn!(
                                            "TOO HIGH WRONG ...: {:?} {:?}",
                                            get_pool_types(&calculators),
                                            get_pubkeys(&calculators)
                                        );
                                    }
                                }
                            } else {
                                warn!("Tip result calculation failed");
                                continue 'outer;
                            }
                        } else {
                            let mut builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());

                            let vtxn: anyhow::Result<VersionedTransaction>;
                            let ttxn: anyhow::Result<VersionedTransaction>;
                            let max_fee = calculate_max_fee(optimize.diff);

                            let preparation = match arb_core::arbitrage::process_arbitrage_v6(
                                &calculators,
                                &WSOL,
                                &mut builder,
                                &mut mint_to_ata,
                                &mint_pair_route,
                                &allowed_token2022,
                                optimize.amounts,
                                optimize.remaining_accounts,
                            ) {
                                Ok(preparation) => preparation,
                                Err(err) => {
                                    warn!("Error processing arbitrage: {:?}", err);
                                    continue 'outer;
                                }
                            };

                            alts.insert(0, alt.clone());

                            let mut compute_unit_limit: u64;
                            compute_unit_limit = if calculators.len() == 2 { 240_000 } else { 390_000 };
                            if optimize.diff > 100_000_000 {
                                compute_unit_limit = 490_000;
                            }

                            let tip_input = arb_core::tip::TipInput {
                                diff: optimize.diff as i64,
                                max_tip: max_fee,
                                priority_fee_pct: cfg.jito_quicknode.priority_fee_percent, // 2,
                                min_ratio: cfg.jito_quicknode.tip_min_percent,             // 20,
                                max_ratio: cfg.jito_quicknode.tip_max_percent,             //  60,
                                provider: arb_core::questdb::ExecutionProviderType::JitoQuicknode,
                                compute_unit_limit,
                                max_priority_fee: Some(cfg.jito_quicknode.max_priority_fee), // 50_000),
                            };
                            let tip_result = arb_core::tip::compute_tip(&tip_input);

                            let use_flash_loan = optimize.amount >= cfg.arbitrage.min_amount_for_flashloan;

                            if let Ok(mut tip_result) = tip_result {
                                if use_flash_loan {
                                    let amount = amount_for_flashloan(optimize.amount);
                                    builder.push_ix(preparation.to_floashloan_ix(
                                        amount,
                                        tip_result.total_tip,
                                        pool_ata,
                                        pool,
                                        token_ata_wsol,
                                    ));
                                } else {
                                    builder.push_ix(preparation.to_instruction(tip_result.total_tip));
                                }
                                if cfg.jito_quicknode.simulate {
                                    let start = Instant::now();
                                    let mut simulation_builder = builder.clone();
                                    simulation_builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                                    simulation_builder
                                        .add_compute_unit_price((tip_result.compute_unit_price).min(1_200_000));
                                    simulation_builder.add_jito_tip_ix(/*tip_result.provider_tip*/ 1);
                                    let txn = simulation_builder.prepare_tx(&keypair, &alts, blockhash);
                                    if let Ok(txn) = txn {
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
                                                    let mut new_cu = cu + 6_000;
                                                    if optimize.diff > 10_000_000 {
                                                        new_cu += 4000;
                                                    }
                                                    if optimize.diff > 1_000_000_000 {
                                                        new_cu += 10_000;
                                                    }
                                                    debug!(
                                                        "CCUL: {:?}, SCUL: {:?}, took: {:?}",
                                                        tip_result.compute_unit_limit, new_cu, elapsed
                                                    );
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
                                builder.add_compute_unit_price((tip_result.compute_unit_price).min(1_200_000));
                                builder.add_compute_accounts_data_size_limit(32_000_000);

                                let mut tip_builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());
                                tip_builder.add_compute_unit_limit(70_000);
                                tip_builder.add_compute_unit_price(1);
                                tip_builder.add_compute_accounts_data_size_limit(1_024_000);
                                let _ = tip_builder.close_ata_ix(&token_ata_wsol);
                                tip_builder.add_create_ata_if_not_exists(&WSOL);
                                tip_builder.add_system_transfer_ix(cfg.wallet_manager.max_wsol, &token_ata_wsol);
                                tip_builder.add_sync_native(&token_ata_wsol);
                                tip_builder.add_jito_tip_ix(tip_result.provider_tip);
                                ttxn = tip_builder.prepare_tx(&keypair, &[], blockhash);

                                vtxn = builder.prepare_tx(&keypair, &alts, blockhash);
                                if let (Ok(vtxn), Ok(ttxn)) = (vtxn, ttxn) {
                                    let buff: Vec<u8> = match check_transaction_size(&vtxn) {
                                        Ok(buffer) => buffer,
                                        Err(err) => {
                                            warn!(
                                                "{:?}, {:?}, {:?}",
                                                err,
                                                get_pool_types(&calculators),
                                                get_pubkeys(&calculators)
                                            );
                                            continue 'outer;
                                        }
                                    };

                                    let tbuffer = bincode::serialize(&ttxn).unwrap_or(vec![]);
                                    let msg = match generate_body_for_jito_from_data_v2(
                                        &buff,
                                        &tbuffer,
                                        &jito_quicknode_region,
                                    ) {
                                        Ok(msg) => msg,
                                        Err(e) => {
                                            warn!("Error generating body for jito: {:?}", e);
                                            continue;
                                        }
                                    };

                                    let res = client.post(cfg.jito_quicknode.quicknode_rpc.clone()).body(msg).send();
                                    match res {
                                        Ok(response) => {
                                            info!("R: {:?}, S: {:?}", response.status(), vtxn.signatures[0])
                                        }
                                        Err(err) => warn!("Error sending tx: {:?}", err),
                                    }

                                    if optimize.diff > 1_000_000_000_000 {
                                        warn!(
                                            "TOO HIGH WRONG ...: {:?} {:?}",
                                            get_pool_types(&calculators),
                                            get_pubkeys(&calculators)
                                        );
                                    }
                                }
                            } else {
                                warn!("Tip result calculation failed");
                                continue 'outer;
                            }
                        }
                    }
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
