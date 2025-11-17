use serde::{Deserialize, Serialize};
use utils::region::Region;

use crate::{DEFAULT_GRPC_AUTH_TOKEN, DEFAULT_NEXT_BLOCK_AUTH_TOKEN, DEFAULT_QUICKNODE_RPC_URL};

// Jito

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct JitoConfig {
    pub simulate: bool,
    pub execution_threads: u64,
    pub filter: u64,
    pub priority_fee_percent: u64,
    pub tip_min_percent: u64,
    pub tip_max_percent: u64,
    pub engines: Vec<String>,
}

impl Default for JitoConfig {
    fn default() -> Self {
        JitoConfig {
            execution_threads: 1,
            simulate: true,
            filter: 50_000,
            priority_fee_percent: 2,
            tip_min_percent: 20,
            tip_max_percent: 50,
            engines: vec![
                "https://frankfurt.mainnet.block-engine.jito.wtf".to_string(),
                "https://amsterdam.mainnet.block-engine.jito.wtf".to_string(),
                "https://london.mainnet.block-engine.jito.wtf".to_string(),
                "https://dublin.mainnet.block-engine.jito.wtf".to_string(),
            ],
        }
    }
}

impl JitoConfig {
    /// Returns the primary and secondary Jito RPC URLs based on the region.
    pub fn get_regions(region: &Region) -> (String, Vec<String>) {
        match region {
            Region::EU => (
                "https://frankfurt.mainnet.block-engine.jito.wtf".to_string(),
                vec![
                    "https://amsterdam.mainnet.block-engine.jito.wtf".to_string(),
                    "https://london.mainnet.block-engine.jito.wtf".to_string(),
                    "https://dublin.mainnet.block-engine.jito.wtf".to_string(),
                ],
            ),
            Region::US => (
                "https://ny.mainnet.block-engine.jito.wtf".to_string(),
                vec![
                    "https://slc.mainnet.block-engine.jito.wtf".to_string(),
                    "https://tokyo.mainnet.block-engine.jito.wtf".to_string(),
                    "https://singapore.mainnet.block-engine.jito.wtf".to_string(),
                ],
            ),
            Region::GB => (
                "https://london.mainnet.block-engine.jito.wtf".to_string(),
                vec![
                    "https://amsterdam.mainnet.block-engine.jito.wtf".to_string(),
                    "https://frankfurt.mainnet.block-engine.jito.wtf".to_string(),
                    "https://dublin.mainnet.block-engine.jito.wtf".to_string(),
                ],
            ),
            Region::JP => (
                "https://tokyo.mainnet.block-engine.jito.wtf".to_string(),
                vec![
                    "https://singapore.mainnet.block-engine.jito.wtf".to_string(),
                    "https://ny.mainnet.block-engine.jito.wtf".to_string(),
                    "https://slc.mainnet.block-engine.jito.wtf".to_string(),
                ],
            ),
            Region::NL => (
                "https://amsterdam.mainnet.block-engine.jito.wtf".to_string(),
                vec![
                    "https://london.mainnet.block-engine.jito.wtf".to_string(),
                    "https://frankfurt.mainnet.block-engine.jito.wtf".to_string(),
                    "https://dublin.mainnet.block-engine.jito.wtf".to_string(),
                ],
            ),
        }
    }
}

// JitoQuicknode

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct JitoQuicknodeConfig {
    pub quicknode_rpc: String,
    pub quicknode_override_ip: String,
    pub simulate: bool,
    pub execution_threads: u64,
    pub filter: u64,
    pub priority_fee_percent: u64,
    pub tip_min_percent: u64,
    pub tip_max_percent: u64,
    pub max_priority_fee: u64,
}

impl Default for JitoQuicknodeConfig {
    fn default() -> Self {
        JitoQuicknodeConfig {
            quicknode_rpc: String::from(DEFAULT_QUICKNODE_RPC_URL),
            quicknode_override_ip: String::from(""),
            execution_threads: 60,
            simulate: true,
            filter: 25_000,
            priority_fee_percent: 2,
            tip_min_percent: 20,
            tip_max_percent: 50,
            max_priority_fee: 50_000,
        }
    }
}

impl JitoQuicknodeConfig {
    /// Returns the primary and secondary Jito QuickNode regions based on the region.
    pub fn get_regions(region: &Region) -> (String, Vec<String>) {
        match region {
            Region::EU => (
                String::from("frankfurt"),
                vec!["amsterdam".to_string(), "london".to_string()],
            ),
            Region::US => ("ny".to_string(), vec!["slc".to_string(), "tokyo".to_string()]),
            Region::GB => (
                "london".to_string(),
                vec!["amsterdam".to_string(), String::from("frankfurt")],
            ),
            Region::JP => (
                "tokyo".to_string(),
                vec!["ny".to_string(), "singapore".to_string(), "slc".to_string()],
            ),
            Region::NL => (
                String::from("frankfurt"),
                vec!["amsterdam".to_string(), "london".to_string()],
            ),
        }
    }
}

