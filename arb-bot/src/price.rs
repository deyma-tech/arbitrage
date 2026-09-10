#![allow(dead_code)]
use ahash::{AHashSet, HashMap, HashSet};
use anyhow::{anyhow, Result};
use anyhow::{format_err, Context};
use arb_core::arbitrage::Opportunity;
use arb_core::arbitrage::{find_optimum_v3, find_optimum_v4};
use arb_core::calculator::CalculatorEnum;
use config::CONFIG as cfg;
use log::debug;
use num_traits::{ToPrimitive, Zero};
use once_cell::sync::Lazy;
use ordered_float::{FloatCore, OrderedFloat};
use rand::Rng;
use solana_sdk::pubkey::Pubkey;
use std::cmp::{Ordering, PartialEq};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;
use utils::constants::WSOL;
use utils::now;
use utils::pool::{sort_mints, PoolType};
use utils::types::MintPair;
// use utils::volumes::{generate, generate_decreasing};

pub static VOLUMES: Lazy<&'static [u64]> = Lazy::new(|| {
    let volumes = cfg.arbitrage.volumes.clone();
    if volumes.is_empty() {
        panic!("No volumes configured");
    }
    // Box<[_]> is leaked only once, but the slice is already &'static
    Box::leak(volumes.into_boxed_slice())
});

#[derive(Debug, Clone)]
pub struct KeyedPrice {
    pub pubkey: Pubkey,
    pub pool_type: PoolType,
    pub price: OrderedFloat<f64>,
    // For all types except Lifinity, use u64::MAX because the data remains valid
    pub slot: u64,
}

impl PartialEq<Self> for KeyedPrice {
    fn eq(&self, other: &Self) -> bool {
        self.pubkey.eq(&other.pubkey)
    }
}

impl Eq for KeyedPrice {}

impl PartialOrd<Self> for KeyedPrice {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.price.partial_cmp(&other.price)
    }
}

impl Ord for KeyedPrice {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.cmp(&other.price)
    }
}

impl Hash for KeyedPrice {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pubkey.hash(state);
    }
}

type PriceSet = BTreeSet<KeyedPrice>;
type PoolPubkeyAndAmount = (Pubkey, u64);

// v1
type VolumeToPriceSet = BTreeMap<u64, PriceSet>;
type MintPairToVolumeToPriceSet = HashMap<MintPair, BTreeMap<u64, PriceSet>>;

type VolumeToPoolPubkeyAndAmount = BTreeMap<u64, PoolPubkeyAndAmount>;
type MintToVolumeToPoolPubkeyAndAmount = HashMap<Pubkey, VolumeToPoolPubkeyAndAmount>;

// v2
type MintToPoolPubkeyAndAmount = HashMap<Pubkey, PoolPubkeyAndAmount>;
type MintPairPriceSet = HashMap<MintPair, PriceSet>;

type Pairs = HashMap<Pubkey, HashSet<Pubkey>>;

#[derive(Debug)]
pub struct Opportunities {
    // <Opportunity, keys volume>
    pub opportunities: Vec<(Opportunity, Vec<Pubkey>, u64)>,
    // <index, (diff, opportunity, volume)>
    pub index_to_opportunity: BTreeMap<usize, (u64, Opportunity, u64)>,
    pub size: usize,
}

impl Opportunities {
    pub fn new(size: usize) -> Self {
        Opportunities {
            opportunities: vec![],
            index_to_opportunity: BTreeMap::new(),
            size,
        }
    }

    pub fn difference(&self, index: usize) -> OrderedFloat<f64> {
        if let Some((diff, _opportunity, _volume)) = self.index_to_opportunity.get(&index) {
            return OrderedFloat((*diff) as f64);
        }
        OrderedFloat(0_f64)
    }

    pub fn insert(&mut self, index: usize, opportunity: Opportunity, volume: u64) {
        self.index_to_opportunity
            .insert(index, (opportunity.diff, opportunity, volume));
    }

    pub fn convert_to_opportunities_with_volume(self, filter: u64) -> Vec<(Opportunity, u64)> {
        self.index_to_opportunity
            .into_iter()
            .map(|(_index, (_diff, opp, volume))| (opp, volume))
            .filter(|(opp, volume)| volume < &VOLUMES[0] && opp.diff > filter)
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Default)]
pub struct PriceMap {
    pub a_to_b: MintPairToVolumeToPriceSet,
    pub b_to_a: MintPairToVolumeToPriceSet,
    pub mint_to_volume: MintToVolumeToPoolPubkeyAndAmount,
    pub pairs: Pairs,
    pub size: usize,
    //pub price_map_trigger_diff: OrderedFloat<f64>,
}

impl PriceMap {
    pub fn new(size: usize) -> Self {
        PriceMap {
            a_to_b: Default::default(),
            b_to_a: Default::default(),
            mint_to_volume: Default::default(),
            pairs: Default::default(),
            size,
            //price_map_trigger_diff: OrderedFloat(cfg.arbitrage.price_map_trigger_diff),
        }
    }

    pub fn process_mint_to_max(&mut self, volume: u64, amount_out: u64, mint: &Pubkey, pool_pubkey: &Pubkey) {
        if let Some(volume_to_max) = self.mint_to_volume.get_mut(mint) {
            if let Some((old_pool_pubkey, old_amount_out)) = volume_to_max.get_mut(&volume) {
                if *old_amount_out < amount_out {
                    *old_pool_pubkey = *pool_pubkey;
                    *old_amount_out = amount_out;
                } else if old_pool_pubkey == pool_pubkey && *old_amount_out > amount_out {
                    *old_amount_out = amount_out;
                }
            } else {
                volume_to_max.insert(volume, (*pool_pubkey, amount_out));
            }
        } else {
            self.mint_to_volume
                .insert(*mint, BTreeMap::from_iter([(volume, (*pool_pubkey, amount_out))]));
        }
    }

    pub fn calculate_price(
        &mut self,
        calculator: &CalculatorEnum,
        a_to_b: bool,
        volume: u64,
        slot: u64,
        mint_pair: Option<MintPair>,
    ) -> anyhow::Result<()> {
        let sorted_mints = mint_pair.unwrap_or(calculator.get_sorted_mints_as_array());
        if a_to_b {
            let volume_in = if sorted_mints[0] != WSOL {
                self.mint_to_volume
                    .get(&sorted_mints[0])
                    .and_then(|map| map.get(&volume))
                    .map(|(_, amount_in)| *amount_in)
                    .unwrap_or(volume)
            } else {
                volume
            };
            let res = calculator.quote_a_to_b_price(volume_in, Some(sorted_mints));
            match res {
                Ok((price, amount_out)) => {
                    let price = OrderedFloat(price);
                    if price.is_finite() && price.is_sign_positive() {
                        let pool_type = calculator.get_pool_type();
                        // let slot_in_keyed_price = if pool_type == PoolType::Lifinity {
                        //     slot + 10
                        // } else {
                        //     u64::MAX
                        // };
                        let slot_in_keyed_price = u64::MAX;
                        let keyed_price = KeyedPrice {
                            pubkey: *calculator.get_pubkey(),
                            pool_type,
                            price,
                            slot: slot_in_keyed_price,
                        };
                        self.update_a_to_b(sorted_mints, volume, keyed_price, slot);
                        self.process_mint_to_max(volume, amount_out, &sorted_mints[1], calculator.get_pubkey());
                    } else {
                        self.remove_from_volume(
                            true,
                            &calculator.get_sorted_mints_as_array(),
                            calculator.get_pubkey(),
                            volume,
                        );
                    }
                    return Ok(());
                }
                Err(_) => {
                    self.remove_from_volume(
                        true,
                        &calculator.get_sorted_mints_as_array(),
                        calculator.get_pubkey(),
                        volume,
                    );
                    return Err(anyhow::anyhow!("Error calculating price"));
                }
            }
        } else if !a_to_b {
            let volume_in = self
                .mint_to_volume
                .get(&sorted_mints[1])
                .and_then(|map| map.get(&volume))
                .map(|(_, amount_in)| *amount_in)
                .unwrap_or(volume);

            let res = calculator.quote_b_to_a_price(volume_in, Some(sorted_mints));
            match res {
                Ok((price, amount_out)) => {
                    let price = OrderedFloat(price);
                    if price.is_finite() && price.is_sign_positive() {
                        let pool_type = calculator.get_pool_type();
                        // let slot_in_keyed_price = if pool_type == PoolType::Lifinity {
                        //     slot + 10
                        // } else {
                        //     u64::MAX
                        // };
                        let slot_in_keyed_price = u64::MAX;
                        let keyed_price = KeyedPrice {
                            pubkey: *calculator.get_pubkey(),
                            pool_type,
                            price,
                            slot: slot_in_keyed_price,
                        };
                        self.update_b_to_a(sorted_mints, volume, keyed_price, slot);
                        if sorted_mints[0] != WSOL {
                            self.process_mint_to_max(volume, amount_out, &sorted_mints[0], calculator.get_pubkey());
                        }
                    } else {
                        self.remove_from_volume(false, &sorted_mints, calculator.get_pubkey(), volume);
                    }
                    return Ok(());
                }
                Err(_) => {
                    self.remove_from_volume(false, &sorted_mints, calculator.get_pubkey(), volume);
                    return Err(anyhow::anyhow!("Error calculating price"));
                }
            }
        }
        Ok(())
    }

