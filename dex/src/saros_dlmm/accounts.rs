use ahash::AHashSet;
use anyhow::Context;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_sdk::{instruction::AccountMeta, pubkey};
use spl_associated_token_account::{get_associated_token_address, get_associated_token_address_with_program_id};
use utils::pool::Pool;

use crate::{
    //meteora_dlmm::events::Swap,
    saros_dlmm::{
        quote::{get_swap_result, SwapMode},
        typedefs::{
            Bin, ConfigAvailability, ConfigStatus, DynamicFeeParameters, QuoteAssetBadgeStatus, StaticFeeParameters,
            SwapType,
        },
        SwapIxArgs, SwapIxData, SwapKeys,
    },
};

pub const BIN_ARRAY_SIZE: u32 = 256;

impl Bin {
    pub fn is_zero_liquidity(&self) -> bool {
        self.total_supply == 0
    }
}

pub const BIN_ARRAY_ACCOUNT_DISCM: [u8; 8] = [92, 142, 92, 220, 5, 148, 70, 181];
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinArray {
    pub pair: Pubkey,
    pub bins: [Bin; 256],
    pub index: u32,
    pub space: [u8; 12],
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinArrayAccount(pub BinArray);
impl BinArrayAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        // use std::io::Read;
        // let mut reader = buf;
        // let mut maybe_discm = [0u8; 8];
        // reader.read_exact(&mut maybe_discm)?;
        // if maybe_discm != BIN_ARRAY_ACCOUNT_DISCM {
        //     return Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!(
        //             "discm does not match. Expected: {:?}. Received: {:?}",
        //             BIN_ARRAY_ACCOUNT_DISCM, maybe_discm
        //         ),
        //     ));
        // }
        // Ok(Self(BinArray::deserialize(&mut reader)?))
        let mut reader = &buf[8..];
        Ok(Self(BinArray::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&BIN_ARRAY_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

impl BinArray {
    pub fn initialize(&mut self, pair: Pubkey, index: u32) {
        self.pair = pair;
        self.index = index;
    }

    pub fn get_index_from_bin_id(bin_id: u32) -> u32 {
        bin_id / BIN_ARRAY_SIZE
    }

    pub fn contains(&self, bin_id: u32) -> bool {
        bin_id / BIN_ARRAY_SIZE == self.index
    }

    pub fn get_bin(&self, bin_id: u32) -> anyhow::Result<&Bin> {
        if !self.contains(bin_id) {
            return Err(anyhow::format_err!("BinNotFound"));
        }

        Ok(&self.bins[(bin_id % BIN_ARRAY_SIZE) as usize])
    }

    pub fn get_bin_mut(&mut self, bin_id: u32) -> anyhow::Result<&mut Bin> {
        if !self.contains(bin_id) {
            return Err(anyhow::format_err!("BinNotFound"));
        }

        Ok(&mut self.bins[(bin_id % BIN_ARRAY_SIZE) as usize])
    }

    pub fn is_zero_liquidity(&self) -> bool {
        for bin in self.bins.iter() {
            if !bin.is_zero_liquidity() {
                return false;
            }
        }
        true
    }
}

pub struct BinArrayPair {
    pub bin_array_lower: BinArray,
    pub bin_array_upper: BinArray,
}

impl Clone for BinArrayPair {
    fn clone(&self) -> Self {
        Self {
            bin_array_lower: self.bin_array_lower,
            bin_array_upper: self.bin_array_upper,
        }
    }
}

impl BinArrayPair {
    pub fn merge(bin_array_lower: BinArray, bin_array_upper: BinArray) -> anyhow::Result<Self> {
        if bin_array_upper.index != bin_array_lower.index + 1 {
            return Err(anyhow::format_err!("inArrayIndexMismatch"));
        }
        Ok(Self {
            bin_array_lower,
            bin_array_upper,
        })
    }

    pub fn get_bin(&self, bin_id: u32) -> anyhow::Result<&Bin> {
        self.bin_array_lower
            .get_bin(bin_id)
            .or_else(|_| self.bin_array_upper.get_bin(bin_id))
    }

    // pub fn get_bin_mut(&mut self, bin_id: u32) -> anyhow::Result<&mut Bin> {
    //     self.bin_array_lower
    //         .get_bin_mut(bin_id)
    //         .or_else(|_| self.bin_array_upper.get_bin_mut(bin_id))
    // }
}

pub const BIN_STEP_CONFIG_ACCOUNT_DISCM: [u8; 8] = [44, 12, 82, 45, 127, 124, 191, 199];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinStepConfig {
    pub bump: u8,
    pub liquidity_book_config: Pubkey,
    pub bin_step: u8,
    pub status: ConfigStatus,
    pub availability: ConfigAvailability,
    pub fee_parameters: StaticFeeParameters,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinStepConfigAccount(pub BinStepConfig);
impl BinStepConfigAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != BIN_STEP_CONFIG_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {BIN_STEP_CONFIG_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(BinStepConfig::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&BIN_STEP_CONFIG_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

pub const LIQUIDITY_BOOK_CONFIG_ACCOUNT_DISCM: [u8; 8] = [173, 36, 130, 129, 45, 178, 44, 86];

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiquidityBookConfig {
    pub preset_authority: Pubkey,
    pub pending_preset_authority: Option<Pubkey>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiquidityBookConfigAccount(pub LiquidityBookConfig);
impl LiquidityBookConfigAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != LIQUIDITY_BOOK_CONFIG_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {LIQUIDITY_BOOK_CONFIG_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(LiquidityBookConfig::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&LIQUIDITY_BOOK_CONFIG_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

pub const PAIR_ACCOUNT_DISCM: [u8; 8] = [85, 72, 49, 176, 182, 228, 141, 82];

#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pair {
    pub bump: [u8; 1],
    pub liquidity_book_config: Pubkey,
    pub bin_step: u8,
    pub bin_step_seed: [u8; 1],
    pub token_mint_x: Pubkey,
    pub token_mint_y: Pubkey,
    pub static_fee_parameters: StaticFeeParameters,
    pub active_id: u32,
    pub dynamic_fee_parameters: DynamicFeeParameters,
    pub protocol_fees_x: u64,
    pub protocol_fees_y: u64,
    pub hook: Option<Pubkey>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PairAccount(pub Pair);

impl PairAccount {
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        let mut reader = &buf[8..];
        Ok(Self(Pair::deserialize(&mut reader)?))
    }
}

// impl PairAccount {
//     pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
//         use std::io::Read;
//         let mut reader = buf;
//         let mut maybe_discm = [0u8; 8];
//         reader.read_exact(&mut maybe_discm)?;
//         if maybe_discm != PAIR_ACCOUNT_DISCM {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!(
//                     "discm does not match. Expected: {:?}. Received: {:?}",
//                     PAIR_ACCOUNT_DISCM, maybe_discm
//                 ),
//             ));
//         }
//         Ok(Self(Pair::deserialize(&mut reader)?))
//     }
//     pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
//         writer.write_all(&PAIR_ACCOUNT_DISCM)?;
//         self.0.serialize(&mut writer)
//     }
//     pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
//         let mut data = Vec::new();
//         self.serialize(&mut data)?;
//         Ok(data)
//     }
// }

pub struct QuoteInput<'a> {
    //pub bin_arrays: &'a BTreeMap<u32, (Pubkey, BinArray)>,
    pub bin_array_pair: &'a BinArrayPair,
}

impl Pool<QuoteInput<'_>> for Pair {
    // https://github.com/saros-xyz/saros-dlmm-sdk-rs/blob/main/saros-sdk/src/math/swap_manager.rs

    fn quote(
        &self,
        a_to_b: bool,
        exact_in: bool,
        amount_in: u64,
        input: &QuoteInput<'_>,
    ) -> anyhow::Result<utils::quote::QuoteResult> {
        let index = self.active_id / BIN_ARRAY_SIZE;
        if exact_in {
            let res = get_swap_result(self, input.bin_array_pair, amount_in, a_to_b, SwapMode::ExactIn);
            if let Ok((amount_out, amount_in_left)) = res {
                if a_to_b {
                    Ok(utils::quote::QuoteResult {
                        amount_a_in: amount_in,
                        amount_b_in: 0,
                        amount_a_out: amount_in_left,
                        amount_b_out: amount_out,
                        indices: Some(vec![index as i32, (index as i32) + 1]),
                        compute_units: 100_000,
                    })
                } else {
                    Ok(utils::quote::QuoteResult {
                        amount_a_in: 0,
                        amount_b_in: amount_in,
                        amount_a_out: amount_out,
                        amount_b_out: amount_in_left,
                        indices: Some(vec![index as i32, (index as i32) + 1]),
                        compute_units: 100_000,
                    })
                }
            } else {
                Err(anyhow::format_err!("SarosDlmm::QuoteErr"))
            }
        } else {
            Err(anyhow::format_err!("SarosDlmm::ExactOutNotImplemented"))
        }
    }

    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.token_mint_x, &self.token_mint_y]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.token_mint_x, self.token_mint_y]
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        &self.token_mint_x
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        &self.token_mint_y
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        a_to_b: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        SwapIxData(SwapIxArgs {
            amount,
            other_amount_threshold: amount_threshold,
            swap_for_y: a_to_b,
            swap_type: if exact_in {
                SwapType::ExactInput
            } else {
                SwapType::ExactOutput
            },
        })
        .try_to_vec()
        .context("SarosDlmm::get_swap_data_ix")
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        _optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<solana_sdk::instruction::AccountMeta>> {
        if let Some(optional_accounts) = optional_accounts {
            if optional_accounts.len() != 2 {
                return Err(anyhow::format_err!(
                    "SarosDlmm::get_swap_keys_ix: optional_accounts not supported"
                ));
            }
            let keys = SwapKeys {
                pair: pool_pubkey,
                token_mint_x: self.token_mint_x,
                token_mint_y: self.token_mint_y,
                bin_array_lower: optional_accounts[0],
                bin_array_upper: optional_accounts[1],
                token_vault_x: if allowed_token2022.contains(&self.token_mint_x) {
                    get_associated_token_address_with_program_id(&pool_pubkey, &self.token_mint_x, &spl_token_2022::ID)
                } else {
                    get_associated_token_address(&pool_pubkey, &self.token_mint_x)
                },
                token_vault_y: if allowed_token2022.contains(&self.token_mint_y) {
                    get_associated_token_address_with_program_id(&pool_pubkey, &self.token_mint_y, &spl_token_2022::ID)
                } else {
                    get_associated_token_address(&pool_pubkey, &self.token_mint_y)
                },
                user_vault_x: if a_to_b {
                    source_token_account
                } else {
                    destination_token_account
                },
                user_vault_y: if a_to_b {
                    destination_token_account
                } else {
                    source_token_account
                },
                user: signer,
                token_program_x: if allowed_token2022.contains(&self.token_mint_x) {
                    spl_token_2022::ID
                } else {
                    spl_token::ID
                },
                token_program_y: if allowed_token2022.contains(&self.token_mint_y) {
                    spl_token_2022::ID
                } else {
                    spl_token::ID
                },
                memo_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                event_authority: pubkey!("AQjz6RZK93SLjxfDGKL9nCYQNSjEbQSdETxwR63jXV8m"),
                program: super::ID,
            };
            Ok(vec![
                AccountMeta {
                    pubkey: keys.pair,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.token_mint_x,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.token_mint_y,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.bin_array_lower,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.bin_array_upper,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.token_vault_x,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.token_vault_y,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.user_vault_x,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.user_vault_y,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: keys.user,
                    is_signer: true,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.token_program_x,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.token_program_y,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.memo_program,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.event_authority,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: keys.program,
                    is_signer: false,
                    is_writable: false,
                },
            ])
        } else {
            Err(anyhow::format_err!(
                "SarosDlmm::get_swap_keys_ix: optional_accounts not supported"
            ))
        }
    }
}

// impl Pair {
//     pub fn is_token_2022(&self) -> bool {
//         self.token_0program == SPL_TOKEN_2022_ID && self.token_1program == SPL_TOKEN_2022_ID
//     }
// }

pub const POSITION_ACCOUNT_DISCM: [u8; 8] = [170, 188, 143, 228, 122, 64, 247, 208];

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    pub pair: Pubkey,
    pub position_mint: Pubkey,
    pub liquidity_shares: [u128; 64],
    pub lower_bin_id: u32,
    pub upper_bin_id: u32,
    pub space: [u8; 8],
}
#[derive(Clone, Debug, PartialEq)]
pub struct PositionAccount(pub Position);
impl PositionAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != POSITION_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {POSITION_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(Position::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&POSITION_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

pub const QUOTE_ASSET_BADGE_ACCOUNT_DISCM: [u8; 8] = [183, 124, 99, 219, 110, 119, 157, 221];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuoteAssetBadge {
    pub bump: u8,
    pub status: QuoteAssetBadgeStatus,
}
#[derive(Clone, Debug, PartialEq)]
pub struct QuoteAssetBadgeAccount(pub QuoteAssetBadge);
impl QuoteAssetBadgeAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != QUOTE_ASSET_BADGE_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {QUOTE_ASSET_BADGE_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(QuoteAssetBadge::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&QUOTE_ASSET_BADGE_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
