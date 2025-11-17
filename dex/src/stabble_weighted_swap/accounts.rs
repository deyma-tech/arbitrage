use crate::stabble_stable_swap::constants::{STABLE_FEE_VAULT, VAULT_PROGRAM};
use crate::stabble_stable_swap::stable_math::{FixedComplement, FixedMul};
use crate::stabble_stable_swap::StableSwapProgramIx::SwapV2;
use crate::stabble_stable_swap::{SwapV2IxArgs, SwapV2Keys};
use crate::stabble_weighted_swap::pda::get_withdraw_authority_address;
use ahash::AHashSet;
use anyhow::{format_err, Context};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use utils::pool::{sort_mints, Pool};
use utils::quote::QuoteResult;
use utils::types::MintPair;

const VAULT_AUTHORITY: Pubkey = pubkey!("7HkzG4LYyCJSrD3gopPQv3VVzQQKbHBZcm9fbjj5fuaH");

#[derive(Debug, Clone)]
pub struct PoolToken {
    pub mint: Pubkey,        // immutable
    pub decimals: u8,        // immutable
    pub scaling_up: bool,    // immutable
    pub scaling_factor: u64, // immutable
    pub balance: u64,
    pub weight: u64, // immutable
}

#[derive(Debug, Clone)]
pub struct PoolWeightedSwap {
    // pub owner: Pubkey,
    pub vault: Pubkey,
    // pub mint: Pubkey,
    // pub authority_bump: u8,
    pub is_active: bool,
    pub invariant: u64,
    pub swap_fee: u64,
    pub tokens: Vec<PoolToken>,
    // pub pending_owner: Option<Pubkey>,
    // pub max_supply: u64,
}

impl PoolWeightedSwap {
    pub const DISCRIMINATOR: [u8; 8] = [241, 154, 109, 4, 17, 177, 109, 188];

    pub fn try_deserialize(data: &[u8]) -> anyhow::Result<Self> {
        let mut offset = 0;

        // Check discriminator
        if data.len() < 8 {
            return Err(format_err!("AccountDiscriminatorNotFound"));
        }
        let discriminator = &data[offset..offset + 8];
        if discriminator != Self::DISCRIMINATOR {
            return Err(format_err!("AccountDiscriminatorMismatch"));
        }
        offset += 40;

        let vault = Pubkey::new_from_array(
            data[offset..offset + 32]
                .try_into()
                .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
        );
        offset += 65;

        let is_active = data[offset] != 0;
        offset += 1;

        let invariant = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
        );
        offset += 8;

