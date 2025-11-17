use ahash::{AHashMap, AHashSet};
use anyhow::Context;
use fred::bytes::Bytes;
use fred::prelude::ServerConfig::Centralized;
use fred::prelude::*;
use fred::types::RespVersion;
use log::{debug, info};
use rayon::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::address_lookup_table::state::AddressLookupTable;
use solana_program::address_lookup_table::AddressLookupTableAccount;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use std::time::{Duration, Instant};
use utils::constants::ADDRESS_LOOKUP_TABLE_PROGRAM_ID;

#[derive(Debug)]
pub enum RedisDB {
    TableKeyToTableData = 0,
    AddressToTableKey = 1,
    PoolPubkeyToTableKeys = 2,
}

pub fn get_address_lookup_table(data: &[u8], key: Pubkey) -> anyhow::Result<AddressLookupTableAccount> {
    // let keys = bytemuck::from_bytes::<[Pubkey;256]>(&data[56..]);
    let mut keys_table: Vec<Pubkey> = vec![]; // keys.iter().map(|(key)| key.clone()).collect();
    let turns = (data.len() - 56) / 32;
    for x in 0..turns {
        let start: usize = 56 + (x * 32);
        let end: usize = 56 + (x * 32) + 32;
        let d: [u8; 32] = data[start..end]
            .try_into()
            .context("get_address_lookup_table: slice with incorrect length")?;
        // debug!("{}", Pubkey::from(d.clone()));
        keys_table.push(Pubkey::from(d));
    }
    // debug!("table have: {} accounts", keys_table.len());
    Ok(AddressLookupTableAccount {
        key,
        addresses: keys_table,
    })
}

// TODO store and update all table accounts - store pubkey and raw account data
// store all addresses as key address and lookup table pubkey as value
// store selected ...
// TODO implement SMEMBERS
// TODO implement SREM etc...

/// searching for keys, key - is always mandatory, if mandatory argument is true if key not found returns error
/// if false function continue searching ...
/// return final hashset and number of found
pub async fn find(
    key: &Pubkey,
    keys: &[&Pubkey],
    redis_client: &RedisClient,
    mandatory: bool,
) -> anyhow::Result<Option<(AHashSet<Pubkey>, usize)>> {
    let primary = redis_client.smembers::<Vec<String>, String>(key.to_string()).await?;
    if primary.is_empty() {
        return Err(anyhow::format_err!("MandatoryNotFound"));
    };
    debug!("for base key found: {:?}", primary.len());
    let mut counter_found: usize = 0;
    let mut intersection = AHashSet::from_iter(primary);
    for key in keys.iter() {
        let secondary = redis_client.smembers::<Vec<String>, String>(key.to_string()).await?;
        if secondary.is_empty() {
            if mandatory {
                return Err(anyhow::format_err!("SecondaryNotFound"));
            }
            continue;
            // return Ok(None);
        };
        let secondary = AHashSet::from_iter(secondary);
        let temp_intersection =
            AHashSet::from_iter(intersection.intersection(&secondary).cloned().collect::<Vec<String>>());
        if !temp_intersection.is_empty() {
            counter_found += 1;
            intersection = temp_intersection;
        }
    }
    debug!("found: {} total: {}", counter_found, keys.len());
    let mut final_set = AHashSet::new();
    for item in intersection.iter() {
        let pubkey = Pubkey::from_str(item)?;
        final_set.insert(pubkey);
    }
    Ok(Some((final_set, counter_found)))
}

pub async fn fetch_and_store_table(rpc: String) -> anyhow::Result<()> {
    // first write address to table key
    let client = RedisClient::new(
        RedisConfig {
            fail_fast: false,
            blocking: Blocking::Interrupt,
            username: None,
            password: None,
            server: Centralized {
                server: Server {
                    host: "localhost".parse().unwrap(),
                    port: 6379,
                },
            },
            version: RespVersion::RESP2,
            database: Some(RedisDB::AddressToTableKey as u8),
        },
        Some(PerformanceConfig::default()),
        Some(ConnectionConfig::default()),
        Some(ReconnectPolicy::default()),
    );
    info!("connecting ..");
    client.connect();
    client.wait_for_connect().await?;
    info!("calling gpa");
    let rpc_client = RpcClient::new_with_timeout(rpc, Duration::from_secs(1_000));
    let start = Instant::now();
    let tables = rpc_client
        .get_program_accounts(&ADDRESS_LOOKUP_TABLE_PROGRAM_ID)
        .await?;
    // let t = tables.clone();
    let elapsed = start.elapsed();
    info!("Loading ALTs, elapsed time: {:?} {:?}", elapsed, tables.len());
    let slot = rpc_client.get_slot().await?;
    let mut address_to_table: AHashMap<String, AHashSet<String>> = AHashMap::new();
    let iter = tables
        .par_iter()
        .map(|(pubkey, account)| (*pubkey, AddressLookupTable::deserialize(account.data.as_slice())))
        .collect::<Vec<_>>();
    for (table_pubkey, table) in iter.iter() {
        if let Ok(alt_account) = table {
            if alt_account.meta.deactivation_slot > slot {
                for address in alt_account.addresses.iter() {
                    address_to_table
                        .entry(address.to_string())
                        .and_modify(|x| {
                            x.insert(table_pubkey.to_string());
                        })
                        .or_insert(AHashSet::from_iter([table_pubkey.to_string()]));
                    // let _ = client
                    //     .sadd::<_, String, String>(address.to_string(), table_pubkey.to_string())
                    //     .await?;
                }
            }
        }
    }
    info!("Intermediary data prepared");
    let mut txn = client.multi();
    for (key, values) in address_to_table.into_iter() {
        for value in values.into_iter() {
            let _: () = txn.sadd::<_, String, String>(key.clone(), value).await?;
        }
        if txn.len() > 1_000 {
            let _: () = txn.exec::<_>(false).await.map_err(|e| anyhow::format_err!("{:?}", e))?;
            txn = client.multi();
        }
    }
    if txn.len() > 0 {
        let _: () = txn.exec::<_>(false).await.map_err(|e| anyhow::format_err!("{:?}", e))?;
    }
    info!("Populating address to table key done.");
    // callers can manage the tokio task driving the connections
    client.select(RedisDB::TableKeyToTableData as u8).await?;
    let mut txn = client.multi();
    for (idx, (pubkey, account)) in tables.iter().enumerate() {
        if idx % 10_000 == 0 {
            info!("Process {} TableToTable", idx);
        }
        let key = pubkey.to_string();
        let _: () = txn
            .set::<_, std::string::String, Bytes>(
                key,
                Bytes::copy_from_slice(account.data.as_slice()),
                None,
                None,
                false,
            )
            .await?;
        if txn.len() > 1_000 {
            let _: () = txn
                .exec::<_>(false)
                .await
                .map_err(|e| anyhow::format_err!("Error in transaction execution: {:?} for key: {}", e, pubkey))?;
            txn = client.multi();
        }
    }
    if txn.len() > 0 {
        let _: () = txn
            .exec::<_>(false)
            .await
            .map_err(|e| anyhow::format_err!("Error in transaction execution: {:?}", e))?;
    }
    info!("Populating table key to table done ..");
    // store all - table key to table data
    client.quit().await?;
    Ok(())
}
