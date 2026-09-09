use ahash::{AHashMap, AHashSet, HashMap};
use arb_core::arbitrage::OpportunityWithCalculators;
use arb_core::calculator::CalculatorEnum;
use arb_core::instruction::ArbitrageCompressedInstructionInput;

use log::{info, warn};
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::priority_fee::spawn_get_priority_fees;
use crate::process::get_channel_for_blockhash;
use crate::setup::{fetch_balance, fetch_blockhash, fetch_flashloan_keys};
use crate::OptimizeResult;
use config::{CONFIG as cfg, DEFAULT_EXECUTION_THREADS};
use solana_sdk::hash::Hash;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::constants::WSOL;

/// Build the only execution ABI enabled by this Chainstack deployment.
/// Unsupported venues/leg counts are rejected instead of falling back to the
/// legacy generic wrapper, whose program ID is not trusted for live funds.
pub fn prepare_executor_v2(
    calculators: &[Box<CalculatorEnum>],
    payer: Pubkey,
    minimum_profit: u64,
    amounts: &[u64],
    remaining_accounts: &[Vec<Pubkey>],
    allowed_token2022: &AHashSet<Pubkey>,
) -> anyhow::Result<ArbitrageCompressedInstructionInput> {
    if cfg.arbitrage.enable_flashloan {
        anyhow::bail!("executor V2 usa WSOL de la wallet; flashloan sigue deshabilitado");
    }
    let executor_program = Pubkey::from_str(&cfg.arb_executor_v2_program_id)
        .map_err(|err| anyhow::anyhow!("ARB_EXECUTOR_V2_PROGRAM_ID inválido: {err}"))?;
    arb_core::arbitrage::process_executor_v2(
        calculators,
        &WSOL,
        amounts,
        remaining_accounts,
        payer,
        executor_program,
        minimum_profit,
        allowed_token2022,
    )
}

pub mod jito;
pub use jito::ProviderJito;
pub mod jito_quicknode;
pub use jito_quicknode::ProviderJitoQuicknode;
// pub mod bloxroute;
// pub use bloxroute::ProviderBloxroute;
pub mod logger;
use crate::providers::bloxroute::ProviderBloxroute;
use crate::providers::nextblock::ProviderNextblock;
pub use logger::ProviderLog;

pub mod bloxroute;
pub mod nextblock;

const _OPTIMIZATION_STEPS: u8 = 8;

//const FEE_RIGHT_SHIFT: u64 = 30;

#[inline]
pub fn amount_for_flashloan(amount: u64) -> u64 {
    if amount < 400_000_000_000 {
        return amount * 125 / 100;
    }
    amount * 105 / 100
}

#[inline]
pub fn amount_for_flashloan_with_tip(amount: u64, tip: u64) -> u64 {
    if amount + tip < 500_000_000_000 {
        (amount * 105 / 100) + tip
    } else {
        1
    }
}

#[derive(Clone, Debug)]
pub enum ProviderType {
    Jito(ProviderJito),
    JitoQuicknode(ProviderJitoQuicknode),
    Bloxroute(ProviderBloxroute),
    Log(ProviderLog),
    Nextblock(ProviderNextblock),
}

impl ProviderType {
    pub fn get_setup(&self) -> Option<SetupResult> {
        match self {
            ProviderType::Jito(provider) => provider.setup.clone(),
            ProviderType::JitoQuicknode(provider) => provider.setup.clone(),
            ProviderType::Bloxroute(provider) => provider.setup.clone(),
            ProviderType::Log(provider) => provider.setup.clone(),
            ProviderType::Nextblock(provider) => provider.setup.clone(),
        }
    }
}

pub async fn get_providers() -> Vec<ProviderType> {
    if !cfg.enable_execution && cfg.providers != "log" {
        warn!("Execution disabled; forcing log provider");
        return vec![ProviderType::Log(ProviderLog::new().await)];
    }
    let providers = cfg.providers.clone();
    if providers == "jjn" {
        return vec![
            ProviderType::JitoQuicknode(ProviderJitoQuicknode::new().await),
            ProviderType::Jito(ProviderJito::new().await),
            ProviderType::Nextblock(ProviderNextblock::new().await),
        ];
    }
    let providers: Vec<&str> = providers.split(',').collect();
    let mut result = Vec::new();
    for provider in providers {
        let provider = match provider {
            "bloxroute" => ProviderType::Bloxroute(ProviderBloxroute::new().await),
            "jitoquicknode" => ProviderType::JitoQuicknode(ProviderJitoQuicknode::new().await),
            "jito" => ProviderType::Jito(ProviderJito::new().await),
            "nextblock" => ProviderType::Nextblock(ProviderNextblock::new().await),
            "log" => ProviderType::Log(ProviderLog::new().await),
            &_ => panic!("UnknownProvider"),
        };
        result.push(provider);
    }
    result
}