    /// Calculates volume based on the previous volume for a given original volume and token mint. Returns 0 if no volume is found.
    fn find_previous_volume(&self, original_volume: u64, token_mint: &Pubkey) -> u64 {
        let lower_volumes = VOLUMES
            .iter()
            .filter(|volume| volume < &&original_volume)
            .collect::<Vec<_>>();
        let token_volumes = self.mint_to_volume.get(token_mint);
        if let Some(token_volumes) = token_volumes {
            for lower_volume in lower_volumes.into_iter() {
                if lower_volume == &0 {
                    debug!("Volume is 0");
                    continue;
                }
                if let Some((_, amount_in)) = token_volumes.get(lower_volume) {
                    let volume = (original_volume / lower_volume) * amount_in;
                    return volume;
                }
            }
        }
        0
    }

    pub fn calculate_price_v2(
        &mut self,
        calculator: &CalculatorEnum,
        a_to_b: bool,
        mut volume: u64,
        slot: u64,
        mint_pair: Option<MintPair>,
    ) -> anyhow::Result<bool> {
        let mut update_price = false;
        let original_volume = volume;
        let sorted_mints = mint_pair.unwrap_or(calculator.get_sorted_mints_as_array());
        if sorted_mints.contains(&WSOL) && !a_to_b {
            volume = self
                .mint_to_volume
                .get(&sorted_mints[1])
                .and_then(|map| map.get(&volume))
                .map(|(_, amount_in)| *amount_in)
                .unwrap_or(self.find_previous_volume(volume, &sorted_mints[1]));
        } else if !sorted_mints.contains(&WSOL) {
            let index = if a_to_b { 0 } else { 1 };
            volume = self
                .mint_to_volume
                .get(&sorted_mints[index])
                .and_then(|map| map.get(&volume))
                .map(|(_, amount_in)| *amount_in)
                .unwrap_or(self.find_previous_volume(volume, &sorted_mints[index]));
        }
        if volume == 0 {
            self.remove_from_volume(a_to_b, &sorted_mints, calculator.get_pubkey(), original_volume);
            return Err(anyhow::anyhow!("NoVolumeFound"));
        }
        let res = calculator.quote_price(a_to_b, volume, Some(sorted_mints));
        match res {
            Ok((price, amount_out)) => {
                let price = OrderedFloat(price);
                if !price.is_zero() && price.is_finite() && price.is_sign_positive() {
                    let pool_type = calculator.get_pool_type();
                    // let slot_in_keyed_price = if pool_type == PoolType::Lifinity {
                    //     slot + 10
                    // } else {
                    //     u64::MAX
                    // };
                    let slot_in_keyed_price = u64::MAX;
                    let keyed_price = KeyedPrice {
                        pubkey: *calculator.get_pubkey(),
                        pool_type,
                        price,
                        slot: slot_in_keyed_price,
                    };
                    let update = self.update(a_to_b, sorted_mints, original_volume, keyed_price, slot);
                    if update {
                        update_price = true;
                    }
                    if a_to_b {
                        self.process_mint_to_max(
                            original_volume,
                            amount_out,
                            &sorted_mints[1],
                            calculator.get_pubkey(),
                        );
                    }
                } else {
                    self.remove_from_volume(
                        a_to_b,
                        &calculator.get_sorted_mints_as_array(),
                        calculator.get_pubkey(),
                        original_volume,
                    );
                }
                Ok(update_price)
            }
            Err(_) => {
                self.remove_from_volume(
                    true,
                    &calculator.get_sorted_mints_as_array(),
                    calculator.get_pubkey(),
                    original_volume,
                );
                Err(anyhow::anyhow!("Error calculating price"))
            }
        }
    }

    pub fn iterate_volumes_v2(&mut self, calculator: &CalculatorEnum, slot: u64) -> bool {
        let mut update_price = false;
        // special case - where orders on raydium amm goes via orderbook ... - ignore and remove ...
        if calculator.get_pool_type() == PoolType::RaydiumAmmV4 {
            if let CalculatorEnum::RaydiumAmmV4(data) = calculator {
                // remove data ...
                if data.amm_info.orderbook_permission() && !data.amm_info.swap_permission() {
                    let mint_pair = calculator.get_sorted_mints_as_array();
                    let pool_pubkey = calculator.get_pubkey();
                    for volume in VOLUMES[1..VOLUMES.len() - 1].iter() {
                        self.remove_from_volume(true, &mint_pair, pool_pubkey, *volume);
                        self.remove_from_volume(false, &mint_pair, pool_pubkey, *volume);
                    }
                }
            }
        }
        let mint_pairs = calculator.get_all_sorted_mints_as_array();
        for mint_pair in mint_pairs.iter() {
            if mint_pair.contains(&WSOL) {
                let res = self.calculate_price_v2(calculator, true, cfg.arbitrage.min_wsol, slot, Some(*mint_pair));
                if res.is_ok() {
                    self.update_pairs(mint_pair);
                    for volume in VOLUMES.iter().rev() {
                        let res = self.calculate_price_v2(calculator, true, *volume, slot, Some(*mint_pair));
                        match res {
                            Ok(update) => {
                                if update {
                                    update_price = true;
                                }
                            }
                            Err(_) => {
                                self.remove_from_volume(true, mint_pair, calculator.get_pubkey(), *volume);
                                break;
                            }
                        }
                    }
                }
                let res = self.calculate_price_v2(calculator, false, cfg.arbitrage.min_wsol, slot, Some(*mint_pair));
                if res.is_ok() {
                    self.update_pairs(mint_pair);
                    for volume in VOLUMES.iter().rev() {
                        let res = self.calculate_price_v2(calculator, false, *volume, slot, Some(*mint_pair));
                        match res {
                            Ok(update) => {
                                if update {
                                    update_price = true;
                                }
                            }
                            Err(_) => {
                                self.remove_from_volume(false, mint_pair, calculator.get_pubkey(), *volume);
                                break;
                            }
                        }
                    }
                }
            } else {
                let res = self.calculate_price_v2(calculator, true, cfg.arbitrage.min_wsol, slot, Some(*mint_pair));
                if res.is_ok() {
                    self.update_pairs(mint_pair);
                    for volume in VOLUMES.iter().rev() {
                        let res = self.calculate_price_v2(calculator, true, *volume, slot, Some(*mint_pair));
                        match res {
                            Ok(update) => {
                                if update {
                                    update_price = true;
                                }
                            }
                            Err(_) => {
                                self.remove_from_volume(true, mint_pair, calculator.get_pubkey(), *volume);
                                break;
                            }
                        }
                    }
                }
                let res = self.calculate_price_v2(calculator, false, cfg.arbitrage.min_wsol, slot, Some(*mint_pair));
                if res.is_ok() {
                    self.update_pairs(mint_pair);
                    for volume in VOLUMES.iter().rev() {
                        let res = self.calculate_price_v2(calculator, false, *volume, slot, Some(*mint_pair));
                        match res {
                            Ok(update) => {
                                if update {
                                    update_price = true;
                                }
                            }
                            Err(_) => {
                                self.remove_from_volume(true, mint_pair, calculator.get_pubkey(), *volume);
                                break;
                            }
                        }
                    }
                }
            }
        }
        update_price
    }

    pub fn remove_from_all_volume(&mut self, mint_pairs: &[MintPair], pool_pubkeys: &[Pubkey]) {
        for mint_pair in mint_pairs.iter() {
            let price_map = self.a_to_b.get_mut(mint_pair);
            if let Some(price_map) = price_map {
                for volume in VOLUMES.iter() {
                    let price_set = price_map.get_mut(volume);
                    if let Some(price_set) = price_set {
                        let to_remove_by_slot = price_set
                            .iter()
                            .filter(|e| pool_pubkeys.contains(&e.pubkey))
                            .cloned()
                            .collect::<Vec<_>>();
                        // println!("to_remove_by_slot: {:?}", to_remove_by_slot);
                        for to_remove in to_remove_by_slot {
                            price_set.remove(&to_remove);
                        }
                    }
                }
            }
            let price_map = self.b_to_a.get_mut(mint_pair);
            if let Some(price_map) = price_map {
                for volume in VOLUMES.iter() {
                    let price_set = price_map.get_mut(volume);
                    if let Some(price_set) = price_set {
                        let to_remove_by_slot = price_set
                            .iter()
                            .filter(|e| pool_pubkeys.contains(&e.pubkey))
                            .cloned()
                            .collect::<Vec<_>>();
                        // println!("to_remove_by_slot: {:?}", to_remove_by_slot);
                        for to_remove in to_remove_by_slot {
                            price_set.remove(&to_remove);
                        }
                    }
                }
            }
        }
    }

    pub fn iterate_volumes(&mut self, calculator: &CalculatorEnum, slot: u64) {
        // special case - where orders on raydium amm goes via orderbook ... - ignore and remove ...
        if calculator.get_pool_type() == PoolType::RaydiumAmmV4 {
            if let CalculatorEnum::RaydiumAmmV4(data) = calculator {
                // remove data ...
                if data.amm_info.orderbook_permission() && !data.amm_info.swap_permission() {
                    let mint_pair = calculator.get_sorted_mints_as_array();
                    let pool_pubkey = calculator.get_pubkey();
                    for volume in VOLUMES.iter() {
                        self.remove_from_volume(true, &mint_pair, pool_pubkey, *volume);
                        self.remove_from_volume(false, &mint_pair, pool_pubkey, *volume);
                    }
                }
            }
        }
        let mint_pairs = calculator.get_all_sorted_mints_as_array();
        for mint in mint_pairs.iter() {
            for volume in VOLUMES.iter().rev() {
                let res = self.calculate_price(calculator, true, *volume, slot, Some(*mint));
                match res {
                    Ok(_) => {}
                    Err(_) => {
                        self.remove_from_volume(true, mint, calculator.get_pubkey(), *volume);
                        break;
                    }
                }
            }
        }
        for mint in mint_pairs.iter() {
            for volume in VOLUMES.iter().rev() {
                let res = self.calculate_price(calculator, false, *volume, slot, Some(*mint));
                match res {
                    Ok(_) => {}
                    Err(_) => {
                        self.remove_from_volume(false, mint, calculator.get_pubkey(), *volume);
                        break;
                    }
                }
            }
        }
    }

