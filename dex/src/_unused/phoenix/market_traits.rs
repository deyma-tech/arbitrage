use borsh::{BorshDeserialize, BorshSerialize};
use itertools::Itertools;
use sokoban::OrderedNodeAllocatorMap;

use super::{enums::Side, order_packet::OrderPacketMetadata, quantities::{BaseLots, BaseLotsPerBaseUnit, QuoteLots, QuoteLotsPerBaseUnitPerTick, Ticks, WrapperU64}, trader_state::TraderState};

pub trait OrderId {
    fn price_in_ticks(&self) -> u64;
}

pub trait RestingOrder {
    fn size(&self) -> u64;
    fn last_valid_slot(&self) -> Option<u64>;
    fn last_valid_unix_timestamp_in_seconds(&self) -> Option<u64>;
    fn is_expired(&self, current_slot: u64, current_unix_timestamp_in_seconds: u64) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LadderOrder {
    pub price_in_ticks: u64,
    pub size_in_base_lots: u64,
}

/// Helpful struct for processing the order book state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ladder {
    pub bids: Vec<LadderOrder>,
    pub asks: Vec<LadderOrder>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedLadderOrder {
    pub price_in_ticks: Ticks,
    pub size_in_base_lots: BaseLots,
}

/// Helpful struct for processing the order book state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLadder {
    pub bids: Vec<TypedLadderOrder>,
    pub asks: Vec<TypedLadderOrder>,
}

/// A wrapper around an matching algorithm implementation that allows arbitrary structs to be
/// used as generic markets.
pub trait Market<
    MarketTraderId: BorshDeserialize + BorshSerialize + Copy,
    MarketOrderId: OrderId,
    MarketRestingOrder: RestingOrder,
    MarketOrderPacket: OrderPacketMetadata,
>
{
    fn get_data_size(&self) -> usize {
        unimplemented!()
    }
    fn get_collected_fee_amount(&self) -> QuoteLots {
        unimplemented!()
    }
    fn get_uncollected_fee_amount(&self) -> QuoteLots {
        unimplemented!()
    }

    fn get_ladder(&self, levels: u64) -> Ladder {
        self.get_ladder_with_expiration(levels, None, None)
    }

    fn get_ladder_with_expiration(
        &self,
        levels: u64,
        last_valid_slot: Option<u64>,
        last_valid_unix_timestamp_in_seconds: Option<u64>,
    ) -> Ladder {
        let ladder = self.get_typed_ladder_with_expiration(
            levels,
            last_valid_slot,
            last_valid_unix_timestamp_in_seconds,
        );
        Ladder {
            bids: ladder
                .bids
                .iter()
                .map(|order| LadderOrder {
                    price_in_ticks: order.price_in_ticks.as_u64(),
                    size_in_base_lots: order.size_in_base_lots.as_u64(),
                })
                .collect(),
            asks: ladder
                .asks
                .iter()
                .map(|order| LadderOrder {
                    price_in_ticks: order.price_in_ticks.as_u64(),
                    size_in_base_lots: order.size_in_base_lots.as_u64(),
                })
                .collect(),
        }
    }

    fn get_typed_ladder(&self, levels: u64) -> TypedLadder {
        self.get_typed_ladder_with_expiration(levels, None, None)
    }

    fn get_typed_ladder_with_expiration(
        &self,
        levels: u64,
        last_valid_slot: Option<u64>,
        last_valid_unix_timestamp_in_seconds: Option<u64>,
    ) -> TypedLadder {
        let slot_expiration = last_valid_slot.unwrap_or(0);
        let unix_timestamp_expiration = last_valid_unix_timestamp_in_seconds.unwrap_or(0);
        let mut bids = vec![];
        let mut asks = vec![];
        for (side, book) in [(Side::Bid, &mut bids), (Side::Ask, &mut asks)].iter_mut() {
            book.extend_from_slice(
                &self
                    .get_book(*side)
                    .iter()
                    .filter_map(|(order_id, resting_order)| {
                        if resting_order.is_expired(slot_expiration, unix_timestamp_expiration) {
                            None
                        } else {
                            Some((order_id.price_in_ticks(), resting_order.size()))
                        }
                    })
                    .group_by(|(price_in_ticks, _)| *price_in_ticks)
                    .into_iter()
                    .take(levels as usize)
                    .map(|(price_in_ticks, group)| TypedLadderOrder {
                        price_in_ticks: Ticks::new(price_in_ticks),
                        size_in_base_lots: BaseLots::new(group.map(|(_, size)| size).sum()),
                    })
                    .collect::<Vec<TypedLadderOrder>>(),
            );
        }
        TypedLadder { bids, asks }
    }

    fn get_taker_fee_bps(&self) -> u64;
    fn get_tick_size(&self) -> QuoteLotsPerBaseUnitPerTick;
    fn get_base_lots_per_base_unit(&self) -> BaseLotsPerBaseUnit;
    fn get_sequence_number(&self) -> u64;
    fn get_registered_traders(&self) -> &dyn OrderedNodeAllocatorMap<MarketTraderId, TraderState>;
    fn get_trader_state(&self, key: &MarketTraderId) -> Option<&TraderState>;
    fn get_trader_state_from_index(&self, index: u32) -> &TraderState;
    fn get_trader_index(&self, trader: &MarketTraderId) -> Option<u32>;
    fn get_trader_id_from_index(&self, trader_index: u32) -> MarketTraderId;
    fn get_book(
        &self,
        side: Side,
    ) -> &dyn OrderedNodeAllocatorMap<MarketOrderId, MarketRestingOrder>;
}
