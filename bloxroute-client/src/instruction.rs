use bincode::serialize;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::transfer;
use solana_sdk::pubkey;
use solana_sdk::transaction::VersionedTransaction;
use utils::base64::b64_encode;
use utils::rnd::{rnd_element, rnd_index};

const PUBKEYS: [Pubkey; 4] = [
    pubkey!("HWEoBxYs7ssKuudEjzjmpfJVX7Dvi7wescFsVx2L5yoY"),
    pubkey!("95cfoy472fcQHaw4tPGBTKpn6ZQnfEPfBgDQx6gcRmRg"),
    pubkey!("3UQUKjhMKaY2S6bjcQD6yHB7utcZt5bfarRCmctpRtUd"),
    pubkey!("FogxVNs6Mm2w9rnGL1vkARSwJxvLE8mujTv3LK8RnUhF"),
];

#[inline(always)]
pub fn get_bloxroute_random_pubkey() -> Pubkey {
    rnd_element(&PUBKEYS).to_owned()
}

pub fn get_bloxroute_random_pubkey_exclude(pubkey: &Pubkey) -> Pubkey {
    let pubkeys = PUBKEYS.iter().filter(|&x| x != pubkey).collect::<Vec<_>>();
    let idx = rnd_index(&pubkeys);
    *pubkeys[idx]
}

pub fn tip_ix(tip: u64, payer: &Pubkey) -> Instruction {
    transfer(payer, &get_bloxroute_random_pubkey(), tip)
}

pub fn tip_ix_with_key(tip: u64, payer: &Pubkey, bloxroute_tip_pubkey: &Pubkey) -> Instruction {
    transfer(payer, bloxroute_tip_pubkey, tip)
}

pub fn tip_ix_exclude(tip: u64, payer: &Pubkey, exclude: &Pubkey) -> Instruction {
    transfer(payer, &get_bloxroute_random_pubkey_exclude(exclude), tip)
}

pub fn versioned_tx_to_string(txn: &VersionedTransaction) -> String {
    // let custom_engine: engine::GeneralPurpose = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
    let tx = serialize(txn).unwrap();
    b64_encode(tx)
}
