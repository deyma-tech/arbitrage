use crate::setup::get_keypair;
use ahash::AHashSet;
use config::CONFIG as cfg;
use futures::StreamExt;
use log::{error, info, warn};
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signer;
use spl_associated_token_account::get_associated_token_address;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use utils::constants::WSOL;
use utils::now;

pub fn rebalance() -> Sender<u64> {
    let max_wsol: u64 = cfg.wallet_manager.max_wsol;
    let delta_max_wsol: u64 = cfg.wallet_manager.delta_max_wsol;
    let keypair = get_keypair();
    let token_ata_wsol = get_associated_token_address(&keypair.pubkey(), &WSOL);
    let (tx_balance, _rx_balance) = tokio::sync::broadcast::channel::<u64>(1000);
    let mut processed = AHashSet::new();
    //let id = Hash::new_unique().to_string()[..6].to_string();
    //let mut toucher = watch_dog.get_toucher(ThreadWatchDogType::Rebalancer);
    // 6 min after start ...
    let update = Arc::new(AtomicU64::new(now::as_secs() + 360));

    // if !cfg.service_name.is_empty() {
    //     std::thread::spawn({
    //         let update = update.clone();
    //         move || loop {
    //             let last_update = update.load(Ordering::Relaxed);
    //             let now = now::as_secs();
    //             if now > last_update {
    //                 let difference = now - last_update;
    //                 warn!("Last update was: {} s before", difference);
    //                 if difference > cfg.forced_restart {
    //                     warn!("forced restart!");
    //                     let systemctl = SystemCtl::default();
    //                     let _ = systemctl.restart(&cfg.service_name);
    //                 }
    //             }
    //             std::thread::sleep(Duration::from_secs(60));
    //         }
    //     });
    // }

    tokio::spawn({
        let rpc_client = RpcClient::new(cfg.rpc.clone());
        let filter = RpcTransactionLogsFilter::Mentions(vec![keypair.pubkey().to_string()]);
        let tx_balance = tx_balance.clone();
        let update = update.clone();
        async move {
            loop {
                let ws_client = match PubsubClient::new(&cfg.ws).await {
                    Ok(client) => client,
                    Err(err) => {
                        error!("Error creating WebSocket client: {:?}", err);
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                        continue;
                    }
                };

                let mut subscriber = match ws_client
                    .logs_subscribe(
                        filter.clone(),
                        RpcTransactionLogsConfig {
                            commitment: Some(CommitmentConfig::finalized()),
                        },
                    )
                    .await
                {
                    Ok((subscriber, _)) => subscriber,
                    Err(err) => {
                        error!("Error subscribing to account: {:?}", err);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };

                while let Some(response) = subscriber.next().await {
                    //toucher.touch();
                    update.store(now::as_secs(), Ordering::Relaxed);

                    if processed.contains(&response.value.signature) {
                        continue;
                    } else {
                        processed.insert(response.value.signature.clone());
                    }

                    info!("wallet update: {}", response.value.signature);
                    let balance = match rpc_client.get_balance(&keypair.pubkey()).await {
                        Ok(balance) => balance,
                        Err(err) => {
                            error!("Error getting balance: {:?}", err);
                            continue;
                        }
                    };
                    let _ = tx_balance.send(balance);
                    let ata_data = match rpc_client.get_account_data(&token_ata_wsol).await {
                        Ok(data) => data,
                        Err(err) => {
                            error!("Error getting account data: {:?}", err);
                            continue;
                        }
                    };
                    let bytes = ata_data[64..72].try_into();
                    if let Ok(bytes) = bytes {
                        let wsol_amount = u64::from_le_bytes(bytes);
                        if wsol_amount > max_wsol + delta_max_wsol {
                            let mut builder = arb_core::instruction::IxBuilder::new(keypair.pubkey());
                            builder.add_compute_unit_price(50_000);
                            builder.add_compute_unit_limit(100_000);

                            builder.add_sync_native(&token_ata_wsol);
                            let _ = builder.close_ata_ix(&token_ata_wsol);
                            builder.add_create_ata_if_not_exists(&WSOL);

                            // Ak MAX WSOL je nie 0, je to pre Jito = doplním prachy do WSOL (nemá flashloan, nemá prachy)
                            if max_wsol > 0 {
                                builder.add_system_transfer_ix(max_wsol, &token_ata_wsol);
                                builder.add_sync_native(&token_ata_wsol);
                            }

                            let blockhash = match rpc_client.get_latest_blockhash().await {
                                Ok(blockhash) => blockhash,
                                Err(err) => {
                                    error!("ErrorGettingLatestBlockhash: {:?}", err);
                                    continue;
                                }
                            };

                            let tx = match builder.prepare_tx(&keypair, &[], blockhash) {
                                Ok(tx) => tx,
                                Err(err) => {
                                    error!("ErrorPreparingTransaction: {:?}", err);
                                    continue;
                                }
                            };

                            for index in 0..6 {
                                if let Ok(signature) = rpc_client.send_and_confirm_transaction(&tx).await {
                                    info!("Rebalancing tx sent: {:?}", signature);
                                    break;
                                } else {
                                    warn!("Error sending transaction, retrying...");
                                    tokio::time::sleep(tokio::time::Duration::from_secs(index + 1)).await;
                                }
                            }
                        }
                    } else {
                        warn!("Error converting bytes to u64");
                        continue;
                    }
                }

                warn!("WebSocket connection closed, reconnecting...");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        }
    });
    tx_balance
}