        let swap_fee = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
        );
        offset += 8;

        // Deserialize tokens
        let token_count = u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
        );
        offset += 4;

        let mut tokens = Vec::with_capacity(token_count as usize);
        for _ in 0..token_count {
            let mint = Pubkey::new_from_array(
                data[offset..offset + 32]
                    .try_into()
                    .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
            );
            offset += 32;

            let decimals = data[offset];
            offset += 1;

            let scaling_up = data[offset] != 0;
            offset += 1;

            let scaling_factor = u64::from_le_bytes(
                data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
            );
            offset += 8;

            let balance = u64::from_le_bytes(
                data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
            );
            offset += 8;

            let weight = u64::from_le_bytes(
                data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
            );
            offset += 8;

            tokens.push(PoolToken {
                mint,
                decimals,
                scaling_up,
                scaling_factor,
                balance,
                weight,
            });
        }

        Ok(Self {
            vault,
            is_active,
            invariant,
            swap_fee,
            tokens,
        })
    }

    pub fn get_normalized_weights(&self) -> Vec<u64> {
        self.tokens.iter().map(|token| token.weight).collect()
    }

    pub fn get_balances(&self) -> Vec<u64> {
        self.tokens.iter().map(|token| token.balance).collect()
    }

    pub fn get_token_index(&self, mint: Pubkey) -> Option<usize> {
        self.tokens.iter().position(|token| token.mint == mint)
    }

    /// scaling up/down from token amount to wrapped balance amount
    pub fn calc_wrapped_amount(&self, amount: u64, token_index: usize) -> Option<u64> {
        let pool_token = self.tokens.get(token_index)?;
        if pool_token.scaling_factor == 1 {
            Some(amount)
        } else if pool_token.scaling_up {
            amount.checked_mul(pool_token.scaling_factor)
        } else {
            amount.checked_div(pool_token.scaling_factor)
        }
    }

    /// scaling up/down from wrapped balance amount to token amount
    pub fn calc_unwrapped_amount(&self, amount: u64, token_index: usize) -> Option<u64> {
        let pool_token = self.tokens.get(token_index)?;
        if pool_token.scaling_factor == 1 {
            Some(amount)
        } else if pool_token.scaling_up {
            amount.checked_div(pool_token.scaling_factor)
        } else {
            amount.checked_mul(pool_token.scaling_factor)
        }
    }

    /// round down token amount not to send the lost amount from wrapped balance amount when it scaled down
    pub fn calc_rounded_amount(&self, amount: u64, token_index: usize) -> Option<u64> {
        let pool_token = self.tokens.get(token_index)?;
        if pool_token.scaling_up {
            Some(amount)
        } else {
            amount
                .checked_div(pool_token.scaling_factor)?
                .checked_mul(pool_token.scaling_factor)
        }
    }

    /// estimated swap amount out
    pub fn get_swap_result(&self, token_in_index: usize, token_out_index: usize, amount_in: u64) -> Option<(u64, u64)> {
        if self.invariant == 0 {
            return Some((0, 0));
        }

        let wrapped_amount_in = self.calc_wrapped_amount(amount_in, token_in_index)?;

        let token_in = self.tokens.get(token_in_index)?;
        let token_out = self.tokens.get(token_out_index)?;
        let wrapped_amount_out_without_fee = super::weighted_math::calc_out_given_in(
            token_in.balance,
            token_in.weight,
            token_out.balance,
            token_out.weight,
            wrapped_amount_in,
        )?;

        let wrapped_amount_out = wrapped_amount_out_without_fee.mul_down(self.swap_fee.complement())?;
        let wrapped_amount_fee = wrapped_amount_out_without_fee.checked_sub(wrapped_amount_out)?;
        let amount_out = self.calc_unwrapped_amount(wrapped_amount_out, token_out_index)?;
        let amount_fee = self.calc_unwrapped_amount(wrapped_amount_fee, token_out_index)?;

        Some((amount_out, amount_fee))
    }
}

// impl PoolWeightedSwap {
//
//     fn get_count_mint_pairs() -> u64 {
//         s
//     }
//     fn get_all_mint_pairs() -> Vec<MintPair> {
//         todo!()
//     }
// }

pub struct QuoteInput {
    pub mint_pair: Option<MintPair>,
}

