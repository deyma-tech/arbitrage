use ahash::AHashSet;
use anyhow::{format_err, Context};
use borsh::BorshDeserialize;
use num_traits::ToPrimitive;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use spl_token_swap::curve::base::SwapCurve;
use spl_token_swap::curve::calculator::TradeDirection;
use spl_token_swap::curve::fees::Fees;
use spl_token_swap::solana_program::program_pack::Pack;
use spl_token_swap::state::SwapV1;
use utils::constants::SPL_TOKEN_2022_ID;
use utils::pool::Pool;
use utils::quote::QuoteResult;

use anyhow::Result;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

// #[derive(Debug, Default, PartialEq)]
// pub struct MarketCurveSwap {
//     pub swap_curve: SwapCurve
// }
//
// impl Clone for MarketCurveSwap {
//     fn clone(&self) -> Self {
//         let mut buffer = [0_u8;33];
//         self.swap_curve.pack_into_slice(&mut buffer);
//         let swap_curve = SwapCurve::unpack_from_slice(&buffer).un_wrap();
//         Self{ swap_curve: swap_curve}
//     }
// }

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Market {
    /// Initialized state.
    pub is_initialized: bool,
    /// Bump seed used in program address.
    /// The program address is created deterministically with the bump seed,
    /// swap program id, and swap account pubkey.  This program address has
    /// authority over the swap's token A account, token B account, and pool
    /// token mint.
    pub bump_seed: u8,

    /// Program ID of the tokens being exchanged.
    pub token_program_id: Pubkey,

    /// Token A
    pub token_a: Pubkey,
    /// Token B
    pub token_b: Pubkey,

    /// Pool tokens are issued when A or B tokens are deposited.
    /// Pool tokens can be withdrawn back to the original A or B token.
    pub pool_mint: Pubkey,

    /// Mint information for token A
    pub token_a_mint: Pubkey,
    /// Mint information for token B
    pub token_b_mint: Pubkey,

    /// Pool token account to receive trading and / or withdrawal fees
    pub pool_fee_account: Pubkey,

    /// All fee information
    pub fees: Fees,

    /// Swap curve parameters, to be unpacked and used by the SwapCurve, which
    /// calculates swaps, deposits, and withdrawals
    pub swap_curve: [u8; 33],
    //pub swap_curve_struct: SwapCurve,
    //swap_curve_cache: RefCell<Option<SwapCurve>>,
}

// impl Clone for Market {
//     fn clone(&self) -> Self {
//         let mut buffer = [0u8; 33];
//         self.swap_curve_struct.pack_into_slice(&mut buffer);
//         let swap_curve_struct = SwapCurve::unpack_from_slice(&buffer).expect("Failed to clone SwapCurve");

//         Market {
//             is_initialized: self.is_initialized,
//             bump_seed: self.bump_seed,
//             token_program_id: self.token_program_id,
//             token_a: self.token_a,
//             token_b: self.token_b,
//             pool_mint: self.pool_mint,
//             token_a_mint: self.token_a_mint,
//             token_b_mint: self.token_b_mint,
//             pool_fee_account: self.pool_fee_account,
//             fees: self.fees.clone(),
//             swap_curve: self.swap_curve,
//             swap_curve_struct,
//         }
//     }
// }

pub struct SyncSwapCurve(pub SwapCurve);

unsafe impl Send for SyncSwapCurve {}
unsafe impl Sync for SyncSwapCurve {}

impl Deref for SyncSwapCurve {
    type Target = SwapCurve;
    fn deref(&self) -> &SwapCurve {
        &self.0
    }
}

lazy_static! {
    static ref SWAP_CURVE_CACHE: Mutex<HashMap<[u8; 33], Arc<SyncSwapCurve>>> = Mutex::new(HashMap::new());
}

pub fn get_swap_curve_cached(swap_curve_bytes: &[u8; 33]) -> Result<Arc<SyncSwapCurve>> {
    let mut cache = SWAP_CURVE_CACHE.lock().unwrap();
    if let Some(existing) = cache.get(swap_curve_bytes) {
        Ok(existing.clone())
    } else {
        let parsed = SwapCurve::unpack_from_slice(swap_curve_bytes)
            .map_err(|e| anyhow::format_err!("SwapCurve unpack failed: {:?}", e))?;
        let arc = Arc::new(SyncSwapCurve(parsed));
        cache.insert(*swap_curve_bytes, arc.clone());
        Ok(arc)
    }
}

