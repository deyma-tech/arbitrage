use std::collections::HashSet;

use crate::constants::JITO_VALIDATORS;
use crate::validators::get_validators_metadata;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JitoValidators {
    pub validators: Vec<Validator>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    #[serde(rename = "vote_account")]
    pub vote_account: String,
    #[serde(rename = "mev_commission_bps")]
    pub mev_commission_bps: i64,
    #[serde(rename = "mev_rewards")]
    pub mev_rewards: i64,
    #[serde(rename = "running_jito")]
    pub running_jito: bool,
    #[serde(rename = "active_stake")]
    pub active_stake: i64,
}

pub async fn get_jito_validators() -> Result<JitoValidators> {
    let result = reqwest::get(JITO_VALIDATORS).await?;
    let jito_validators = result.json::<JitoValidators>().await?;
    Ok(jito_validators)
}

pub struct JitoValidator {
    pub node_keys: HashSet<String>,
}

impl JitoValidator {
    pub async fn new() -> Self {
        let jito_validators = get_jito_validators().await.unwrap();
        let validators_metadata = get_validators_metadata();
        let mut node_keys = HashSet::new();
        for jito_validator in jito_validators.validators {
            let vote_account = jito_validator.vote_account.replace("\"", "");
            if jito_validator.running_jito {
                let jito_node_key = validators_metadata
                    .validators
                    .iter()
                    .find(|v| v.vote_account == vote_account)
                    .context(format!("missing vote account: {vote_account}"))
                    .unwrap()
                    .node_key
                    .clone();
                node_keys.insert(jito_node_key);
            }
        }
        Self { node_keys }
    }

    pub fn is_jito(&self, node_key: &str) -> bool {
        self.node_keys.contains(node_key)
    }
}
