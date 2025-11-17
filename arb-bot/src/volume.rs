use ahash::HashMap;
use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;

// TODO: remove me
pub fn process_mint_to_max(
    volume: u64,
    mut amount_out: u64,
    mint: &Pubkey,
    pool_pubkey: &Pubkey,
    mint_to_max: &mut HashMap<Pubkey, BTreeMap<u64, (Pubkey, u64)>>,
) {
    if mint_to_max.contains_key(mint) {
        let volume_to_max = mint_to_max.get_mut(mint);
        match volume_to_max {
            None => {
                // if no amount in for vomule and mint
                mint_to_max.insert(*mint, BTreeMap::from_iter([(volume, (*pool_pubkey, amount_out))]));
            }
            Some(volume_to_max) => {
                let result = volume_to_max.get_mut(&volume);
                match result {
                    None => {
                        volume_to_max.insert(volume, (*pool_pubkey, amount_out));
                    }
                    Some((old_pool_pubkey, old_amount_out)) => {
                        if old_amount_out < &mut amount_out {
                            *old_pool_pubkey = *pool_pubkey;
                            *old_amount_out = amount_out;
                        } else if old_pool_pubkey == pool_pubkey && old_amount_out > &mut amount_out {
                            *old_amount_out = amount_out;
                        }
                    }
                }
            }
        }
    } else {
        mint_to_max.insert(*mint, BTreeMap::from_iter([(volume, (*pool_pubkey, amount_out))]));
    }
}
