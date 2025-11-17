use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;

solana_program::declare_id!("GAMMA7meSFWaBXF25oSUgmGRwaW6sCMFLmBNiMSdbHVT");
pub mod accounts;
mod calculator;
pub mod errors;
mod fee;
pub mod instructions;
pub mod typedefs;

//   let (authority, __bump) = Pubkey::find_program_address(&[AUTH_SEED.as_bytes()], &program.id());
// ALfS4oPB5684XwTvCjWw7XddFfmyTNdcY7xHxbh2Ui8s

pub const GAMMA_AUTHORITY: Pubkey = pubkey!("ALfS4oPB5684XwTvCjWw7XddFfmyTNdcY7xHxbh2Ui8s");
