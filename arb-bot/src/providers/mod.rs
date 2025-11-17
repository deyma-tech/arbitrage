use ahash::{AHashMap, AHashSet, HashMap};
use arb_core::arbitrage::OpportunityWithCalculators;

use log::info;
use solana_program::pubkey::Pubkey;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::priority_fee::spawn_get_priority_fees;
use crate::process::get_channel_for_blockhash;
use crate::rebalancer::rebalance;
use crate::setup::{fetch_alt, fetch_balance, fetch_blockhash, fetch_flashloan_keys};
use crate::OptimizeResult;
use config::{CONFIG as cfg, DEFAULT_EXECUTION_THREADS};
use solana_sdk::hash::Hash;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::constants::WSOL;

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
        let balance = fetch_balance().await;
        let blockhash = fetch_blockhash().await;
        let flashloan_keys = fetch_flashloan_keys().await;
        let alt = fetch_alt().await;
        // TODO - true need to go into config !!!!
        let turn_on_priority_fee = cfg.providers.as_str() == "helius_swqos" || cfg.providers.as_str() == "allh+";
        let tx_balance = rebalance();
        let tx_priority_fee = if turn_on_priority_fee {
            Some(spawn_get_priority_fees())
        } else {
            None
        };
        let tx_blockhash = get_channel_for_blockhash(cfg.blockhash_and_simulate_rpc.clone()).await;

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
            ProviderType::Log(_) => DEFAULT_EXECUTION_THREADS, // default thread count for Log provider
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
