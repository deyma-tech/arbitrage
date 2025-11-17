use crate::{
    get_address_lookup_tables, optimize_convex, optimize_universal, optimize_v4, optimize_v5, OptimizeFn,
    OptimizeResult,
};
use ahash::AHashMap;
use arb_core::{arbitrage::OpportunityWithCalculators, calculator::get_pubkeys_as_ref};
use config::CONFIG as cfg;
use log::debug;
use solana_sdk::{message::AddressLookupTableAccount, pubkey::Pubkey};
use std::sync::{atomic::AtomicU16, Arc};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use utils::rnd::rnd_index;
use utils::{constants::WSOL, now};

#[allow(clippy::type_complexity)]
pub fn spawn_optimization(
    minimum_filter: u64,
) -> anyhow::Result<(
    Vec<crossbeam_channel::Sender<(OpportunityWithCalculators, u64)>>,
    crossbeam_channel::Receiver<(OpportunityWithCalculators, OptimizeResult)>,
)> {
    let optimization_threads = cfg.arbitrage.optimization_threads;
    let mut tx_arbitrages = Vec::with_capacity(optimization_threads);

    let (tx_final_arbitrage, rx_final_arbitrage) =
        crossbeam_channel::unbounded::<(OpportunityWithCalculators, OptimizeResult)>();

    for _ in 0..optimization_threads {
        let (tx_arbitrage, rx_arbitrage) = crossbeam_channel::unbounded::<(OpportunityWithCalculators, u64)>();

        tx_arbitrages.push(tx_arbitrage);

        let tx_final_arbitrage = tx_final_arbitrage.clone();

        let optimizers: Vec<(String, OptimizeFn)> = vec![
            ("v4".to_string(), optimize_v4),
            ("v5".to_string(), optimize_v5),
            ("uni".to_string(), optimize_universal),
            ("convex".to_string(), optimize_convex),
        ];

        let optimizer = optimizers
            .iter()
            .find(|(name, _)| *name == cfg.arbitrage.optimize_method)
            .map(|(_, func)| *func)
            .unwrap_or(optimize_convex);

        std::thread::spawn(move || {
            while let Ok((opportunity, volume)) = rx_arbitrage.recv() {
                // toucher.touch();
                let timestamp = now::as_millis();
                if timestamp > opportunity.timestamp + 50 {
                    continue;
                }
                let mint_pair_route = opportunity.mint_pair_route.iter().collect::<Vec<_>>();
                match optimizer(
                    opportunity.diff,
                    opportunity.slot,
                    volume,
                    &opportunity.calculators,
                    &mint_pair_route,
                    &WSOL,
                ) {
                    Ok(optimize_result) => {
                        if optimize_result.diff > minimum_filter
                            && optimize_result.amounts.len() >= 2
                            && optimize_result.remaining_accounts.len() >= 2
                        {
                            let _ = tx_final_arbitrage.send((opportunity, optimize_result));
                        } else {
                            debug!("spawn_optimization: filtered out: {:?}", optimize_result);
                        }
                    }
                    Err(e) => {
                        debug!("spawn_optimization: filtered out: {:?}", e);
                        continue;
                    }
                };
            } // while
        });
    } // for
    Ok((tx_arbitrages, rx_final_arbitrage))
}

/// With optionals
type ExecutionArbitrageSenderV2 = UnboundedSender<(
    OpportunityWithCalculators,
    OptimizeResult,
    Vec<AddressLookupTableAccount>,
    Vec<AddressLookupTableAccount>,
)>;

#[derive(Debug, Default, Clone)]
struct ArbitrageFilter {
    pub _millis_per_request: u128,
    pub cursor: usize,
    pub _timestamp: u128,
    pub _extra_calls: Arc<AtomicU16>,
}

impl ArbitrageFilter {
    pub fn new(millis_per_request: u128) -> Self {
        let timestamp = now::as_millis();
        let cursor = 0;
        let extra_calls = Arc::new(AtomicU16::new(0));
        ArbitrageFilter {
            _millis_per_request: millis_per_request,
            cursor,
            _timestamp: timestamp,
            _extra_calls: extra_calls,
        }
    }
}

// #[allow(clippy::too_many_arguments)]
// pub fn spawn_filter_arbitrages_v7(
//     limit_per_sec: u64,
//     mut rx_final_arbitrage: UnboundedReceiver<(OpportunityWithCalculators, OptimizeResult)>,
//     tx_jito_quicknode_execution_arbitrages: Vec<ExecutionArbitrageSenderV2>,
//     tx_jito_execution_arbitrages: Vec<ExecutionArbitrageSenderV2>,
//     tx_nextblock_execution_arbitrages: Vec<ExecutionArbitrageSenderV2>,
//     tx_bloxroute_execution_arbitrages: Vec<ExecutionArbitrageSenderV2>,
//     pool_to_table_key: AHashMap<Pubkey, Pubkey>,
//     table_key_to_table: AHashMap<Pubkey, AddressLookupTableAccount>,
//     pool_to_optionals: AHashMap<Pubkey, Pubkey>,
// ) {
//     // let mut toucher = watch_dog.get_toucher(ThreadWatchDogType::Filter);

