use ahash::AHashSet;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::str::FromStr;

use crate::quote::QuoteResult;
use crate::types::MintPair;

const WSOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub fn sort_mints(pubkey_a: Pubkey, pubkey_b: Pubkey) -> MintPair {
    if pubkey_b == WSOL {
        [pubkey_b, pubkey_a]
    } else if pubkey_a == WSOL {
        [pubkey_a, pubkey_b]
    } else {
        let mut mints = [pubkey_a, pubkey_b];
        mints.sort();
        mints
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum PoolType {
    RaydiumClmmV3,
    #[default]
    RaydiumAmmV4,
    Orca,
    MeteoraDlmm,
    PumpAmm,
    StabbleStableSwap,
    StabbleWeightedSwap,
    //Lifinity,
    RaydiumCpmm,
    OrcaSwapV2,
    SarosAmm,
    //GooseGamma,
    MeteoraDammV2,
    FusionAmm,
    SarosDlmm,
}

impl PoolType {
    #[inline]
    pub fn to_offset(&self) -> u16 {
        match &self {
            PoolType::RaydiumClmmV3 => 8,
            PoolType::RaydiumAmmV4 => 1,
            PoolType::Orca => 8,
            PoolType::MeteoraDlmm => 8,
            PoolType::PumpAmm => 8,
            PoolType::StabbleStableSwap => 9,
            PoolType::StabbleWeightedSwap => 9,
            //PoolType::Lifinity => 8,
            PoolType::RaydiumCpmm => 8,
            PoolType::OrcaSwapV2 => 1,
            PoolType::SarosAmm => 1,
            //PoolType::GooseGamma => 8,
            PoolType::MeteoraDammV2 => 8,
            PoolType::FusionAmm => 8, // TODO: Fusion for get_swap_ix
            PoolType::SarosDlmm => 8, // TODO: Saros for get_swap_ix
        }
    }
}

impl Debug for PoolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            PoolType::RaydiumClmmV3 => write!(f, "raydium_clmm_v3"),
            PoolType::RaydiumAmmV4 => write!(f, "raydium_amm_v4"),
            PoolType::Orca => write!(f, "orca_whirlpool"),
            PoolType::MeteoraDlmm => write!(f, "meteora_dlmm"),
            PoolType::PumpAmm => write!(f, "pump_amm"),
            PoolType::StabbleStableSwap => write!(f, "stabble_stable_swap"),
            PoolType::StabbleWeightedSwap => write!(f, "stabble_weighted_swap"),
            //PoolType::Lifinity => write!(f, "lifinity_v2"),
            PoolType::RaydiumCpmm => write!(f, "raydium_cpmm"),
            PoolType::OrcaSwapV2 => write!(f, "orca_swap_v2"),
            PoolType::SarosAmm => write!(f, "saros_amm"),
            //PoolType::GooseGamma => write!(f, "goose_gamma"),
            PoolType::MeteoraDammV2 => write!(f, "meteora_damm_v2"),
            PoolType::FusionAmm => write!(f, "fusion_amm"),
            PoolType::SarosDlmm => write!(f, "saros_dlmm"),
        }
    }
}

impl Display for PoolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            PoolType::RaydiumClmmV3 => write!(f, "raydium_clmm_v3"),
            PoolType::RaydiumAmmV4 => write!(f, "raydium_amm_v4"),
            PoolType::Orca => write!(f, "orca_whirlpool"),
            PoolType::MeteoraDlmm => write!(f, "meteora_dlmm"),
            PoolType::PumpAmm => write!(f, "pump_amm"),
            PoolType::StabbleStableSwap => write!(f, "stabble_stable_swap"),
            PoolType::StabbleWeightedSwap => write!(f, "stabble_weighted_swap"),
            //PoolType::Lifinity => write!(f, "lifinity_v2"),
            PoolType::RaydiumCpmm => write!(f, "raydium_cpmm"),
            PoolType::OrcaSwapV2 => write!(f, "orca_swap_v2"),
            PoolType::SarosAmm => write!(f, "saros_amm"),
            //PoolType::GooseGamma => write!(f, "goose_gamma"),
            PoolType::MeteoraDammV2 => write!(f, "meteora_damm_v2"),
            PoolType::FusionAmm => write!(f, "fusion_amm"),
            PoolType::SarosDlmm => write!(f, "saros_dlmm"),
        }
    }
}