    pub fn insert_or_replace(set: &mut BTreeSet<KeyedPrice>, new_entry: KeyedPrice, _slot: u64) {
        // let to_remove_by_slot = set.iter().filter(|e| e.slot < slot).cloned().collect::<Vec<_>>();
        // // println!("to_remove_by_slot: {:?}", to_remove_by_slot);
        // for to_remove in to_remove_by_slot {
        //     set.remove(&to_remove);
        // }
        if let Some(existing) = set.iter().find(|e| e.pubkey == new_entry.pubkey) {
            let to_remove = existing.clone();
            set.remove(&to_remove);
        }
        set.insert(new_entry);
    }

    pub fn insert_v2(set: &mut BTreeSet<KeyedPrice>, new_entry: KeyedPrice, _slot: u64) {
        // let to_remove_by_slot = set.iter().filter(|e| e.slot < slot).cloned().collect::<Vec<_>>();
        // // println!("to_remove_by_slot: {:?}", to_remove_by_slot);
        // for to_remove in to_remove_by_slot {
        //     set.remove(&to_remove);
        // }
        // if let Some(existing) = set.iter().find(|e| e.pubkey == new_entry.pubkey) {
        //     let to_remove = existing.clone();
        //     set.remove(&to_remove);
        // }
        set.insert(new_entry);
    }

    pub fn remove_v2(set: &mut BTreeSet<KeyedPrice>, new_entry: &KeyedPrice, _slot: u64) {
        // let to_remove_by_slot = set.iter().filter(|e| e.slot < slot).cloned().collect::<Vec<_>>();
        // // println!("to_remove_by_slot: {:?}", to_remove_by_slot);
        // for to_remove in to_remove_by_slot {
        //     set.remove(&to_remove);
        // }
        if let Some(existing) = set.iter().find(|e| e.pubkey == new_entry.pubkey) {
            let to_remove = existing.clone();
            set.remove(&to_remove);
        }
    }

    // TODO if new price is worst with same key as is on the end - old values will be not pushed out
    pub fn update_a_to_b(&mut self, mints: MintPair, volume: u64, keyed_price: KeyedPrice, slot: u64) {
        // && !mints.contains(&USDC) && !mints.contains(&USDT)
        if !mints.contains(&WSOL) {
            if !self.pairs.contains_key(&mints[0]) {
                self.pairs.insert(mints[0], HashSet::from_iter([mints[1]]));
            } else {
                let existing = self.pairs.get_mut(&mints[0]);
                match existing {
                    None => {
                        self.pairs.insert(mints[0], HashSet::from_iter([mints[1]]));
                    }
                    Some(existing) => {
                        existing.insert(mints[1]);
                    }
                }
            }
            if !self.pairs.contains_key(&mints[1]) {
                self.pairs.insert(mints[1], HashSet::from_iter([mints[0]]));
            } else {
                let existing = self.pairs.get_mut(&mints[1]);
                match existing {
                    None => {
                        self.pairs.insert(mints[1], HashSet::from_iter([mints[0]]));
                    }
                    Some(existing) => {
                        existing.insert(mints[0]);
                    }
                }
            }
        }
        let volume_to_set = self.a_to_b.get_mut(&mints);
        if let Some(volume_to_set) = volume_to_set {
            if let Some(price_set) = volume_to_set.get_mut(&volume) {
                Self::insert_or_replace(price_set, keyed_price, slot);
                while price_set.len() > self.size {
                    price_set.pop_first();
                }
            } else {
                let mut price_set = BTreeSet::new();
                price_set.insert(keyed_price);
                while price_set.len() > self.size {
                    price_set.pop_first();
                }
                volume_to_set.insert(volume, price_set);
            }
        } else {
            // does not exists
            let mut price_set = BTreeSet::new();
            price_set.insert(keyed_price);
            let mut volume_to_set = BTreeMap::new();
            volume_to_set.insert(volume, price_set);
            self.a_to_b.insert(mints, volume_to_set);
        }
    }

    pub fn update_b_to_a(&mut self, mints: MintPair, volume: u64, keyed_price: KeyedPrice, slot: u64) {
        let volume_to_set = self.b_to_a.get_mut(&mints);
        if let Some(volume_to_set) = volume_to_set {
            if let Some(price_set) = volume_to_set.get_mut(&volume) {
                Self::insert_or_replace(price_set, keyed_price, slot);
                while price_set.len() > self.size {
                    price_set.pop_last();
                }
            } else {
                let mut price_set = BTreeSet::new();
                price_set.insert(keyed_price);
                while price_set.len() > self.size {
                    price_set.pop_last();
                }
                volume_to_set.insert(volume, price_set);
            }
        } else {
            let mut price_set = BTreeSet::new();
            price_set.insert(keyed_price);
            let mut volume_to_set = BTreeMap::new();
            volume_to_set.insert(volume, price_set);
            self.b_to_a.insert(mints, volume_to_set);
        }
    }

    pub fn update(&mut self, a_to_b: bool, mints: MintPair, volume: u64, keyed_price: KeyedPrice, slot: u64) -> bool {
        let mut update_price = false;
        let map = if a_to_b { &mut self.a_to_b } else { &mut self.b_to_a };
        let volume_to_set = map.get_mut(&mints);
        if let Some(volume_to_set) = volume_to_set {
            if let Some(price_set) = volume_to_set.get_mut(&volume) {
                Self::remove_v2(price_set, &keyed_price, slot);
                if price_set.is_empty() {
                    update_price = true;
                } else if a_to_b {
                    let maximum = price_set.last();
                    if let Some(kp) = maximum {
                        if keyed_price.price > kp.price {
                            // * self.price_map_trigger_diff {
                            update_price = true;
                        }
                    }
                } else {
                    let minimum = price_set.first();
                    if let Some(kp) = minimum {
                        if keyed_price.price < kp.price {
                            // / self.price_map_trigger_diff {
                            update_price = true;
                        }
                    }
                }
                Self::insert_v2(price_set, keyed_price, slot);
                while price_set.len() > self.size {
                    if a_to_b {
                        price_set.pop_first();
                    } else {
                        price_set.pop_last();
                    }
                }
            } else {
                update_price = true;
                let mut price_set = BTreeSet::new();
                price_set.insert(keyed_price);
                while price_set.len() > self.size {
                    if a_to_b {
                        price_set.pop_first();
                    } else {
                        price_set.pop_last();
                    }
                }
                volume_to_set.insert(volume, price_set);
            }
        } else {
            update_price = true;
            let mut price_set = BTreeSet::new();
            price_set.insert(keyed_price);
            let mut volume_to_set = BTreeMap::new();
            volume_to_set.insert(volume, price_set);
            map.insert(mints, volume_to_set);
        }
        update_price
    }

    fn update_pairs(&mut self, mint_pair: &MintPair) {
        if !mint_pair.contains(&WSOL) {
            if !self.pairs.contains_key(&mint_pair[0]) {
                self.pairs.insert(mint_pair[0], HashSet::from_iter([mint_pair[1]]));
            } else {
                let existing = self.pairs.get_mut(&mint_pair[0]);
                match existing {
                    None => {
                        self.pairs.insert(mint_pair[0], HashSet::from_iter([mint_pair[1]]));
                    }
                    Some(existing) => {
                        existing.insert(mint_pair[1]);
                    }
                }
            }
            if !self.pairs.contains_key(&mint_pair[1]) {
                self.pairs.insert(mint_pair[1], HashSet::from_iter([mint_pair[0]]));
            } else {
                let existing = self.pairs.get_mut(&mint_pair[1]);
                match existing {
                    None => {
                        self.pairs.insert(mint_pair[1], HashSet::from_iter([mint_pair[0]]));
                    }
                    Some(existing) => {
                        existing.insert(mint_pair[0]);
                    }
                }
            }
        }
    }

    fn remove_from_volume(&mut self, a_to_b: bool, mints: &MintPair, pool_pubkey: &Pubkey, volume_to_remove: u64) {
        let volumes_to_remove = VOLUMES.iter().filter(|&v| v >= &volume_to_remove);
        let price_map = if a_to_b {
            self.a_to_b.get_mut(mints)
        } else {
            self.b_to_a.get_mut(mints)
        };
        if let Some(price_map) = price_map {
            for volume in volumes_to_remove {
                let price_set = price_map.get_mut(volume);
                if let Some(price_set) = price_set {
                    let to_remove_by_slot = price_set
                        .iter()
                        .filter(|e| &e.pubkey == pool_pubkey)
                        .cloned()
                        .collect::<Vec<_>>();
                    // println!("to_remove_by_slot: {:?}", to_remove_by_slot);
                    for to_remove in to_remove_by_slot {
                        price_set.remove(&to_remove);
                    }
                }
            }
        }
    }