//     let millis_per_request = (1000 / limit_per_sec) as u128;

//     let arbitrage_filter = ArbitrageFilter::new(millis_per_request);

//     // let mut queue = Queue::new(300);
//     let mut time = now::as_secs();

//     std::thread::spawn(move || {
//         let mut arbitrage_filter = arbitrage_filter.clone();
//         let mut buffer = vec![];
//         loop {
//             let _size = rx_final_arbitrage.blocking_recv_many(&mut buffer, 100);
//             let timestamp = now::as_secs();
//             buffer.sort_by(|(_, optimize), (_, optimize1)| optimize.diff.cmp(&optimize1.diff));
//             // if there is more
//             if buffer.len() > 6 {
//                 let _ = buffer.drain(5..);
//             }
//             while let Some((opportunity, optimize_result)) = buffer.pop() {
//                 let pool_pubkeys = get_pubkeys_as_ref(&opportunity.calculators);
//                 let alts = get_address_lookup_tables(&pool_pubkeys, &pool_to_table_key, &table_key_to_table);
//                 let optional_alts = get_address_lookup_tables(&pool_pubkeys, &pool_to_optionals, &table_key_to_table);
//                 //let optionals = Option;
//                 // Jito
//                 if !tx_jito_execution_arbitrages.is_empty()
//                     && optimize_result.diff > cfg.jito.filter
//                     && timestamp > time + 1
//                 {
//                     debug!("Jito: {:?}", optimize_result);
//                     let _ = tx_jito_execution_arbitrages[0].send((
//                         opportunity.clone(),
//                         optimize_result.clone(),
//                         alts.clone(),
//                         optional_alts.clone(),
//                     ));
//                     time = timestamp;
//                 }
//                 // nextblock
//                 if !tx_nextblock_execution_arbitrages.is_empty() && optimize_result.diff > cfg.nextblock.filter {
//                     debug!("Nextblock: {:?}", optimize_result);
//                     let idx = rnd_index(&tx_nextblock_execution_arbitrages);
//                     let _ = tx_nextblock_execution_arbitrages[idx].send((
//                         opportunity.clone(),
//                         optimize_result.clone(),
//                         alts.clone(),
//                         optional_alts.clone(),
//                     ));
//                 }
//                 // bloxroute
//                 if !tx_bloxroute_execution_arbitrages.is_empty() && optimize_result.diff > cfg.bloxroute.filter {
//                     debug!("Bloxroute: {:?}", optimize_result);
//                     let idx = rnd_index(&tx_bloxroute_execution_arbitrages);
//                     let _ = tx_bloxroute_execution_arbitrages[idx].send((
//                         opportunity.clone(),
//                         optimize_result.clone(),
//                         alts.clone(),
//                         optional_alts.clone(),
//                     ));
//                 }

//                 // JitoQuicknode
//                 if !tx_jito_quicknode_execution_arbitrages.is_empty() {
//                     // queue.push(optimize_result.diff);
//                     send_arbitrage_v3(
//                         &mut arbitrage_filter.cursor,
//                         &tx_jito_quicknode_execution_arbitrages,
//                         opportunity.clone(),
//                         optimize_result.clone(),
//                         alts.clone(),
//                         optional_alts.clone(),
//                     );
//                 }
//             }
//             // toucher.touch();
//         } // while
//     });
// }

pub type UpdateAlts = (
    Box<AHashMap<Pubkey, Pubkey>>,
    Box<AHashMap<Pubkey, AddressLookupTableAccount>>,
    Box<AHashMap<Pubkey, Pubkey>>,
);

