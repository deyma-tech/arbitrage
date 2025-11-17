use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use sokoban::{FromSlice, OrderedNodeAllocatorMap, RedBlackTree, ZeroCopy, SENTINEL};
use std::fmt::Debug;
use super::{enums::Side, market_traits::{Market, OrderId, RestingOrder}, order_packet::OrderPacket, quantities::{AdjustedQuoteLots, BaseLots, BaseLotsPerBaseUnit, QuoteLots, QuoteLotsPerBaseUnitPerTick, Ticks, WrapperU64}, trader_state::TraderState};

#[repr(C)]
#[derive(
    Eq, PartialEq, Debug, Default, Copy, Clone, Zeroable, Pod,
)]
pub struct FIFOOrderId {
    /// The price of the order, in ticks. Each market has a designated
    /// tick size (some number of quote lots per base unit) that is used to convert the price to ticks.
    /// For example, if the tick size is 0.01, then a price of 1.23 is converted to 123 ticks.
    /// If the quote lot size is 0.001, this means that there is a spacing of 10 quote lots
    /// in between each tick.
    pub price_in_ticks: Ticks,

    /// This is the unique identifier of the order, which is used to determine the side of the order.
    /// It is derived from the sequence number of the market.
    ///
    /// If the order is a bid, the sequence number will have its bits inverted, and if it is an ask,
    /// the sequence number will be used as is.
    ///
    /// The way to identify the side of the order is to check the leading bit of `order_id`.
    /// A leading bit of 0 indicates an ask, and a leading bit of 1 indicates a bid. See Side::from_order_id.
    pub order_sequence_number: u64,
}

impl OrderId for FIFOOrderId {
    fn price_in_ticks(&self) -> u64 {
        self.price_in_ticks.as_u64()
    }
}

impl FIFOOrderId {
    pub fn new_from_untyped(price_in_ticks: u64, order_sequence_number: u64) -> Self {
        FIFOOrderId {
            price_in_ticks: Ticks::new(price_in_ticks),
            order_sequence_number,
        }
    }

    pub fn new(price_in_ticks: Ticks, order_sequence_number: u64) -> Self {
        FIFOOrderId {
            price_in_ticks,
            order_sequence_number,
        }
    }
}

impl PartialOrd for FIFOOrderId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // The ordering of the `FIFOOrderId` struct is determined by the price of the order. If the price is the same,
        // then the order with the lower sequence number is considered to be the lower order.
        //
        // Asks are sorted in ascending order, and bids are sorted in descending order.
        let (tick_cmp, seq_cmp) = match Side::from_order_sequence_number(self.order_sequence_number)
        {
            Side::Bid => (
                other.price_in_ticks.partial_cmp(&self.price_in_ticks)?,
                other
                    .order_sequence_number
                    .partial_cmp(&self.order_sequence_number)?,
            ),
            Side::Ask => (
                self.price_in_ticks.partial_cmp(&other.price_in_ticks)?,
                self.order_sequence_number
                    .partial_cmp(&other.order_sequence_number)?,
            ),
        };
        if tick_cmp == std::cmp::Ordering::Equal {
            Some(seq_cmp)
        } else {
            Some(tick_cmp)
        }
    }
}

impl Ord for FIFOOrderId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}


#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct FIFORestingOrder {
    pub trader_index: u64,
    pub num_base_lots: BaseLots, // Number of base lots quoted
    pub last_valid_slot: u64,
    pub last_valid_unix_timestamp_in_seconds: u64,
}

impl FIFORestingOrder {
    pub fn new_default(trader_index: u64, num_base_lots: BaseLots) -> Self {
        FIFORestingOrder {
            trader_index,
            num_base_lots,
            last_valid_slot: 0,
            last_valid_unix_timestamp_in_seconds: 0,
        }
    }

    pub fn new(
        trader_index: u64,
        num_base_lots: BaseLots,
        last_valid_slot: Option<u64>,
        last_valid_unix_timestamp_in_seconds: Option<u64>,
    ) -> Self {
        FIFORestingOrder {
            trader_index,
            num_base_lots,
            last_valid_slot: last_valid_slot.unwrap_or(0),
            last_valid_unix_timestamp_in_seconds: last_valid_unix_timestamp_in_seconds.unwrap_or(0),
        }
    }

    pub fn new_with_last_valid_slot(
        trader_index: u64,
        num_base_lots: BaseLots,
        last_valid_slot: u64,
    ) -> Self {
        FIFORestingOrder {
            trader_index,
            num_base_lots,
            last_valid_slot,
            last_valid_unix_timestamp_in_seconds: 0,
        }
    }

