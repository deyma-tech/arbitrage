use arb_core::gpa::PoolToCalculator;
use std::collections::BTreeMap;
use std::time::Duration;
use utils::rnd::rnd_range;

use ahash::{AHashMap, AHashSet};
use config::CONFIG as cfg;
use futures::StreamExt;
use log::{error, info, warn};
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::rpc_response::Response;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
};
use solana_sdk::{commitment_config::CommitmentConfig, message::AddressLookupTableAccount, pubkey::Pubkey};
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use utils::constants::ADDRESS_LOOKUP_TABLE_PROGRAM_ID;
use utils::now;
use utils::safe::ResultExt;

#[derive(Debug, Clone)]
pub enum TableResponse {
    RemoveTable(Pubkey),
    UpdateTable(AddressLookupTableAccount),
    AddAndRefresh(Pubkey, AddressLookupTableAccount),
}

impl TableResponse {
    pub fn apply(
        self,
        pool_pubkey_to_table_key: &mut AHashMap<Pubkey, Pubkey>,
        table_key_to_table: &mut AHashMap<Pubkey, AddressLookupTableAccount>,
    ) {
        match self {
            TableResponse::RemoveTable(table_key) => {
                table_key_to_table.remove(&table_key);
            }
            TableResponse::UpdateTable(lookup_table_account) => {
                table_key_to_table.insert(lookup_table_account.key, lookup_table_account);
            }
            TableResponse::AddAndRefresh(pool_pubkey, lookup_table_account) => {
                pool_pubkey_to_table_key.insert(pool_pubkey, lookup_table_account.key);
                table_key_to_table.insert(lookup_table_account.key, lookup_table_account);
            }
        }
    }
}

