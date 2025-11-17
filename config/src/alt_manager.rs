use serde::{Deserialize, Serialize};

use crate::{DEFAULT_ALT_MANAGER_PATH, DEFAULT_RPC_URL, DEFAULT_WS_URL};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
#[allow(unused)]
pub struct AltManagerSetting {
    pub rpc: String,
    pub ws: String,
    pub path: String,
}

impl Default for AltManagerSetting {
    fn default() -> Self {
        AltManagerSetting {
            rpc: String::from(DEFAULT_RPC_URL),
            ws: String::from(DEFAULT_WS_URL),
            path: String::from(DEFAULT_ALT_MANAGER_PATH),
        }
    }
}