    pub fn new_with_last_valid_unix_timestamp(
        trader_index: u64,
        num_base_lots: BaseLots,
        last_valid_unix_timestamp_in_seconds: u64,
    ) -> Self {
        FIFORestingOrder {
            trader_index,
            num_base_lots,
            last_valid_slot: 0,
            last_valid_unix_timestamp_in_seconds,
        }
    }
}

impl RestingOrder for FIFORestingOrder {
    fn size(&self) -> u64 {
        self.num_base_lots.as_u64()
    }

    fn last_valid_slot(&self) -> Option<u64> {
        if self.last_valid_slot == 0 {
            None
        } else {
            Some(self.last_valid_slot)
        }
    }

    fn last_valid_unix_timestamp_in_seconds(&self) -> Option<u64> {
        if self.last_valid_unix_timestamp_in_seconds == 0 {
            None
        } else {
            Some(self.last_valid_unix_timestamp_in_seconds)
        }
    }

    fn is_expired(&self, current_slot: u64, current_unix_timestamp_in_seconds: u64) -> bool {
        (self.last_valid_slot != 0 && self.last_valid_slot < current_slot)
            || (self.last_valid_unix_timestamp_in_seconds != 0
                && self.last_valid_unix_timestamp_in_seconds < current_unix_timestamp_in_seconds)
    }
}


#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable)]
pub struct FIFOMarket<
    MarketTraderId: Debug
        + PartialOrd
        + Ord
        + Default
        + Copy
        + Clone
        + Zeroable
        + Pod
        + BorshDeserialize
        + BorshSerialize,        
    const BIDS_SIZE: usize,
    const ASKS_SIZE: usize,
    const NUM_SEATS: usize,
> {
    /// Padding
    pub _padding: [u64; 32],

    /// Number of base lots in a base unit. For example, if the lot size is 0.001 SOL, then base_lots_per_base_unit is 1000.
    pub base_lots_per_base_unit: BaseLotsPerBaseUnit,

    /// Tick size in quote lots per base unit. For example, if the tick size is 0.01 USDC and the quote lot size is 0.001 USDC, then tick_size_in_quote_lots_per_base_unit is 10.
    pub tick_size_in_quote_lots_per_base_unit: QuoteLotsPerBaseUnitPerTick,

    /// The sequence number of the next event.
    order_sequence_number: u64,

    /// There are no maker fees. Taker fees are charged on the quote lots transacted in the trade, in basis points.
    pub taker_fee_bps: u64,

    /// Amount of fees collected from the market in its lifetime, in quote lots.
    collected_quote_lot_fees: QuoteLots,

    /// Amount of unclaimed fees accrued to the market, in quote lots.
    unclaimed_quote_lot_fees: QuoteLots,

    /// Red-black tree representing the bids in the order book.
    pub bids: RedBlackTree<FIFOOrderId, FIFORestingOrder, BIDS_SIZE>,

    /// Red-black tree representing the asks in the order book.
    pub asks: RedBlackTree<FIFOOrderId, FIFORestingOrder, ASKS_SIZE>,

    /// Red-black tree representing the authorized makers in the market.
    pub traders: RedBlackTree<MarketTraderId, TraderState, NUM_SEATS>,
}

unsafe impl<
        MarketTraderId: Debug
            + PartialOrd
            + Ord
            + Default
            + Copy
            + Clone
            + Zeroable
            + Pod
            + BorshDeserialize
            + BorshSerialize,
        const BIDS_SIZE: usize,
        const ASKS_SIZE: usize,
        const NUM_SEATS: usize,
    > Pod for FIFOMarket<MarketTraderId, BIDS_SIZE, ASKS_SIZE, NUM_SEATS>
{
}

impl<
        MarketTraderId: Debug
            + PartialOrd
            + Ord
            + Default
            + Copy
            + Clone
            + Zeroable
            + Pod
            + BorshDeserialize
            + BorshSerialize,
        const BIDS_SIZE: usize,
        const ASKS_SIZE: usize,
        const NUM_SEATS: usize,
    > FromSlice for FIFOMarket<MarketTraderId, BIDS_SIZE, ASKS_SIZE, NUM_SEATS>
{
    fn new_from_slice(data: &mut [u8]) -> &mut Self {
        let market = Self::load_mut_bytes(data).unwrap();
        assert_eq!(market.base_lots_per_base_unit, BaseLotsPerBaseUnit::ZERO);
        assert_eq!(market.order_sequence_number, 0);
        market.initialize();
        market
    }
}