    // pub fn max(
    //     &mut self,
    //     mint_pair: &MintPair,
    //     volume: &u64,
    //     slot: u64,
    //     filter_out: Option<PoolType>,
    //     size: usize,
    // ) -> Vec<KeyedPrice> {
    //     let mut result = vec![];
    //     match self.a_to_b.get_mut(mint_pair) {
    //         None => {}
    //         Some(volume_map) => match volume_map.get_mut(volume) {
    //             None => {}
    //             Some(price_set) => {
    //                 while let Some(last) = price_set.pop_last() {
    //                     if last.slot < slot {
    //                         continue;
    //                     }
    //                     price_set.insert(last.clone());
    //                     if let Some(filter) = filter_out {
    //                         if last.pool_type != filter {
    //                             result.push(last);
    //                         }
    //                     } else {
    //                         result.push(last);
    //                     };
    //                     if result.len() >= size {
    //                         break;
    //                     }
    //                 }
    //             }
    //         },
    //     }
    //     result
    // }

    pub fn take_max(
        &mut self,
        mint_pair: &MintPair,
        volume: &u64,
        _slot: u64,
        _filter_out: Option<Vec<PoolType>>,
        index: usize,
    ) -> Option<KeyedPrice> {
        //let mut result = vec![];
        match self.a_to_b.get_mut(mint_pair) {
            None => {}
            Some(volume_map) => match volume_map.get_mut(volume) {
                None => {}
                Some(price_set) => {
                    // let to_be_removed = price_set
                    //     .iter()
                    //     .filter(|keyed_price| keyed_price.slot < slot)
                    //     .cloned()
                    //     .collect::<Vec<_>>();
                    // for kp in to_be_removed.into_iter() {
                    //     let kp = kp.clone();
                    //     let _ = price_set.remove(&kp);
                    // }
                    // if price_set.len() < index + 1 {
                    //     return None;
                    // }
                    // if let Some(filter) = filter_out {
                    //     let keyed_price = price_set
                    //         .iter()
                    //         .filter(|kp| !filter.contains(&kp.pool_type))
                    //         .rev()
                    //         .nth(index);
                    //     if let Some(keyed_price) = keyed_price {
                    //         return Some(keyed_price.clone());
                    //     }
                    // } else {
                    let keyed_price = price_set.iter().rev().nth(index);
                    if let Some(keyed_price) = keyed_price {
                        return Some(keyed_price.clone());
                    }
                    //}
                }
            },
        }
        //if index >= result.len() {
        None
        //}
        //Some(result.remove(index))
    }

    // pub fn min(
    //     &mut self,
    //     mint_pair: &MintPair,
    //     volume: &u64,
    //     slot: u64,
    //     filter_out: Option<PoolType>,
    //     size: usize,
    // ) -> Vec<KeyedPrice> {
    //     let mut result = vec![];
    //     match self.b_to_a.get_mut(mint_pair) {
    //         None => {}
    //         Some(volume_map) => match volume_map.get_mut(volume) {
    //             None => {}
    //             Some(price_set) => {
    //                 while let Some(first) = price_set.pop_first() {
    //                     if first.slot < slot {
    //                         continue;
    //                     }
    //                     price_set.insert(first.clone());
    //                     if let Some(filter) = filter_out {
    //                         if first.pool_type != filter {
    //                             result.push(first);
    //                         }
    //                     } else {
    //                         result.push(first);
    //                     };
    //                     if result.len() >= size {
    //                         break;
    //                     }
    //                 }
    //             }
    //         },
    //     }
    //     result
    // }

    pub fn take_min(
        &mut self,
        mint_pair: &MintPair,
        volume: &u64,
        _slot: u64,
        _filter_out: Option<Vec<PoolType>>,
        index: usize,
    ) -> Option<KeyedPrice> {
        //let mut result = vec![];
        match self.b_to_a.get_mut(mint_pair) {
            None => {}
            Some(volume_map) => match volume_map.get_mut(volume) {
                None => {}
                Some(price_set) => {
                    // let to_be_removed = price_set
                    //     .iter()
                    //     .filter(|keyed_price| keyed_price.slot < slot)
                    //     .cloned()
                    //     .collect::<Vec<_>>();
                    // for kp in to_be_removed.into_iter() {
                    //     let _ = price_set.remove(&kp);
                    // }
                    // if price_set.len() < index + 1 {
                    //     return None;
                    // }
                    // if let Some(filter) = filter_out {
                    //     let keyed_price = price_set.iter().filter(|kp| !filter.contains(&kp.pool_type)).nth(index);
                    //     if let Some(keyed_price) = keyed_price {
                    //         return Some(keyed_price.clone());
                    //     }
                    // } else {
                    let keyed_price = price_set.iter().nth(index);
                    if let Some(keyed_price) = keyed_price {
                        return Some(keyed_price.clone());
                    }
                    //}
                }
            },
        }
        //if index >= result.len() {
        None
        //}
        //Some(result.remove(index))
    }

    pub fn get_pairs(&self, mint: &Pubkey) -> Vec<MintPair> {
        let mut pairs = vec![];
        if let Some(mint_pairs) = self.pairs.get(mint) {
            for mint_pair in mint_pairs.iter() {
                let pair = sort_mints(*mint, *mint_pair);
                pairs.push(pair);
            }
        }
        pairs
    }

    pub fn find_arbitrage2(
        &mut self,
        mints: &MintPair,
        slot: u64,
        filter: f64,
        index: usize,
    ) -> Vec<(Opportunity, u64)> {
        if index >= self.size {
            return vec![];
        }
        let mut selection = Opportunities::new(self.size);
        let mut compare: Option<OrderedFloat<f64>> = None;
        for volume in VOLUMES[1..VOLUMES.len() - 1].iter().rev() {
            let keyed_price1 = match self.take_max(mints, volume, slot, None, index) {
                None => break,
                Some(keyed_price) => keyed_price,
            };
            let keyed_price2 = match self.take_min(mints, volume, slot, None, index) {
                None => break,
                Some(keyed_price) => keyed_price,
            };
            let diff = (keyed_price1.price - keyed_price2.price) * OrderedFloat(*volume as f64);
            // println!("diff: {} {} {}", diff, volume, index);
            if diff.is_sign_positive() && diff.is_finite() && diff > selection.difference(index) {
                selection.insert(
                    index,
                    Opportunity {
                        slot,
                        route: vec![
                            (
                                keyed_price1.pubkey,
                                keyed_price1.pool_type,
                                keyed_price1.price.to_le_bytes(),
                            ),
                            (
                                keyed_price2.pubkey,
                                keyed_price2.pool_type,
                                keyed_price2.price.to_le_bytes(),
                            ),
                        ],
                        diff: diff.0 as u64,
                        mint_pair_route: vec![*mints, *mints],
                        timestamp: now::as_millis(),
                    },
                    *volume,
                );
                compare = Some(diff);
            } else {
                // break if the difference is not increasing ...
                if let Some(compare_diff) = compare {
                    if diff < compare_diff {
                        break;
                    } else {
                        compare = Some(diff);
                    }
                } else {
                    compare = Some(diff);
                }
            }
        }
        // TODO get index ... and sent to channel ...
        selection.convert_to_opportunities_with_volume(filter as u64)
    }

    // pub fn find_arbitrage2_additional(&mut self, mints: &MintPair, slot: u64, filter: f64) -> Vec<(Opportunity, u64)> {
    //     let mut selection = Opportunities::new(self.size);
    //     for (index, index1) in [(0, 1), (1, 0)] {
    //         let mut compare: Option<OrderedFloat<f64>> = None;
    //         for volume in VOLUMES[1..VOLUMES.len() - 1].iter().rev() {
    //             let keyed_price1 = match self.take_max(mints, volume, slot, None, index) {
    //                 None => break,
    //                 Some(keyed_price) => keyed_price,
    //             };
    //             let keyed_price2 = match self.take_min(mints, volume, slot, None, index1) {
    //                 None => break,
    //                 Some(keyed_price) => keyed_price,
    //             };
    //             let diff = (keyed_price1.price - keyed_price2.price) * OrderedFloat(*volume as f64);
    //             if diff.is_sign_positive() && diff.is_finite() && diff > selection.difference(index) {
    //                 selection.insert(
    //                     index,
    //                     Opportunity {
    //                         slot,
    //                         route: vec![
    //                             (
    //                                 keyed_price1.pubkey,
    //                                 keyed_price1.pool_type,
    //                                 keyed_price1.price.to_le_bytes(),
    //                             ),
    //                             (
    //                                 keyed_price2.pubkey,
    //                                 keyed_price2.pool_type,
    //                                 keyed_price2.price.to_le_bytes(),
    //                             ),
    //                         ],
    //                         diff: diff.0 as u64,
    //                         mint_pair_route: vec![*mints, *mints],
    //                         timestamp: now::as_millis(),
    //                     },
    //                     *volume,
    //                 );
    //                 compare = Some(diff);
    //             } else {
    //                 // break if the difference is not increasing ...
    //                 if let Some(compare_diff) = compare {
    //                     if diff < compare_diff {
    //                         break;
    //                     } else {
    //                         compare = Some(diff);
    //                     }
    //                 } else {
    //                     compare = Some(diff);
    //                 }
    //             }
    //         }
    //     }
    //     selection.convert_to_opportunities_with_volume(filter as u64)
    // }

