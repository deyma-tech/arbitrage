use solana_program::pubkey::Pubkey;

pub fn get_authority(amm_pubkey: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[amm_pubkey.as_ref()], &super::ID).0
}