impl<
        MarketTraderId: Debug
            + PartialOrd
            + Ord
            + Default
            + Copy
            + Clone
            + Zeroable
            + Pod
            + BorshDeserialize
            + BorshSerialize,
        const BIDS_SIZE: usize,
        const ASKS_SIZE: usize,
        const NUM_SEATS: usize,
    > ZeroCopy for FIFOMarket<MarketTraderId, BIDS_SIZE, ASKS_SIZE, NUM_SEATS>
{
}

impl<
        MarketTraderId: Debug
            + PartialOrd
            + Ord
            + Default
            + Copy
            + Clone
            + Zeroable
            + Pod
            + BorshDeserialize
            + BorshSerialize,
        const BIDS_SIZE: usize,
        const ASKS_SIZE: usize,
        const NUM_SEATS: usize,
    > Market<MarketTraderId, FIFOOrderId, FIFORestingOrder, OrderPacket>
    for FIFOMarket<MarketTraderId, BIDS_SIZE, ASKS_SIZE, NUM_SEATS>
{
    fn get_data_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    fn get_taker_fee_bps(&self) -> u64 {
        self.taker_fee_bps
    }

    fn get_tick_size(&self) -> QuoteLotsPerBaseUnitPerTick {
        self.tick_size_in_quote_lots_per_base_unit
    }

    fn get_base_lots_per_base_unit(&self) -> BaseLotsPerBaseUnit {
        self.base_lots_per_base_unit
    }

    fn get_sequence_number(&self) -> u64 {
        self.order_sequence_number
    }

    fn get_collected_fee_amount(&self) -> QuoteLots {
        self.collected_quote_lot_fees
    }

    fn get_uncollected_fee_amount(&self) -> QuoteLots {
        self.unclaimed_quote_lot_fees
    }

    fn get_registered_traders(&self) -> &dyn OrderedNodeAllocatorMap<MarketTraderId, TraderState> {
        &self.traders as &dyn OrderedNodeAllocatorMap<MarketTraderId, TraderState>
    }

    fn get_trader_state(&self, trader_id: &MarketTraderId) -> Option<&TraderState> {
        self.get_registered_traders().get(trader_id)
    }

    fn get_trader_state_from_index(&self, index: u32) -> &TraderState {
        &self.traders.get_node(index).value
    }

    #[inline(always)]
    fn get_trader_index(&self, trader_id: &MarketTraderId) -> Option<u32> {
        let addr = self.traders.get_addr(trader_id);
        if addr == SENTINEL {
            None
        } else {
            Some(addr)
        }
    }

    fn get_trader_id_from_index(&self, trader_index: u32) -> MarketTraderId {
        self.traders.get_node(trader_index).key
    }

    #[inline(always)]
    fn get_book(&self, side: Side) -> &dyn OrderedNodeAllocatorMap<FIFOOrderId, FIFORestingOrder> {
        match side {
            Side::Bid => &self.bids,
            Side::Ask => &self.asks,
        }
    }
}


