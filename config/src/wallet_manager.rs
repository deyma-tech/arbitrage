use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WalletManagerConfig {
    pub max_wsol: u64,
    pub delta_max_wsol: u64,
}

impl Default for WalletManagerConfig {
    fn default() -> Self {
        WalletManagerConfig {
            max_wsol: 2_000_000_000,
            delta_max_wsol: 100_000_000,
        }
    }
}