impl Market {
    // pub fn get_swap_curve_cached(&self) -> anyhow::Result<std::cell::Ref<SwapCurve>> {
    //     if self.swap_curve_cache.borrow().is_none() {
    //         let curve = SwapCurve::unpack_from_slice(&self.swap_curve)?;
    //         *self.swap_curve_cache.borrow_mut() = Some(curve);
    //     }
    //     Ok(std::cell::Ref::map(self.swap_curve_cache.borrow(), |opt| opt.as_ref().unwrap()))
    // }

    pub fn try_deserialize(data: &[u8]) -> anyhow::Result<Self> {
        let result = SwapV1::unpack(&data[1..]);
        match result {
            Ok(swap_v1) => {
                let mut buffer = [0_u8; 33];
                swap_v1.swap_curve.pack_into_slice(&mut buffer);

                // let swap_curve_struct = SwapCurve::unpack_from_slice(&buffer)
                //     .context("swap_curve_struct")?;

                let market = Market {
                    is_initialized: swap_v1.is_initialized,
                    bump_seed: swap_v1.nonce,
                    token_program_id: Pubkey::try_from_slice(swap_v1.token_program_id.as_ref())
                        .context("token_program_id")?,
                    token_a: Pubkey::try_from_slice(swap_v1.token_a.as_ref()).context("token_a")?,
                    token_b: Pubkey::try_from_slice(swap_v1.token_b.as_ref()).context("token_b")?,
                    pool_mint: Pubkey::try_from_slice(swap_v1.pool_mint.as_ref()).context("pool_mint")?,
                    token_a_mint: Pubkey::try_from_slice(swap_v1.token_a_mint.as_ref()).context("token_a_mint")?,
                    token_b_mint: Pubkey::try_from_slice(swap_v1.token_b_mint.as_ref()).context("token_b_mint")?,
                    pool_fee_account: Pubkey::try_from_slice(swap_v1.pool_fee_account.as_ref())
                        .context("pool_fee_account")?,
                    fees: swap_v1.fees,
                    swap_curve: buffer,
                    //swap_curve_struct,
                    //swap_curve_cache: RefCell::new(None),
                };

                Ok(market)
            }
            Err(err) => Err(anyhow::format_err!("Failed to deserialize Market: {:?}", err)),
        }
    }
}

pub const MARKET_LENGTH: usize = 324;

pub fn get_authority(market_key: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[market_key.as_ref()], &super::ID).0
}

pub struct QuoteInput {
    pub coin_a: u64,
    pub coin_b: u64,
    //pub swap_curve: SyncSwapCurve,
}

