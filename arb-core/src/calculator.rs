use std::collections::BTreeMap;

use anyhow::Context;
use dex::meteora_dlmm::{BinArray, BinArrayBitmapExtension};
use dex::orca::OrcaTickArray;
use dex::orca::OrcaWhirlpool;
use dex::raydium_amm::AmmInfo;
use dex::raydium_clmm::AmmConfig;
use log::warn;
use solana_sdk::pubkey::Pubkey;
use utils::pool::Pool;

use utils::pool::PoolType;

use dex::pump_amm::GlobalConfig;
use utils::types::MintPair;

pub trait FirstPriceQuote {
    fn get_all_mint_pairs(&self) -> Vec<MintPair>;
    fn get_pubkey(&self) -> &Pubkey;
    fn get_sorted_mints(&self) -> Vec<&Pubkey>;
    fn get_sorted_mints_as_str(&self) -> String;
    fn get_sorted_mints_as_array(&self) -> MintPair;
    fn get_pool_type(&self) -> PoolType;
    fn quote(&self, amount: u64, sorted_a_b: bool, mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)>;
}

#[derive(Debug, Clone)]
pub struct OrcaData {
    pub whirlpool: OrcaWhirlpool,
    pub whirlpool_pubkey: Pubkey,
    pub tick_arrays_map: BTreeMap<i32, (Pubkey, OrcaTickArray)>,
    pub oracle: Option<dex::orca::Oracle>,
}

impl FirstPriceQuote for OrcaData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.whirlpool.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.whirlpool_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.whirlpool.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.whirlpool.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.whirlpool.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::Orca
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.whirlpool.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.whirlpool.get_a_mint());

        /*
        let a_to_b = if sorted_a_b && starts_with_a {
            true
        } else if sorted_a_b && !starts_with_a {
            false
        } else if !sorted_a_b && starts_with_a {
            false
        } else if !sorted_a_b && !starts_with_a {
            true
        } else {
            return Err(anyhow::format_err!("BrokenLogic"));
        };
         */

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::orca::QuoteInput {
            map: &self.tick_arrays_map,
            oracle: self.oracle.as_ref(),
        };

        let result = self.whirlpool.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct RaydiumClmmV3Data {
    pub amm_config: AmmConfig,
    pub pool_state: dex::raydium_clmm::PoolState,
    pub pool_state_pubkey: Pubkey,
    pub tick_arrays_map: BTreeMap<i32, (Pubkey, dex::raydium_clmm::TickArrayState)>,
    pub bitmap_extension: Option<Pubkey>,
    pub bitmap_extension_data: Option<dex::raydium_clmm::TickArrayBitmapExtension>,
}

impl RaydiumClmmV3Data {
    pub fn get_input(&self) -> dex::raydium_clmm::QuoteInput<'_> {
        dex::raydium_clmm::QuoteInput {
            amm_config: &self.amm_config,
            tick_arrays: &self.tick_arrays_map,
            bitmap_extension: self.bitmap_extension_data.as_ref(),
        }
    }
}

