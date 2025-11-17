use questdb::ingress::Buffer;
use solana_program::clock::Slot;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::collections::BTreeMap;
use std::fmt::Debug;
use utils::pool::{Pool, PoolType};
use utils::quote::QuoteResult;

const TABLE: &str = "pools";

#[derive(Clone, Copy, Default, Debug)]
pub struct GeneralInputs {
    pub amount: u64,
    pub a_to_b: bool,
    pub exact_in: bool,
}

impl GeneralInputs {
    pub fn update_buffer(&self, buffer: &mut Buffer) {
        buffer
            .column_bool("a_to_b", self.a_to_b)
            .unwrap()
            .column_bool("exact_in", self.exact_in)
            .unwrap()
            .column_i64("amount", self.amount as i64)
            .unwrap();
    }
}

pub enum QuoteInputEnum<'a> {
    RaydiumClmmV3(&'a dex::raydium_clmm::QuoteInput<'a>),
    RaydiumAmmV4(&'a dex::raydium_amm::QuoteInput),
    Orca(&'a BTreeMap<i32, (Pubkey, dex::orca::TickArray)>),
    MeteoraDlmm(&'a dex::meteora_dlmm::quote::QuoteInput<'a>),
}

#[allow(clippy::too_many_arguments)]
pub fn update_buffer<T, I: Pool<I>>(
    pool: &dyn Pool<I>,
    pool_pk: &Pubkey,
    signature: Signature,
    slot: Slot,
    pool_type: PoolType,
    general_inputs: GeneralInputs,
    quote_result: QuoteResult,
    buffer: &mut Buffer,
) {
    // TODO
    let pool_type = format!("{pool_type:?}");
    buffer
        .table(TABLE)
        .unwrap()
        .column_str("pool_type", pool_type)
        .unwrap()
        .column_str("pubkey", pool_pk.to_string())
        .unwrap()
        .column_str("mint_id", pool.get_sorted_mints_as_str())
        .unwrap()
        .column_str("a_mint", pool.get_a_mint().to_string())
        .unwrap()
        .column_str("b_mint", pool.get_b_mint().to_string())
        .unwrap()
        .column_str("signature", signature.to_string())
        .unwrap()
        .column_i64("slot", slot as i64)
        .unwrap();
    general_inputs.update_buffer(buffer);
    quote_result.update_buffer(buffer);
    buffer.at_now().unwrap();
}