// TODO: unused
pub async fn process_alt_cache(
    mut rx: UnboundedReceiver<PoolToCalculator>,
    mut pool_pubkey_to_table_key: AHashMap<Pubkey, Pubkey>,
    mut table_key_to_table: AHashMap<Pubkey, AddressLookupTableAccount>,
) -> (tokio::sync::broadcast::Sender<TableResponse>, tokio::runtime::Runtime) {
    let (tx_tables, mut rx_tables) = tokio::sync::mpsc::unbounded_channel::<(u64, u64, AddressLookupTableAccount)>();

    let (tx_table_response, mut _rx_response) = tokio::sync::broadcast::channel::<TableResponse>(100);
    info!("TB ALT cache starts");

    tokio::spawn({
        async move {
            loop {
                info!("TB Subscribing to websocket ...");
                let ws_client = match PubsubClient::new(&cfg.ws).await {
                    Ok(client) => client,
                    Err(err) => {
                        error!("Error creating WebSocket client: {:?}", err);
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                        continue;
                    }
                };

                let mut subscriber = match ws_client
                    .program_subscribe(
                        &ADDRESS_LOOKUP_TABLE_PROGRAM_ID,
                        Some(RpcProgramAccountsConfig {
                            filters: None,
                            account_config: RpcAccountInfoConfig {
                                encoding: None,
                                data_slice: None,
                                commitment: Some(CommitmentConfig::finalized()),
                                min_context_slot: None,
                            },
                            with_context: Some(true),
                            sort_results: None,
                        }),
                    )
                    .await
                {
                    Ok((subscriber, _)) => subscriber,
                    Err(err) => {
                        error!("Error subscribing to account: {:?}", err);
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                        continue;
                    }
                };

                while let Some(response) = subscriber.next().await {
                    let Response { context, value } = response;
                    let slot = context.slot;
                    let pubkey = Pubkey::from_str_const(&value.pubkey);
                    let data = value.account.data.decode();
                    match data {
                        None => {}
                        Some(data) => {
                            let lookup_table_account =
                                match arb_core::table::get_address_lookup_table(data.as_slice(), pubkey) {
                                    Ok(lookup_table_account) => lookup_table_account,
                                    Err(err) => {
                                        warn!("Failed to parse AddressLookupTableAccount: {:?}", err);
                                        continue;
                                    }
                                };

                            let lookup_table = match AddressLookupTable::deserialize(data.as_slice()) {
                                Ok(lt) => lt,
                                Err(err) => {
                                    warn!("Failed to parse AddressLookupTableAccount: {:?}", err);
                                    continue;
                                }
                            };
                            let deactivation_slot = lookup_table.meta.deactivation_slot;
                            info!("TB Table: {} updated data sent", lookup_table_account.key);
                            match tx_tables.send((slot, deactivation_slot, lookup_table_account)) {
                                Ok(_) => {}
                                Err(err) => {
                                    warn!("Failed to send table update: {:?}", err);
                                    continue;
                                }
                            }
                        }
                    }
                }

                warn!("WebSocket connection closed, reconnecting...");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        }
    });

    let (tx_tick, mut rx_tick) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(150));
        loop {
            interval.tick().await;
            let _ = tx_tick.send(now::as_millis());
        }
    });

    let table_runtime = tokio::runtime::Runtime::new().or_panic("FailedToCreateRuntime");
    table_runtime.spawn({
        info!("TB runtime spawned");
        let tx_table_response = tx_table_response.clone();
        let mut new_tables: AHashMap<Pubkey, AddressLookupTableAccount> = AHashMap::default();
        let mut deactivation_slot_to_table_key: BTreeMap<u64, AHashSet<Pubkey>> = BTreeMap::new();
        let mut delayed_to_pool_calculator: BTreeMap<u64, PoolToCalculator> = BTreeMap::new();

        async move {
            loop {
                select! {

                    Some((slot, deactivation_slot, lookup_table_account)) = rx_tables.recv() => {
                        if slot > deactivation_slot && table_key_to_table.contains_key(&lookup_table_account.key) {
                            table_key_to_table.remove(&lookup_table_account.key);
                            let  _ = tx_table_response.send(TableResponse::RemoveTable(lookup_table_account.key));
                        } else if table_key_to_table.contains_key(&lookup_table_account.key) {
                            table_key_to_table.insert(lookup_table_account.key, lookup_table_account.clone());
                            deactivation_slot_to_table_key
                                .entry(deactivation_slot)
                                .and_modify(|e| { e.insert(lookup_table_account.key); })
                                .or_insert(AHashSet::from([lookup_table_account.key]));
                            let  _ = tx_table_response.send(TableResponse::UpdateTable(lookup_table_account));
                        } else {
                            new_tables.insert(lookup_table_account.key, lookup_table_account.clone());
                            deactivation_slot_to_table_key
                                .entry(deactivation_slot)
                                .and_modify(|e| { e.insert(lookup_table_account.key); })
                                .or_insert(AHashSet::from([lookup_table_account.key]));
                        }

                        let first_value = deactivation_slot_to_table_key.first_key_value();
                        if let Some((first_deactivation_slot, _pubkeys)) = first_value {
                            if *first_deactivation_slot < slot {
                                let keys_to_remove = deactivation_slot_to_table_key.iter()
                                    .filter(|(deactivation_slot, _)| *deactivation_slot < &slot)
                                    .map(|(slot, _pubkeys)| *slot).collect::<Vec<u64>>();
                                for key in keys_to_remove.iter() {
                                    let pubkeys = deactivation_slot_to_table_key.remove(key);
                                    if let Some(pubkeys) = pubkeys {
                                        for pubkey in pubkeys.iter() {
                                            if let Some(_removed) = new_tables.remove(pubkey) {
                                                new_tables.remove(pubkey);
                                            }
                                        }
                                        for pubkey in pubkeys.iter() {
                                            if let Some(_removed) = table_key_to_table.remove(pubkey) {
                                                let  _ = tx_table_response.send(TableResponse::RemoveTable(*pubkey));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                    }

                    Some(pool_to_calculator) = rx.recv() => {
                        if pool_pubkey_to_table_key.contains_key(&pool_to_calculator.pubkey) {
                            continue;
                        }
                        info!("TB New pool: {}", pool_to_calculator.pubkey);
                        delayed_to_pool_calculator.insert((now::as_millis() + 300_000) as u64, pool_to_calculator);
                    }

                    Some(tick) = rx_tick.recv() => {
                        let keys_to_check = delayed_to_pool_calculator.iter()
                            .filter(|(timestamp, _)| **timestamp < tick as u64)
                            .map(|(timestamp, _)| *timestamp)
                            .collect::<Vec<u64>>();
                        info!("TB Searching tables for {} pools", keys_to_check.len());
                        for key in keys_to_check.iter() {
                            let removed = delayed_to_pool_calculator.remove(key);
                            let search_result = search(&removed, & table_key_to_table, & new_tables);
                            if let Some(table_response) = search_result {
                                let _ = tx_table_response.send(table_response.clone());
                                if let TableResponse::AddAndRefresh(pool_pubkey,table) = table_response {
                                        info!("TB for pool: {} found table: {}", pool_pubkey, table.key);
                                        pool_pubkey_to_table_key.insert(pool_pubkey, table.key);
                                        if !table_key_to_table.contains_key(&table.key) {
                                            let removed = new_tables.remove(&table.key);
                                            match removed {
                                            None => {}
                                            Some(removed_table) => {
                                                table_key_to_table.insert(removed_table.key, removed_table);
                                            }
                                        }
                                        }
                                }
                            } else {
                            // reinsert ...
                                match removed {
                                    None => {}
                                    Some(pool_to_calculator) => {
                                        let random_delay = rnd_range(0, 900_000);
                                        delayed_to_pool_calculator.insert((now::as_millis() + 300_000) as u64 + random_delay, pool_to_calculator);
                                    }
                                }
                            }
                        }

                    }

                }
            }
        }
    });

    (tx_table_response, table_runtime)
}

fn search(
    pool_to_calculator: &Option<PoolToCalculator>,
    table_table: &AHashMap<Pubkey, AddressLookupTableAccount>,
    new_tables: &AHashMap<Pubkey, AddressLookupTableAccount>,
) -> Option<TableResponse> {
    match pool_to_calculator {
        None => {}
        Some(pool_to_calculator) => {
            let found = table_table
                .iter()
                .find(|(_pubkey, table)| address_lookup_table_contains(pool_to_calculator, table));
            if let Some((_, table)) = found {
                return Some(TableResponse::AddAndRefresh(pool_to_calculator.pubkey, table.clone()));
            }
            let found = new_tables
                .iter()
                .find(|(_pubkey, table)| address_lookup_table_contains(pool_to_calculator, table));
            if let Some((_, table)) = found {
                return Some(TableResponse::AddAndRefresh(pool_to_calculator.pubkey, table.clone()));
            }
        }
    }
    None
}

fn address_lookup_table_contains(pool_to_calculator: &PoolToCalculator, table: &AddressLookupTableAccount) -> bool {
    if !table.addresses.contains(&pool_to_calculator.pubkey) {
        return false;
    }
    for key in pool_to_calculator.keys.iter().flatten() {
        if table.addresses.contains(key) {
            continue;
        } else {
            return false;
        }
    }
    true
}

pub fn get_address_lookup_tables(
    pool_pubkeys: &[&Pubkey],
    pool_to_table_key: &AHashMap<Pubkey, Pubkey>,
    table_key_to_table: &AHashMap<Pubkey, AddressLookupTableAccount>,
) -> Vec<AddressLookupTableAccount> {
    let mut alts = vec![];

    for pool in pool_pubkeys.iter() {
        let table_key = pool_to_table_key.get(pool);
        if let Some(table_key) = table_key {
            let table = table_key_to_table.get(table_key);
            if let Some(table) = table {
                if !alts.contains(table) {
                    alts.push(table.clone());
                }
            }
        }
    }

    alts
}

pub fn get_address_lookup_tables_no_ref(
    pool_pubkeys: &[Pubkey],
    pool_to_table_key: &AHashMap<Pubkey, Pubkey>,
    table_key_to_table: &AHashMap<Pubkey, AddressLookupTableAccount>,
) -> Vec<AddressLookupTableAccount> {
    let mut alts = vec![];

    for pool in pool_pubkeys.iter() {
        let table_key = pool_to_table_key.get(pool);
        if let Some(table_key) = table_key {
            let table = table_key_to_table.get(table_key);
            if let Some(table) = table {
                if !alts.contains(table) {
                    alts.push(table.clone());
                }
            }
        }
    }
    alts
}
