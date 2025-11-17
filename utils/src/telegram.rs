use std::sync::{Arc, OnceLock};

use log::{error, warn};
use teloxide::prelude::*;

pub struct Telegram {
    pub bot: Option<Bot>,
    pub notification_channel_id: Option<i64>,
}

impl Telegram {
    pub fn new(teloxide_token: Option<String>, notification_channel_id: Option<i64>) -> Self {
        // Some() and is not empty
        let token = match teloxide_token.filter(|t| !t.trim().is_empty()) {
            Some(token) => token,
            None => match std::env::var("TELOXIDE_TOKEN") {
                Ok(token) if !token.trim().is_empty() => token,
                _ => {
                    warn!("Telegram is not configured. Missing or empty TELOXIDE_TOKEN");
                    return Self {
                        bot: None,
                        notification_channel_id: None,
                    };
                }
            },
        };
        Self {
            bot: Some(Bot::new(token)),
            notification_channel_id,
        }
    }

    pub async fn send_message(&self, channel_id: i64, message: &str) {
        if channel_id == 0 {
            return;
        }
        if let Some(bot) = &self.bot {
            let channel = ChatId(channel_id);
            match bot.send_message(channel, message).send().await {
                Ok(_) => {}
                Err(e) => error!("Failed to send message! {e}"),
            }
        }
    }
}

impl Default for Telegram {
    fn default() -> Self {
        Self::new(None, None)
    }
}

static TELEGRAM: OnceLock<Arc<Telegram>> = OnceLock::new();

pub fn init(teloxide_token: &str, notification_channel_id: Option<i64>) -> Arc<Telegram> {
    TELEGRAM
        .get_or_init(|| Arc::new(Telegram::new(Some(teloxide_token.to_string()), notification_channel_id)))
        .clone()
}

pub async fn notify(message: &str) {
    if let Some(telegram) = TELEGRAM.get() {
        if let Some(notification_channel_id) = telegram.notification_channel_id {
            if notification_channel_id == 0 {
                return;
            }
            telegram.send_message(notification_channel_id, message).await;
        }
    }
}
