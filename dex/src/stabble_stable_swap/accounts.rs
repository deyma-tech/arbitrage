use crate::stabble_stable_swap::constants::{STABLE_FEE_VAULT, VAULT_AUTHORITY, VAULT_PROGRAM};
use crate::stabble_stable_swap::stable_math::{CheckedMulDiv, FixedComplement, FixedMul};
use crate::stabble_stable_swap::StableSwapProgramIx::SwapV2;
use crate::stabble_stable_swap::{stable_math, PoolToken, SwapV2IxArgs, SwapV2Keys};
use ahash::AHashSet;
use anyhow::{format_err, Context};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::time::{SystemTime, UNIX_EPOCH};
use utils::pool::{sort_mints, Pool};
use utils::quote::QuoteResult;
use utils::types::MintPair;

pub(crate) const WSOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub const POOL_ACCOUNT_DISCM: [u8; 8] = [241, 154, 109, 4, 17, 177, 109, 188];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct PoolSwap {
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub mint: Pubkey,
    pub authority_bump: u8,
    pub is_active: bool,
    pub amp_initial_factor: u16,
    pub amp_target_factor: u16,
    pub ramp_start_ts: i64,
    pub ramp_stop_ts: i64,
    pub swap_fee: u64,
    pub tokens: Vec<PoolToken>,
    // pub pending_owner: Option<Pubkey>,
    // pub max_supply: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolAccount(pub PoolSwap);
impl PoolAccount {
    // pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
    //     use std::io::Read;
    //     let mut reader = buf;
    //     let mut maybe_discm = [0u8; 8];
    //     reader.read_exact(&mut maybe_discm)?;
    //     if maybe_discm != POOL_ACCOUNT_DISCM {
    //         return Err(std::io::Error::other(format!(
    //             "discm does not match. Expected: {POOL_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
    //         )));
    //     }
    //     Ok(Self(PoolSwap::deserialize(&mut reader)?))
    // }
    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {
        let mut offset = 134;

        let token_count = u32::from_le_bytes(
            buf[offset..offset + 4]
                .try_into()
                .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
        );
        offset += 4;

        let mut tokens = Vec::with_capacity(token_count as usize);
        for _ in 0..token_count {
            let mint = Pubkey::new_from_array(
                buf[offset..offset + 32]
                    .try_into()
                    .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
            );
            offset += 32;

            let decimals = buf[offset];
            offset += 1;

            let scaling_up = buf[offset] != 0;
            offset += 1;

            let scaling_factor = u64::from_le_bytes(
                buf[offset..offset + 8]
                    .try_into()
                    .map_err(|_| format_err!("AccountDidNotDeserialize"))?,
            );
            offset += 8;

            let balance = u64::from_le_bytes(
                buf[offset..offset + 8]
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
            });
        }

        let pool = PoolSwap {
            owner: Pubkey::new_from_array(
                buf[8..40]
                    .try_into()
                    .context("StabbleStableSwap::FailedToDeser::Owner")?,
            ),
            vault: Pubkey::new_from_array(
                buf[40..72]
                    .try_into()
                    .context("StabbleStableSwap::FailedToDeser::Vault")?,
            ),
            mint: Pubkey::new_from_array(
                buf[72..104]
                    .try_into()
                    .context("StabbleStableSwap::FailedToDeser::Mint")?,
            ),
            authority_bump: buf[104],
            is_active: buf[105] != 0,
            amp_initial_factor: u16::from_le_bytes(
                buf[106..108]
                    .try_into()
                    .context("StabbleStableSwap::FailedToDeser::AmpInitialFactor")?,
            ),
            amp_target_factor: u16::from_le_bytes(
                buf[108..110]
                    .try_into()
                    .context("StabbleStableSwap::FailedToDeser::AmpTargetFactor")?,
            ),
            ramp_start_ts: i64::from_le_bytes(
                buf[110..118]
                    .try_into()
                    .context("StabbleStableSwap::FailedToDeser::RampStartTs")?,
            ),
            ramp_stop_ts: i64::from_le_bytes(
                buf[118..126]
                    .try_into()
                    .context("StabbleStableSwap::FailedToDeser::RampStopTs")?,
            ),
            swap_fee: u64::from_le_bytes(
                buf[126..134]
                    .try_into()
                    .context("StabbleStableSwap::FailedToDeser::SwapFee")?,
            ),
            tokens,
        };
        Ok(Self(pool))
    }

    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&POOL_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

impl PoolSwap {
    pub fn get_amplification(&self, current_ts: i64) -> Option<u64> {
        let amp_initial_factor = self.amp_initial_factor as u64;
        let amp_target_factor = self.amp_target_factor as u64;

        let amp = if current_ts <= self.ramp_start_ts {
            amp_initial_factor.saturating_mul(stable_math::AMP_PRECISION)
        } else if current_ts >= self.ramp_stop_ts {
            amp_target_factor.saturating_mul(stable_math::AMP_PRECISION)
        } else {
            let ramp_elapsed = (current_ts.saturating_sub(self.ramp_start_ts) as u64)
                .checked_div(60)?
                .checked_mul(60)?;
            let ramp_duration = self.ramp_stop_ts.saturating_sub(self.ramp_start_ts) as u64;
            if amp_initial_factor <= amp_target_factor {
                let amp_offset = (amp_target_factor.saturating_sub(amp_initial_factor))
                    .saturating_mul(stable_math::AMP_PRECISION)
                    .checked_mul_div_down(ramp_elapsed, ramp_duration)?;
                amp_initial_factor
                    .saturating_mul(stable_math::AMP_PRECISION)
                    .saturating_add(amp_offset)
            } else {
                let amp_offset = (amp_initial_factor.saturating_sub(amp_target_factor))
                    .saturating_mul(stable_math::AMP_PRECISION)
                    .checked_mul_div_down(ramp_elapsed, ramp_duration)?;
                amp_initial_factor
                    .saturating_mul(stable_math::AMP_PRECISION)
                    .saturating_sub(amp_offset)
            }
        };

        Some(amp)
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
    pub fn get_swap_result(
        &self,
        current_ts: i64,
        token_in_index: usize,
        token_out_index: usize,
        amount_in: u64,
    ) -> Option<(u64, u64)> {
        let amplification = self.get_amplification(current_ts)?;
        let balances = self.get_balances();
        let current_invariant = stable_math::calc_invariant(amplification, &balances)?;

        let wrapped_amount_in = self.calc_wrapped_amount(amount_in, token_in_index)?;
        let wrapped_amount_out_without_fee = stable_math::calc_out_given_in(
            amplification,
            &balances,
            token_in_index,
            token_out_index,
            wrapped_amount_in,
            current_invariant,
        )?;

        let wrapped_amount_out = wrapped_amount_out_without_fee.mul_down(self.swap_fee.complement())?;
        let wrapped_amount_fee = wrapped_amount_out_without_fee.checked_sub(wrapped_amount_out)?;
        let amount_out = self.calc_unwrapped_amount(wrapped_amount_out, token_out_index)?;
        let amount_fee = self.calc_unwrapped_amount(wrapped_amount_fee, token_out_index)?;
        Some((amount_out, amount_fee))
    }
}

pub struct QuoteInput {
    pub mint_pair: Option<MintPair>,
}

impl Pool<QuoteInput> for PoolSwap {
    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![&self.vault, &self.mint]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![self.vault, self.mint]
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
            .position(|x| &x.mint == if a_to_b { self.get_a_mint() } else { self.get_b_mint() });
        let mut idx_out = self
            .tokens
            .iter()
            .position(|x| &x.mint == if !a_to_b { self.get_a_mint() } else { self.get_b_mint() });
        if let Some(mint_pair) = input.mint_pair {
            idx_in = self
                .tokens
                .iter()
                .position(|x| &x.mint == if a_to_b { &mint_pair[0] } else { &mint_pair[1] });
            idx_out = self
                .tokens
                .iter()
                .position(|x| &x.mint == if !a_to_b { &mint_pair[0] } else { &mint_pair[1] });
        }
        if exact_in {
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| format_err!("Stabble Stable Swap: quote(). {:?}", e))?
                .as_secs();
            // let amplification = self.get_amplification(t as i64);
            if let (Some(idx_in), Some(idx_out)) = (idx_in, idx_out) {
                let in_balance = self.tokens[idx_in].balance;
                if a_to_b && amount_in > in_balance {
                    return Err(format_err!("OutOfLiquidity:StableStableSwap"));
                }
                let result = self.get_swap_result(t as i64, idx_in, idx_out, amount_in);
                if let Some((amount_out, _amount_fee)) = result {
                    return Ok(QuoteResult {
                        amount_a_in: if a_to_b { amount_in } else { 0 },
                        amount_b_in: if !a_to_b { amount_in } else { 0 },
                        amount_a_out: if !a_to_b { amount_out } else { 0 },
                        amount_b_out: if a_to_b { amount_out } else { 0 },
                        indices: None,
                        compute_units: 103_000,
                    });
                } else {
                    return Err(format_err!("CalculationFailed - get_swap_result"));
                }
            } else {
                return Err(format_err!("CalculationFailed - quote"));
            }
        }
        Err(format_err!("NotAllowed (StabbleStableSwap)"))
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        if self.tokens.iter().any(|token| token.mint == WSOL) {
            &WSOL
        } else {
            &self.tokens[0].mint
        }
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        // returns
        if self.tokens.iter().any(|token| token.mint == WSOL) {
            &self
                .tokens
                .iter()
                .find(|token| token.mint != WSOL)
                // TODO: unwrap is safe here, because we check if WSOL exists
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
            .context("Stabble Stable Swap: SwapV2IxArgs");
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
            withdraw_authority: super::get_withdraw_authority_address(&self.vault),
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

pub const STRATEGY_ACCOUNT_DISCM: [u8; 8] = [174, 110, 39, 119, 82, 106, 169, 102];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Strategy {
    pub pool: Pubkey,
    pub is_active: bool,
    pub amp_min_factor: u16,
    pub amp_max_factor: u16,
    pub ramp_min_step: u16,
    pub ramp_max_step: u16,
    pub ramp_min_duration: u32,
    pub ramp_max_duration: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct StrategyAccount(pub Strategy);
impl StrategyAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != STRATEGY_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {STRATEGY_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(Strategy::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&STRATEGY_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const VAULT_ACCOUNT_DISCM: [u8; 8] = [211, 8, 232, 43, 2, 152, 117, 119];
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub struct Vault {
    pub admin: Pubkey,
    pub withdraw_authority: Pubkey,
    pub withdraw_authority_bump: u8,
    pub authority_bump: u8,
    pub is_active: bool,
    pub beneficiary: Pubkey,
    pub beneficiary_fee: u64,
    pub pending_admin: Option<Pubkey>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct VaultAccount(pub Vault);
impl VaultAccount {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        use std::io::Read;
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != VAULT_ACCOUNT_DISCM {
            return Err(std::io::Error::other(format!(
                "discm does not match. Expected: {VAULT_ACCOUNT_DISCM:?}. Received: {maybe_discm:?}"
            )));
        }
        Ok(Self(Vault::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&VAULT_ACCOUNT_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