// Bloxroute

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct BloxrouteConfig {
    pub grpc_auth_token: String,
    pub execution_threads: u64,
    pub filter: u64,
    pub simulate: bool,
    /// Jito filter for Bloxroute, 0 means no processing
    pub filter_jito_secondary: u64,
    /// Paladin filter for Bloxroute, 0 means no processing
    pub filter_paladin: u64,
    pub filter_paladin_secondary: u64,
    /// SwQoS filter for Bloxroute, 0 means no processing
    pub filter_swqos: u64,
    /// Bloxroute fee && tip && limits
    pub priority_fee_percent: u64,
    pub tip_min_percent: u64,
    pub tip_max_percent: u64,
    pub max_priority_fee: u64,
    // Paladin fee && tip
    pub paladin_priority_fee_percent: u64,
    pub paladin_tip_min_percent: u64,
    pub paladin_tip_max_percent: u64,
    pub paladin_max_priority_fee: u64,
    // SwQoS fee && tip
    pub swqos_priority_fee_percent: u64,
    pub swqos_tip_min_percent: u64,
    pub swqos_max_percent: u64,
    // other
    pub use_global: bool,
    pub use_ofr: bool,
    pub ofr_http_server: String,
}

impl Default for BloxrouteConfig {
    fn default() -> Self {
        BloxrouteConfig {
            grpc_auth_token: String::from(DEFAULT_GRPC_AUTH_TOKEN),
            execution_threads: 5,
            filter: 1_050_000,
            simulate: true,
            filter_jito_secondary: 100_000_000,
            filter_paladin: 200_000_000,
            filter_paladin_secondary: 100_000_000,
            filter_swqos: 0,
            priority_fee_percent: 2,
            tip_min_percent: 20,
            tip_max_percent: 50,
            max_priority_fee: 50_000,
            paladin_priority_fee_percent: 44,
            paladin_tip_min_percent: 20,
            paladin_tip_max_percent: 50,
            paladin_max_priority_fee: 0,
            swqos_priority_fee_percent: 2,
            swqos_tip_min_percent: 20,
            swqos_max_percent: 50,
            use_global: false,
            use_ofr: false,
            ofr_http_server: "http://localhost:6080".to_string(),
        }
    }
}

impl BloxrouteConfig {
    /// Returns the primary and secondary Bloxroute RPC URLs based on the region.
    pub fn get_regions(region: &Region) -> (String, Vec<String>) {
        match region {
            Region::EU => (
                "https://germany.solana.dex.blxrbdn.com".to_string(),
                vec![
                    "https://uk.solana.dex.blxrbdn.com".to_string(),
                    "https://amsterdam.solana.dex.blxrbdn.com".to_string(),
                ],
            ),
            Region::US => (
                "https://ny.solana.dex.blxrbdn.com".to_string(),
                vec![
                    "https://la.solana.dex.blxrbdn.com".to_string(),
                    "https://tokyo.solana.dex.blxrbdn.com".to_string(),
                ],
            ),
            Region::GB => (
                "https://uk.solana.dex.blxrbdn.com".to_string(),
                vec![
                    "https://amsterdam.solana.dex.blxrbdn.com".to_string(),
                    "https://germany.solana.dex.blxrbdn.com".to_string(),
                ],
            ),
            Region::JP => (
                "https://tokyo.solana.dex.blxrbdn.com".to_string(),
                vec![
                    "https://ny.solana.dex.blxrbdn.com".to_string(),
                    "https://la.solana.dex.blxrbdn.com".to_string(),
                ],
            ),
            Region::NL => (
                "https://amsterdam.solana.dex.blxrbdn.com".to_string(),
                vec![
                    "https://uk.solana.dex.blxrbdn.com".to_string(),
                    "https://germany.solana.dex.blxrbdn.com".to_string(),
                ],
            ),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
#[allow(unused)]
pub struct JitoSender {
    pub url: String,
    pub channel: String,
}

impl Default for JitoSender {
    fn default() -> Self {
        JitoSender {
            url: String::from("https://mainnet.block-engine.jito.wtf"),
            channel: String::from("0"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct NextblockConfig {
    pub grpc_auth_token: String,
    pub simulate: bool,
    pub execution_threads: u64,
    pub filter: u64,
    pub priority_fee_percent: u64,
    pub tip_min_percent: u64,
    pub tip_max_percent: u64,
    pub max_priority_fee: u64,
    pub revert_on_fail: bool,
}

impl Default for NextblockConfig {
    fn default() -> Self {
        NextblockConfig {
            grpc_auth_token: String::from(DEFAULT_NEXT_BLOCK_AUTH_TOKEN),
            execution_threads: 10,
            filter: 1_050_000,
            simulate: true,
            priority_fee_percent: 2,
            tip_min_percent: 20,
            tip_max_percent: 50,
            max_priority_fee: 50_000,
            revert_on_fail: true,
        }
    }
}

impl NextblockConfig {
    /// Returns the primary and secondary Bloxroute RPC URLs based on the region.
    pub fn get_regions(region: &Region) -> String {
        match region {
            Region::EU => "http://fra.nextblock.io".to_string(),
            Region::US => "http://ny.nextblock.io".to_string(),
            // http://slc.nextblock.io
            // http://sgp.nextblock.io
            Region::GB => "http://london.nextblock.io".to_string(),
            Region::JP => "http://tokyo.nextblock.io".to_string(),
            Region::NL => "http://fra.nextblock.io".to_string(),
        }
    }
}