    pub fn find_arbitrage3(
        &mut self,
        mints: &MintPair,
        slot: u64,
        filter: f64,
        index: usize,
    ) -> Vec<(Opportunity, u64)> {
        if index >= self.size {
            return vec![];
        }

        let mints_a = [WSOL, mints[0]];
        let mints_b = [WSOL, mints[1]];

        let mints_a2 = [WSOL, mints[1]];
        let mints_b2 = [WSOL, mints[0]];

        let mut selection = Opportunities::new(self.size * 2);

        let mut compare: Option<OrderedFloat<f64>> = None;
        for volume in VOLUMES[1..VOLUMES.len() - 1].iter().rev() {
            let keyed_price1 = match self.take_max(&mints_a, volume, slot, None, index) {
                None => break,
                Some(keyed_price) => keyed_price,
            };
            let keyed_price2 = match self.take_max(mints, volume, slot, None, index) {
                None => break,
                Some(keyed_price) => keyed_price,
            };
            let keyed_price3 = match self.take_min(&mints_b, volume, slot, None, index) {
                None => break,
                Some(keyed_price) => keyed_price,
            };

            if (keyed_price2.price == 0.0) || (keyed_price3.price == 0.0) {
                continue;
            }
            let diff = (keyed_price1.price - (keyed_price3.price / keyed_price2.price)) * OrderedFloat(*volume as f64);
            if diff.is_sign_positive() && diff.is_finite() && diff > selection.difference(index) {
                selection.insert(
                    index,
                    Opportunity {
                        slot,
                        route: vec![
                            (
                                keyed_price1.pubkey,
                                keyed_price1.pool_type,
                                keyed_price1.price.to_le_bytes(),
                            ),
                            (
                                keyed_price2.pubkey,
                                keyed_price2.pool_type,
                                keyed_price2.price.to_le_bytes(),
                            ),
                            (
                                keyed_price3.pubkey,
                                keyed_price3.pool_type,
                                keyed_price3.price.to_le_bytes(),
                            ),
                        ],
                        diff: diff.0 as u64,
                        mint_pair_route: vec![mints_a, *mints, mints_b],
                        timestamp: now::as_millis(),
                    },
                    *volume,
                );
                compare = Some(diff);
            } else {
                // break if the difference is not increasing ...
                if let Some(compare_diff) = compare {
                    if diff < compare_diff {
                        break;
                    } else {
                        compare = Some(diff);
                    }
                } else {
                    compare = Some(diff);
                }
            }
        }

        let mut compare: Option<OrderedFloat<f64>> = None;
        for volume in VOLUMES[1..VOLUMES.len() - 1].iter().rev() {
            let keyed_price1 = match self.take_max(&mints_a2, volume, slot, None, index) {
                None => break,
                Some(keyed_price) => keyed_price,
            };
            let keyed_price2 = match self.take_min(mints, volume, slot, None, index) {
                None => break,
                Some(keyed_price) => keyed_price,
            };
            let keyed_price3 = match self.take_min(&mints_b2, volume, slot, None, index) {
                None => break,
                Some(keyed_price) => keyed_price,
            };

            let diff = (keyed_price1.price - (keyed_price3.price * keyed_price2.price)) * OrderedFloat(*volume as f64);

            if diff.is_sign_positive() && diff.is_finite() && diff > selection.difference(index) {
                selection.insert(
                    index + 1,
                    Opportunity {
                        slot,
                        route: vec![
                            (
                                keyed_price1.pubkey,
                                keyed_price1.pool_type,
                                keyed_price1.price.to_le_bytes(),
                            ),
                            (
                                keyed_price2.pubkey,
                                keyed_price2.pool_type,
                                keyed_price2.price.to_le_bytes(),
                            ),
                            (
                                keyed_price3.pubkey,
                                keyed_price3.pool_type,
                                keyed_price3.price.to_le_bytes(),
                            ),
                        ],
                        diff: diff.0 as u64,
                        mint_pair_route: vec![mints_a2, *mints, mints_b2],
                        timestamp: now::as_millis(),
                    },
                    *volume,
                );
                compare = Some(diff);
            } else {
                // break if the difference is not increasing ...
                if let Some(compare_diff) = compare {
                    if diff < compare_diff {
                        break;
                    } else {
                        compare = Some(diff);
                    }
                } else {
                    compare = Some(diff);
                }
            }
        }

        selection.convert_to_opportunities_with_volume(filter as u64)
    }

    // pub fn find_arbitrage3_additional(&mut self, mints: &MintPair, slot: u64, filter: f64) -> Vec<(Opportunity, u64)> {
    //     let mints_a = [WSOL, mints[0]];
    //     let mints_b = [WSOL, mints[1]];

    //     let mints_a2 = [WSOL, mints[1]];
    //     let mints_b2 = [WSOL, mints[0]];

    //     let mut selection = Opportunities::new(self.size * 2);

    //     let combinations = [(0, 1, 1), (1, 1, 0)];

    //     for (i, (index1, index2, index3)) in combinations.into_iter().enumerate() {
    //         let mut compare: Option<OrderedFloat<f64>> = None;
    //         for volume in VOLUMES[1..VOLUMES.len() - 1].iter().rev() {
    //             let keyed_price1 = match self.take_max(&mints_a, volume, slot, None, index1) {
    //                 None => break,
    //                 Some(keyed_price) => keyed_price,
    //             };
    //             let keyed_price2 = match self.take_max(mints, volume, slot, None, index2) {
    //                 None => break,
    //                 Some(keyed_price) => keyed_price,
    //             };
    //             let keyed_price3 = match self.take_min(&mints_b, volume, slot, None, index3) {
    //                 None => break,
    //                 Some(keyed_price) => keyed_price,
    //             };

    //             if (keyed_price2.price == 0.0) || (keyed_price3.price == 0.0) {
    //                 continue;
    //             }
    //             let diff =
    //                 (keyed_price1.price - (keyed_price3.price / keyed_price2.price)) * OrderedFloat(*volume as f64);
    //             if diff.is_sign_positive() && diff.is_finite() && diff > selection.difference(i) {
    //                 selection.insert(
    //                     i,
    //                     Opportunity {
    //                         slot,
    //                         route: vec![
    //                             (
    //                                 keyed_price1.pubkey,
    //                                 keyed_price1.pool_type,
    //                                 keyed_price1.price.to_le_bytes(),
    //                             ),
    //                             (
    //                                 keyed_price2.pubkey,
    //                                 keyed_price2.pool_type,
    //                                 keyed_price2.price.to_le_bytes(),
    //                             ),
    //                             (
    //                                 keyed_price3.pubkey,
    //                                 keyed_price3.pool_type,
    //                                 keyed_price3.price.to_le_bytes(),
    //                             ),
    //                         ],
    //                         diff: diff.0 as u64,
    //                         mint_pair_route: vec![mints_a, *mints, mints_b],
    //                         timestamp: now::as_millis(),
    //                     },
    //                     *volume,
    //                 );
    //                 compare = Some(diff);
    //             } else {
    //                 // break if the difference is not increasing ...
    //                 if let Some(compare_diff) = compare {
    //                     if diff < compare_diff {
    //                         break;
    //                     } else {
    //                         compare = Some(diff);
    //                     }
    //                 } else {
    //                     compare = Some(diff);
    //                 }
    //             }
    //         }
    //     }

    //     for (i, (index1, index2, index3)) in combinations.into_iter().enumerate() {
    //         let mut compare: Option<OrderedFloat<f64>> = None;
    //         for volume in VOLUMES[1..VOLUMES.len() - 1].iter().rev() {
    //             let keyed_price1 = match self.take_max(&mints_a2, volume, slot, None, index1) {
    //                 None => break,
    //                 Some(keyed_price) => keyed_price,
    //             };
    //             let keyed_price2 = match self.take_min(mints, volume, slot, None, index2) {
    //                 None => break,
    //                 Some(keyed_price) => keyed_price,
    //             };
    //             let keyed_price3 = match self.take_min(&mints_b2, volume, slot, None, index3) {
    //                 None => break,
    //                 Some(keyed_price) => keyed_price,
    //             };

    //             let diff =
    //                 (keyed_price1.price - (keyed_price3.price * keyed_price2.price)) * OrderedFloat(*volume as f64);

    //             if diff.is_sign_positive() && diff.is_finite() && diff > selection.difference(i) {
    //                 selection.insert(
    //                     i,
    //                     Opportunity {
    //                         slot,
    //                         route: vec![
    //                             (
    //                                 keyed_price1.pubkey,
    //                                 keyed_price1.pool_type,
    //                                 keyed_price1.price.to_le_bytes(),
    //                             ),
    //                             (
    //                                 keyed_price2.pubkey,
    //                                 keyed_price2.pool_type,
    //                                 keyed_price2.price.to_le_bytes(),
    //                             ),
    //                             (
    //                                 keyed_price3.pubkey,
    //                                 keyed_price3.pool_type,
    //                                 keyed_price3.price.to_le_bytes(),
    //                             ),
    //                         ],
    //                         diff: diff.0 as u64,
    //                         mint_pair_route: vec![mints_a2, *mints, mints_b2],
    //                         timestamp: now::as_millis(),
    //                     },
    //                     *volume,
    //                 );
    //                 compare = Some(diff);
    //             } else {
    //                 // break if the difference is not increasing ...
    //                 if let Some(compare_diff) = compare {
    //                     if diff < compare_diff {
    //                         break;
    //                     } else {
    //                         compare = Some(diff);
    //                     }
    //                 } else {
    //                     compare = Some(diff);
    //                 }
    //             }
    //         }
    //     }

    //     selection.convert_to_opportunities_with_volume(filter as u64)
    // }