impl<
        MarketTraderId: Debug
            + PartialOrd
            + Ord
            + Default
            + Copy
            + Clone
            + Zeroable
            + Pod
            + BorshDeserialize
            + BorshSerialize,
        const BIDS_SIZE: usize,
        const ASKS_SIZE: usize,
        const NUM_SEATS: usize,
    > FIFOMarket<MarketTraderId, BIDS_SIZE, ASKS_SIZE, NUM_SEATS>
{
    pub fn new(
        tick_size_in_quote_lots_per_base_unit: QuoteLotsPerBaseUnitPerTick,
        base_lots_per_base_unit: BaseLotsPerBaseUnit,
    ) -> Self {
        let mut market = Self::default();
        market.set_initial_params(
            tick_size_in_quote_lots_per_base_unit,
            base_lots_per_base_unit,
        );
        market
    }

    fn initialize(&mut self) {
        self.bids.initialize();
        self.asks.initialize();
        self.traders.initialize();
    }

    fn initialize_with_params_inner(
        &mut self,
        tick_size_in_quote_lots_per_base_unit: QuoteLotsPerBaseUnitPerTick,
        base_lots_per_base_unit: BaseLotsPerBaseUnit,
    ) {
        self.initialize();
        self.set_initial_params(
            tick_size_in_quote_lots_per_base_unit,
            base_lots_per_base_unit,
        );
    }

    fn set_initial_params(
        &mut self,
        tick_size_in_quote_lots_per_base_unit: QuoteLotsPerBaseUnitPerTick,
        base_lots_per_base_unit: BaseLotsPerBaseUnit,
    ) {
        assert!(tick_size_in_quote_lots_per_base_unit % base_lots_per_base_unit == 0);

        // Ensure there is no re-entrancy
        assert_eq!(self.order_sequence_number, 0);
        self.tick_size_in_quote_lots_per_base_unit = tick_size_in_quote_lots_per_base_unit;
        self.base_lots_per_base_unit = base_lots_per_base_unit;
        // After setting the initial params, this function can never be called again
        self.order_sequence_number += 1;
    }

    #[inline]
    /// Round up the fee to the nearest adjusted quote lot
    fn compute_fee(&self, size_in_adjusted_quote_lots: AdjustedQuoteLots) -> AdjustedQuoteLots {
        AdjustedQuoteLots::new(
            ((size_in_adjusted_quote_lots.as_u128() * self.taker_fee_bps as u128 + 10000 - 1)
                / 10000) as u64,
        )
    }

    #[inline]
    /// Quote lot budget with fees adjusted (buys)
    ///
    /// The desired result is adjusted_quote_lots / (1 + fee_bps). We approach this result by taking
    /// (size_in_lots * u64::MAX) / (u64::MAX * (1 + fee_bps)) for accurate numerical precision.
    /// This will never overflow at any point in the calculation because all intermediate values
    /// will be stored in a u128. There is only a single multiplication of u64's which will be
    /// strictly less than u128::MAX
    fn adjusted_quote_lot_budget_post_fee_adjustment_for_buys(
        &self,
        size_in_adjusted_quote_lots: AdjustedQuoteLots,
    ) -> Option<AdjustedQuoteLots> {
        let fee_adjustment = self.compute_fee(AdjustedQuoteLots::MAX).as_u128() + u64::MAX as u128;
        // Return an option to catch truncation from downcasting to u64
        u64::try_from(size_in_adjusted_quote_lots.as_u128() * u64::MAX as u128 / fee_adjustment)
            .ok()
            .map(AdjustedQuoteLots::new)
    }

    #[inline]
    /// Quote lot budget with fees adjusted (sells)
    ///
    /// The desired result is adjusted_quote_lots / (1 - fee_bps). We approach this result by taking
    /// (size_in_lots * u64::MAX) / (u64::MAX * (1 - fee_bps)) for accurate numerical precision.
    /// This will never overflow at any point in the calculation because all intermediate values
    /// will be stored in a u128. There is only a single multiplication of u64's which will be
    /// strictly less than u128::MAX
    fn adjusted_quote_lot_budget_post_fee_adjustment_for_sells(
        &self,
        size_in_adjusted_quote_lots: AdjustedQuoteLots,
    ) -> Option<AdjustedQuoteLots> {
        let fee_adjustment = u64::MAX as u128 - self.compute_fee(AdjustedQuoteLots::MAX).as_u128();
        // Return an option to catch truncation from downcasting to u64
        u64::try_from(size_in_adjusted_quote_lots.as_u128() * u64::MAX as u128 / fee_adjustment)
            .ok()
            .map(AdjustedQuoteLots::new)
    }

    #[inline]
    /// Adjusted quote lots, rounded up to the nearest multiple of base_lots_per_base_unit
    pub fn round_adjusted_quote_lots_up(
        &self,
        num_adjusted_quote_lots: AdjustedQuoteLots,
    ) -> AdjustedQuoteLots {
        ((num_adjusted_quote_lots
            + AdjustedQuoteLots::new(self.base_lots_per_base_unit.as_u64() - 1))
        .unchecked_div::<BaseLotsPerBaseUnit, QuoteLots>(self.base_lots_per_base_unit))
            * self.base_lots_per_base_unit
    }

    #[inline]
    /// Adjusted quote lots, rounded down to the nearest multiple of base_lots_per_base_unit
    pub fn round_adjusted_quote_lots_down(
        &self,
        num_adjusted_quote_lots: AdjustedQuoteLots,
    ) -> AdjustedQuoteLots {
        num_adjusted_quote_lots
            .unchecked_div::<BaseLotsPerBaseUnit, QuoteLots>(self.base_lots_per_base_unit)
            * self.base_lots_per_base_unit
    }

}
