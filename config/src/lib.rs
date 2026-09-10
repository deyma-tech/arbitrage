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
pub use providers::{BloxrouteConfig, HttpRelayConfig, JitoConfig, JitoQuicknodeConfig};
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

pub const PROVIDERS: [&str; 7] = [
    "jito",
    "jitoquicknode",
    "bloxroute",
    "nextblock",
    "astralane",
    "nozomi",
    "log",
];
pub const DEFAULT_PROVIDER: &str = "log";

pub const REGIONS: [&str; 2] = ["EU", "US"];
pub const DEFAULT_REGION: &str = "EU";

pub const DEFAULT_JITO_QUICKNODE_LIMIT: u64 = 25;

pub const FORCED_RESTART: u64 = 600;
/// Runtime ALT metadata is local to the checkout/process and must not depend
/// on the old VM-only `/home/ubuntu/arbitrage` path.
pub const DEFAULT_ALT_MANAGER_PATH: &str = "data";

pub const DEFAULT_NEXT_BLOCK_AUTH_TOKEN: &str = "***";
pub const DEFAULT_ASTRALANE_ENDPOINT: &str = "https://lim.gateway.astralane.io/iris2";
pub const DEFAULT_NOZOMI_ENDPOINT: &str = "https://nozomi.temporal.xyz/api/sendTransaction2";

