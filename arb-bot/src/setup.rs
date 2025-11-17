use ahash::AHashMap;
use config::CONFIG as cfg;
use kv::{Config, Raw, Store};
use log::{info, warn};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;
use utils::constants::{ADDRESS_LOOKUP_TABLE, FLASHLOAN_ID};
use utils::safe::ResultExt;

use crate::optimizer::UpdateAlts;

pub fn get_keypair() -> Keypair {
    let result = solana_sdk::signature::read_keypair_file(&cfg.keypair);
    match result {
        Ok(keypair) => keypair,
        Err(err) => {
            panic!("{err:?}")
        }
    }
}

pub async fn fetch_balance() -> u64 {
    let keypair = solana_sdk::signature::read_keypair_file(&cfg.keypair);
    let rpc_client = RpcClient::new(cfg.rpc.clone());
    match keypair {
        Ok(keypair) => {
            let balance = rpc_client.get_balance(&keypair.pubkey()).await;
            match balance {
                Ok(balance) => balance,
                Err(_) => {
                    panic!("FailedToLoadBalance")
                }
            }
        }
        Err(_) => {
            panic!("FailedTooLoadKeypair")
        }
    }
}

pub async fn fetch_flashloan_keys() -> AHashMap<Pubkey, (Pubkey, Pubkey)> {
    let rpc_client = RpcClient::new(cfg.rpc.clone());
    let accounts = rpc_client
        .get_program_accounts(&FLASHLOAN_ID)
        .await
        .or_panic("FailedToLoadFlashloanAccounts");

    let mut flashloan_keys = AHashMap::new();
    for (pool_pubkey, account) in accounts.iter() {
        if account.data.len() == 65 {
            let mint = Pubkey::try_from(&account.data[..32]);
            let mint = match mint {
                Ok(mint) => mint,
                Err(_) => {
                    warn!("FailedToDeserializeMint");
                    continue;
                }
            };
            let ata = get_associated_token_address(pool_pubkey, &mint);
            info!("Flashloan pool: {} mint: {}, pool ata: {}", pool_pubkey, mint, ata);
            flashloan_keys.insert(mint, (*pool_pubkey, ata));
        }
    }
    if flashloan_keys.is_empty() {
        panic!("No flashloan pools found");
    }
    flashloan_keys
}

pub async fn fetch_alt() -> AddressLookupTableAccount {
    let rpc_client = RpcClient::new(cfg.rpc.clone());
    let alt = rpc_client.get_account_data(&ADDRESS_LOOKUP_TABLE).await;
    match alt {
        Ok(data) => {
            arb_core::table::get_address_lookup_table(&data, ADDRESS_LOOKUP_TABLE).or_panic("FailedToDeserializeALT")
        }
        Err(_) => panic!("FailedToLoadALT"),
    }
}

pub async fn fetch_blockhash() -> Hash {
    let rpc_client = RpcClient::new(cfg.blockhash_and_simulate_rpc.clone());
    rpc_client
        .get_latest_blockhash()
        .await
        .or_panic("FailedToLoadBlockHash")
}

