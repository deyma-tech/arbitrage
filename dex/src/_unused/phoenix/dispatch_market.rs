use log::warn;
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use super::accounts::MarketSizeParams;
use super::fifo::{FIFOOrderId, FIFORestingOrder, FIFOMarket};
use super::market_traits::Market;
use super::market_wrapper::MarketWrapper;
use super::order_packet::OrderPacket;
use sokoban::ZeroCopy;

macro_rules! fifo_market {
    ($num_bids:literal, $num_asks:literal, $num_seats:literal, $bytes:expr) => {
        FIFOMarket::<Pubkey, $num_bids, $num_asks, $num_seats>::load_bytes($bytes)
            .ok_or(anyhow::format_err!("FailedToLoadMarketFromAccount"))?
            as &dyn Market<Pubkey, FIFOOrderId, FIFORestingOrder, OrderPacket>
    };
}

/// Loads a market from a given buffer and known market params.
pub fn load_with_dispatch<'a>(
    market_size_params: &'a MarketSizeParams,
    bytes: &'a [u8],
) -> Result<MarketWrapper<'a, Pubkey, FIFOOrderId, FIFORestingOrder, OrderPacket>> {
    dispatch_market(market_size_params, bytes)
}


fn dispatch_market<'a>(
    market_size_params: &'a MarketSizeParams,
    bytes: &'a [u8],
) -> Result<MarketWrapper<'a, Pubkey, FIFOOrderId, FIFORestingOrder, OrderPacket>> {
    let market = match (
        market_size_params.bids_size,
        market_size_params.asks_size,
        market_size_params.num_seats,
    ) {
        (512, 512, 128) => fifo_market!(512, 512, 128, bytes),
        (512, 512, 1025) => fifo_market!(512, 512, 1025, bytes),
        (512, 512, 1153) => fifo_market!(512, 512, 1153, bytes),
        (1024, 1024, 128) => fifo_market!(1024, 1024, 128, bytes),
        (1024, 1024, 2049) => fifo_market!(1024, 1024, 2049, bytes),
        (1024, 1024, 2177) => fifo_market!(1024, 1024, 2177, bytes),
        (2048, 2048, 128) => fifo_market!(2048, 2048, 128, bytes),
        (2048, 2048, 4097) => fifo_market!(2048, 2048, 4097, bytes),
        (2048, 2048, 4225) => fifo_market!(2048, 2048, 4225, bytes),
        (4096, 4096, 128) => fifo_market!(4096, 4096, 128, bytes),
        (4096, 4096, 8193) => fifo_market!(4096, 4096, 8193, bytes),
        (4096, 4096, 8321) => fifo_market!(4096, 4096, 8321, bytes),
        _ => {
            warn!("Invalid parameters for market");
            return Err(anyhow::format_err!("InvalidMarketParameters"));
        }
    };
    Ok(MarketWrapper::<
        Pubkey,
        FIFOOrderId,
        FIFORestingOrder,
        OrderPacket,
    >::new(market))
}

// pub async fn del_me() {
//     let market_data = client.get_account(market).await.unwrap().data;
//     let (header_bytes, bytes) = market_data.split_at(size_of::<MarketHeader>());
//     let header = Box::new(MarketHeader::load_bytes(header_bytes).unwrap());
//     let full_market = load_with_dispatch(&header.market_size_params, bytes).unwrap();
// }