#[allow(clippy::too_many_arguments)]
pub fn spawn_filter_arbitrages_v8(
    limit_per_sec: u64,
    rx_final_arbitrage: crossbeam_channel::Receiver<(OpportunityWithCalculators, OptimizeResult)>,
    tx_jito_quicknode_execution_arbitrages: Vec<ExecutionArbitrageSenderV2>,
    tx_jito_execution_arbitrages: Vec<ExecutionArbitrageSenderV2>,
    tx_nextblock_execution_arbitrages: Vec<ExecutionArbitrageSenderV2>,
    tx_bloxroute_execution_arbitrages: Vec<ExecutionArbitrageSenderV2>,
    pool_to_table_key: AHashMap<Pubkey, Pubkey>,
    table_key_to_table: AHashMap<Pubkey, AddressLookupTableAccount>,
    pool_to_optionals: AHashMap<Pubkey, Pubkey>,
    mut rx_update_alts: Receiver<UpdateAlts>,
) {
    let mut pool_to_table_key = Box::new(pool_to_table_key);
    let mut table_key_to_table = Box::new(table_key_to_table);
    let mut pool_to_optionals = Box::new(pool_to_optionals);

    let millis_per_request = (1000 / limit_per_sec) as u128;
    let arbitrage_filter = ArbitrageFilter::new(millis_per_request);
    let time = now::as_secs();
    let mut jito_cursor_to_time = AHashMap::new();
    if !tx_jito_execution_arbitrages.is_empty() {
        for i in 0..tx_jito_execution_arbitrages.len() {
            jito_cursor_to_time.insert(i, time);
        }
    }
    std::thread::spawn(move || {
        let mut arbitrage_filter = arbitrage_filter.clone();
        let mut buffer = vec![];
        let mut jito_cursor = 0;
        let mut zero = 0;
        loop {
            //let _size = rx_final_arbitrage.blocking_recv_many(&mut buffer, 100);
            if !rx_final_arbitrage.is_empty() {
                while let Ok((opportunity, optimize_result)) = rx_final_arbitrage.try_recv() {
                    buffer.push((opportunity, optimize_result));
                }
            } else {
                continue;
            }

            let timestamp = now::as_secs();
            buffer.sort_by(|(_, optimize), (_, optimize1)| optimize.diff.cmp(&optimize1.diff));
            // if there is more
            if buffer.len() > 6 {
                let _ = buffer.drain(5..);
            }
            while let Some((opportunity, optimize_result)) = buffer.pop() {
                let pool_pubkeys = get_pubkeys_as_ref(&opportunity.calculators);
                let alts = get_address_lookup_tables(&pool_pubkeys, &pool_to_table_key, &table_key_to_table);
                let optional_alts = get_address_lookup_tables(&pool_pubkeys, &pool_to_optionals, &table_key_to_table);
                //let optionals = Option;
                // Jito
                if !tx_jito_execution_arbitrages.is_empty() && optimize_result.diff > cfg.jito.filter {
                    debug!("Jito: {:?}", optimize_result);
                    let jito_cursor_time = jito_cursor_to_time.get_mut(&jito_cursor).unwrap_or(&mut zero);
                    if timestamp > *jito_cursor_time + 1 {
                        let _ = tx_jito_execution_arbitrages[jito_cursor].send((
                            opportunity.clone(),
                            optimize_result.clone(),
                            alts.clone(),
                            optional_alts.clone(),
                        ));
                        jito_cursor = (jito_cursor + 1) % tx_jito_execution_arbitrages.len();
                        jito_cursor_to_time.insert(jito_cursor, timestamp);
                    }
                }
                // nextblock
                if !tx_nextblock_execution_arbitrages.is_empty() && optimize_result.diff > cfg.nextblock.filter {
                    debug!("Nextblock: {:?}", optimize_result);
                    let idx = rnd_index(&tx_nextblock_execution_arbitrages);
                    let _ = tx_nextblock_execution_arbitrages[idx].send((
                        opportunity.clone(),
                        optimize_result.clone(),
                        alts.clone(),
                        optional_alts.clone(),
                    ));
                }
                // bloxroute
                if !tx_bloxroute_execution_arbitrages.is_empty() && optimize_result.diff > cfg.bloxroute.filter {
                    debug!("Bloxroute: {:?}", optimize_result);
                    let idx = rnd_index(&tx_bloxroute_execution_arbitrages);
                    let _ = tx_bloxroute_execution_arbitrages[idx].send((
                        opportunity.clone(),
                        optimize_result.clone(),
                        alts.clone(),
                        optional_alts.clone(),
                    ));
                }

                // JitoQuicknode
                if !tx_jito_quicknode_execution_arbitrages.is_empty() {
                    // queue.push(optimize_result.diff);
                    send_arbitrage_v3(
                        &mut arbitrage_filter.cursor,
                        &tx_jito_quicknode_execution_arbitrages,
                        opportunity.clone(),
                        optimize_result.clone(),
                        alts.clone(),
                        optional_alts.clone(),
                    );
                }
            }
            // toucher.touch();

            if !rx_update_alts.is_empty() {
                while let Some((new_pool_to_table_key, new_table_key_to_table, new_pool_to_optionals)) =
                    rx_update_alts.blocking_recv()
                {
                    pool_to_table_key = new_pool_to_table_key;
                    table_key_to_table = new_table_key_to_table;
                    pool_to_optionals = new_pool_to_optionals;
                }
            }
        }
    });
}

fn send_arbitrage_v3(
    cursor: &mut usize,
    tx_execution_arbitrages: &[ExecutionArbitrageSenderV2],
    opportunity: OpportunityWithCalculators,
    optimize: OptimizeResult,
    alts: Vec<AddressLookupTableAccount>,
    optional_alts: Vec<AddressLookupTableAccount>,
) {
    let _ = tx_execution_arbitrages[*cursor].send((opportunity, optimize, alts, optional_alts));
    *cursor += 1;
    if *cursor >= tx_execution_arbitrages.len() {
        *cursor = 0;
    }
}