impl Pool<QuoteInput> for Market {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.token_a, &self.token_b]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.token_a, self.token_b]
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount_in: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
        if exact_in {
            if a_to_b {
                if amount_in > input.coin_a {
                    return Err(format_err!("OrcaSwapV2PoolOutOfLiquidity"));
                }
            } else if amount_in > input.coin_b {
                return Err(format_err!("OrcaSwapV2PoolOutOfLiquidity"));
            }

            // TODO deserializing !!!!!
            let swap_curve = SwapCurve::unpack_from_slice(&self.swap_curve).context("swap_curve")?;

            //let swap_curve = self.get_swap_curve_cached()?;

            // let swap_curve_arc = get_swap_curve_cached(&self.swap_curve).context("getting SwapCurve from cache")?;
            // let swap_curve: &SyncSwapCurve = &swap_curve_arc;

            let result = swap_curve.swap(
                amount_in as u128,
                if a_to_b {
                    input.coin_a as u128
                } else {
                    input.coin_b as u128
                },
                if a_to_b {
                    input.coin_b as u128
                } else {
                    input.coin_a as u128
                },
                if a_to_b {
                    TradeDirection::AtoB
                } else {
                    TradeDirection::BtoA
                },
                &self.fees,
            );
            match result {
                None => Err(anyhow::format_err!("CalculationFailed")),
                Some(result) => {
                    let destination_amount_swapped = result.destination_amount_swapped;
                    let mut quote_result = QuoteResult {
                        amount_a_in: 0,
                        amount_b_in: 0,
                        amount_a_out: 0,
                        amount_b_out: 0,
                        indices: None,
                        compute_units: 58_500,
                    };
                    if a_to_b {
                        quote_result.amount_a_in = amount_in;
                        quote_result.amount_b_out = destination_amount_swapped.to_u64().context("Overflow")?;
                    } else {
                        quote_result.amount_b_in = amount_in;
                        quote_result.amount_a_out = destination_amount_swapped.to_u64().context("Overflow")?;
                    }
                    Ok(quote_result)
                }
            }
        } else {
            Err(anyhow::format_err!("NotAllowed (OrcaSwapV2)"))
        }
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        &self.token_a_mint
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        &self.token_b_mint
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        _a_to_b: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        if exact_in {
            let swap = spl_token_swap::instruction::Swap {
                amount_in: amount,
                minimum_amount_out: amount_threshold,
            };
            Ok(spl_token_swap::instruction::SwapInstruction::Swap(swap).pack())
        } else {
            Ok(vec![])
        }
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        _optional_account: Option<Pubkey>,
        _optional_accounts: Option<Vec<Pubkey>>,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        Ok(vec![
            AccountMeta::new_readonly(pool_pubkey, false),
            AccountMeta::new_readonly(get_authority(&pool_pubkey), false),
            AccountMeta::new_readonly(signer, true),
            AccountMeta::new(source_token_account, false),
            AccountMeta::new(if a_to_b { self.token_a } else { self.token_b }, false),
            AccountMeta::new(if a_to_b { self.token_b } else { self.token_a }, false),
            AccountMeta::new(destination_token_account, false),
            AccountMeta::new(self.pool_mint, false),
            AccountMeta::new(self.pool_fee_account, false),
            AccountMeta::new_readonly(self.token_program_id, false),
            // AccountMeta::new_readonly(*pool_pubkey, false),
            // AccountMeta::new_readonly(*authority_pubkey, false),
            // AccountMeta::new_readonly(*user_transfer_authority_pubkey, true),
            // AccountMeta::new(*source_pubkey, false),
            // AccountMeta::new(*swap_source_pubkey, false),
            // AccountMeta::new(*swap_destination_pubkey, false),
            // AccountMeta::new(*destination_pubkey, false),
            // AccountMeta::new(*pool_mint_pubkey, false),
            // AccountMeta::new(*pool_fee_pubkey, false),
            // AccountMeta::new_readonly(*source_mint_pubkey, false),
            // AccountMeta::new_readonly(*destination_mint_pubkey, false),
            // AccountMeta::new_readonly(*source_token_program_id, false),
            // AccountMeta::new_readonly(*destination_token_program_id, false),
            // AccountMeta::new_readonly(*pool_token_program_id, false),
        ])
    }
}

impl Market {
    pub fn is_token_2022(&self) -> bool {
        self.token_program_id == SPL_TOKEN_2022_ID
    }
}

// struct Market {
//     swap: spl_token_swap::
// }

// #[derive(Copy, Clone, Debug )]
// pub struct Fees {
//     pub trade_fee_numerator: u64, // 8
//     pub trade_fee_denominator: u64, // 8, 16
//     pub owner_trade_fee_numerator: u64, // 8, 24
//     pub owner_trade_fee_denominator: u64, // 8, 32
//     pub owner_withdraw_fee_numerator: u64, // 8, 40
//     pub owner_withdraw_fee_denominator: u64, // 8, 48
//     pub host_fee_numerator: u64, // 8, 56
//     pub host_fee_denominator: u64, // 8, 64
// }
// //
// /// Curve types supported by the token-swap program.
// #[repr(C)]
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum SwapCurve {
//     /// Uniswap-style constant product curve, invariant = token_a_amount * token_b_amount
//     ConstantProduct,
//     /// Flat line, always providing 1:1 from one token to another
//     ConstantPrice,
//     /// Stable, like uniswap, but with wide zone of 1:1 instead of one point
//     Stable,
//     /// Offset curve, like Uniswap, but the token B side has a faked offset
//     Offset,
// }
//
// // 324 bytes ...
// #[derive(Copy, Clone, Debug )]
// pub struct Market {
//     pub is_initialized: bool, // 1
//     pub bump_seed: u8, // 1, 2
//     pub token_program_id: Pubkey, // 32, 34
//     pub token_a: Pubkey, // 32, 66
//     pub token_b: Pubkey, // 32, 98
//     pub pool_mint: Pubkey, // 32, 130
//     pub token_a_mint: Pubkey, // 32, 162
//     pub token_b_mint: Pubkey, // 32, 194
//     pub pool_fee_account: Pubkey, // 32, 226
//     pub fees: Fees, // 64, 290
//     pub swap_curve: SwapCurve,
// }
