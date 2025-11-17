use config::CONFIG as cfg;
use solana_client::nonblocking::rpc_client::RpcClient;
use tokio::sync::broadcast::Sender;

pub fn spawn_get_priority_fees() -> Sender<u64> {
    let (tx_priority_fee, _rx_balance) = tokio::sync::broadcast::channel::<u64>(100);
    let tx_priority_fee_2 = tx_priority_fee.clone();
    tokio::spawn({
        let rpc_client = RpcClient::new(cfg.blockhash_and_simulate_rpc.clone());
        let programs = [
            dex::orca::ID,
            dex::stabble_stable_swap::ID,
            dex::raydium_clmm::id(),
            dex::raydium_amm::ID,
            dex::pump_amm::ID,
            //dex::lifinity::ID,
            dex::pump_amm::ID,
        ];
        async move {
            loop {
                let fees = rpc_client.get_recent_prioritization_fees(&programs).await;
                if let Ok(fees) = fees {
                    let max_fee = fees.iter().max_by_key(|fee| fee.prioritization_fee);
                    if let Some(priority_fee) = max_fee {
                        let _ = tx_priority_fee_2.send(priority_fee.prioritization_fee);
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
        }
    });
    tx_priority_fee
}