impl FirstPriceQuote for RaydiumClmmV3Data {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.pool_state.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.pool_state_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.pool_state.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.pool_state.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.pool_state.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::RaydiumClmmV3
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.pool_state.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.pool_state.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = self.get_input();
        let result = self.pool_state.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct RaydiumAmmV4Data {
    pub amm_info_pubkey: Pubkey,
    pub amm_info: AmmInfo,
    pub total_pc: u64,
    pub total_coin: u64,
}

impl FirstPriceQuote for RaydiumAmmV4Data {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.amm_info.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.amm_info_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.amm_info.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.amm_info.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.amm_info.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::RaydiumAmmV4
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.amm_info.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.amm_info.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::raydium_amm::QuoteInput {
            total_pc: self.total_pc,
            total_coin: self.total_coin,
        };
        let result = self.amm_info.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct PumpAmmData {
    pub pool_pubkey: Pubkey,
    pub pool: dex::pump_amm::Pool,
    pub config_pubkey: Pubkey,
    pub config: GlobalConfig,
    pub base: u64,
    pub quote: u64,
    pub fee: u64,
}

impl FirstPriceQuote for crate::calculator::PumpAmmData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.pool.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.pool_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.pool.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.pool.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.pool.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::PumpAmm
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.pool.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.pool.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::pump_amm::QuoteInput {
            base: self.base,
            quote: self.quote,
            fee: self.fee,
        };
        let result = self.pool.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeteoraDlmmData {
    pub lb_pair_pubkey: Pubkey,
    pub lb_pair: dex::meteora_dlmm::accounts::LbPair,
    pub bin_arrays_map: BTreeMap<i32, (Pubkey, BinArray)>,
    pub bitmap_extension_pubkey: Option<Pubkey>,
    pub bitmap_extension: Option<BinArrayBitmapExtension>,
    pub slot: u64,
}

impl MeteoraDlmmData {
    fn get_input(&self) -> dex::meteora_dlmm::quote::QuoteInput<'_> {
        dex::meteora_dlmm::quote::QuoteInput {
            bin_arrays: &self.bin_arrays_map,
            bitmap_extension: self.bitmap_extension.as_ref(),
            current_slot: self.slot,
        }
    }
}

impl FirstPriceQuote for MeteoraDlmmData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.lb_pair.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.lb_pair_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.lb_pair.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.lb_pair.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.lb_pair.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::MeteoraDlmm
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.lb_pair.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.lb_pair.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = self.get_input();
        let result = self.lb_pair.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct StabbleStableSwapData {
    pub pool_swap_pubkey: Pubkey,
    pub pool_swap: dex::stabble_stable_swap::PoolSwap,
}

impl FirstPriceQuote for crate::calculator::StabbleStableSwapData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.pool_swap.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.pool_swap_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.pool_swap.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.pool_swap.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.pool_swap.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::StabbleStableSwap
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.pool_swap.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.pool_swap.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::stabble_stable_swap::QuoteInput { mint_pair: None };
        let result = self.pool_swap.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct StabbleWeightedSwapData {
    pub pool_swap_pubkey: Pubkey,
    pub pool_swap: dex::stabble_weighted_swap::accounts::PoolWeightedSwap,
}

impl FirstPriceQuote for crate::calculator::StabbleWeightedSwapData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.pool_swap.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.pool_swap_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.pool_swap.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.pool_swap.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.pool_swap.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::StabbleWeightedSwap
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let mut sorted_pair = self.pool_swap.get_sorted_mints_as_array();
        if let Some(mint_pair) = mint_pair {
            sorted_pair = mint_pair;
        }