pub fn get_tables() -> (
    AHashMap<Pubkey, Pubkey>,
    AHashMap<Pubkey, AddressLookupTableAccount>,
    AHashMap<Pubkey, Pubkey>,
) {
    let kv_cfg = Config::new(&cfg.atl_manager.path);
    let store = match Store::new(kv_cfg) {
        Ok(store) => store,
        Err(err) => {
            warn!("FailedToLoadStore: {:?}", err);
            return (AHashMap::default(), AHashMap::default(), AHashMap::default());
        }
    };
    let map_bucket = store.bucket::<Raw, Raw>(Some("map_bucket")).unwrap();

    let tables_bucket = store.bucket::<Raw, Raw>(Some("tables_bucket")).unwrap();

    let mut pool_to_table_key = AHashMap::new();
    let mut table_key_to_table = AHashMap::new();
    for result in map_bucket.iter() {
        match result {
            Ok(item) => {
                let pool_pubkey = item.key::<Raw>();
                let table_pubkey = item.value::<Raw>();
                if let (Ok(pool_pubkey), Ok(table_pubkey)) = (pool_pubkey, table_pubkey) {
                    // info!("searching: {:?}", pool_pubkey);
                    let table = tables_bucket.get(&table_pubkey);
                    let pool_pubkey = Pubkey::new_from_array(pool_pubkey.to_vec().try_into().unwrap());
                    if let Ok(Some(table)) = table {
                        let table_pubkey = Pubkey::new_from_array(table_pubkey.to_vec().try_into().unwrap());
                        // info!("found");
                        let lookup_table =
                            match arb_core::table::get_address_lookup_table(table.to_vec().as_slice(), table_pubkey) {
                                Ok(lookup_table) => {
                                    // info!("found");
                                    lookup_table
                                }
                                Err(_) => {
                                    continue;
                                }
                            };
                        table_key_to_table.insert(table_pubkey, lookup_table);
                        pool_to_table_key.insert(pool_pubkey, table_pubkey);
                    }
                }
            }
            Err(_) => continue,
        }
    }
    if pool_to_table_key.is_empty() && table_key_to_table.is_empty() {
        warn!("Empty Tables!!!!");
    }

    let optionals_bucket = store.bucket::<Raw, Raw>(Some("optionals_bucket")).unwrap();

    let mut pool_pubkey_to_optionals = AHashMap::default();
    for result in optionals_bucket.iter() {
        match result {
            Ok(item) => {
                let pool_pubkey = item.key::<Raw>();
                let optional_pubkey = item.value::<Raw>();
                if let (Ok(pool_pubkey), Ok(optional_pubkey)) = (pool_pubkey, optional_pubkey) {
                    let pool_pubkey = Pubkey::new_from_array(pool_pubkey.to_vec().try_into().unwrap());
                    let optional_pubkey = Pubkey::new_from_array(optional_pubkey.to_vec().try_into().unwrap());
                    pool_pubkey_to_optionals.insert(pool_pubkey, optional_pubkey);
                }
            }
            Err(_) => continue,
        }
    }

    if let Err(e) = map_bucket.flush() {
        warn!("Error flushing map_bucket: {:?}", e);
    }

    if let Err(e) = optionals_bucket.flush() {
        warn!("Error flushing optionals_bucket: {:?}", e);
    }

    if let Err(e) = tables_bucket.flush() {
        warn!("Error flushing tables_bucket: {:?}", e);
    }

    drop(map_bucket);
    drop(tables_bucket);
    drop(optionals_bucket);
    drop(store);

    (pool_to_table_key, table_key_to_table, pool_pubkey_to_optionals)
}

pub fn get_tables_v1(
    store: &Store,
) -> (
    AHashMap<Pubkey, Pubkey>,
    AHashMap<Pubkey, AddressLookupTableAccount>,
    AHashMap<Pubkey, Pubkey>,
) {
    let map_bucket = store.bucket::<Raw, Raw>(Some("map_bucket")).unwrap();

    let tables_bucket = store.bucket::<Raw, Raw>(Some("tables_bucket")).unwrap();

    let mut pool_to_table_key = AHashMap::new();
    let mut table_key_to_table = AHashMap::new();
    for result in map_bucket.iter() {
        match result {
            Ok(item) => {
                let pool_pubkey = item.key::<Raw>();
                let table_pubkey = item.value::<Raw>();
                if let (Ok(pool_pubkey), Ok(table_pubkey)) = (pool_pubkey, table_pubkey) {
                    // info!("searching: {:?}", pool_pubkey);
                    let table = tables_bucket.get(&table_pubkey);
                    let pool_pubkey = Pubkey::new_from_array(pool_pubkey.to_vec().try_into().unwrap());
                    if let Ok(Some(table)) = table {
                        let table_pubkey = Pubkey::new_from_array(table_pubkey.to_vec().try_into().unwrap());
                        // info!("found");
                        let lookup_table =
                            match arb_core::table::get_address_lookup_table(table.to_vec().as_slice(), table_pubkey) {
                                Ok(lookup_table) => {
                                    // info!("found");
                                    lookup_table
                                }
                                Err(_) => {
                                    continue;
                                }
                            };
                        table_key_to_table.insert(table_pubkey, lookup_table);
                        pool_to_table_key.insert(pool_pubkey, table_pubkey);
                    }
                }
            }
            Err(_) => continue,
        }
    }
    if pool_to_table_key.is_empty() && table_key_to_table.is_empty() {
        warn!("Empty Tables!!!!");
    }

    let optionals_bucket = store.bucket::<Raw, Raw>(Some("optionals_bucket")).unwrap();

    let mut pool_pubkey_to_optionals = AHashMap::default();
    for result in optionals_bucket.iter() {
        match result {
            Ok(item) => {
                let pool_pubkey = item.key::<Raw>();
                let optional_pubkey = item.value::<Raw>();
                if let (Ok(pool_pubkey), Ok(optional_pubkey)) = (pool_pubkey, optional_pubkey) {
                    let pool_pubkey = Pubkey::new_from_array(pool_pubkey.to_vec().try_into().unwrap());
                    let optional_pubkey = Pubkey::new_from_array(optional_pubkey.to_vec().try_into().unwrap());
                    pool_pubkey_to_optionals.insert(pool_pubkey, optional_pubkey);
                }
            }
            Err(_) => continue,
        }
    }

    if let Err(e) = map_bucket.flush() {
        warn!("Error flushing map_bucket: {:?}", e);
    }

    if let Err(e) = optionals_bucket.flush() {
        warn!("Error flushing optionals_bucket: {:?}", e);
    }

    if let Err(e) = tables_bucket.flush() {
        warn!("Error flushing tables_bucket: {:?}", e);
    }

    drop(map_bucket);
    drop(tables_bucket);
    drop(optionals_bucket);

    (pool_to_table_key, table_key_to_table, pool_pubkey_to_optionals)
}