    pub fn find_arbitrage(
        &mut self,
        mints: &MintPair,
        slot: u64,
        filter: f64,
        index: usize,
    ) -> Vec<(Opportunity, u64)> {
        let mut opportunities = vec![];

        if mints[0] == WSOL || mints[1] == WSOL {
            if cfg.arbitrage.c2 {
                opportunities.extend(self.find_arbitrage2(mints, slot, filter, index));
                // if index == 0 {
                //     opportunities.extend(self.find_arbitrage2_additional(mints, slot, filter));
                // }
            }
            if cfg.arbitrage.c3 && mints[1] != WSOL {
                let pairs = self.get_pairs(&mints[1]);
                for pair in pairs.iter() {
                    opportunities.extend(self.find_arbitrage3(pair, slot, filter, index));
                    // if index == 0 {
                    //     opportunities.extend(self.find_arbitrage3_additional(pair, slot, filter));
                    // }
                }
            }
        } else if cfg.arbitrage.c3 {
            opportunities.extend(self.find_arbitrage3(mints, slot, filter, index));
            //opportunities.extend(self.find_arbitrage3_additional(mints, slot, filter));
        }

        opportunities.sort_by(|(opp1, _), (opp2, _)| opp1.diff.cmp(&opp2.diff));
        // there can be duplicate opportunities ...
        let mut filtrated = vec![];
        let mut already_inserted = AHashSet::new();
        while let Some((opp, volume)) = opportunities.pop() {
            if !cfg.arbitrage.allow_dlmm_dlmm
                && opp.route.len() == 2
                && opp
                    .route
                    .iter()
                    .all(|(_, pool_type, _)| *pool_type == PoolType::MeteoraDlmm)
            {
                continue;
            }
            let pubkeys = opp.route.iter().map(|(pubkey, _, _)| *pubkey).collect::<Vec<_>>();
            if !already_inserted.contains(&pubkeys) {
                filtrated.push((opp, volume));
                already_inserted.insert(pubkeys);
            }
        }
        filtrated
    }
}

// #[derive(Debug, Default)]
// pub struct PriceMapInVolume {
//     pub a_to_b: MintPairPriceSet,
//     pub b_to_a: MintPairPriceSet,
//     pub volume: u64,
//     pub mint_to_volume: MintToPoolPubkeyAndAmount,
//     pub pairs: Pairs,
//     pub size: usize,
// }

#[derive(Debug, Default, Clone)]
pub struct OptimizeResult {
    // TODO: remove me later
    pub amount: u64,
    // TODO: remove me later
    pub diff: u64,
    pub remaining_accounts: Vec<Vec<Pubkey>>,
    pub amounts: Vec<u64>,
}

pub type OptimizeFn = fn(
    diff: u64,
    slot: u64,
    volume: u64,
    calculators: &[Box<CalculatorEnum>],
    mint_pair_route: &[&MintPair],
    starting_mint: &Pubkey,
) -> anyhow::Result<OptimizeResult>;

pub fn optimize_v4(
    diff: u64,
    slot: u64,
    volume: u64,
    calculators: &[Box<CalculatorEnum>],
    mint_pair_route: &[&MintPair],
    starting_mint: &Pubkey,
) -> anyhow::Result<OptimizeResult> {
    if diff == 0 {
        return Err(format_err!("ZeroDiff"));
    }

    let mut steps = 4;
    if volume > 10_000_000 {
        steps += 4;
    }

    let mut diff_to_helpers = BTreeMap::new();
    // input volume is minimum
    let mut amount_up = volume + (volume / 100);
    let mut amount_down = volume - (volume / 100);
    for _ in 0..3 {
        amount_up = amount_up.min((VOLUMES[0] / 100) * 95);
        amount_down = amount_down.min((VOLUMES[0] / 100) * 94);
        let mut results = vec![];
        results.push(diff as i64);
        let result_up = find_optimum_v4(slot, amount_up, calculators, starting_mint, mint_pair_route);
        let result_down = find_optimum_v4(slot, amount_down, calculators, starting_mint, mint_pair_route);
        // info!("OPT UP {:?} DOWN: {:?}", result_up, result_down);
        if result_up.is_err() && result_down.is_ok() {
            if let Ok((diff_down, amounts, optionals)) = result_down {
                diff_to_helpers.insert(diff_down, (amount_down, amounts, optionals));
                let amount = amount_down;
                let direction_up = false;
                results.push(diff_down);
                let result = iterate(
                    results,
                    amount,
                    slot,
                    calculators,
                    starting_mint,
                    mint_pair_route,
                    direction_up,
                    steps,
                );
                if let Ok(mut iteration_last_result) = result {
                    diff_to_helpers.insert(
                        iteration_last_result.diff as i64,
                        (
                            iteration_last_result.amount,
                            iteration_last_result.amounts.clone(),
                            iteration_last_result.remaining_accounts.clone(),
                        ),
                    );
                    if iteration_last_result.diff > 0 {
                        // info!("Original Value: {:?}, Iteration: {:?}", last_result, iteration_last_result);
                        iteration_last_result.amount = ((iteration_last_result.amount / 1000) * 1000) + 1000;
                        return Ok(iteration_last_result);
                    }
                } else {
                    amount_up = (volume / 100) * 81;
                    amount_down = (volume / 100) * 80;
                }
            }
        } else if result_up.is_ok() && result_down.is_err() {
            if let Ok((diff_up, amounts, optionals)) = result_up {
                diff_to_helpers.insert(diff_up, (amount_up, amounts, optionals));
                let amount = amount_down;
                let direction_up = true;
                results.push(diff_up);
                let result = iterate(
                    results,
                    amount,
                    slot,
                    calculators,
                    starting_mint,
                    mint_pair_route,
                    direction_up,
                    steps,
                );
                if let Ok(mut iteration_last_result) = result {
                    diff_to_helpers.insert(
                        iteration_last_result.diff as i64,
                        (
                            iteration_last_result.amount,
                            iteration_last_result.amounts.clone(),
                            iteration_last_result.remaining_accounts.clone(),
                        ),
                    );
                    if iteration_last_result.diff > 0 {
                        // info!("Original Value: {:?}, Iteration: {:?}", last_result, iteration_last_result);
                        iteration_last_result.amount = ((iteration_last_result.amount / 1000) * 1000) + 1000;
                        return Ok(iteration_last_result);
                    }
                } else {
                    amount_up = (volume / 100) * 91;
                    amount_down = (volume / 100) * 90;
                }
            }
        } else if result_up.is_ok() && result_down.is_ok() {
            if let (Ok((diff_down, damounts, doptionals)), Ok((diff_up, uamounts, uoptionals))) =
                (result_down, result_up)
            {
                diff_to_helpers.insert(diff_up, (amount_up, uamounts, uoptionals));
                diff_to_helpers.insert(diff_down, (amount_down, damounts, doptionals));
                let amount = if diff_down >= diff_up { amount_down } else { amount_up };
                let direction_up = diff_down < diff_up; // if diff_down >= diff_up { false } else { true };
                if diff_down >= diff_up {
                    results.push(diff_down)
                } else {
                    results.push(diff_up)
                };
                let result = iterate(
                    results,
                    amount,
                    slot,
                    calculators,
                    starting_mint,
                    mint_pair_route,
                    direction_up,
                    steps,
                );
                if let Ok(mut iteration_last_result) = result {
                    diff_to_helpers.insert(
                        iteration_last_result.diff as i64,
                        (
                            iteration_last_result.amount,
                            iteration_last_result.amounts.clone(),
                            iteration_last_result.remaining_accounts.clone(),
                        ),
                    );
                    if iteration_last_result.diff > 0 {
                        // info!("Original Value: {:?}, Iteration: {:?}", last_result, iteration_last_result);
                        iteration_last_result.amount = ((iteration_last_result.amount / 1000) * 1000) + 1000;
                        return Ok(iteration_last_result);
                    }
                } else {
                    amount_up = (volume / 100) * 91;
                    amount_down = (volume / 100) * 90;
                }
            }
        } else if result_up.is_err() && result_down.is_err() {
            //
            amount_up = (volume / 100) * 91;
            amount_down = (volume / 100) * 90;
        }
    }
    let (diff, (amount, amounts, optionals)) = diff_to_helpers.into_iter().max().unwrap_or((0, (0, vec![], vec![])));
    Ok(OptimizeResult {
        amount,
        diff: diff.to_u64().unwrap_or(0),
        remaining_accounts: optionals,
        amounts,
    })
}

