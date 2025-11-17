use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Telegram {
    pub teloxide_token: String,
    pub notification_channel_id: i64,
}
