use config::CONFIG as cfg;
use solana_client::nonblocking::rpc_client::RpcClient;
use validators::{jito_validators::JitoValidator, schedule::process_leader_schedule};

#[tokio::main]
pub async fn main() {
    let client = RpcClient::new(cfg.rpc.clone());
    let schedule = process_leader_schedule(&client).await.unwrap();

    let jito_validator = JitoValidator::new().await;
    println!("Jito nodekeys: {:?}", jito_validator.node_keys);
    for (slot, node_key) in schedule {
        if jito_validator.is_jito(&node_key) {
            println!("{} {} Jito", slot, node_key);
        } else {
            println!("{} {} N0", slot, node_key);
        }
    }
}
