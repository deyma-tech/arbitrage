solana_program::declare_id!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
pub mod accounts;

pub use accounts::*;
use solana_program::pubkey::Pubkey;
use spl_token_swap::solana_program::pubkey;
pub mod typedefs;
pub use typedefs::*;
pub mod instructions;
pub use instructions::*;
pub mod errors;
pub use errors::*;
pub mod calculator;
pub mod events;
pub mod fees;

pub use events::*;

pub const RAYDIUM_VAULT_AUTHORITY: Pubkey = pubkey!("GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL");
