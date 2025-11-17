use ahash::AHashSet;
use arb_core::arbitrage::OpportunityWithCalculators;
use log::{debug, info, warn};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use std::time::Instant;
use tokio::sync::broadcast::Receiver;
use utils::transaction::check_transaction_size;

use super::{Provider, SetupResult};
use crate::providers::amount_for_flashloan;
use crate::OptimizeResult;
use arb_core::calculator::{get_pool_types, get_pubkeys};
use arb_core::fee::calculate_max_fee;
use config::{CONFIG as cfg, DEFAULT_EXECUTION_THREADS};
use solana_sdk::hash::Hash;
use solana_sdk::signature::Signer;
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::constants::{ALLOWED_TOKEN_2022, WSOL};

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
        DEFAULT_EXECUTION_THREADS
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
        let token_ata_wsol = setup.token_ata_wsol;
        //let _regions = setup.regions;

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

        info!("Using log provider, setup successful");

        let exec_runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => panic!("Error creating tokio runtime: {err:?}"),
        };

        let id = Hash::new_unique().to_string()[..6].to_string();
        info!("Spawning log provider with id: {}", id);
        let mut counter = 0;

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

                            let max_fee = calculate_max_fee(balance);

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

                            if optimize.diff > 1_000_000_000 {
                                info!("Diff: opp={:?}, opt={:?}, max_fee={:?}, comb={:?}, slot={:?}, bh={:?}, pt={:?}, pk={:?}, id={}", opportunity.diff, optimize.diff, max_fee, calculators.len(), opportunity.slot, blockhash, get_pool_types(&calculators), get_pubkeys(&calculators), id);
                            }

                            if optimize.diff > 10_000 && counter % 10_000 == 0 {

                                alts.insert(0, alt.clone());

                                let compute_unit_limit = 1_000_000;
                                builder.add_compute_unit_limit(compute_unit_limit as u32);
                                builder.add_compute_unit_price(10_000);
                                let amount = amount_for_flashloan(optimize.amount);
                                builder.push_ix(preparation.to_floashloan_ix(amount, 0, pool_ata, pool, token_ata_wsol));

                                let txn = builder.prepare_tx(&keypair, &alts, blockhash);

                                info!("Preparation of tx: {:?}", start.elapsed());
                                if let Ok(txn) = txn {
                                    let _ = check_transaction_size(&txn);
                                    // let unsigned = txn.message.header().num_readonly_unsigned_accounts;
                                    // txn.message.
                                    // let signed = txn.message.header().num_readonly_signed_accounts;
                                    let simulation = rpc_client.simulate_transaction(&txn).await;
                                    info!("Simulating tx: {:?} {:?} {:?} {:?}, {:?}", simulation, calculators.iter().map(|c| c.get_pubkey()).collect::<Vec<_>>(), calculators.iter().map(|c| c.get_pool_type()).collect::<Vec<_>>(), mint_pair_route, id); // unsigned, signed);
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
