use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use sokoban::node_allocator::ZeroCopy;
use solana_sdk::pubkey::Pubkey;

use super::{quantities::{BaseAtomsPerBaseLot, QuoteAtomsPerBaseUnitPerTick, QuoteAtomsPerQuoteLot, WrapperU64}, status::MarketStatus};

#[derive(Default, Debug, Copy, Clone,  Pod, BorshDeserialize, BorshSerialize, Zeroable)]
#[repr(C)]
pub struct MarketSizeParams {
    pub bids_size: u64,
    pub asks_size: u64,
    pub num_seats: u64,
}
impl ZeroCopy for MarketSizeParams {}


#[derive(Debug, Copy, Clone, Pod, BorshDeserialize, BorshSerialize, Zeroable)]
#[repr(C)]
pub struct TokenParams {
    /// Number of decimals for the token (e.g. 9 for SOL, 6 for USDC).
    pub decimals: u32,

    /// Bump used for generating the PDA for the market's token vault.
    pub vault_bump: u32,

    /// Pubkey of the token mint.
    pub mint_key: Pubkey,

    /// Pubkey of the token vault.
    pub vault_key: Pubkey,
}
impl ZeroCopy for TokenParams {}


#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct MarketHeader {
    pub discriminant: u64,
    pub status: u64,
    pub market_size_params: MarketSizeParams,
    pub base_params: TokenParams,
    base_lot_size: BaseAtomsPerBaseLot,
    pub quote_params: TokenParams,
    quote_lot_size: QuoteAtomsPerQuoteLot,
    tick_size_in_quote_atoms_per_base_unit: QuoteAtomsPerBaseUnitPerTick,
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub market_sequence_number: u64,
    pub successor: Pubkey,
    pub raw_base_units_per_base_unit: u32,
    _padding1: u32,
    _padding2: [u64; 32],
}
impl ZeroCopy for MarketHeader {}

impl MarketHeader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        market_size_params: MarketSizeParams,
        base_params: TokenParams,
        base_lot_size: BaseAtomsPerBaseLot,
        quote_params: TokenParams,
        quote_lot_size: QuoteAtomsPerQuoteLot,
        tick_size_in_quote_atoms_per_base_unit: QuoteAtomsPerBaseUnitPerTick,
        authority: Pubkey,
        successor: Pubkey,
        fee_recipient: Pubkey,
        raw_base_units_per_base_unit: u32,
    ) -> Self {
        Self {
            discriminant: 0_u64, //get_discriminant::<MarketHeader>().unwrap(),
            status: MarketStatus::PostOnly as u64,
            market_size_params,
            base_params,
            base_lot_size,
            quote_params,
            quote_lot_size,
            tick_size_in_quote_atoms_per_base_unit,
            authority,
            fee_recipient,
            market_sequence_number: 0,
            successor,
            raw_base_units_per_base_unit,
            _padding1: 0,
            _padding2: [0; 32],
        }
    }

    /// Converts a price from quote atoms per base unit to ticks.
    pub fn price_in_ticks(&self, price: u64) -> u64 {
        price / self.tick_size_in_quote_atoms_per_base_unit.as_u64()
    }

    pub fn get_base_lot_size(&self) -> BaseAtomsPerBaseLot {
        self.base_lot_size
    }

    pub fn get_quote_lot_size(&self) -> QuoteAtomsPerQuoteLot {
        self.quote_lot_size
    }

    pub fn get_tick_size_in_quote_atoms_per_base_unit(&self) -> QuoteAtomsPerBaseUnitPerTick {
        self.tick_size_in_quote_atoms_per_base_unit
    }

    pub fn increment_sequence_number(&mut self) {
        self.market_sequence_number += 1;
    }

    pub fn from_bytes(data: &'_ [u8]) -> Option<&'_ Self> {
        MarketHeader::load_bytes(data)
    }

}