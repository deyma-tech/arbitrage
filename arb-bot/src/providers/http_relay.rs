use ahash::AHashSet;
use arb_core::arbitrage::OpportunityWithCalculators;
use arb_core::fee::calculate_max_fee;
use config::{HttpRelayConfig, CONFIG as cfg};
use log::{debug, info, warn};
use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::base64::b64_encode;
use utils::constants::ALLOWED_TOKEN_2022;
use utils::transaction::check_transaction_size;

use crate::providers::{
    fetch_nonce_context, prepare_transaction, require_min_net_profit, NonceLease, Provider, SetupResult,
};
use crate::OptimizeResult;

#[derive(Clone, Copy, Debug)]
pub enum HttpRelayKind {
    Astralane,
    Nozomi,
}

#[derive(Clone, Debug)]
pub struct ProviderHttpRelay {
    pub kind: HttpRelayKind,
    pub counter: Arc<AtomicU64>,
    pub setup: Option<SetupResult>,
}

impl ProviderHttpRelay {
    pub async fn new(kind: HttpRelayKind) -> Self {
        let mut provider = Self {
            kind,
            counter: Arc::new(AtomicU64::new(0)),
            setup: None,
        };
        provider.setup = Some(provider.setup().await);
        provider
    }

    fn config(&self) -> &HttpRelayConfig {
        match self.kind {
            HttpRelayKind::Astralane => &cfg.astralane,
            HttpRelayKind::Nozomi => &cfg.nozomi,
        }
    }

    fn name(&self) -> &'static str {
        match self.kind {
            HttpRelayKind::Astralane => "Astralane",
            HttpRelayKind::Nozomi => "Nozomi",
        }
    }

    fn provider_type(&self) -> arb_core::questdb::ExecutionProviderType {
        match self.kind {
            HttpRelayKind::Astralane => arb_core::questdb::ExecutionProviderType::Astralane,
            HttpRelayKind::Nozomi => arb_core::questdb::ExecutionProviderType::Nozomi,
        }
    }

    fn add_tip(&self, builder: &mut arb_core::instruction::IxBuilder, amount: u64) {
        match self.kind {
            HttpRelayKind::Astralane => builder.add_astralane_tip_ix(amount),
            HttpRelayKind::Nozomi => builder.add_nozomi_tip_ix(amount),
        }
    }

    async fn submit(&self, client: &Client, content: String) -> anyhow::Result<String> {
        let config = self.config();
        let request = match self.kind {
            HttpRelayKind::Astralane => client
                .post(&config.endpoint)
                .query(&[
                    ("api-key", config.api_key.as_str()),
                    ("method", "sendTransaction"),
                    ("mev-protect", "false"),
                    ("swqos-only", "false"),
                ])
                .header(reqwest::header::CONTENT_TYPE, "text/plain")
                .body(content),
            HttpRelayKind::Nozomi => client
                .post(&config.endpoint)
                .query(&[("c", config.api_key.as_str())])
                .header(reqwest::header::CONTENT_TYPE, "text/plain")
                .body(content),
        };
        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("{} returned HTTP {}: {}", self.name(), status, body);
        }
        Ok(body)
    }
}

impl Provider for ProviderHttpRelay {
    fn get_filter(&self) -> u64 {
        self.config().filter
    }