pub fn get_tables_v2() -> anyhow::Result<UpdateAlts> {
    let kv_cfg = Config::new(&cfg.atl_manager.path);
    let store = match Store::new(kv_cfg) {
        Ok(store) => store,
        Err(err) => return Err(anyhow::format_err!("FailedToLoadStore: {:?}", err)),
    };

    let map_bucket = store.bucket::<Raw, Raw>(Some("map_bucket"))?;
    let tables_bucket = store.bucket::<Raw, Raw>(Some("tables_bucket"))?;

    let mut pool_to_table_key = AHashMap::new();
    let mut table_key_to_table = AHashMap::new();
    for result in map_bucket.iter() {
        match result {
            Ok(item) => {
                let pool_pubkey = item.key::<Raw>();
                let table_pubkey = item.value::<Raw>();
                if let (Ok(pool_pubkey), Ok(table_pubkey)) = (pool_pubkey, table_pubkey) {
                    // info!("searching: {:?}", pool_pubkey);
                    let table = tables_bucket.get(&table_pubkey);
                    let pool_pubkey = Pubkey::new_from_array(pool_pubkey.to_vec().try_into().unwrap());
                    if let Ok(Some(table)) = table {
                        let table_pubkey = Pubkey::new_from_array(table_pubkey.to_vec().try_into().unwrap());
                        // info!("found");
                        let lookup_table =
                            match arb_core::table::get_address_lookup_table(table.to_vec().as_slice(), table_pubkey) {
                                Ok(lookup_table) => {
                                    // info!("found");
                                    lookup_table
                                }
                                Err(_) => {
                                    continue;
                                }
                            };
                        table_key_to_table.insert(table_pubkey, lookup_table);
                        pool_to_table_key.insert(pool_pubkey, table_pubkey);
                    }
                }
            }
            Err(_) => continue,
        }
    }
    if pool_to_table_key.is_empty() && table_key_to_table.is_empty() {
        warn!("Empty Tables!!!!");
    }

    let optionals_bucket = store.bucket::<Raw, Raw>(Some("optionals_bucket"))?;

    let mut pool_pubkey_to_optionals = AHashMap::default();
    for result in optionals_bucket.iter() {
        match result {
            Ok(item) => {
                let pool_pubkey = item.key::<Raw>();
                let optional_pubkey = item.value::<Raw>();
                if let (Ok(pool_pubkey), Ok(optional_pubkey)) = (pool_pubkey, optional_pubkey) {
                    let pool_pubkey = Pubkey::new_from_array(pool_pubkey.to_vec().try_into().unwrap());
                    let optional_pubkey = Pubkey::new_from_array(optional_pubkey.to_vec().try_into().unwrap());
                    pool_pubkey_to_optionals.insert(pool_pubkey, optional_pubkey);
                }
            }
            Err(_) => continue,
        }
    }

    // if let Err(e) = map_bucket.flush() {
    //     warn!("Error flushing map_bucket: {:?}", e);
    // }

    // if let Err(e) = optionals_bucket.flush() {
    //     warn!("Error flushing optionals_bucket: {:?}", e);
    // }

    // if let Err(e) = tables_bucket.flush() {
    //     warn!("Error flushing tables_bucket: {:?}", e);
    // }

    drop(map_bucket);
    drop(tables_bucket);
    drop(optionals_bucket);
    drop(store);

    Ok((
        Box::new(pool_to_table_key),
        Box::new(table_key_to_table),
        Box::new(pool_pubkey_to_optionals),
    ))
}