pub fn optimize_v5(
    diff: u64,
    slot: u64,
    volume: u64,
    calculators: &[Box<CalculatorEnum>],
    mint_pair_route: &[&MintPair],
    starting_mint: &Pubkey,
) -> anyhow::Result<OptimizeResult> {
    // searching for best volume is searching for maximum on concave function
    if diff == 0 {
        return Err(format_err!("ZeroDiff"));
    }
    let default = (0, vec![], vec![]);
    // let mut last_result = OptimizeResult { amount: volume, diff };
    // input volume is minimum
    let mut coefficient = if volume >= 100_000_000_000 { 2.5 } else { 50.0 };
    // 50 25 12.5 6.25 3.
    // first calculation
    let mut results = BTreeMap::new();
    // results.insert(diff, (volume);
    let mut amount = ((volume as f64) * coefficient) as u64;
    let (mut diff_1, amounts, optionals) =
        find_optimum_v4(slot, amount, calculators, starting_mint, mint_pair_route).unwrap_or((0, vec![], vec![]));
    results.insert(diff_1.to_u64().unwrap_or(0), (amount, amounts, optionals));
    let (mut diff_2, mut amounts, mut optionals) =
        find_optimum_v4(slot, (amount * 101) / 100, calculators, starting_mint, mint_pair_route).unwrap_or((
            0,
            vec![],
            vec![],
        ));
    results.insert(diff_2.to_u64().unwrap_or(0), (amount, amounts, optionals));
    let mut first_volume = volume;
    for _ in 0..6 {
        coefficient /= 2.0;
        if diff_2 > diff_1 {
            first_volume = (amount * 101) / 100;
            amount += (volume as f64 * coefficient) as u64;
            // println!("{}", amount);
            (diff_1, amounts, optionals) = find_optimum_v4(slot, amount, calculators, starting_mint, mint_pair_route)
                .unwrap_or((0, vec![], vec![]));
            results.insert(diff_1.to_u64().unwrap_or(0), (amount, amounts, optionals));
            (diff_2, amounts, optionals) =
                find_optimum_v4(slot, (amount * 101) / 100, calculators, starting_mint, mint_pair_route).unwrap_or((
                    0,
                    vec![],
                    vec![],
                ));
            results.insert(diff_2.to_u64().unwrap_or(0), (amount, amounts, optionals));
        } else if diff_1 > diff_2 {
            if amount > first_volume {
                amount = (amount - first_volume) / 2 + first_volume;
            } else {
                amount = (amount / 95) * 100
            }
            // println!("{}", amount);
            (diff_1, amounts, optionals) = find_optimum_v4(slot, amount, calculators, starting_mint, mint_pair_route)
                .unwrap_or((0, vec![], vec![]));
            results.insert(diff_1.to_u64().unwrap_or(0), (amount, amounts, optionals));
            (diff_2, amounts, optionals) =
                find_optimum_v4(slot, (amount * 101) / 100, calculators, starting_mint, mint_pair_route).unwrap_or((
                    0,
                    vec![],
                    vec![],
                ));
            results.insert(diff_2.to_u64().unwrap_or(0), (amount, amounts, optionals));
        } else {
            let (_max_diff, (volume_for_max_diff, _, _)) = results.iter().max().unwrap_or((&0, &default));
            amount = (volume_for_max_diff * 99) / 100;
            // println!("{}", amount);
            (diff_1, amounts, optionals) = find_optimum_v4(slot, amount, calculators, starting_mint, mint_pair_route)
                .unwrap_or((0, vec![], vec![]));
            results.insert(diff_1.to_u64().unwrap_or(0), (amount, amounts, optionals));
            (diff_2, amounts, optionals) =
                find_optimum_v4(slot, (amount * 101) / 100, calculators, starting_mint, mint_pair_route).unwrap_or((
                    0,
                    vec![],
                    vec![],
                ));
            results.insert(diff_2.to_u64().unwrap_or(0), (amount, amounts, optionals));
        }
        // if break if there the difference is less than 2 %
        // if (diff_2 - diff_1).abs() < diff_1.abs() / 50 {
        //     break;
        // }
        // let (max_difference, max_volume) = results.iter().max().unwrap_or((&0, &0));
        // let length = results.iter().len();
        // if length > 2 {
        //     let (second_max_difference, second_max_volume) = results.iter().nth(length -2).unwrap_or((&0, &0));
        //     if ((max_difference - second_max_difference)/ ( second_max_difference * 100_000) ) < 10 {
        //         break;
        //     }
        // }
    }
    let (max_diff, (volume_for_max_diff, amounts, optionals)) = results.iter().max().unwrap_or((&0, &default));
    // warn!("OPT FailedToOptimizeReturnsOriginalAmount");
    let last_result = OptimizeResult {
        amount: *volume_for_max_diff,
        diff: *max_diff,
        remaining_accounts: optionals.to_vec(),
        amounts: amounts.to_vec(),
    };
    Ok(last_result)
}

#[allow(clippy::too_many_arguments)]
pub fn iterate(
    mut results: Vec<i64>,
    mut amount: u64,
    slot: u64,
    calculators: &[Box<CalculatorEnum>],
    starting_mint: &Pubkey,
    mint_pair_route: &[&MintPair],
    mut direction_up: bool,
    steps: u8,
) -> anyhow::Result<OptimizeResult> {
    let last = results.last().context("EmptyValue")?;
    let mut last_result = OptimizeResult {
        amount,
        diff: if last >= &0 { *last as u64 } else { 0_u64 },
        remaining_accounts: vec![],
        amounts: vec![],
    };
    let mut diff_to_helpers = BTreeMap::new();
    let mut diff_to_volume = BTreeMap::new();
    for step in 0..steps {
        // -2 - 3/ -2
        let coeff = results[results.len() - 1] - results[results.len() - 2];
        let coefficient = (coeff.abs() as f64).log2() / 2.0;
        // println!("coef: {} {:?} {}", coefficient, results, amount);
        // log 10 of 0.00001 is -
        let mut addition = ((amount as f64 * coefficient.min(10.0)) / 100.0) as u64;
        if let (Some(maximum), Some(last)) = (results.iter().max(), results.last()) {
            // if there is already maximum not reason to go far ...
            if maximum != last {
                addition /= 1 + step as u64;
            }
        }
        if results[results.len() - 1] > results[results.len() - 2] {
            if direction_up {
                amount += addition;
            } else {
                amount -= addition;
            }
        } else if direction_up {
            amount -= addition;
            direction_up = false;
        } else {
            amount += addition;
            direction_up = true;
        }

        // cap amount ...
        amount = amount.min((VOLUMES[0] / 100) * 95);
        // println!("AMOUNT: {}", amount);
        let result = find_optimum_v4(slot, amount, calculators, starting_mint, mint_pair_route);
        // info!("iteration result: {:?}", result);
        if let Ok((difference, amounts, optionals)) = result {
            diff_to_helpers.insert(difference, (amount, amounts, optionals));
            // println!("AMOUNT: {} DIFF: {} DUP: {}", amount, difference, direction_up);
            diff_to_volume.insert(difference, amount);
            results.push(difference);
            if difference.is_positive() && difference as u64 > last_result.diff {
                last_result.amount = amount;
                last_result.diff = difference as u64;
            }
        } else {
            // if there is error go down ...
            amount /= 10;
            amount *= 9;
        }
    }
    let (_, (_, amounts, optionals)) = diff_to_helpers.into_iter().max().unwrap_or((0, (0, vec![], vec![])));
    if let Some((top_difference, volume)) = diff_to_volume.last_key_value() {
        if top_difference.is_positive() {
            return Ok(OptimizeResult {
                amount: *volume,
                diff: top_difference.unsigned_abs(),
                remaining_accounts: optionals,
                amounts,
            });
        }
    }
    Ok(last_result)
}

// uni

#[derive(Debug, Clone)]
pub struct Budget {
    pub max_evals: usize,
    pub top_k: usize,
    pub mc_n: usize,
    pub risk_lambda: f64,
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            max_evals: 200,
            top_k: 3,
            mc_n: 8,
            risk_lambda: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub tick_aware: bool,
    pub use_bayesian: bool,
}

// impl Default for Options {
//     fn default() -> Self {
//         Options {
//             tick_aware: false,
//             use_bayesian: false,
//         }
//     }
// }

// Evaluator wrapper: for now we call find_optimum_v3 and return i64 profit or error
fn evaluate(
    slot: u64,
    volume: u64,
    calculators: &[Box<CalculatorEnum>],
    starting_mint: &Pubkey,
    mint_pair_route: &[&MintPair],
) -> Result<i64> {
    // NOTE: find_optimum_v3 signature in arb_core::arbitrage may differ; adapt as needed.
    match find_optimum_v3(slot, volume, calculators, starting_mint, mint_pair_route) {
        Ok(v) => Ok(v),
        Err(_) => Err(anyhow!("EvalError")),
    }
}

// Very small discrete hill-climb used as local refinement when brent isn't available
fn hill_climb_local<F>(vol: u64, min_v: u64, max_v: u64, budget: &mut Budget, mut eval_fn: F) -> (u64, i64)
where
    F: FnMut(u64) -> Result<i64>,
{
    let mut best_v = vol;
    let mut best_score = eval_fn(vol).unwrap_or(i64::MIN);
    // budget.max_evals = budget.max_evals; // touch to avoid unused warning

    // quick exponential expansion to escape small starts (try multipliers)
    for &mul in [2u64, 3u64, 5u64, 10u64].iter() {
        if budget.max_evals == 0 {
            break;
        }
        let cand = best_v.saturating_mul(mul);
        if cand <= best_v || cand > max_v {
            continue;
        }
        if let Ok(score) = eval_fn(cand) {
            budget.max_evals = budget.max_evals.saturating_sub(1);
            if score > best_score {
                best_score = score;
                best_v = cand;
                // continue expanding further
                continue;
            }
        } else {
            budget.max_evals = budget.max_evals.saturating_sub(1);
        }
    }

    // step sizes: 1%, 2%, 5%, 10% scaled
    let steps = [1, 2, 5, 10, 20];
    for s in steps.iter() {
        let mut changed = true;
        while changed {
            changed = false;
            for dir in [-1i64, 1i64].iter() {
                let delta = ((best_v as f64) * (*s as f64) / 100.0) as u64;
                let cand = if *dir > 0 {
                    best_v.saturating_add(delta)
                } else {
                    best_v.saturating_sub(delta)
                };
                if cand < min_v || cand > max_v || cand == best_v {
                    continue;
                }
                if budget.max_evals == 0 {
                    return (best_v, best_score);
                }
                match eval_fn(cand) {
                    Ok(score) => {
                        budget.max_evals = budget.max_evals.saturating_sub(1);
                        if score > best_score {
                            best_score = score;
                            best_v = cand;
                            changed = true;
                        }
                    }
                    Err(_) => {
                        budget.max_evals = budget.max_evals.saturating_sub(1);
                    }
                }
            }
        }
    }
    (best_v, best_score)
}