impl Pool<QuoteInput> for PoolWeightedSwap {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.vault]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.vault]
    }

    fn get_count_mint_pairs(&self) -> u64 {
        let number_of_tokens = self.tokens.iter().len();
        (number_of_tokens * (number_of_tokens - 1)) as u64
    }
    fn get_all_mint_pairs(&self) -> Vec<MintPair> {
        let mints = self.tokens.iter().map(|x| &x.mint).collect::<Vec<_>>();
        let second = mints.clone();
        // println!("seconds: {:?}", second);
        let mut pairs = vec![];
        for mint in mints.iter() {
            let filtrated = second.iter().filter(|pubkey| *pubkey != mint).collect::<Vec<_>>();
            for second_mint in filtrated.into_iter() {
                // println!("{} {}", mint, second_mint);
                pairs.push(sort_mints(**mint, **second_mint));
            }
        }
        pairs.sort();
        pairs.dedup();
        pairs
    }

    fn quote(&self, a_to_b: bool, exact_in: bool, amount_in: u64, input: &QuoteInput) -> anyhow::Result<QuoteResult> {
        let mut idx_in = self
            .tokens
            .iter()
            .position(|x| &x.mint == if a_to_b { self.get_a_mint() } else { self.get_b_mint() })
            .context("SWSQuote1")?;
        let mut idx_out = self
            .tokens
            .iter()
            .position(|x| &x.mint == if !a_to_b { self.get_a_mint() } else { self.get_b_mint() })
            .context("SWSQuote2")?;
        if let Some(mint_pair) = input.mint_pair {
            idx_in = self
                .tokens
                .iter()
                .position(|x| &x.mint == if a_to_b { &mint_pair[0] } else { &mint_pair[1] })
                .context("SWSQuote3")?;
            idx_out = self
                .tokens
                .iter()
                .position(|x| &x.mint == if !a_to_b { &mint_pair[0] } else { &mint_pair[1] })
                .context("SWSQuote4")?;
        }
        if exact_in {
            let in_balance = self.tokens[idx_in].balance;
            if a_to_b && amount_in > in_balance {
                return Err(format_err!("OutOfLiquidity:StabbleWeightedSwap"));
            }
            // let amplification = self.get_amplification(t as i64);
            let result = self.get_swap_result(idx_in, idx_out, amount_in);
            if let Some((amount_out, _amount_fee)) = result {
                return Ok(QuoteResult {
                    amount_a_in: if a_to_b { amount_in } else { 0 },
                    amount_b_in: if !a_to_b { amount_in } else { 0 },
                    amount_a_out: if !a_to_b { amount_out } else { 0 },
                    amount_b_out: if a_to_b { amount_out } else { 0 },
                    indices: None,
                    compute_units: 55_000,
                });
            }
        }
        Err(format_err!("NotAllowed  (StabbleWeightedSwap)"))
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        if self
            .tokens
            .iter()
            .any(|token| token.mint == crate::stabble_stable_swap::accounts::WSOL)
        {
            &crate::stabble_stable_swap::accounts::WSOL
        } else {
            &self.tokens[0].mint
        }
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        // returns
        if self
            .tokens
            .iter()
            .any(|token| token.mint == crate::stabble_stable_swap::accounts::WSOL)
        {
            &self
                .tokens
                .iter()
                .find(|token| token.mint != crate::stabble_stable_swap::accounts::WSOL)
                .unwrap()
                .mint
        } else {
            &self.tokens[1].mint
        }
    }

    fn get_program_id(&self) -> Pubkey {
        super::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        _a_to_b: bool,
        exact_in: bool,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<u8>> {
        if exact_in {
            return SwapV2(SwapV2IxArgs {
                amount_in: Some(amount),
                minimum_amount_out: amount_threshold,
            })
            .try_to_vec()
            .context("Stabble Weighed Swap: SwapV2IxArgs");
        }
        Ok(vec![])
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        _optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
        _allowed_token2022: &AHashSet<Pubkey>,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        let (mut mint_in, mut mint_out) = if a_to_b {
            (*self.get_a_mint(), *self.get_b_mint())
        } else {
            (*self.get_b_mint(), *self.get_a_mint())
        };
        if let Some(optionals) = optional_accounts {
            if optionals.len() == 2 {
                if a_to_b {
                    (mint_in, mint_out) = (optionals[0], optionals[1]);
                } else {
                    (mint_in, mint_out) = (optionals[1], optionals[0]);
                }
            }
        }
        let vault_token_in = get_associated_token_address(&VAULT_AUTHORITY, &mint_in);
        let vault_token_out = get_associated_token_address(&VAULT_AUTHORITY, &mint_out);
        let beneficiary_token_out = get_associated_token_address(&STABLE_FEE_VAULT, &mint_out);
        let keys = SwapV2Keys {
            user: signer,
            mint_in,
            mint_out,
            user_token_in: source_token_account,
            user_token_out: destination_token_account,
            vault_token_in,
            vault_token_out,
            beneficiary_token_out,
            pool: pool_pubkey,
            withdraw_authority: get_withdraw_authority_address(&self.vault),
            vault: self.vault,
            vault_authority: VAULT_AUTHORITY,
            vault_program: VAULT_PROGRAM,
            token_program: spl_token::ID,
            token_2022program: pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
        };
        Ok(vec![
            AccountMeta {
                pubkey: keys.user,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.mint_in,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.mint_out,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.user_token_in,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.user_token_out,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_token_in,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_token_out,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.beneficiary_token_out,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.withdraw_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_authority,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_2022program,
                is_signer: false,
                is_writable: false,
            },
        ])
    }
}
