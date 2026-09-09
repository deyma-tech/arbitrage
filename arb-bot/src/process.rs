use ahash::AHashSet;
use arb_core::calculator::CalculatorEnum;
use arb_core::gpa::{GPAResult, PoolToCalculator};
use log::{debug, error, warn};
use solana_client::nonblocking::rpc_client;
use solana_program::clock::Slot;
use solana_program::hash::Hash;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use tokio::sync::broadcast::Sender;
use utils::deserialize::{deserialize_v2, Message, MessagesV2};
use utils::pool::PoolType;
use utils::safe::ResultExt;

pub fn process_message(
    msg: &Message,
    result: &mut GPAResult,
    token22_set: &mut AHashSet<Pubkey>,
    data_set: &mut AHashSet<Vec<u8>>,
) -> Option<PoolToCalculator> {
    if msg.owner == spl_token::ID {
        arb_core::gpa::process_token_account(msg.pubkey, msg.data.as_slice(), &mut result.ta_amount_map);
    } else if msg.owner == dex::raydium_amm::ID {
        let res = arb_core::gpa::raydium::process_raydium_amm(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.raydium_amm_v4_map,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Raydium AMM error. {:?}", e),
        }
    } else if msg.owner == dex::pump_amm::ID {
        let res = arb_core::gpa::pump::process_pump_amm(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.pump_amm_map,
            &mut result.pump_amm_config,
            &mut result.pump_amm_fee_config,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Pump AMM error. {:?}", e),
        }
    } else if msg.owner == dex::raydium_clmm::ID {
        let res = arb_core::gpa::raydium::process_raydium_clmm(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.raydium_clmm_tick_arrays,
            &mut result.raydium_clmm_pools,
            &mut result.raydium_clmm_config,
            &mut result.pool_type_and_pubkey,
            &mut result.raydium_bitmap_extensions,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Raydium CLMM error. {:?}", e),
        }
    } else if msg.owner == dex::raydium_cpmm::ID {
        let res = arb_core::gpa::raydium::process_raydium_cpmm(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.raydium_cpmm_configs,
            &mut result.raydium_cpmm_pools,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Raydium CPMM error. {:?}", e),
        }
    } else if msg.owner == dex::meteora_dlmm::ID {
        let res = arb_core::gpa::meteora::process_meteora_dlmm(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.meteora_dlmm_pools,
            &mut result.meteora_dlmm_bin_arrays,
            &mut result.meteora_dlmm_bitmap_extensions,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Meteora DLMM error. {:?}", e),
        }
    } else if msg.owner == dex::meteora_damm_v2::ID {
        let res = arb_core::gpa::meteora::process_meteora_damm_v2(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.meteora_damm_v2_pools,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Meteora DAMM V2 error. {:?}", e),
        }
    } else if msg.owner == spl_token_2022::ID {
        arb_core::gpa::process_token_2022_account(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.ta_amount_map,
            token22_set,
            data_set,
        );
    } else if msg.owner == dex::orca::ID {
        let res = arb_core::gpa::orca::process_orca(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.orca_pools,
            Some(&mut result.orca_oracles), // None for skipping oracle address derivation
            &mut result.orca_pool_to_oracle,
            &mut result.orca_tick_arrays,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Orca error. {:?}", e),
        }
    } else if msg.owner == dex::stabble_stable_swap::ID {
        let res = arb_core::gpa::stabble::process_stabble_stable_swap(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.stabble_stable_swap_map,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Stabble Stable Swap error. {:?}", e),
        }
    } else if msg.owner == dex::stabble_weighted_swap::ID {
        let res = arb_core::gpa::stabble::process_stabble_weighted_swap(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.stabble_weighted_swap_map,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Stabble Weight Swap. {:?}", e),
        }
    }
    // else if msg.owner == dex::lifinity::ID {
    //     let res = arb_core::gpa::lifinity::process_lifinity(
    //         msg.pubkey,
    //         msg.data.as_slice(),
    //         &mut result.lifinity_amm_map,
    //         &mut result.lifinity_authorities,
    //         &mut result.pool_type_and_pubkey,
    //     );
    //     match res {
    //         Ok(new_pool) => return new_pool,
    //         Err(e) => warn!("process_message: Lifinity error. {:?}", e),
    //     }
    // }
    // else if msg.owner == dex::lifinity::ORACLE_PUBKEY {
    //     arb_core::gpa::lifinity::process_pyth(
    //         msg.pubkey,
    //         msg.data.as_slice(),
    //         &mut result.pyth_map,
    //         &mut result.pool_type_and_pubkey,
    //     );
    // }
    else if msg.owner == dex::orca_swap_v2::ID {
        return arb_core::gpa::orca::process_orca_swap_v2(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.orca_swap_v2_markets,
            &mut result.orca_swap_v2_authorities,
            &mut result.pool_type_and_pubkey,
        );
    } else if msg.owner == dex::saros_amm::ID {
        return arb_core::gpa::saros::process_saros_amm(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.saros_amm_markets,
            &mut result.saros_amm_authorities,
            &mut result.pool_type_and_pubkey,
        );
    }
    // else if msg.owner == dex::goose_gamma::ID {
    //     let res = arb_core::gpa::goose_gama::process_goose_gamma(
    //         msg.pubkey,
    //         msg.data.as_slice(),
    //         &mut result.goose_gamma_pools,
    //         &mut result.goose_gamma_configs,
    //         &mut result.goose_gamma_observation_states,
    //         &mut result.pool_type_and_pubkey,
    //     );
    //     match res {
    //         Ok(new_pool) => return new_pool,
    //         Err(e) => warn!("process_message: Goose Gamma error. {:?}", e),
    //     }
    // }
    else if msg.owner == dex::fusion_amm::ID {
        let res = arb_core::gpa::fusion::process_fusion(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.fusion_amm_pools,
            &mut result.fusion_amm_tick_arrays,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Fusion AMM error. {:?}", e),
        }
    } else if msg.owner == dex::saros_dlmm::ID {
        let res = arb_core::gpa::saros::process_saros_dlmm(
            msg.pubkey,
            msg.data.as_slice(),
            &mut result.saros_dlmm_pools,
            &mut result.saros_dlmm_bin_arrays,
            &mut result.pool_type_and_pubkey,
        );
        match res {
            Ok(new_pool) => return new_pool,
            Err(e) => warn!("process_message: Saros DLMM error. {:?}", e),
        }
    } else {
        return None;
    }
    None
}

