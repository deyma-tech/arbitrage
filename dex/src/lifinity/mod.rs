solana_program::declare_id!("2wT8Yq49kHgDzXuPxZSaeLaH1qbmGXtEyPy64bL7aD3c");
pub mod accounts;

pub use accounts::*;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
pub mod typedefs;
pub use typedefs::*;
pub mod instructions;
pub use instructions::*;
pub mod errors;
pub mod pc;
pub mod pda;

pub use errors::*;

// oracle pubkey ...
pub const ORACLE_PUBKEY: Pubkey = pubkey!("8BR3zs8zSXetpnDjCtHWnkpSkNSydWb3PTTDuVKku2uu");
