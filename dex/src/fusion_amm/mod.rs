use solana_sdk::pubkey::Pubkey;

solana_program::declare_id!("fUSioN9YKKSa3CUC2YUc4tPkHJ5Y6XW1yz8y6F7qWz9");
pub mod accounts;
pub use accounts::*;
pub mod typedefs;

// transferred here from Orca
pub fn get_tick_array_address_fusion(whirlpool: &Pubkey, start_tick_index: i32) -> anyhow::Result<(Pubkey, u8)> {
    let start_tick_index_str = start_tick_index.to_string();
    let seeds = &[b"tick_array", whirlpool.as_ref(), start_tick_index_str.as_bytes()];

    Pubkey::try_find_program_address(seeds, &ID).ok_or(anyhow::format_err!("FusionAMM::ProgramError::InvalidSeeds"))
}