// pub async fn partial_deser(tx: UnboundedSender<Messages>) {
//     let nats_client = async_nats::connect("nats://localhost:4222").await.unwrap();
//     let mut subscriber = nats_client.subscribe("accounts").await.unwrap();
//     while let Some(msg) = subscriber.next().await {
//         let payload = msg.payload.to_vec();
//         let data = deserialize(payload);
//         if let Ok(messages) = data {
//             let _ = tx.send(messages);
//         } else {
//             error!("Deserialize error: {:?}", data);
//         }
//     }
// }

pub fn partial_deser_v2(tx: crossbeam_channel::Sender<MessagesV2>) {
    let ctx = zmq2::Context::new();

    let socket = ctx.socket(zmq2::SUB).unwrap();
    socket
        .connect("ipc:///tmp/accounts_zmq_v2.sock")
        .or_panic("FailedToConnect");
    let _ = socket.set_subscribe(b"");

    loop {
        let message = socket.recv_bytes(0);
        if let Ok(msg) = message {
            let data = deserialize_v2(msg);
            if let Ok(messages) = data {
                let _ = tx.send(messages);
            } else {
                error!("Deserialize error: {:?}", data);
            }
        }
    }
}

pub async fn get_slot(tx: Sender<(u64, String)>) {
    let ctx = zmq2::Context::new();
    let socket = ctx.socket(zmq2::SUB).unwrap();
    socket
        .connect("ipc:///tmp/slot_zmq_v2.sock")
        .or_panic("FailedToConnect");
    let _ = socket.set_subscribe(b"");
    loop {
        let result = socket.recv_bytes(0);
        if let Ok(payload) = result {
            let slot = u64::from_le_bytes((&payload[..8]).try_into().or_panic("FailedToConvertSlot"));
            let status = String::from_utf8(payload[8..].to_vec()).or_panic("FailedToConvertStatus");
            let _ = tx.send((slot, status));
        }
    }
}

