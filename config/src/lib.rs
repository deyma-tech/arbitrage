use std::str::FromStr;

use alt_manager::AltManagerSetting;
use arbitrage::ArbitrageSettings;
use log::LevelFilter;
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Item, Value};
use utils::{region::Region, safe::ResultExt};

pub mod alt_manager;
pub mod arbitrage;
pub mod telegram;
pub mod wallet_manager;

pub mod providers;
pub use providers::{BloxrouteConfig, JitoConfig, JitoQuicknodeConfig};
use wallet_manager::WalletManagerConfig;

mod level_filter_serde;
use crate::{providers::NextblockConfig, telegram::Telegram};
use level_filter_serde::SerdeLevelFilter;

pub const DEFAULT_RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=***";
pub const DEFAULT_BLOCKHASH_RPC_URL: &str = "http://localhost:7799";
pub const DEFAULT_QUICKNODE_RPC_URL: &str = "https://convincing-holy-bush.solana-mainnet.quiknode.pro/***/";
pub const DEFAULT_HELIUS_SWQOS_RPC_URL: &str = "https://staked.helius-rpc.com?api-key=***";
//pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";
//pub const DEFAULT_WS_URL: &str = "ws://localhost:7800";
pub const DEFAULT_WS_URL: &str = "wss://mainnet.helius-rpc.com/?api-key=***";
pub const DEFAULT_HELIUS_WS_URL: &str = "wss://mainnet.helius-rpc.com/?api-key=***";

pub const DEFAULT_QUEST_DB: &str = "tcp::addr=localhost:5009;";

//pub const DEFAULT_GRPC_PEM: &str = "/etc/ssl/cert.pem";
pub const DEFAULT_GRPC_PEM: &str = "/etc/ssl/certs/ISRG_Root_X1.pem";
pub const DEFAULT_GRPC_AUTH_TOKEN: &str = "***";

//pub const DEFAULT_KEYPAIR: &str = "/Users/oxy/.config/solana/id.json";
pub const DEFAULT_KEYPAIR: &str = "/home/ubuntu/.config/solana/id.json";

pub const DEFAULT_EXECUTION_THREADS: u64 = 10;
pub const DEFAULT_OPTIMIZATION_THREADS: usize = 48;

pub const PROVIDERS: [&str; 4] = ["jito", "jitoquicknode", "bloxroute", "log"];
pub const DEFAULT_PROVIDER: &str = "log";

pub const REGIONS: [&str; 2] = ["EU", "US"];
pub const DEFAULT_REGION: &str = "EU";

pub const DEFAULT_JITO_QUICKNODE_LIMIT: u64 = 25;

pub const FORCED_RESTART: u64 = 600;
pub const DEFAULT_ALT_MANAGER_PATH: &str = "/home/ubuntu/arbitrage/tables";

pub const DEFAULT_NEXT_BLOCK_AUTH_TOKEN: &str = "***";

pub const DEFAULT_SERVICE_NAME: &str = "arb.service";

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
#[allow(unused)]
pub struct Config {
    pub name: String,
    pub log_level: SerdeLevelFilter,
    pub rpc: String,
    pub ws: String,
    pub blockhash_and_simulate_rpc: String,
    pub grpc_pem: String,
    pub grpc_auth_token: String,
    pub keypair: String,
    pub arbitrage: ArbitrageSettings,
    pub region: String,
    // providers
    pub providers: String,
    pub jito: providers::JitoConfig,
    pub jito_quicknode: providers::JitoQuicknodeConfig,
    pub bloxroute: providers::BloxrouteConfig,
    pub nextblock: NextblockConfig,
    pub wallet_manager: WalletManagerConfig,
    // address lookup table
    pub atl_manager: AltManagerSetting,
    pub telegram: Telegram,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            log_level: SerdeLevelFilter(LevelFilter::Info),
            rpc: String::from(DEFAULT_RPC_URL),
            name: "".to_string(),
            ws: String::from(DEFAULT_WS_URL),
            blockhash_and_simulate_rpc: String::from(DEFAULT_BLOCKHASH_RPC_URL),
            grpc_pem: String::from(DEFAULT_GRPC_PEM),
            grpc_auth_token: String::from(DEFAULT_GRPC_AUTH_TOKEN),
            keypair: String::from(DEFAULT_KEYPAIR),
            arbitrage: ArbitrageSettings::default(),
            region: String::from(DEFAULT_REGION),
            // providers
            providers: String::from("all"),
            jito: providers::JitoConfig::default(),
            jito_quicknode: providers::JitoQuicknodeConfig::default(),
            bloxroute: providers::BloxrouteConfig::default(),
            nextblock: providers::NextblockConfig::default(),
            wallet_manager: WalletManagerConfig::default(),
            // address lookup table
            atl_manager: AltManagerSetting::default(),
            telegram: Telegram::default(),
        }
    }
}

