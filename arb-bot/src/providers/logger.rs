use ahash::{AHashMap, AHashSet};
use arb_core::arbitrage::OpportunityWithCalculators;
use log::{debug, info, warn};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use std::time::{Duration, Instant};
use tokio::sync::broadcast::Receiver;
use utils::transaction::check_transaction_size;

use super::{fetch_nonce_context, prepare_transaction, NonceLease, Provider, SetupResult};
use crate::OptimizeResult;
use arb_core::calculator::{get_pool_types, get_pubkeys};
use arb_core::fee::calculate_max_fee;
use config::{CONFIG as cfg, DEFAULT_EXECUTION_THREADS};
use solana_sdk::hash::Hash;
use solana_sdk::signature::Signer;
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::{constants::ALLOWED_TOKEN_2022, types::MintPair};

#[derive(Clone, Debug)]
pub struct ProviderLog {
    // counter: Arc<AtomicU64>,
    pub setup: Option<SetupResult>,
}

impl ProviderLog {
    pub async fn new() -> ProviderLog {
        let mut provider = ProviderLog {
            setup: None,
            //start_time: Instant::now(),
            // counter: Arc::new(AtomicU64::new(0)),
        };
        provider.setup = Some(provider.setup().await);
        provider
    }
}

impl Provider for ProviderLog {
    #[inline(always)]
    fn get_filter(&self) -> u64 {
        1
    }

    #[inline(always)]
    fn get_execution_threads(&self) -> u64 {
        if cfg.enable_execution {
            DEFAULT_EXECUTION_THREADS
        } else {
            1
        }
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

        let setup = match self.setup {
            Some(setup) => setup,
            None => panic!("Failed to load setup"),
        };

        let keypair = setup.keypair;
        // Executor V2 uses the wallet's existing WSOL ATA directly.

        let mut balance = setup.balance;
        let mut blockhash = setup.blockhash;
        let mut rx_balance = setup.tx_balance.subscribe();
        let mut rx_blockhash = setup.tx_blockhash.subscribe();

        let alt = setup.alt;
        let nonce_manager = setup.nonce_manager.clone();

        let mut allowed_token2022 = AHashSet::from_iter(ALLOWED_TOKEN_2022.iter().cloned());

        info!("Using log provider, setup successful");

        let exec_runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => panic!("Error creating tokio runtime: {err:?}"),
        };

        let id = Hash::new_unique().to_string()[..6].to_string();
        info!("Spawning log provider with id: {}", id);
        let mut counter = 0;
        let mut dry_run_last_processed: AHashMap<(Vec<Pubkey>, Vec<MintPair>, u64), Instant> = AHashMap::new();
        let mut dry_run_simulation_window_started = Instant::now();
        let mut dry_run_simulations_in_window = 0u64;