pub async fn get_channel_for_blockhash(blockhash_rpc: String) -> Sender<Hash> {
    let (tx_blockhash, _rx_blockhash) = tokio::sync::broadcast::channel::<Hash>(100);
    tokio::spawn({
        let tx_blockhash = tx_blockhash.clone();
        async move {
            let mut rpc_client =
                rpc_client::RpcClient::new_with_commitment(blockhash_rpc.clone(), CommitmentConfig::processed());
            let mut valid_blockhash = rpc_client.get_latest_blockhash().await.unwrap_or(Hash::default());
            loop {
                let blockhash_result = rpc_client.get_latest_blockhash().await;
                match blockhash_result {
                    Ok(blockhash) => {
                        if blockhash != valid_blockhash {
                            let _ = tx_blockhash.send(blockhash);
                            valid_blockhash = blockhash;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                    }
                    Err(e) => {
                        warn!("Error getting blockhash: {:?}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        rpc_client = rpc_client::RpcClient::new_with_commitment(
                            blockhash_rpc.clone(),
                            CommitmentConfig::processed(),
                        );
                    }
                }
            }
        }
    });
    tx_blockhash
}

// TODO: volať na result get_partial_data
pub fn add_to_calculator(
    pool_type: &PoolType,
    pubkey: &Pubkey,
    result: &mut GPAResult,
    calculator: &mut Vec<CalculatorEnum>,
    //signature: &Signature,
    slot: Slot,
    rpc_client: &solana_client::rpc_client::RpcClient,
) {
    let data = result.get_partial_data(pool_type, pubkey, slot, rpc_client);
    match data {
        Some(data) => calculator.push(data),
        None => debug!("DataNotFound: {:?}, {:?}", pool_type, pubkey),
    }
}

// pub async fn process_bloxroute(
//     grpc: &str,
//     number_of_channels: usize,
// ) -> Vec<
//     UnboundedSender<(
//         VersionedTransaction,
//         u64,
//         ExecutionProviderType,
//         Option<VersionedTransaction>,
//     )>,
// > {
//     let mut bloxroute_tx = vec![];
//     let api_counter = Arc::new(AtomicU64::new(0));
//     let timestamp = Arc::new(AtomicU64::new(now::as_millis() as u64));
//     for _ in 0..number_of_channels {
//         let grpc = grpc.to_string();
//         let grpc_pem = cfg.grpc_pem.to_string();
//         let api_counter = api_counter.clone();
//         let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(
//             VersionedTransaction,
//             u64,
//             ExecutionProviderType,
//             Option<VersionedTransaction>,
//         )>();

//         // let simulate_rpc = RpcClient::new_with_commitment(
//         //     simulation_rpc.to_string(),
//         //     CommitmentConfig::processed(),
//         // );
//         bloxroute_tx.push(tx);
//         let api_counter = api_counter.clone();
//         tokio::spawn({
//             let t = timestamp.clone();
//             let channel = create_grpc_channel(&grpc, &grpc_pem).await;
//             let auth_token = cfg.bloxroute.grpc_auth_token.clone();
//             let mut grpc_client = ApiClient::with_interceptor(channel, move |mut req: Request<()>| {
//                 let auth_token = match auth_token.parse() {
//                     Ok(token) => token,
//                     Err(_) => {
//                         error!("Failed to parse auth token");
//                         return Err(tonic::Status::unauthenticated("Invalid auth token"));
//                     }
//                 };
//                 req.metadata_mut().insert("authorization", auth_token);
//                 Ok(req)
//             });
//             // let mut counter = 0;
//             async move {
//                 while let Some((txn, min_reward, execution_provider, ttxn)) = rx.recv().await {
//                     let timestamp = t.load(Relaxed);
//                     let recent_timestamp = now::as_millis() as u64;
//                     if recent_timestamp - timestamp <= 60_001 && api_counter.load(Relaxed) >= 1000 {
//                         continue;
//                     }
//                     // counter += 1;
//                     let recent_timestamp = now::as_millis() as u64;
//                     if recent_timestamp - timestamp <= 60_001 && api_counter.load(Relaxed) >= 1000 {
//                         continue;
//                     }
//                     // if simulate {
//                     //     if counter % 1000 == 0 {
//                     //         info!("{} processed", counter);
//                     //     }
//                     //     counter += 0;
//                     //     let result = simulate_rpc.simulate_transaction(&txn).await;
//                     //     if result.is_err() {
//                     //         continue;
//                     //     }
//                     //     let simulation = result.un_wrap();
//                     //     if simulation.value.err.is_some() {
//                     //         continue;
//                     //     }
//                     // }
//                     // if is over 1 min reset ...
//                     if recent_timestamp - timestamp > 60_001 {
//                         api_counter.store(0, Relaxed);
//                         t.store(recent_timestamp, Relaxed);
//                     }
//                     if api_counter.load(Relaxed) < 1000 {
//                         api_counter.fetch_add(3, Relaxed);
//                     }
//                     let tx_message = versioned_tx_to_string(&txn);
//                     match execution_provider {
//                         ExecutionProviderType::Paladin => {
//                             let submit = PostSubmitPaladinRequest {
//                                 transaction: Some(TransactionMessageV2 { content: tx_message }),
//                                 revert_protection: Some(true),
//                                 timestamp: None,
//                             };
//                             let res = grpc_client.post_submit_paladin_v2(submit).await;
//                             info!("Bx-Paladin {:?} {}", res, api_counter.load(Relaxed));
//                         }
//                         ExecutionProviderType::BloxrouteSwQoS => {
//                             let submit = PostSubmitRequest {
//                                 transaction: Some(TransactionMessage {
//                                     content: tx_message,
//                                     is_cleanup: false,
//                                 }),
//                                 skip_pre_flight: false,
//                                 front_running_protection: Some(false),
//                                 tip: Some(min_reward),
//                                 use_staked_rp_cs: Some(true),
//                                 allow_back_run: Some(false),
//                                 fast_best_effort: Some(true), // vyhodnotiť podľa situácie
//                                 revenue_address: None,
//                                 sniping: None,
//                                 submit_protection: None,
//                                 timestamp: None,
//                             };
//                             let res = grpc_client.post_submit(submit).await;
//                             info!("Bx-SwQoS {:?} {}", res, api_counter.load(Relaxed));
//                         }
//                         ExecutionProviderType::BloxrouteJito => {
//                             let submit = PostSubmitRequest {
//                                 transaction: Some(TransactionMessage {
//                                     content: tx_message,
//                                     is_cleanup: false,
//                                 }),
//                                 skip_pre_flight: false,
//                                 front_running_protection: Some(true),
//                                 tip: None,
//                                 use_staked_rp_cs: Some(false),
//                                 allow_back_run: Some(false),
//                                 fast_best_effort: Some(false),
//                                 revenue_address: None,
//                                 sniping: None,
//                                 submit_protection: None,
//                                 timestamp: None,
//                             };
//                             let res = grpc_client.post_submit(submit).await;
//                             info!("Bx-jito {:?} {}", res, api_counter.load(Relaxed));
//                         }
//                         ExecutionProviderType::BloxrouteJitoBundle => {
//                             let entry = PostSubmitRequestEntry {
//                                 transaction: Some(TransactionMessage {
//                                     content: tx_message,
//                                     is_cleanup: false,
//                                 }),
//                                 skip_pre_flight: false,
//                             };
//                             let entry2 = if let Some(ttxn) = ttxn {
//                                 let tx_message2 = versioned_tx_to_string(&ttxn);
//                                 PostSubmitRequestEntry {
//                                     transaction: Some(TransactionMessage {
//                                         content: tx_message2,
//                                         is_cleanup: false,
//                                     }),
//                                     skip_pre_flight: false,
//                                 }
//                             } else {
//                                 warn!("BloxrouteJitoBundle missing second transaction");
//                                 continue;
//                             };
//                             let request = PostSubmitBatchRequest {
//                                 entries: vec![entry, entry2],
//                                 submit_strategy: i32::from(SubmitStrategy::PSubmitAll),
//                                 use_bundle: Some(true),
//                                 front_running_protection: None,
//                                 timestamp: None,
//                                 submit_protection: None,
//                             };
//                             let res = grpc_client.post_submit_batch_v2(request).await;
//                             info!("Bx-jito-bundle {:?} {}", res, api_counter.load(Relaxed));
//                         }
//                         _ => {
//                             warn!("Unsupported execution provider: {}", execution_provider.to_string());
//                         }
//                     };
//                 }
//             }
//         });
//     }
//     bloxroute_tx
// }

// pub async fn process_bloxroute_ofr(
//     number_of_channels: usize,
// ) -> Vec<
//     UnboundedSender<(
//         VersionedTransaction,
//         u64,
//         ExecutionProviderType,
//         Option<VersionedTransaction>,
//     )>,
// > {
//     let mut bloxroute_tx = vec![];
//     let api_counter = Arc::new(AtomicU64::new(0));
//     let timestamp = Arc::new(AtomicU64::new(now::as_millis() as u64));
//     for _ in 0..number_of_channels {
//         let api_counter = api_counter.clone();
//         let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(
//             VersionedTransaction,
//             u64,
//             ExecutionProviderType,
//             Option<VersionedTransaction>,
//         )>();
//         bloxroute_tx.push(tx);
//         let api_counter = api_counter.clone();
//         tokio::spawn({
//             let t = timestamp.clone();
//             let auth_token = cfg.bloxroute.grpc_auth_token.clone();
//             let mut headers = HeaderMap::new();
//             headers.insert("Authorization", HeaderValue::from_str(&auth_token).unwrap());
//             let client = Client::builder()
//                 .default_headers(headers)
//                 .pool_idle_timeout(None)
//                 .pool_max_idle_per_host(200)
//                 .tcp_keepalive(Some(std::time::Duration::from_secs(15)))
//                 .build()
//                 .unwrap();
//             async move {
//                 while let Some((txn, _min_reward, execution_provider, _ttxn)) = rx.recv().await {
//                     let timestamp = t.load(Relaxed);
//                     let recent_timestamp = now::as_millis() as u64;
//                     if recent_timestamp - timestamp <= 60_001 && api_counter.load(Relaxed) >= 1000 {
//                         continue;
//                     }
//                     let recent_timestamp = now::as_millis() as u64;
//                     if recent_timestamp - timestamp <= 60_001 && api_counter.load(Relaxed) >= 1000 {
//                         continue;
//                     }
//                     if recent_timestamp - timestamp > 60_001 {
//                         api_counter.store(0, Relaxed);
//                         t.store(recent_timestamp, Relaxed);
//                     }
//                     if api_counter.load(Relaxed) < 1000 {
//                         api_counter.fetch_add(3, Relaxed);
//                     }
//                     let tx_message = versioned_tx_to_string(&txn);
//                     match execution_provider {
//                         // ExecutionProviderType::Paladin => {
//                         //     let submit = PostSubmitPaladinRequest {
//                         //         transaction: Some(TransactionMessageV2 { content: tx_message }),
//                         //         revert_protection: Some(true),
//                         //         timestamp: None,
//                         //     };
//                         //     let res = grpc_client.post_submit_paladin_v2(submit).await;
//                         //     info!("Bx-Paladin {:?} {}", res, api_counter.load(Relaxed));
//                         // }
//                         // ExecutionProviderType::BloxrouteSwQoS => {
//                         //     let submit = PostSubmitRequest {
//                         //         transaction: Some(TransactionMessage {
//                         //             content: tx_message,
//                         //             is_cleanup: false,
//                         //         }),
//                         //         skip_pre_flight: false,
//                         //         front_running_protection: Some(false),
//                         //         tip: Some(min_reward),
//                         //         use_staked_rp_cs: Some(true),
//                         //         allow_back_run: Some(false),
//                         //         fast_best_effort: Some(true), // vyhodnotiť podľa situácie
//                         //         revenue_address: None,
//                         //         sniping: None,
//                         //         submit_protection: None,
//                         //         timestamp: None,
//                         //     };
//                         //     let res = grpc_client.post_submit(submit).await;
//                         //     info!("Bx-SwQoS {:?} {}", res, api_counter.load(Relaxed));
//                         // }
//                         ExecutionProviderType::BloxrouteJito => {
//                             let request_json = json!({
//                                 "transaction": { "content": tx_message, "isCleanup": false },
//                                 "skipPreFlight": false,
//                                 "frontRunningProtection": true,
//                                 "useStakedRPCs": false,
//                                 "fastBestEffort": false,
//                                 "revertProtection": true,
//                                 "submitProtection": "SP_LOW"
//                             });
//                             match client
//                                 .post(format!("{}/submit", cfg.bloxroute.ofr_http_server))
//                                 .json(&request_json)
//                                 .send()
//                                 .await
//                             {
//                                 Ok(res) => info!("Bx-jito {:?} {}", res, api_counter.load(Relaxed)),
//                                 Err(err) => warn!("Bx-jito error {:?} {}", err, api_counter.load(Relaxed)),
//                             };
//                         }
//                         // ExecutionProviderType::BloxrouteJitoBundle => {
//                         //     let entry = PostSubmitRequestEntry {
//                         //         transaction: Some(TransactionMessage {
//                         //             content: tx_message,
//                         //             is_cleanup: false,
//                         //         }),
//                         //         skip_pre_flight: false,
//                         //     };
//                         //     let entry2 = if let Some(ttxn) = ttxn {
//                         //         let tx_message2 = versioned_tx_to_string(&ttxn);
//                         //         PostSubmitRequestEntry {
//                         //             transaction: Some(TransactionMessage {
//                         //                 content: tx_message2,
//                         //                 is_cleanup: false,
//                         //             }),
//                         //             skip_pre_flight: false,
//                         //         }
//                         //     } else {
//                         //         warn!("BloxrouteJitoBundle missing second transaction");
//                         //         continue;
//                         //     };
//                         //     let request = PostSubmitBatchRequest {
//                         //         entries: vec![entry, entry2],
//                         //         submit_strategy: i32::from(SubmitStrategy::PSubmitAll),
//                         //         use_bundle: Some(true),
//                         //         front_running_protection: None,
//                         //         timestamp: None,
//                         //         submit_protection: None,
//                         //     };
//                         //     let res = grpc_client.post_submit_batch_v2(request).await;
//                         //     info!("Bx-jito-bundle {:?} {}", res, api_counter.load(Relaxed));
//                         // }
//                         _ => {
//                             warn!("Unsupported execution provider: {}", execution_provider.to_string());
//                         }
//                     };
//                 }
//             }
//         });
//     }
//     bloxroute_tx
// }

// pub async fn process_jito_quicknode(
//     rpc: &str,
//     number_of_channels: u64,
//     limit: u64,
// ) -> Vec<UnboundedSender<(VersionedTransaction, u64, String)>> {
//     let counter = Arc::new(AtomicU64::new(0));
//     let timestamp = Arc::new(AtomicU64::new(now::as_millis() as u64));

//     let mut jito_tx: Vec<UnboundedSender<(VersionedTransaction, u64, String)>> = vec![];
//     for _ in 0..number_of_channels {
//         let rpc = rpc.to_string();
//         let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(VersionedTransaction, u64, String)>();

//         let client = reqwest::ClientBuilder::new()
//             .default_headers(header::HeaderMap::from_iter(
//                 vec![(
//                     header::CONTENT_TYPE,
//                     header::HeaderValue::from_static("application/json"),
//                 )]
//                 .into_iter(),
//             ))
//             .build();
//         let client = match client {
//             Ok(client) => client,
//             Err(e) => {
//                 error!("Error creating reqwest client: {:?}", e);
//                 continue;
//             }
//         };

//         jito_tx.push(tx);
//         tokio::spawn({
//             let t = timestamp.clone();
//             let counter = counter.clone();
//             async move {
//                 while let Some((txn, _, jito_short)) = rx.recv().await {
//                     let timestamp = t.load(Relaxed);
//                     let recent_timestamp = now::as_millis() as u64;
//                     if recent_timestamp - timestamp <= 1001 && counter.load(Relaxed) >= (limit - 1) {
//                         continue;
//                     }
//                     // if is over 1s reset ...
//                     if recent_timestamp - timestamp > 1001 {
//                         counter.store(0, Relaxed);
//                         t.store(recent_timestamp, Relaxed);
//                     }
//                     // if simulate {
//                     //     let result = simulate_rpc.simulate_transaction(&txn).await;
//                     //     if result.is_err() {
//                     //         continue;
//                     //     }
//                     //     let simulation = result.unw_rap();
//                     //     if simulation.value.err.is_some() {
//                     //         continue;
//                     //     }
//                     // }

//                     if counter.load(Relaxed) < (limit - 1) {
//                         counter.fetch_add(1, Relaxed);
//                     } else {
//                         continue;
//                     }

//                     let msg = match generate_body_for_jito(&txn, &jito_short) {
//                         Ok(msg) => msg,
//                         Err(e) => {
//                             warn!("Error generating body for jito: {:?}", e);
//                             continue;
//                         }
//                     };
//                     let res = client.post(rpc.clone()).body(msg).send().await;
//                     if res.is_err() {
//                         warn!("Error sending tx: {:?}", res);
//                     }

//                     info!("tx quicknode jito: {} {}", txn.signatures[0], counter.load(Relaxed),);
//                 }
//             }
//         });
//     }
//     jito_tx
// }

// pub async fn process_jito(
//     jito_channels: Vec<String>,
//     nats: String,
// ) -> anyhow::Result<Vec<Sender<VersionedTransaction>>> {
//     let mut jito_tx = vec![];
//     for channel in jito_channels {
//         let (tx, mut rx) = tokio::sync::broadcast::channel::<VersionedTransaction>(1000);
//         jito_tx.push(tx);
//         tokio::spawn({
//             let mut timestamp = now::as_millis();
//             let channel = channel.clone();
//             let nats_client = connect(nats.clone()).await.or_panic("FailedToConnectToNATS");
//             let subject = Subject::from(channel.clone());
//             async move {
//                 while let Ok(txn) = rx.recv().await {
//                     let recent_timestamp = now::as_millis();
//                     if recent_timestamp - timestamp <= 1001 {
//                         continue;
//                     }
//                     timestamp = recent_timestamp;
//                     let buffer = bincode::serialize(&txn);
//                     let buffer = match buffer {
//                         Ok(buffer) => buffer,
//                         Err(e) => {
//                             warn!("Error serializing transaction: {:?}", e);
//                             continue;
//                         }
//                     };
//                     let _ = nats_client
//                         .publish(subject.clone(), Bytes::from(buffer))
//                         .await;
//                     let _ = nats_client.flush().await;
//                     info!("tx jito: {} {}", txn.signatures[0], subject);
//                 }
//             }
//         });
//     }
//     Ok(jito_tx)
// }

// if node_key found and it fills conditions of inclusion it return node key as option or none
// pub async fn check_inclusion(
//     prohibited_slot: &mut HashSet<u64>,
//     slot: u64,
//     schedule: &mut HashMap<u64, String>,
//     schedule_rpc: &RpcClient,
//     node_keys_to_country: &HashMap<String, String>,
//     countries_to_include: &Vec<&str>,
//     include: bool,
// ) -> Option<String> {
//     if prohibited_slot.contains(&slot) {
//         return None;
//     }
//     match schedule.get(&slot) {
//         None => {
//             let new_schedule = get_leader_schedule_as_slot_to_node_key(schedule_rpc).await;
//             let new_schedule = match new_schedule {
//                 Ok(new_schedule) => new_schedule,
//                 Err(e) => {
//                     warn!("Error getting schedule: {:?}", e);
//                     return None;
//                 }
//             };
//             schedule.clear();
//             schedule.extend(new_schedule);
//             warn!("missing schedule");
//             return None;
//         }
//         Some(node_key) => {
//             if let Some(country) = node_keys_to_country.get(node_key) {
//                 if !countries_to_include.is_empty() {
//                     if (include && !countries_to_include.contains(&&**country))
//                         || (!include && countries_to_include.contains(&&**country))
//                     {
//                         warn!("excluded country: {} slot: {}", country, slot);
//                         prohibited_slot.insert(slot);
//                         return None;
//                     } else {
//                         return Some(node_key.to_string());
//                     }
//                 }
//             }
//         }
//     }
//     None
// }
