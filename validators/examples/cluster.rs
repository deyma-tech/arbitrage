use anyhow::format_err;
use config;
use config::CONFIG as cfg;
use solana_client::nonblocking::rpc_client::RpcClient;

pub async fn process_leader_schedule(rpc_client: &RpcClient) -> anyhow::Result<Vec<(u64, String)>> {
    let epoch_info = rpc_client.get_epoch_info().await?;
    let epoch = epoch_info.epoch;

    let epoch_schedule = rpc_client.get_epoch_schedule().await?;
    let first_slot_in_epoch = epoch_schedule.get_first_slot_in_epoch(epoch);

    let leader_schedule = rpc_client.get_leader_schedule(Some(first_slot_in_epoch)).await?;
    if leader_schedule.is_none() {
        return Err(format_err!("Unable to fetch leader schedule for slot {first_slot_in_epoch}").into());
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
    // let jito_validators = get_jito_validators().await.unwrap();
    let client = RpcClient::new(cfg.rpc.clone());
    let cluster_info = client.get_cluster_nodes().await.unwrap();
    // let mut node_key2ip = HashMap::new();
    for item in cluster_info.iter() {
        println!("{:?}", item);
    }
}