    fn get_execution_threads(&self) -> u64 {
        let configured = self.config().execution_threads;
        if cfg.nonce_accounts.is_empty() {
            configured
        } else {
            configured.min(cfg.nonce_accounts.len() as u64)
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
        let setup = self.setup.clone().expect("Failed to load setup");
        let keypair = setup.keypair;
        let mut balance = setup.balance;
        let mut blockhash = setup.blockhash;
        let mut rx_balance = setup.tx_balance.subscribe();
        let mut rx_blockhash = setup.tx_blockhash.subscribe();
        let alt = setup.alt;
        let nonce_manager = setup.nonce_manager.clone();
        let kind = self.kind;
        let name = self.name();
        let config = self.config().clone();
        let counter = self.counter.clone();

        let runtime = tokio::runtime::Runtime::new().expect("failed to create relay runtime");
        runtime.spawn(async move {
            let client = Client::builder()
                .tcp_keepalive(Duration::from_secs(30))
                .build()
                .expect("failed to create relay HTTP client");
            let relay = ProviderHttpRelay {
                kind,
                counter,
                setup: None,
            };
            let simulate_rpc = RpcClient::new_with_commitment(
                cfg.blockhash_and_simulate_rpc.clone(),
                CommitmentConfig::processed(),
            );
            let mut allowed_token2022 = AHashSet::from_iter(ALLOWED_TOKEN_2022.iter().cloned());
            let mut buffer = Vec::new();
            let mut next_astralane_submission = Instant::now();

            info!("{} relay active: endpoint={}", name, config.endpoint);
            loop {
                select! {
                    Ok(new_balance) = rx_balance.recv() => balance = new_balance,
                    Ok(new_blockhash) = rx_blockhash.recv() => blockhash = new_blockhash,
                    Ok(token_set) = rx_token2022_bc.recv() => allowed_token2022 = token_set,
                    _ = rx_arbitrage.recv_many(&mut buffer, 100) => {
                        let now = utils::now::as_millis();
                        buffer.retain(|(opportunity, _, _, _)| {
                            now >= opportunity.timestamp && now - opportunity.timestamp < 50
                        });
                        buffer.sort_by(|(_, left, _, _), (_, right, _, _)| left.diff.cmp(&right.diff));
                        let Some((opportunity, optimize, mut alts, _)) = buffer.pop() else {
                            buffer.clear();
                            continue;
                        };
                        buffer.clear();
                        if optimize.diff < config.filter {
                            continue;
                        }

                        let calculators = opportunity.calculators;
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
                                debug!("{} rejected before preparation: {}", name, err);
                                continue;
                            }
                        };
                        let compute_unit_limit = if optimize.diff > 10_000_000 { 600_000 } else { 300_000 };
                        let tip_input = arb_core::tip::TipInput {
                            diff: optimize.diff as i64,
                            max_tip: calculate_max_fee(balance),
                            priority_fee_pct: config.priority_fee_percent,
                            min_ratio: config.tip_min_percent,
                            max_ratio: config.tip_max_percent,
                            provider: relay.provider_type(),
                            compute_unit_limit,
                            max_priority_fee: Some(config.max_priority_fee),
                        };
                        let mut tip_result = match arb_core::tip::compute_tip(&tip_input) {
                            Ok(result) => result,
                            Err(err) => {
                                debug!("{} tip calculation rejected: {}", name, err);
                                continue;
                            }
                        };
                        if require_min_net_profit(optimize.diff, tip_result.total_tip).is_err() {
                            continue;
                        }

                        alts.insert(0, alt.clone());
                        let mut nonce_lease = match fetch_nonce_context(&simulate_rpc, &nonce_manager).await {
                            Ok(lease) => lease,
                            Err(err) => {
                                warn!("{} nonce acquisition failed: {}", name, err);
                                continue;
                            }
                        };
                        let nonce_context = nonce_lease.as_ref().map(NonceLease::context);

                        if config.simulate {
                            let mut simulation_builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());
                            simulation_builder.push_ix(preparation.to_instruction(tip_result.total_tip));
                            simulation_builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                            simulation_builder.add_compute_unit_price(tip_result.compute_unit_price.min(1_200_000));
                            relay.add_tip(&mut simulation_builder, tip_result.provider_tip);
                            let txn = match prepare_transaction(&mut simulation_builder, &keypair, &alts, blockhash, nonce_context) {
                                Ok(txn) => txn,
                                Err(err) => {
                                    debug!("{} simulation preparation failed: {}", name, err);
                                    continue;
                                }
                            };
                            if let Err(err) = check_transaction_size(&txn) {
                                debug!("{} simulation transaction too large: {}", name, err);
                                continue;
                            }
                            let started = Instant::now();
                            let simulation = match simulate_rpc.simulate_transaction(&txn).await {
                                Ok(result) => result,
                                Err(err) => {
                                    warn!("{} simulation RPC failed: {}", name, err);
                                    continue;
                                }
                            };
                            if simulation.value.err.is_some() {
                                debug!("{} simulation rejected: {:?}", name, simulation.value.err);
                                continue;
                            }
                            if let Some(units) = simulation.value.units_consumed {
                                if units < 50_000 {
                                    continue;
                                }
                                tip_result.set_final_compute_unit_limit(units + 20_000);
                                debug!("{} simulation ok: units={}, elapsed={:?}", name, units, started.elapsed());
                            }
                        }

                        let net = match require_min_net_profit(optimize.diff, tip_result.total_tip) {
                            Ok(net) => net,
                            Err(err) => {
                                debug!("{} final net-profit reject: {}", name, err);
                                continue;
                            }
                        };
                        let mut builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());
                        builder.push_ix(preparation.to_instruction(tip_result.total_tip));
                        builder.add_compute_unit_limit(tip_result.compute_unit_limit as u32);
                        builder.add_compute_unit_price(tip_result.compute_unit_price);
                        relay.add_tip(&mut builder, tip_result.provider_tip);
                        let txn = match prepare_transaction(&mut builder, &keypair, &alts, blockhash, nonce_context) {
                            Ok(txn) => txn,
                            Err(err) => {
                                debug!("{} transaction preparation failed: {}", name, err);
                                continue;
                            }
                        };
                        if let Err(err) = check_transaction_size(&txn) {
                            debug!("{} transaction too large: {}", name, err);
                            continue;
                        }
                        let signature = txn.signatures.first().copied();
                        let content = b64_encode(bincode::serialize(&txn).expect("failed to serialize transaction"));
                        if matches!(kind, HttpRelayKind::Astralane) {
                            let now = Instant::now();
                            if now < next_astralane_submission {
                                tokio::time::sleep(next_astralane_submission - now).await;
                            }
                            // Astralane free tier: 5 single transactions per second.
                            next_astralane_submission = Instant::now() + Duration::from_millis(200);
                        }
                        match relay.submit(&client, content).await {
                            Ok(response) => {
                                info!("{} accepted transaction: signature={:?}, net={}, response={}", name, signature, net, response);
                                if let Some(signature) = signature {
                                    if let Some(lease) = nonce_lease.as_mut() {
                                        match lease.wait_for_finalized(&simulate_rpc, &signature, Duration::from_millis(cfg.nonce_confirmation_timeout_ms)).await {
                                            Ok(()) => info!("{} transaction finalized: signature={}", name, signature),
                                            Err(err) => warn!("{} transaction finalization failed: signature={}, error={}", name, signature, err),
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
                                warn!("{} submission failed: {}", name, err);
                            }
                        }
                    }
                }
            }
        });
        Some(runtime)
    }
}