/// Returns modeled profit after the provider's calculated total cost.
/// `total_cost` includes provider tip, priority fee, and base transaction fee.
pub(super) fn require_min_net_profit(gross_profit: u64, total_cost: u64) -> anyhow::Result<u64> {
    let net_profit = net_profit_after_cost(gross_profit, total_cost)
        .ok_or_else(|| anyhow::anyhow!("GrossProfitBelowExecutionCost"))?;
    if net_profit < cfg.arbitrage.min_net_profit_lamports {
        anyhow::bail!(
            "NetProfitBelowMinimum: gross={}, cost={}, net={}, minimum={}",
            gross_profit,
            total_cost,
            net_profit,
            cfg.arbitrage.min_net_profit_lamports
        );
    }
    Ok(net_profit)
}

#[inline]
fn net_profit_after_cost(gross_profit: u64, total_cost: u64) -> Option<u64> {
    gross_profit.checked_sub(total_cost)
}

#[cfg(test)]
mod tests {
    use super::net_profit_after_cost;

    #[test]
    fn net_profit_cannot_underflow() {
        assert_eq!(net_profit_after_cost(1_000, 1_001), None);
    }

    #[test]
    fn net_profit_subtracts_provider_cost() {
        assert_eq!(net_profit_after_cost(10_000_000, 1_005_000), Some(8_995_000));
    }
}

#[derive(Clone, Debug)]
pub struct SetupResult {
    pub keypair: Arc<Keypair>,
    pub token_ata_wsol: Pubkey,
    // pub record_account: Pubkey,
    pub balance: u64,
    pub blockhash: Hash,
    pub flashloan_keys: AHashMap<Pubkey, (Pubkey, Pubkey)>,
    pub alt: AddressLookupTableAccount,
    pub tx_balance: Sender<u64>,
    pub tx_blockhash: Sender<Hash>,
    pub tx_priority_fee: Option<Sender<u64>>,
    pub mint_to_ata: HashMap<Pubkey, Pubkey>,
}

#[allow(async_fn_in_trait)]
pub trait Provider {
    fn get_filter(&self) -> u64;
    fn get_execution_threads(&self) -> u64;

    async fn setup(&self) -> SetupResult {
        let keypair = match solana_sdk::signature::read_keypair_file(&cfg.keypair) {
            Ok(keypair) => keypair,
            Err(err) => {
                panic!("Error reading keypair file: {err:?}");
            }
        };
        let keypair = Arc::new(keypair);
        let token_ata_wsol = get_associated_token_address(&keypair.pubkey(), &WSOL);
        //let regions = get_region();

        // let record_account = get_record_account(&keypair.pubkey());
        let balance = if cfg.enable_execution {
            fetch_balance().await
        } else {
            // Dry-run does not use balance-based fee or capital guards.
            info!("Execution disabled; skipping wallet balance RPC setup");
            0
        };
        let needs_blockhash = cfg.enable_execution || cfg.arbitrage.dry_run_simulate;
        let (blockhash, tx_blockhash) = if needs_blockhash {
            let blockhash = fetch_blockhash().await;
            let tx_blockhash = get_channel_for_blockhash(cfg.blockhash_and_simulate_rpc.clone()).await;
            (blockhash, tx_blockhash)
        } else {
            // Dry-run never builds or submits a transaction, so there is no
            // reason to query or refresh blockhashes against the RPC.
            info!("Execution disabled; skipping blockhash RPC setup");
            let (tx_blockhash, _) = tokio::sync::broadcast::channel::<Hash>(100);
            (Hash::default(), tx_blockhash)
        };
        let flashloan_keys = if cfg.arbitrage.enable_flashloan {
            fetch_flashloan_keys().await
        } else {
            info!("Flashloan disabled; not loading flashloan accounts");
            AHashMap::default()
        };
        // The old hardcoded ALT is a placeholder and is not part of the
        // verified executor V2 ABI. Keep the setup empty; callers may still
        // add validated route-specific tables from the optimizer.
        let alt = AddressLookupTableAccount {
            key: Pubkey::default(),
            addresses: vec![],
        };
        // TODO - true need to go into config !!!!
        let turn_on_priority_fee = cfg.providers.as_str() == "helius_swqos" || cfg.providers.as_str() == "allh+";
        // The legacy rebalancer can close/recreate the WSOL ATA and submit a
        // native transfer. Executor V2 is wallet-funded, so do not start that
        // side-effecting task; providers use the initial balance snapshot.
        let (tx_balance, _) = tokio::sync::broadcast::channel(1);
        let tx_priority_fee = if turn_on_priority_fee {
            Some(spawn_get_priority_fees())
        } else {
            None
        };

        info!("ALT: {:?}", alt.addresses.len());
        info!("wallet: {}", keypair.pubkey());
        info!("ata wsol: {:?}", token_ata_wsol);
        // info!("record account: {:?}", record_account);

        SetupResult {
            keypair,
            token_ata_wsol,
            //regions,
            // record_account,
            balance,
            blockhash,
            flashloan_keys,
            alt,
            tx_balance,
            tx_blockhash,
            tx_priority_fee,
            mint_to_ata: HashMap::default(),
        }
    }

