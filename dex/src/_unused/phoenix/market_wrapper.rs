use super::market_traits::Market;


/// Struct that holds an object implementing the Market trait.
pub struct MarketWrapper<'a, MarketTraderId, MarketOrderId, MarketRestingOrder, MarketOrderPacket> {
    pub inner: &'a dyn Market<MarketTraderId, MarketOrderId, MarketRestingOrder, MarketOrderPacket>,
}

impl<'a, MarketTraderId, MarketOrderId, MarketRestingOrder, MarketOrderPacket>
    MarketWrapper<'a, MarketTraderId, MarketOrderId, MarketRestingOrder, MarketOrderPacket>
{
    pub fn new(
        market: &'a dyn Market<
            MarketTraderId,
            MarketOrderId,
            MarketRestingOrder,
            MarketOrderPacket,
        >,
    ) -> Self {
        Self { inner: market }
    }
}