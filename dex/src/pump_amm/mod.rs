solana_program::declare_id!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
pub const GLOBAL_CONFIG_ID: Pubkey = pubkey!("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");

pub mod accounts;
pub use accounts::*;
pub mod typedefs;
use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;
pub use typedefs::*;
pub mod calculator;
pub mod instructions;
pub use instructions::*;
pub mod fees;
pub use fees::*;

pub fn get_fee_config_pda(pool_pubkey: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"fee_config", pool_pubkey.as_ref()], &ID)
}