        let starts_with_a = sorted_pair.first() == Some(self.pool_swap.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::stabble_weighted_swap::accounts::QuoteInput {
            mint_pair: Some(sorted_pair),
        };
        let result = self.pool_swap.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct LifinityData {
//     pub amm_pubkey: Pubkey,
//     pub amm: dex::lifinity::accounts::Amm,
//     pub slot: u64,
//     pub total_pc: u64,
//     pub total_coin: u64,
//     pub oracle_main: Price,
//     pub oracle_sub: Price,
//     pub oracle_pc: Price,
// }

// impl FirstPriceQuote for crate::calculator::LifinityData {
//     fn get_all_mint_pairs(&self) -> Vec<MintPair> {
//         self.amm.get_all_mint_pairs()
//     }
//     fn get_pubkey(&self) -> &Pubkey {
//         &self.amm_pubkey
//     }

//     fn get_sorted_mints(&self) -> Vec<&Pubkey> {
//         self.amm.get_sorted_mints()
//     }

//     fn get_sorted_mints_as_str(&self) -> String {
//         self.amm.get_sorted_mints_as_str()
//     }

//     fn get_sorted_mints_as_array(&self) -> MintPair {
//         self.amm.get_sorted_mints_as_array()
//     }

//     fn get_pool_type(&self) -> PoolType {
//         PoolType::Lifinity
//     }

//     fn quote(&self, amount: u64, sorted_a_b: bool, _mint_piar: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
//         let sorted_pair = self.amm.get_sorted_mints();

//         let starts_with_a = sorted_pair.first() == Some(&self.amm.get_a_mint());

//         let a_to_b = match (sorted_a_b, starts_with_a) {
//             (true, true) | (false, false) => true,
//             (true, false) | (false, true) => false,
//         };

//         let input = dex::lifinity::accounts::QuoteInput {
//             coin_a: self.total_coin,
//             coin_b: self.total_pc,
//             slot: self.slot,
//             oracle_main: &self.oracle_main,
//             oracle_sub: &self.oracle_sub,
//             oracle_pc: &self.oracle_pc,
//         };
//         let result = self.amm.quote(a_to_b, true, amount, &input)?;

//         if a_to_b {
//             let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
//             Ok((
//                 if !starts_with_a {
//                     1.0 / a_to_b_price
//                 } else {
//                     a_to_b_price
//                 },
//                 result.amount_b_out,
//             ))
//         } else {
//             let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
//             Ok((
//                 if starts_with_a {
//                     1.0 / a_to_b_price
//                 } else {
//                     a_to_b_price
//                 },
//                 result.amount_a_out,
//             ))
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct RaydiumCpmmData {
    pub pool_state_pubkey: Pubkey,
    pub pool_state: dex::raydium_cpmm::PoolState,
    pub amm_config: dex::raydium_cpmm::AmmConfig,
    pub total_pc: u64,
    pub total_coin: u64,
}

impl FirstPriceQuote for crate::calculator::RaydiumCpmmData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.pool_state.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.pool_state_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.pool_state.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.pool_state.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.pool_state.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::RaydiumCpmm
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.pool_state.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.pool_state.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::raydium_cpmm::QuoteInput {
            coin_a: self.total_coin,
            coin_b: self.total_pc,
            config: &self.amm_config,
        };
        let result = self.pool_state.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrcaSwapV2Data {
    pub market_pubkey: Pubkey,
    pub market: dex::orca_swap_v2::accounts::Market,
    pub total_coin: u64,
    pub total_pc: u64,
}

impl FirstPriceQuote for crate::calculator::OrcaSwapV2Data {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.market.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.market_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.market.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.market.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.market.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::OrcaSwapV2
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.market.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.market.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::orca_swap_v2::accounts::QuoteInput {
            coin_a: self.total_coin,
            coin_b: self.total_pc,
        };
        let result = self.market.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SarosAmmData {
    pub market_pubkey: Pubkey,
    pub market: dex::saros_amm::accounts::Market,
    pub total_coin: u64,
    pub total_pc: u64,
}

impl FirstPriceQuote for crate::calculator::SarosAmmData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.market.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.market_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.market.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.market.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.market.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::SarosAmm
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.market.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.market.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::saros_amm::accounts::QuoteInput {
            coin_a: self.total_coin,
            coin_b: self.total_pc,
        };
        let result = self.market.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct GooseGammaData {
//     pub pool_pubkey: Pubkey,
//     pub pool: dex::goose_gamma::accounts::PoolState,
//     pub config: dex::goose_gamma::accounts::AmmConfig,
//     pub observation_state: dex::goose_gamma::accounts::ObservationState,
//     pub coin_a: u64,
//     pub coin_b: u64,
// }

// impl FirstPriceQuote for GooseGammaData {
//     fn get_all_mint_pairs(&self) -> Vec<MintPair> {
//         self.pool.get_all_mint_pairs()
//     }
//     fn get_pubkey(&self) -> &Pubkey {
//         &self.pool_pubkey
//     }

//     fn get_sorted_mints(&self) -> Vec<&Pubkey> {
//         self.pool.get_sorted_mints()
//     }

//     fn get_sorted_mints_as_str(&self) -> String {
//         self.pool.get_sorted_mints_as_str()
//     }

//     fn get_sorted_mints_as_array(&self) -> MintPair {
//         self.pool.get_sorted_mints_as_array()
//     }

//     fn get_pool_type(&self) -> PoolType {
//         PoolType::GooseGamma
//     }

//     fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
//         let sorted_pair = self.pool.get_sorted_mints();

//         let starts_with_a = sorted_pair.first() == Some(&self.pool.get_a_mint());

//         let a_to_b = match (sorted_a_b, starts_with_a) {
//             (true, true) | (false, false) => true,
//             (true, false) | (false, true) => false,
//         };

//         let input = dex::goose_gamma::accounts::QuoteInput {
//             coin_a: self.coin_a,
//             coin_b: self.coin_b,
//             amm_config: &self.config,
//             observation_state: &self.observation_state,
//         };
//         let result = self.pool.quote(a_to_b, true, amount, &input)?;

//         if a_to_b {
//             let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
//             Ok((
//                 if !starts_with_a {
//                     1.0 / a_to_b_price
//                 } else {
//                     a_to_b_price
//                 },
//                 result.amount_b_out,
//             ))
//         } else {
//             let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
//             Ok((
//                 if starts_with_a {
//                     1.0 / a_to_b_price
//                 } else {
//                     a_to_b_price
//                 },
//                 result.amount_a_out,
//             ))
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct MeteoraDammV2Data {
    pub pool_pubkey: Pubkey,
    pub pool: dex::meteora_damm_v2::PoolState,
    pub slot: u64,
}

impl FirstPriceQuote for MeteoraDammV2Data {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.pool.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.pool_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.pool.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.pool.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.pool.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::MeteoraDammV2
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.pool.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.pool.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::meteora_damm_v2::QuoteInput { slot: self.slot };
        let result = self.pool.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct FusionAmmData {
    pub pool_pubkey: Pubkey,
    pub pool: dex::fusion_amm::accounts::FusionPoolWhirlpool,
    pub tick_arrays_map: BTreeMap<i32, (Pubkey, OrcaTickArray)>,
}

impl FirstPriceQuote for FusionAmmData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.pool.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.pool_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.pool.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.pool.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.pool.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::FusionAmm
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.pool.whirlpool.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.pool.whirlpool.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let input = dex::orca::QuoteInput {
            map: &self.tick_arrays_map,
            oracle: None,
        };

        let result = self.pool.whirlpool.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SarosDlmmData {
    pub pool_pubkey: Pubkey,
    pub pair: dex::saros_dlmm::accounts::Pair,
    pub bin_arrays_map: BTreeMap<u32, (Pubkey, dex::saros_dlmm::accounts::BinArray)>,
}

impl FirstPriceQuote for SarosDlmmData {
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        self.pair.get_all_mint_pairs()
    }
    fn get_pubkey(&self) -> &Pubkey {
        &self.pool_pubkey
    }

    fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        self.pair.get_sorted_mints()
    }

    fn get_sorted_mints_as_str(&self) -> String {
        self.pair.get_sorted_mints_as_str()
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        self.pair.get_sorted_mints_as_array()
    }

    fn get_pool_type(&self) -> PoolType {
        PoolType::SarosDlmm
    }

    fn quote(&self, amount: u64, sorted_a_b: bool, _mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        let sorted_pair = self.pair.get_sorted_mints();

        let starts_with_a = sorted_pair.first() == Some(&self.pair.get_a_mint());

        let a_to_b = match (sorted_a_b, starts_with_a) {
            (true, true) | (false, false) => true,
            (true, false) | (false, true) => false,
        };

        let index = self.pair.active_id / dex::saros_dlmm::BIN_ARRAY_SIZE;
        let (_bin_array_lower_pubkey, bin_array_lower) =
            self.bin_arrays_map.get(&index).context("NotFound Active Bin Array")?;
        let (_bin_array_upper_pubkey, bin_array_upper) = self
            .bin_arrays_map
            .get(&(index + 1))
            .context("NotFound Active Bin Array")?;
        let bin_array_pair = dex::saros_dlmm::BinArrayPair {
            bin_array_lower: *bin_array_lower,
            bin_array_upper: *bin_array_upper,
        };
        let input = dex::saros_dlmm::QuoteInput {
            bin_array_pair: &bin_array_pair,
        };

        let result = self.pair.quote(a_to_b, true, amount, &input)?;

        if a_to_b {
            let a_to_b_price = result.amount_b_out as f64 / result.amount_a_in as f64;
            Ok((
                if !starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_b_out,
            ))
        } else {
            let a_to_b_price = result.amount_a_out as f64 / result.amount_b_in as f64;
            Ok((
                if starts_with_a {
                    1.0 / a_to_b_price
                } else {
                    a_to_b_price
                },
                result.amount_a_out,
            ))
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum CalculatorEnum {
    Orca(OrcaData),                   // box
    RaydiumClmmV3(RaydiumClmmV3Data), // box
    RaydiumAmmV4(RaydiumAmmV4Data),
    MeteoraDlmm(MeteoraDlmmData), // box
    PumpAmm(PumpAmmData),
    StabbleStableSwap(StabbleStableSwapData),
    StabbleWeightedSwap(StabbleWeightedSwapData),
    //Lifinity(LifinityData), // box ??
    RaydiumCpmm(RaydiumCpmmData),
    OrcaSwapV2(OrcaSwapV2Data),
    SarosAmm(SarosAmmData),
    //GooseGamma(GooseGammaData),
    MeteoraDammV2(MeteoraDammV2Data),
    FusionAmm(FusionAmmData),
    SarosDlmm(SarosDlmmData),
}

impl CalculatorEnum {
    pub fn get_mandatory_pubkeys(&self) -> Vec<&'_ Pubkey> {
        match self {
            CalculatorEnum::Orca(data) => data.whirlpool.get_keys(),
            CalculatorEnum::RaydiumClmmV3(data) => data.pool_state.get_keys(),
            CalculatorEnum::RaydiumAmmV4(data) => data.amm_info.get_keys(),
            CalculatorEnum::MeteoraDlmm(data) => data.lb_pair.get_keys(),
            CalculatorEnum::PumpAmm(data) => data.pool.get_keys(),
            CalculatorEnum::StabbleStableSwap(data) => data.pool_swap.get_keys(),
            CalculatorEnum::StabbleWeightedSwap(data) => data.pool_swap.get_keys(),
            //CalculatorEnum::Lifinity(data) => data.amm.get_keys(),
            CalculatorEnum::RaydiumCpmm(data) => data.pool_state.get_keys(),
            CalculatorEnum::OrcaSwapV2(data) => data.market.get_keys(),
            CalculatorEnum::SarosAmm(data) => data.market.get_keys(),
            //CalculatorEnum::GooseGamma(data) => data.pool.get_keys(),
            CalculatorEnum::MeteoraDammV2(data) => data.pool.get_keys(),
            CalculatorEnum::FusionAmm(data) => data.pool.get_keys(),
            CalculatorEnum::SarosDlmm(data) => data.pair.get_keys(),
        }
    }

    pub fn get_optional_pubkeys(&self) -> Option<Vec<&'_ Pubkey>> {
        match self {
            CalculatorEnum::Orca(orca) => Some(
                orca.tick_arrays_map
                    .iter()
                    .map(|(_, (pubkey, _))| pubkey)
                    .collect::<Vec<_>>(),
            ),
            CalculatorEnum::RaydiumClmmV3(data) => Some(
                data.tick_arrays_map
                    .iter()
                    .map(|(_, (pubkey, _))| pubkey)
                    .collect::<Vec<_>>(),
            ),
            CalculatorEnum::MeteoraDlmm(data) => Some(
                data.bin_arrays_map
                    .iter()
                    .map(|(_, (pubkey, _))| pubkey)
                    .collect::<Vec<_>>(),
            ),
            // CalculatorEnum::Lifinity(data) => Some(vec![
            //     &data.amm.fee_account,
            //     &data.amm.oracle_main_account,
            //     &data.amm.oracle_sub_account,
            //     &data.amm.oracle_pc_account,
            // ]),
            _ => None,
        }
    }

    pub fn quote_a_to_b_price(&self, amount: u64, mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        match self {
            CalculatorEnum::Orca(data) => data.quote(amount, true, None),
            CalculatorEnum::RaydiumClmmV3(data) => data.quote(amount, true, None),
            CalculatorEnum::RaydiumAmmV4(data) => data.quote(amount, true, None),
            CalculatorEnum::MeteoraDlmm(data) => data.quote(amount, true, None),
            CalculatorEnum::PumpAmm(data) => data.quote(amount, true, None),
            CalculatorEnum::StabbleStableSwap(data) => data.quote(amount, true, mint_pair),
            CalculatorEnum::StabbleWeightedSwap(data) => data.quote(amount, true, mint_pair),
            //CalculatorEnum::Lifinity(data) => data.quote(amount, true, None),
            CalculatorEnum::RaydiumCpmm(data) => data.quote(amount, true, None),
            CalculatorEnum::OrcaSwapV2(data) => data.quote(amount, true, None),
            CalculatorEnum::SarosAmm(data) => data.quote(amount, true, None),
            //CalculatorEnum::GooseGamma(data) => data.quote(amount, true, None),
            CalculatorEnum::MeteoraDammV2(data) => data.quote(amount, true, None),
            CalculatorEnum::FusionAmm(data) => data.quote(amount, true, None),
            CalculatorEnum::SarosDlmm(data) => data.quote(amount, true, None),
        }
    }

    pub fn quote_b_to_a_price(&self, amount: u64, mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        match self {
            CalculatorEnum::Orca(data) => data.quote(amount, false, None),
            CalculatorEnum::RaydiumClmmV3(data) => data.quote(amount, false, None),
            CalculatorEnum::RaydiumAmmV4(data) => data.quote(amount, false, None),
            CalculatorEnum::MeteoraDlmm(data) => data.quote(amount, false, None),
            CalculatorEnum::PumpAmm(data) => data.quote(amount, false, None),
            CalculatorEnum::StabbleStableSwap(data) => data.quote(amount, false, mint_pair),
            CalculatorEnum::StabbleWeightedSwap(data) => data.quote(amount, false, mint_pair),
            //CalculatorEnum::Lifinity(data) => data.quote(amount, false, None),
            CalculatorEnum::RaydiumCpmm(data) => data.quote(amount, false, None),
            CalculatorEnum::OrcaSwapV2(data) => data.quote(amount, false, None),
            CalculatorEnum::SarosAmm(data) => data.quote(amount, false, None),
            //CalculatorEnum::GooseGamma(data) => data.quote(amount, false, None),
            CalculatorEnum::MeteoraDammV2(data) => data.quote(amount, false, None),
            CalculatorEnum::FusionAmm(data) => data.quote(amount, false, None),
            CalculatorEnum::SarosDlmm(data) => data.quote(amount, false, None),
        }
    }

    pub fn quote_price(&self, a_to_b: bool, amount: u64, mint_pair: Option<MintPair>) -> anyhow::Result<(f64, u64)> {
        match self {
            CalculatorEnum::Orca(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::RaydiumClmmV3(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::RaydiumAmmV4(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::MeteoraDlmm(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::PumpAmm(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::StabbleStableSwap(data) => data.quote(amount, a_to_b, mint_pair),
            CalculatorEnum::StabbleWeightedSwap(data) => data.quote(amount, a_to_b, mint_pair),
            //CalculatorEnum::Lifinity(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::RaydiumCpmm(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::OrcaSwapV2(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::SarosAmm(data) => data.quote(amount, a_to_b, None),
            //CalculatorEnum::GooseGamma(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::MeteoraDammV2(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::FusionAmm(data) => data.quote(amount, a_to_b, None),
            CalculatorEnum::SarosDlmm(data) => data.quote(amount, a_to_b, None),
        }
    }

    pub fn get_pubkey(&self) -> &Pubkey {
        match self {
            CalculatorEnum::Orca(data) => data.get_pubkey(),
            CalculatorEnum::RaydiumClmmV3(data) => data.get_pubkey(),
            CalculatorEnum::RaydiumAmmV4(data) => data.get_pubkey(),
            CalculatorEnum::MeteoraDlmm(data) => data.get_pubkey(),
            CalculatorEnum::PumpAmm(data) => data.get_pubkey(),
            CalculatorEnum::StabbleStableSwap(data) => data.get_pubkey(),
            CalculatorEnum::StabbleWeightedSwap(data) => data.get_pubkey(),
            //CalculatorEnum::Lifinity(data) => data.get_pubkey(),
            CalculatorEnum::RaydiumCpmm(data) => data.get_pubkey(),
            CalculatorEnum::OrcaSwapV2(data) => data.get_pubkey(),
            CalculatorEnum::SarosAmm(data) => data.get_pubkey(),
            //CalculatorEnum::GooseGamma(data) => data.get_pubkey(),
            CalculatorEnum::MeteoraDammV2(data) => data.get_pubkey(),
            CalculatorEnum::FusionAmm(data) => data.get_pubkey(),
            CalculatorEnum::SarosDlmm(data) => data.get_pubkey(),
        }
    }

    pub fn get_sorted_mints(&self) -> Vec<&Pubkey> {
        match self {
            CalculatorEnum::Orca(data) => data.get_sorted_mints(),
            CalculatorEnum::RaydiumClmmV3(data) => data.get_sorted_mints(),
            CalculatorEnum::RaydiumAmmV4(data) => data.get_sorted_mints(),
            CalculatorEnum::MeteoraDlmm(data) => data.get_sorted_mints(),
            CalculatorEnum::PumpAmm(data) => data.get_sorted_mints(),
            CalculatorEnum::StabbleStableSwap(data) => data.get_sorted_mints(),
            CalculatorEnum::StabbleWeightedSwap(data) => data.get_sorted_mints(),
            //CalculatorEnum::Lifinity(data) => data.get_sorted_mints(),
            CalculatorEnum::RaydiumCpmm(data) => data.get_sorted_mints(),
            CalculatorEnum::OrcaSwapV2(data) => data.get_sorted_mints(),
            CalculatorEnum::SarosAmm(data) => data.get_sorted_mints(),
            //CalculatorEnum::GooseGamma(data) => data.get_sorted_mints(),
            CalculatorEnum::MeteoraDammV2(data) => data.get_sorted_mints(),
            CalculatorEnum::FusionAmm(data) => data.get_sorted_mints(),
            CalculatorEnum::SarosDlmm(data) => data.get_sorted_mints(),
        }
    }

    pub fn get_sorted_mints_as_str(&self) -> String {
        match self {
            CalculatorEnum::Orca(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::RaydiumClmmV3(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::RaydiumAmmV4(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::MeteoraDlmm(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::PumpAmm(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::StabbleStableSwap(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::StabbleWeightedSwap(data) => data.get_sorted_mints_as_str(),
            //CalculatorEnum::Lifinity(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::RaydiumCpmm(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::OrcaSwapV2(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::SarosAmm(data) => data.get_sorted_mints_as_str(),
            //CalculatorEnum::GooseGamma(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::MeteoraDammV2(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::FusionAmm(data) => data.get_sorted_mints_as_str(),
            CalculatorEnum::SarosDlmm(data) => data.get_sorted_mints_as_str(),
        }
    }

    pub fn get_sorted_mints_as_array(&self) -> MintPair {
        match self {
            CalculatorEnum::Orca(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::RaydiumClmmV3(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::RaydiumAmmV4(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::MeteoraDlmm(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::PumpAmm(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::StabbleStableSwap(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::StabbleWeightedSwap(data) => data.get_sorted_mints_as_array(),
            //CalculatorEnum::Lifinity(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::RaydiumCpmm(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::OrcaSwapV2(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::SarosAmm(data) => data.get_sorted_mints_as_array(),
            //CalculatorEnum::GooseGamma(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::MeteoraDammV2(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::FusionAmm(data) => data.get_sorted_mints_as_array(),
            CalculatorEnum::SarosDlmm(data) => data.get_sorted_mints_as_array(),
        }
    }

    pub fn get_all_sorted_mints_as_array(&self) -> Vec<MintPair> {
        match self {
            CalculatorEnum::Orca(data) => data.get_all_mint_pairs(),
            CalculatorEnum::RaydiumClmmV3(data) => data.get_all_mint_pairs(),
            CalculatorEnum::RaydiumAmmV4(data) => data.get_all_mint_pairs(),
            CalculatorEnum::MeteoraDlmm(data) => data.get_all_mint_pairs(),
            CalculatorEnum::PumpAmm(data) => data.get_all_mint_pairs(),
            CalculatorEnum::StabbleStableSwap(data) => data.get_all_mint_pairs(),
            CalculatorEnum::StabbleWeightedSwap(data) => data.get_all_mint_pairs(),
            //CalculatorEnum::Lifinity(data) => data.get_all_mint_pairs(),
            CalculatorEnum::RaydiumCpmm(data) => data.get_all_mint_pairs(),
            CalculatorEnum::OrcaSwapV2(data) => data.get_all_mint_pairs(),
            CalculatorEnum::SarosAmm(data) => data.get_all_mint_pairs(),
            //CalculatorEnum::GooseGamma(data) => data.get_all_mint_pairs(),
            CalculatorEnum::MeteoraDammV2(data) => data.get_all_mint_pairs(),
            CalculatorEnum::FusionAmm(data) => data.get_all_mint_pairs(),
            CalculatorEnum::SarosDlmm(data) => data.get_all_mint_pairs(),
        }
    }

    pub fn get_pool_type(&self) -> PoolType {
        match self {
            CalculatorEnum::Orca(data) => data.get_pool_type(),
            CalculatorEnum::RaydiumClmmV3(data) => data.get_pool_type(),
            CalculatorEnum::RaydiumAmmV4(data) => data.get_pool_type(),
            CalculatorEnum::MeteoraDlmm(data) => data.get_pool_type(),
            CalculatorEnum::PumpAmm(data) => data.get_pool_type(),
            CalculatorEnum::StabbleStableSwap(data) => data.get_pool_type(),
            CalculatorEnum::StabbleWeightedSwap(data) => data.get_pool_type(),
            //CalculatorEnum::Lifinity(data) => data.get_pool_type(),
            CalculatorEnum::RaydiumCpmm(data) => data.get_pool_type(),
            CalculatorEnum::OrcaSwapV2(data) => data.get_pool_type(),
            CalculatorEnum::SarosAmm(data) => data.get_pool_type(),
            //CalculatorEnum::GooseGamma(data) => data.get_pool_type(),
            CalculatorEnum::MeteoraDammV2(data) => data.get_pool_type(),
            CalculatorEnum::FusionAmm(data) => data.get_pool_type(),
            CalculatorEnum::SarosDlmm(data) => data.get_pool_type(),
        }
    }

    pub fn get_compute_units(&self) -> ComputeUnitsPerPoolType {
        match self {
            CalculatorEnum::Orca(_) => ComputeUnitsPerPoolType::Variable(88_000),
            CalculatorEnum::RaydiumClmmV3(_) => ComputeUnitsPerPoolType::Variable(98_000),
            CalculatorEnum::MeteoraDlmm(_) => ComputeUnitsPerPoolType::Variable(98_000),
            // CalculatorEnum::MeteoraDlmm(_) => {
            //     ComputeUnitsPerPoolType::Variable(if volume >= 50_000_000_000 {
            //         180_000
            //     } else {
            //         70_000
            //     })
            // }
            //
            CalculatorEnum::RaydiumAmmV4(_) => ComputeUnitsPerPoolType::Constant(32_000),
            CalculatorEnum::RaydiumCpmm(_) => ComputeUnitsPerPoolType::Constant(49_000),
            CalculatorEnum::PumpAmm(_) => ComputeUnitsPerPoolType::Constant(62_000),
            //CalculatorEnum::Lifinity(_) => ComputeUnitsPerPoolType::Constant(73_000),
            CalculatorEnum::OrcaSwapV2(_) => ComputeUnitsPerPoolType::Variable(59_000),
            CalculatorEnum::SarosAmm(_) => ComputeUnitsPerPoolType::Variable(59_000),
            CalculatorEnum::StabbleStableSwap(data) => {
                if data.get_all_mint_pairs().len() == 1 {
                    ComputeUnitsPerPoolType::Constant(90_000)
                } else {
                    ComputeUnitsPerPoolType::Constant(110_000)
                }
            }
            CalculatorEnum::StabbleWeightedSwap(data) => {
                if data.get_all_mint_pairs().len() == 1 {
                    ComputeUnitsPerPoolType::Constant(60_000)
                } else {
                    ComputeUnitsPerPoolType::Constant(75_000)
                }
            } // 316410
            //CalculatorEnum::GooseGamma(_) => ComputeUnitsPerPoolType::Constant(75_000),
            CalculatorEnum::MeteoraDammV2(_) => ComputeUnitsPerPoolType::Constant(65_000),
            CalculatorEnum::FusionAmm(_) => ComputeUnitsPerPoolType::Constant(88_000),
            CalculatorEnum::SarosDlmm(_) => ComputeUnitsPerPoolType::Constant(100_000),
        }
    }
}

impl CalculatorEnum {
    pub fn get_program_id(&self) -> Pubkey {
        match self {
            CalculatorEnum::RaydiumClmmV3(_) => dex::raydium_clmm::ID,
            CalculatorEnum::MeteoraDlmm(_) => dex::meteora_dlmm::ID,
            CalculatorEnum::Orca(_) => dex::orca::ID,
            CalculatorEnum::StabbleStableSwap(_) => dex::stabble_stable_swap::ID,
            CalculatorEnum::StabbleWeightedSwap(_) => dex::stabble_weighted_swap::ID,
            //CalculatorEnum::Lifinity(_) => dex::lifinity::ID,
            CalculatorEnum::RaydiumCpmm(_) => dex::raydium_cpmm::ID,
            CalculatorEnum::OrcaSwapV2(_) => dex::orca_swap_v2::ID,
            CalculatorEnum::SarosAmm(_) => dex::saros_amm::ID,
            //CalculatorEnum::GooseGamma(_) => dex::goose_gamma::ID,
            CalculatorEnum::RaydiumAmmV4(_) => dex::raydium_amm::ID,
            CalculatorEnum::PumpAmm(_) => dex::pump_amm::ID,
            CalculatorEnum::MeteoraDammV2(_) => dex::meteora_damm_v2::ID,
            CalculatorEnum::FusionAmm(_) => dex::fusion_amm::ID,
            CalculatorEnum::SarosDlmm(_) => dex::saros_dlmm::ID,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputeUnitsPerPoolType {
    Constant(u64),
    Variable(u64),
    Meteora(u64),
    //Variable(fn(u64) -> u64),
}

impl ComputeUnitsPerPoolType {
    pub fn is_constant(&self) -> bool {
        matches!(self, ComputeUnitsPerPoolType::Constant(_))
    }

    pub fn get_value(&self) -> u64 {
        match self {
            ComputeUnitsPerPoolType::Constant(value) => *value,
            ComputeUnitsPerPoolType::Variable(value) => *value,
            ComputeUnitsPerPoolType::Meteora(value) => *value,
            //ComputeUnitsPerPoolType::Variable(func) => func(0),
        }
    }

    pub fn apply_volume(&self, volume: u64) -> u64 {
        match self {
            ComputeUnitsPerPoolType::Variable(value) => *value + apply_volume_for_cu(volume, None),
            ComputeUnitsPerPoolType::Constant(value) => *value,
            ComputeUnitsPerPoolType::Meteora(value) => *value + (apply_volume_for_cu(volume, Some(25_000_f64)) * 8),
        }
    }
}

const ONE_SOL: f64 = 1_000_000_000_f64;
const CU_MULTIPLICATOR: f64 = 18_000_f64;

fn apply_volume_for_cu(volume: u64, multiplicator: Option<f64>) -> u64 {
    if volume > ONE_SOL as u64 {
        return 0;
    }
    let one_sol_log = ONE_SOL.ln();
    let volume_log = (volume as f64).ln();
    let result = volume_log - one_sol_log;
    if result.is_sign_negative() {
        return 0;
    }
    let extra_cu = match multiplicator {
        None => result * CU_MULTIPLICATOR,
        Some(multiplicator) => result * multiplicator,
    };
    if extra_cu.is_finite() && extra_cu.is_sign_positive() {
        (extra_cu as u64).min(280_000)
    } else {
        warn!("Extra compute units is not finite or positive: {}", extra_cu);
        0
    }
}

pub fn get_pool_types(calculators: &[Box<CalculatorEnum>]) -> Vec<PoolType> {
    calculators
        .iter()
        .map(|calculator| calculator.get_pool_type())
        .collect()
}

pub fn get_pubkeys(calculators: &[Box<CalculatorEnum>]) -> Vec<Pubkey> {
    calculators
        .iter()
        .map(|calculator| calculator.get_pubkey())
        .cloned()
        .collect()
}

pub fn get_pubkeys_as_ref(calculators: &[Box<CalculatorEnum>]) -> Vec<&Pubkey> {
    calculators.iter().map(|calculator| calculator.get_pubkey()).collect()
}