    fn run(
        self,
        rx_arbitrage: UnboundedReceiver<(
            OpportunityWithCalculators,
            OptimizeResult,
            Vec<AddressLookupTableAccount>,
            Vec<AddressLookupTableAccount>,
        )>,
        tx_token2022_bc: Receiver<AHashSet<Pubkey>>,
        cfg_option: Option<String>,
    ) -> Option<tokio::runtime::Runtime>;

    fn increase_counter_and_log(counter: Arc<AtomicU64>) {
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self::log_counter(counter);
    }

    fn log_counter(counter: Arc<AtomicU64>) {
        let count = counter.load(std::sync::atomic::Ordering::Relaxed);
        if count.is_multiple_of(1000) {
            info!("Jito counter: {}", count);
        }
    }
}

impl Provider for ProviderType {
    fn get_filter(&self) -> u64 {
        match self {
            ProviderType::Jito(provider) => provider.get_filter(),
            ProviderType::JitoQuicknode(provider) => provider.get_filter(),
            ProviderType::Bloxroute(provider) => provider.get_filter(),
            ProviderType::Nextblock(provider) => provider.get_filter(),
            ProviderType::Log(provider) => provider.get_filter(),
        }
    }

    fn get_execution_threads(&self) -> u64 {
        match self {
            ProviderType::Jito(_) => cfg.jito.execution_threads,
            ProviderType::JitoQuicknode(_) => cfg.jito_quicknode.execution_threads,
            ProviderType::Bloxroute(_) => cfg.bloxroute.execution_threads,
            ProviderType::Nextblock(_) => cfg.nextblock.execution_threads,
            ProviderType::Log(_) => {
                if cfg.enable_execution {
                    DEFAULT_EXECUTION_THREADS
                } else {
                    1
                }
            }
        }
    }

    fn run(
        self,
        rx_arbitrage: UnboundedReceiver<(
            OpportunityWithCalculators,
            OptimizeResult,
            Vec<AddressLookupTableAccount>,
            Vec<AddressLookupTableAccount>,
        )>,
        rx_token2022_bc: Receiver<AHashSet<Pubkey>>,
        cfg_option: Option<String>,
    ) -> Option<tokio::runtime::Runtime> {
        match self {
            ProviderType::Jito(provider) => {
                info!("using jito provider");
                provider.run(rx_arbitrage, rx_token2022_bc, cfg_option)
            }
            ProviderType::JitoQuicknode(provider) => {
                info!("using jito quicknode provider");
                provider.run(rx_arbitrage, rx_token2022_bc, cfg_option)
            }
            ProviderType::Bloxroute(provider) => {
                info!("using bloxroute provider");
                provider.run(rx_arbitrage, rx_token2022_bc, cfg_option)
            }
            ProviderType::Log(provider) => {
                info!("using log provider");
                provider.run(rx_arbitrage, rx_token2022_bc, cfg_option)
            }
            ProviderType::Nextblock(provider) => {
                info!("using nextblock provider");
                provider.run(rx_arbitrage, rx_token2022_bc, cfg_option)
            }
        }
    }
}