pub const DEFAULT_SERVICE_NAME: &str = "arb.service";
pub const DEFAULT_ARB_EXECUTOR_V2_PROGRAM_ID: &str = "HPXVR7EQc1KE9XAY4wPoTakBTFgs4SNZW75zBaVfufW3";

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
#[allow(unused)]
pub struct Config {
    pub name: String,
    pub log_level: SerdeLevelFilter,
    pub rpc: String,
    pub ws: String,
    pub blockhash_and_simulate_rpc: String,
    /// Durable nonce accounts used by execution providers. Values may be
    /// supplied as a TOML array or through ARB_NONCE_ACCOUNTS as a comma-
    /// separated list.
    pub nonce_accounts: Vec<String>,
    /// How long a submitted nonce transaction may remain unfinalized before
    /// its nonce account is quarantined rather than reused.
    pub nonce_confirmation_timeout_ms: u64,
    pub grpc_pem: String,
    pub grpc_auth_token: String,
    pub keypair: String,
    /// Sending transactions is opt-in and remains disabled by default.
    pub enable_execution: bool,
    /// First canary uses only one Jito region to avoid duplicate fills while
    /// multi-relay fan-out is being validated.
    pub canary_single_provider: bool,
    /// Deployed isolated PumpSwap/DLMM executor. This is never inferred from
    /// the legacy SWAP_PROGRAM_ID constant.
    pub arb_executor_v2_program_id: String,
    pub arbitrage: ArbitrageSettings,
    pub region: String,
    // providers
    pub providers: String,
    pub jito: providers::JitoConfig,
    pub jito_quicknode: providers::JitoQuicknodeConfig,
    pub bloxroute: providers::BloxrouteConfig,
    pub nextblock: NextblockConfig,
    pub astralane: providers::HttpRelayConfig,
    pub nozomi: providers::HttpRelayConfig,
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
            nonce_accounts: vec![],
            nonce_confirmation_timeout_ms: 30_000,
            grpc_pem: String::from(DEFAULT_GRPC_PEM),
            grpc_auth_token: String::from(DEFAULT_GRPC_AUTH_TOKEN),
            keypair: String::from(DEFAULT_KEYPAIR),
            enable_execution: false,
            canary_single_provider: false,
            arb_executor_v2_program_id: String::from(DEFAULT_ARB_EXECUTOR_V2_PROGRAM_ID),
            arbitrage: ArbitrageSettings::default(),
            region: String::from(DEFAULT_REGION),
            // providers
            // Dry-run is the safe default. Sending must be explicitly selected.
            providers: String::from(DEFAULT_PROVIDER),
            jito: providers::JitoConfig::default(),
            jito_quicknode: providers::JitoQuicknodeConfig::default(),
            bloxroute: providers::BloxrouteConfig::default(),
            nextblock: providers::NextblockConfig::default(),
            astralane: providers::HttpRelayConfig {
                endpoint: String::from(DEFAULT_ASTRALANE_ENDPOINT),
                ..providers::HttpRelayConfig::default()
            },
            nozomi: providers::HttpRelayConfig {
                endpoint: String::from(DEFAULT_NOZOMI_ENDPOINT),
                ..providers::HttpRelayConfig::default()
            },
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

/// Load simple KEY=VALUE entries without adding a runtime dependency. Existing
/// process environment variables win over values from this local file.
pub fn load_dotenv(path: &str) {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return,
    };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || std::env::var_os(key).is_some() {
            continue;
        }
        let value = value
            .trim()
            .trim_matches(|character| character == '"' || character == '\'');
        std::env::set_var(key, value);
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Self {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .unwrap();
        let mut cfg: Config = cfg.try_deserialize().or_panic("FailedToDeserializeConfig");

        // Explicit TOML values remain authoritative. Helius is preferred for
        // HTTP reads (balance, nonce state and simulation) when configured;
        // Yellowstone/Chainstack remains the live account-stream transport.
        if cfg.rpc == DEFAULT_RPC_URL {
            if let Some(value) = first_env(&["HELIUS_RPC_URL", "SOLANA_RPC_URL", "CHAINSTACK_TRADER_RPC_URL"]) {
                cfg.rpc = value;
            }
        }
        if cfg.ws == DEFAULT_WS_URL {
            if let Some(value) = first_env(&["CHAINSTACK_TRADER_WS_URL", "SOLANA_WS_URL"]) {
                cfg.ws = value;
            }
        }
        if cfg.blockhash_and_simulate_rpc == DEFAULT_BLOCKHASH_RPC_URL {
            cfg.blockhash_and_simulate_rpc = cfg.rpc.clone();
        }
        if cfg.nonce_accounts.is_empty() {
            if let Some(value) = first_env(&["ARB_NONCE_ACCOUNTS"]) {
                cfg.nonce_accounts = value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
            }
        }
        if cfg.nonce_confirmation_timeout_ms == 30_000 {
            if let Some(value) = first_env(&["ARB_NONCE_CONFIRMATION_TIMEOUT_MS"]) {
                cfg.nonce_confirmation_timeout_ms = value
                    .parse()
                    .unwrap_or_else(|err| panic!("ARB_NONCE_CONFIRMATION_TIMEOUT_MS inválido {value}: {err}"));
            }
        }
        if cfg.keypair == DEFAULT_KEYPAIR {
            if let Some(value) = first_env(&["BOT_KEYPAIR_PATH"]) {
                cfg.keypair = value;
            }
        }
        if cfg.arb_executor_v2_program_id == DEFAULT_ARB_EXECUTOR_V2_PROGRAM_ID {
            if let Some(value) = first_env(&["ARB_EXECUTOR_V2_PROGRAM_ID", "SWAP_PROGRAM_ID"]) {
                cfg.arb_executor_v2_program_id = value;
            }
        }
        if cfg.nextblock.grpc_auth_token == DEFAULT_NEXT_BLOCK_AUTH_TOKEN {
            if let Some(value) = first_env(&["NEXTBLOCK_API_KEY"]) {
                cfg.nextblock.grpc_auth_token = value;
            }
        }
        if cfg.astralane.api_key.is_empty() {
            if let Some(value) = first_env(&["ASTRALANE_API_KEY"]) {
                cfg.astralane.api_key = value;
            }
        }
        if cfg.nozomi.api_key.is_empty() {
            if let Some(value) = first_env(&["NOZOMI_API_KEY"]) {
                cfg.nozomi.api_key = value;
            }
        }
        if cfg.astralane.endpoint.is_empty() {
            cfg.astralane.endpoint = DEFAULT_ASTRALANE_ENDPOINT.to_string();
        }
        if cfg.astralane.endpoint == DEFAULT_ASTRALANE_ENDPOINT {
            if let Some(value) = first_env(&["ASTRALANE_ENDPOINT"]) {
                cfg.astralane.endpoint = value;
            }
        }
        if cfg.nozomi.endpoint.is_empty() {
            cfg.nozomi.endpoint = DEFAULT_NOZOMI_ENDPOINT.to_string();
        }
        if cfg.nozomi.endpoint == DEFAULT_NOZOMI_ENDPOINT {
            if let Some(value) = first_env(&["NOZOMI_ENDPOINT"]) {
                cfg.nozomi.endpoint = value;
            }
        }
        cfg
    }
}

fn first_env(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    })
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
