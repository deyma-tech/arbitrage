use anyhow::format_err;
use config::CONFIG as cfg;
use ipgeolocate::{Locator, Service};
use solana_client::nonblocking::rpc_client::RpcClient;
use std::collections::HashMap;
use std::time::Duration;

pub async fn process_leader_schedule(rpc_client: &RpcClient) -> anyhow::Result<Vec<(u64, String)>> {
    let epoch_info = rpc_client.get_epoch_info().await?;
    let epoch = epoch_info.epoch;

    let epoch_schedule = rpc_client.get_epoch_schedule().await?;
    let first_slot_in_epoch = epoch_schedule.get_first_slot_in_epoch(epoch);

    let leader_schedule = rpc_client.get_leader_schedule(Some(first_slot_in_epoch)).await?;
    if leader_schedule.is_none() {
        return Err(format_err!(
            "Unable to fetch leader schedule for slot {first_slot_in_epoch}"
        ));
    }
    let leader_schedule = leader_schedule.unwrap();

    let mut leader_per_slot_index = Vec::new();
    for (pubkey, leader_slots) in leader_schedule.iter() {
        for slot_index in leader_slots.iter() {
            if *slot_index >= leader_per_slot_index.len() {
                leader_per_slot_index.resize(*slot_index + 1, "?");
            }
            leader_per_slot_index[*slot_index] = pubkey;
        }
    }

    let mut leader_schedule_entries = vec![];
    // node key ...
    for (slot_index, leader) in leader_per_slot_index.iter().enumerate() {
        leader_schedule_entries.push((first_slot_in_epoch + slot_index as u64, leader.to_string()));
    }

    Ok(leader_schedule_entries)
}

#[tokio::main]
pub async fn main() {
    // fMvCgRs5B4g9DRJiSE7RQVcU
    //let jito_validators = get_jito_validators().await.unwrap();
    let client = RpcClient::new(cfg.rpc.clone());
    let cluster_info = client.get_cluster_nodes().await.unwrap();
    let mut node_key2ip = HashMap::new();
    for item in cluster_info.iter() {
        let ip; // = "".to_string();
        if let Some(tpu) = item.tpu {
            ip = tpu.ip().to_string();
            node_key2ip.insert(item.pubkey.clone(), ip);
        } else if let Some(gossip) = item.gossip {
            let ip = gossip.ip().to_string();
            node_key2ip.insert(item.pubkey.clone(), ip);
        }
    }
    println!("total: {}", cluster_info.len());
    // let slot = client.get_slot().await.unwrap();
    // let slot = client.get_leader_schedule(Some(slot + 100)).await.unwrap().unwrap();
    // for (validator_key, slots) in slot.iter(){
    //     println!("{} {:?}", validator_key, slots)
    // }
    let vote = client.get_vote_accounts().await.unwrap();

    let service = Service::IpApi;
    for item in vote.current.iter() {
        let node_pubkey = item.node_pubkey.clone();
        if let Some(ip) = node_key2ip.get(&node_pubkey) {
            let msg = format!(
                "node key: {}, vote account:{}, ip: {}",
                node_pubkey, item.vote_pubkey, ip
            );
            match Locator::get(ip, service).await {
                Ok(ip) => println!(
                    "{}, city: {}, country: {}, latitude: {}, longitude: {} region: {}",
                    msg, ip.city, ip.country, ip.latitude, ip.longitude, ip.region
                ),
                Err(error) => println!("{msg}, Error: {error}"),
            };
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    println!("{}", node_key2ip.len());
    for item in vote.delinquent.iter() {
        let node_pubkey = item.node_pubkey.clone();
        if let Some(ip) = node_key2ip.get(&node_pubkey) {
            let msg = format!(
                "node key: {}, vote account:{}, ip: {}",
                node_pubkey, item.vote_pubkey, ip
            );
            match Locator::get(ip, service).await {
                Ok(ip) => println!(
                    "{}, city: {}, country: {}, latitude: {}, longitude: {}, region: {}, isp: {}",
                    msg, ip.city, ip.country, ip.latitude, ip.longitude, ip.region, ip.isp
                ),
                Err(error) => println!("{msg}, Error: {error}"),
            };
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    // println!("{:?}", vote);
    // println!("{}", vote.current.len())
}