impl Config {
    pub fn get_region(&self) -> Region {
        match self.region.as_str() {
            "EU" => Region::EU,
            "US" => Region::US,
            "GB" => Region::GB,
            "JP" => Region::JP,
            "NL" => Region::NL,
            _ => {
                panic!("Invalid region specified.");
            }
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Self {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .unwrap();
        cfg.try_deserialize().or_panic("FailedToDeserializeConfig")
    }
}

static CONFIG_CELL: OnceCell<Config> = OnceCell::new();

pub static CONFIG: Lazy<&Config> = Lazy::new(|| CONFIG_CELL.get().expect("Settings has not been initialized"));

pub fn init(path: &str) {
    CONFIG_CELL
        .set(Config::load_from_file(path))
        .expect("Settings has been already initialized");
}

/*
/// Takes the entire TOML string and in each line of the form `key = number` inserts underscores into the number
fn pretty_with_underscores(toml_str: &str) -> String {
    // regex: captures lines `  key = 1234567`
    let re = Regex::new(r"(?m)^(\s*\w+\s*=\s*)(\d{4,})\s*$").unwrap();
    re.replace_all(toml_str, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let val = &caps[2];
        let n: u64 = val.parse().unwrap();
        let s = format_with_underscores(n);
        format!("{prefix}{s}")
    })
    .to_string()
}

// make 2_000_000_000 from 2000000000
fn format_with_underscores(n: u64) -> String {
    let s = n.to_string();
    let rev: Vec<char> = s.chars().rev().collect();
    let groups: Vec<String> = rev
        .chunks(3)
        .map(|chunk| chunk.iter().rev().collect())
        .collect();
    groups.into_iter().rev().collect::<Vec<_>>().join("_")
}
*/

/// Formats all integer values in TOML (including arrays) and leaves strings unchanged.
fn pretty_with_underscores(toml_str: &str) -> String {
    let mut doc = toml_str.parse::<DocumentMut>().expect("Error parsing TOML string");

    fn walk(item: &mut Item) {
        match item {
            Item::Value(val) => match val {
                Value::Integer(int) => {
                    // If the integer has a prefix or suffix, we need to preserve it
                    let prefix = int.decor().prefix().and_then(|r| r.as_str()).unwrap_or("");
                    let suffix = int.decor().suffix().and_then(|r| r.as_str()).unwrap_or("");
                    let n = *int.value();
                    let s = format_with_underscores(n);
                    let mut new_val = Value::from_str(&s).expect("Parsing formatted integer");
                    new_val = new_val.decorated(prefix, suffix);
                    *item = Item::Value(new_val);
                }
                Value::Array(arr) => {
                    for elem in arr.iter_mut() {
                        if let Value::Integer(int) = elem {
                            let prefix = int.decor().prefix().and_then(|r| r.as_str()).unwrap_or("");
                            let suffix = int.decor().suffix().and_then(|r| r.as_str()).unwrap_or("");
                            let n = *int.value();
                            let s = format_with_underscores(n);
                            let mut new_elem = Value::from_str(&s).expect("Parsing formatted integer");
                            new_elem = new_elem.decorated(prefix, suffix);
                            *elem = new_elem;
                        }
                    }
                }
                // Value::InlineTable(inline) => {
                //     for (_, itm) in inline.iter_mut() {
                //         walk(itm);
                //     }
                // }
                _ => {}
            },
            Item::Table(table) => {
                for (_, itm) in table.iter_mut() {
                    walk(itm);
                }
            }
            _ => {}
        }
    }

    // Start walking the document from the root
    for (_, itm) in doc.as_table_mut().iter_mut() {
        walk(itm);
    }

    doc.to_string()
}

fn format_with_underscores(n: i64) -> String {
    let s = n.to_string();
    let rev: Vec<char> = s.chars().rev().collect();
    let groups: Vec<String> = rev.chunks(3).map(|chunk| chunk.iter().rev().collect()).collect();
    groups.into_iter().rev().collect::<Vec<_>>().join("_")
}

pub fn dump_config(config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let toml_pretty = toml::to_string_pretty(config)?;
    let with_unders = pretty_with_underscores(&toml_pretty);
    Ok(with_unders)
}