pub fn optimize_universal(
    diff: u64,
    slot: u64,
    volume: u64,
    calculators: &[Box<CalculatorEnum>],
    mint_pair_route: &[&MintPair],
    starting_mint: &Pubkey,
) -> anyhow::Result<OptimizeResult> {
    if diff == 0 {
        return Err(anyhow!("ZeroDiff"));
    }

    let mut budget = Budget {
        max_evals: 200,
        top_k: 1, // ak fia nemá lokálne maximá, top_k == 1 => výrazne rýchlejšie
        mc_n: 8,
        risk_lambda: 0.0,
    };

    let mut max_volume = 500_000_000_000;

    // coarse log grid sampling
    let points = 20_usize.min(budget.max_evals.max(8));
    let mut samples: Vec<(u64, i64)> = Vec::new();
    for i in 0..points {
        let t = (i as f64) / ((points - 1) as f64);
        let logv = (volume as f64).ln() * (1.0 - t) + (max_volume as f64).ln() * t;
        let v = logv.exp().round() as u64;
        if v == 0 {
            continue;
        }
        if budget.max_evals == 0 {
            break;
        }
        match evaluate(slot, v, calculators, starting_mint, mint_pair_route) {
            Ok(p) => {
                samples.push((v, p));
                budget.max_evals = budget.max_evals.saturating_sub(1);
            }
            Err(_) => {
                // samples.push((v, i64::MIN));
                samples.push((v, 0_i64));
                max_volume = v * 90 / 100; // if error, reduce max_volume
                budget.max_evals = budget.max_evals.saturating_sub(1);
            }
        }
    }

    if samples.is_empty() {
        return Ok(OptimizeResult {
            amount: volume,
            diff,
            remaining_accounts: vec![],
            amounts: vec![],
        });
    }

    // pick top_k samples by profit
    samples.sort_by_key(|(_, p)| OrderedFloat::from(*p as f64));
    samples.reverse();
    let top_k = budget.top_k.min(samples.len());

    let best = OptimizeResult {
        amount: volume,
        diff,
        remaining_accounts: vec![],
        amounts: vec![],
    };
    let mut candidates: Vec<(u64, i64)> = Vec::new();
    for (v, _p) in samples.iter().take(top_k) {
        // local refinement via hill-climb
        let (best_v, best_p) = hill_climb_local(*v, volume, max_volume, &mut budget, |vv| {
            evaluate(slot, vv, calculators, starting_mint, mint_pair_route)
        });
        candidates.push((best_v, best_p));
        if budget.max_evals == 0 {
            break;
        }
    }

    // If no candidates found, return default
    if candidates.is_empty() {
        return Ok(best);
    }

    // Monte-Carlo robustness: for each candidate compute mean/std of profits using small perturbations
    let mut rng = rand::thread_rng();
    let mut best_utility: f64 = f64::NEG_INFINITY;
    let mut best_final = best;

    for (cand_v, cand_p) in candidates.into_iter() {
        if budget.mc_n == 0 || budget.max_evals == 0 {
            // no MC, fallback to raw profit
            let mean = cand_p as f64;
            let std = 0.0_f64;
            let utility = mean - budget.risk_lambda * std;
            if utility > best_utility {
                best_utility = utility;
                best_final.amount = cand_v;
                best_final.diff = cand_p as u64;
            }
            continue;
        }

        let mut vals: Vec<f64> = Vec::with_capacity(budget.mc_n);
        for _ in 0..budget.mc_n {
            if budget.max_evals == 0 {
                break;
            }
            // perturbation: ±0.1% .. ±0.5%
            let eps = (rng.gen_range(1u32..=5u32) as f64) / 1000.0; // 0.001 .. 0.005
            let sign = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
            let mut pert_v = ((cand_v as f64) * (1.0 + sign * eps)).round() as u64;
            if pert_v < volume {
                pert_v = volume;
            }
            if pert_v > max_volume {
                pert_v = max_volume;
            }
            let profit = match evaluate(slot, pert_v, calculators, starting_mint, mint_pair_route) {
                Ok(p) => p as f64,
                Err(_) => i64::MIN as f64,
            };
            vals.push(profit);
            budget.max_evals = budget.max_evals.saturating_sub(1);
        }
        if vals.is_empty() {
            continue;
        }
        let mean = vals.iter().sum::<f64>() / (vals.len() as f64);
        let var = vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / (vals.len() as f64);
        let std = var.sqrt();
        let utility = mean - budget.risk_lambda * std;
        if utility > best_utility {
            // choose candidate with best utility
            best_utility = utility;
            // for final diff pick mean rounded to u64 if positive else keep cand_p
            best_final.amount = cand_v;
            best_final.diff = if mean.is_sign_positive() {
                mean as u64
            } else {
                cand_p as u64
            };
        }
    }

    // final snap to 100 units
    best_final.amount = (best_final.amount / 100) * 100;

    if best_final.diff > u64::MAX - 100_000_000_000 {
        return Err(anyhow!("NoProfit"));
    }

    Ok(best_final)
}

pub fn optimize_convex(
    diff: u64,
    slot: u64,
    volume: u64,
    //max_volume: u64,
    calculators: &[Box<CalculatorEnum>],
    mint_pair_route: &[&MintPair],
    starting_mint: &Pubkey,
) -> anyhow::Result<OptimizeResult> {
    let mut diff_to_helpers = BTreeMap::new();
    const DELTA_FACTOR: u64 = 2;
    let mut lo = volume;
    let mut delta: u64 = lo / DELTA_FACTOR;
    let mut hi = VOLUMES.iter().copied().max().unwrap_or(500_000_000_000);
    // Aby sme sa vyhli nekonečnej slučke, skončíme keď je interval malý
    while hi.saturating_sub(lo) > 5 {
        // Pozície tretín (pozor na pretečenie)
        let third = (hi - lo) / 3;
        let m1 = lo + third;
        let m2 = hi - third;

        let (f1, amounts, remaining_accounts) = find_optimum_v4(slot, m1, calculators, starting_mint, mint_pair_route)
            .unwrap_or((i64::MIN, vec![], vec![]));
        diff_to_helpers.insert(f1, (amounts, remaining_accounts));
        let (f2, amounts, remaining_accounts) = find_optimum_v4(slot, m2, calculators, starting_mint, mint_pair_route)
            .unwrap_or((i64::MIN, vec![], vec![]));
        diff_to_helpers.insert(f2, (amounts, remaining_accounts));

        // Pri unimodálnej funkcii platí:
        // - ak f1 < f2, maximum je vpravo od m1 (posuň lo)
        // - inak je vľavo od m2 (posuň hi)
        delta = lo / DELTA_FACTOR;
        if f1 < f2 {
            lo = m1 + delta;
        } else {
            hi = m2.saturating_sub(delta);
        }
    }

    // Dofinišujeme lineárnym dohľadaním v úzkom intervale
    let mut best_v = lo;
    let (mut best_s, amounts, remaining_accounts) =
        find_optimum_v4(slot, lo, calculators, starting_mint, mint_pair_route).unwrap_or((i64::MIN, vec![], vec![]));
    diff_to_helpers.insert(best_s, (amounts, remaining_accounts));
    let mut v = lo + delta;
    while v <= hi {
        let (s, amounts, remaining_accounts) =
            find_optimum_v4(slot, v, calculators, starting_mint, mint_pair_route).unwrap_or((i64::MIN, vec![], vec![]));
        diff_to_helpers.insert(s, (amounts, remaining_accounts));
        if s > best_s {
            let diff = s - best_s;
            best_s = s;
            best_v = v;
            if diff > best_s / 20 {
                break;
            }
        }
        v += delta;
    }
    let (_, (amounts, remaining_accounts)) = diff_to_helpers.into_iter().max().unwrap_or((0, (vec![], vec![])));
    Ok(OptimizeResult {
        amount: if best_s > diff as i64 { best_v } else { volume },
        diff: if best_s > (diff as i64) { best_s as u64 } else { diff },
        remaining_accounts,
        amounts,
    })
}

//

pub fn optimize_benchmark(
    diff: u64,
    slot: u64,
    volume: u64,
    calculators: &[Box<CalculatorEnum>],
    mint_pair_route: &[&MintPair],
    starting_mint: &Pubkey,
) -> anyhow::Result<OptimizeResult> {
    if diff == 0 {
        return Err(format_err!("ZeroDiff"));
    }
    //info!("STARTING VOLUME: {}", volume);
    let min_step = (volume * 2) / 200;
    let mut last_result = OptimizeResult {
        amount: volume,
        diff,
        remaining_accounts: vec![],
        amounts: vec![],
    };
    // input volume is minimum
    let mut map = BTreeMap::new();
    // next volume is maximum
    for step in 0..20_000 {
        // TODO denominator depends
        let amount = step * min_step;
        let result = find_optimum_v3(slot, amount, calculators, starting_mint, mint_pair_route);
        if let Ok(diff) = result {
            //info!("DIFF: {}, AMOUNT: {}", diff, amount);
            map.insert(diff, amount);
        } else {
            break;
        }
    }
    if let Some((diff, volume)) = map.last_key_value() {
        if *diff > 0_i64 {
            last_result.amount = *volume;
            last_result.diff = ((*diff as u64) / 100) * 100;
        }
    }
    Ok(last_result)
}