        exec_runtime.spawn({
            async move {
                let rpc_client = RpcClient::new_with_commitment(cfg.rpc.clone(), CommitmentConfig::processed());
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

                        // blockhash
                        Ok(new_blockhash) = rx_blockhash.recv() => {
                            blockhash = new_blockhash;
                        }

                        // opportunities
                        Some((opportunity, optimize, mut alts, _)) = rx_arbitrage.recv() => {
                            counter += 1;

                            let start = Instant::now();

                            let calculators = opportunity.calculators;
                            let mint_pair_route = opportunity.mint_pair_route.iter().collect::<Vec<_>>();
                            let mint_route_key = opportunity.mint_pair_route.clone();
                            let route_pubkeys = calculators.iter().map(|calculator| *calculator.get_pubkey()).collect::<Vec<_>>();
                            let dry_run_simulate = !cfg.enable_execution && cfg.arbitrage.dry_run_simulate;

                            if !cfg.enable_execution {
                                let now = Instant::now();
                                let dedup_window = Duration::from_millis(cfg.arbitrage.dry_run_dedup_window_ms);
                                if counter % 256 == 0 {
                                    dry_run_last_processed.retain(|_, last| now.duration_since(*last) < dedup_window.saturating_mul(2));
                                }
                                let dedup_key = (route_pubkeys.clone(), mint_route_key, optimize.amount);
                                let should_process = dry_run_last_processed
                                    .get(&dedup_key)
                                    .map(|last| now.duration_since(*last) >= dedup_window)
                                    .unwrap_or(true);
                                if !should_process {
                                    continue 'outer;
                                }
                                dry_run_last_processed.insert(dedup_key, now);

                                let estimated_cost = cfg.arbitrage.estimated_execution_cost_lamports;
                                let estimated_net = optimize.diff.saturating_sub(estimated_cost);
                                info!(
                                    "DRY-RUN candidate: gross={}, estimated_cost={}, estimated_net={}, amount={}, slot={}, route={:?}, types={:?}, mints={:?}",
                                    optimize.diff,
                                    estimated_cost,
                                    estimated_net,
                                    optimize.amount,
                                    opportunity.slot,
                                    route_pubkeys,
                                    calculators.iter().map(|c| c.get_pool_type()).collect::<Vec<_>>(),
                                    mint_pair_route
                                );

                                if !dry_run_simulate {
                                    continue 'outer;
                                }

                                if now.duration_since(dry_run_simulation_window_started) >= Duration::from_secs(1) {
                                    dry_run_simulation_window_started = now;
                                    dry_run_simulations_in_window = 0;
                                }
                                if dry_run_simulations_in_window >= cfg.arbitrage.dry_run_max_simulations_per_second {
                                    debug!(
                                        "DRY-RUN simulation rate limit reached: max_per_second={}",
                                        cfg.arbitrage.dry_run_max_simulations_per_second
                                    );
                                    continue 'outer;
                                }
                                dry_run_simulations_in_window += 1;
                            }

                            let max_fee = calculate_max_fee(balance);

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
                                    warn!("DRY-RUN rejected before simulation: {:?}, pools: {:?}, types: {:?}, volume: {:?}", err,
                                        calculators.iter().map(|c| c.get_pubkey()).collect::<Vec<_>>(),
                                        calculators.iter().map(|c| c.get_pool_type()).collect::<Vec<_>>(),
                                        optimize.amount,
                                    );
                                    continue 'outer;
                                }
                            };

                            if optimize.diff > 1_000_000_000 {
                                info!("Diff: opp={:?}, opt={:?}, max_fee={:?}, comb={:?}, slot={:?}, bh={:?}, pt={:?}, pk={:?}, id={}", opportunity.diff, optimize.diff, max_fee, calculators.len(), opportunity.slot, blockhash, get_pool_types(&calculators), get_pubkeys(&calculators), id);
                            }

                            let should_simulate = dry_run_simulate || (cfg.enable_execution && optimize.diff > 10_000 && counter % 10_000 == 0);
                            if should_simulate {

                                alts.insert(0, alt.clone());

                                let compute_unit_limit = 1_000_000;
                                builder.add_compute_unit_limit(compute_unit_limit as u32);
                                builder.add_compute_unit_price(10_000);
                                // Executor V2 is wallet-funded; the flashloan
                                // wrapper is intentionally unavailable here.
                                builder.push_ix(preparation.to_instruction(0));

                                let nonce_lease = match fetch_nonce_context(&rpc_client, &nonce_manager).await {
                                    Ok(context) => context,
                                    Err(err) => {
                                        warn!("DRY-RUN nonce read failed: {:?}, route={:?}", err, route_pubkeys);
                                        continue 'outer;
                                    }
                                };
                                let nonce_context = nonce_lease.as_ref().map(NonceLease::context);
                                let txn = prepare_transaction(&mut builder, &keypair, &alts, blockhash, nonce_context);

                                info!("Preparation of tx: {:?}", start.elapsed());
                                match txn {
                                    Ok(txn) => {
                                        if let Err(err) = check_transaction_size(&txn) {
                                            warn!("DRY-RUN transaction size check failed: {:?}, route={:?}", err, route_pubkeys);
                                            continue 'outer;
                                        }
                                        let simulation = rpc_client.simulate_transaction(&txn).await;
                                        match simulation {
                                            Ok(simulation) if simulation.value.err.is_none() => info!(
                                                "DRY-RUN simulation ok: units_consumed={:?}, route={:?}, types={:?}, mints={:?}, slot={}, elapsed={:?}, id={}",
                                                simulation.value.units_consumed,
                                                route_pubkeys,
                                                calculators.iter().map(|c| c.get_pool_type()).collect::<Vec<_>>(),
                                                mint_pair_route,
                                                opportunity.slot,
                                                start.elapsed(),
                                                id
                                            ),
                                            Ok(simulation) => warn!(
                                                "DRY-RUN simulation rejected: err={:?}, units_consumed={:?}, route={:?}, types={:?}, mints={:?}, slot={}, elapsed={:?}, id={}",
                                                simulation.value.err,
                                                simulation.value.units_consumed,
                                                route_pubkeys,
                                                calculators.iter().map(|c| c.get_pool_type()).collect::<Vec<_>>(),
                                                mint_pair_route,
                                                opportunity.slot,
                                                start.elapsed(),
                                                id
                                            ),
                                            Err(err) => warn!(
                                                "DRY-RUN simulation RPC failed: {:?}, route={:?}, slot={}, elapsed={:?}, id={}",
                                                err,
                                                route_pubkeys,
                                                opportunity.slot,
                                                start.elapsed(),
                                                id
                                            ),
                                        }
                                    }
                                    Err(err) => {
                                        warn!(
                                            "DRY-RUN transaction preparation failed: {:?}, route={:?}, slot={}",
                                            err, route_pubkeys, opportunity.slot
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Some(exec_runtime)
    }
}