impl FromStr for PoolType {
    type Err = String;

    fn from_str(input: &str) -> Result<PoolType, Self::Err> {
        match input.to_lowercase().as_str() {
            "raydium_clmm_v3" => Ok(PoolType::RaydiumClmmV3),
            "raydium_amm_v4" => Ok(PoolType::RaydiumAmmV4),
            "orca_whirlpool" => Ok(PoolType::Orca),
            "meteora_dlmm" => Ok(PoolType::MeteoraDlmm),
            "pump_amm" => Ok(PoolType::PumpAmm),
            "stabble_stable_swap" => Ok(PoolType::StabbleStableSwap),
            "stabble_weighted_swap" => Ok(PoolType::StabbleWeightedSwap),
            //"lifinity_v2" => Ok(PoolType::Lifinity),
            "raydium_cpmm" => Ok(PoolType::RaydiumCpmm),
            "orca_swap_v2" => Ok(PoolType::OrcaSwapV2),
            "saros_amm" => Ok(PoolType::SarosAmm),
            //"goose_gamma" => Ok(PoolType::GooseGamma),
            "meteora_damm_v2" => Ok(PoolType::MeteoraDammV2),
            "fusion_amm" => Ok(PoolType::FusionAmm),
            "saros_dlmm" => Ok(PoolType::SarosDlmm),
            _ => Err(format!("Unknown pool type: {input}")),
        }
    }
}

pub trait Pool<T> {
    fn get_count_mint_pairs(&self) -> u64 {
        1
    }
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        vec![self.get_sorted_mints_as_array()]
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount_in: u64, input: &T) -> anyhow::Result<QuoteResult>;

    fn get_keys(&self) -> Vec<&'_ Pubkey>;

    fn copy_keys(&self) -> Vec<Pubkey>;

    fn get_a_mint(&self) -> &'_ Pubkey;

    fn get_b_mint(&self) -> &'_ Pubkey;

    fn get_a_b_mints(&self) -> (&'_ Pubkey, &'_ Pubkey) {
        (self.get_a_mint(), self.get_b_mint())
    }

    fn get_sorted_mints(&self) -> Vec<&'_ Pubkey> {
        let mut mints = vec![self.get_a_mint(), self.get_b_mint()];
        if mints[1] == &WSOL {
            (mints[0], mints[1]) = (mints[1], mints[0]);
        }
        if mints[0] != &WSOL {
            mints.sort()
        }
        mints
    }

    fn get_sorted_mints_as_array(&self) -> MintPair {
        sort_mints(*self.get_a_mint(), *self.get_b_mint())
    }

    #[inline(always)]
    fn has_wsol(&self) -> bool {
        self.get_a_mint() == &WSOL || self.get_b_mint() == &WSOL
    }

    fn get_sorted_mints_as_str(&self) -> String {
        let mints = self
            .get_sorted_mints()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        mints.join("")
    }

    fn update_new_tokens(&self, token_mints: &mut HashSet<Pubkey>) -> Option<Pubkey> {
        let new_a = token_mints.insert(*self.get_a_mint());
        let new_b = token_mints.insert(*self.get_b_mint());
        if new_a || new_b {
            return Some(if new_a { *self.get_a_mint() } else { *self.get_b_mint() });
        }
        None
    }

    fn get_program_id(&self) -> Pubkey;

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        a_to_b: bool,
        exact_in: bool,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>>;

    #[allow(clippy::too_many_arguments)]
    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
        allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<AccountMeta>>;
}
