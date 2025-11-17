use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;

#[inline]
pub fn create_ata_token_or_not(
    funding: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
    token_program: Option<&Pubkey>,
) -> Instruction {
    spl_associated_token_account::instruction::create_associated_token_account_idempotent(
        funding,
        owner,
        mint,
        token_program.unwrap_or(&spl_token::id()),
    )
}
