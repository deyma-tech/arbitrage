//! Host-side instruction builder for the isolated PumpSwap/DLMM executor V2.
//!
//! This module deliberately does not send transactions. It packages the two
//! already-quoted venue legs into the fixed ABI consumed by
//! `arb_executor_v2`.

use crate::calculator::{CalculatorEnum, MeteoraDlmmData, PumpAmmData};
use ahash::AHashSet;
use anyhow::{bail, Result};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use utils::constants::WSOL;

pub const EXECUTOR_V2_PROGRAM_ID: Pubkey = pubkey!("HPXVR7EQc1KE9XAY4wPoTakBTFgs4SNZW75zBaVfufW3");
pub const DLMM_TO_PUMPSWAP_TAG: [u8; 8] = *b"arbv2d02";
pub const PUMPSWAP_TO_DLMM_TAG: [u8; 8] = *b"arbv2p02";

const PUMP_AMM: Pubkey = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
const PUMP_FEE_PROGRAM: Pubkey = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
const PUMP_GLOBAL_CONFIG: Pubkey = pubkey!("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");
const DLMM_PROGRAM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
const MEMO_PROGRAM: Pubkey = pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
const ASSOCIATED_TOKEN_PROGRAM: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const SYSTEM_PROGRAM: Pubkey = pubkey!("11111111111111111111111111111111");
const TOKEN_PROGRAM: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_2022_PROGRAM: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const BREAKING_RECIPIENT: Pubkey = pubkey!("5YxQFdt3Tr9zJLvkFccqXVUwhdTWJQc1fFg2YPbxvxeD");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    PumpToDlmm,
    DlmmToPump,
}

/// Builds the exact V2 outer instruction from the existing two-leg quote.
/// `amounts` must be `[input, first_output, final_output]` and
/// `remaining_accounts` must contain the two quote-side account lists.
pub fn build_instruction(
    calculators: &[Box<CalculatorEnum>],
    starting_mint: &Pubkey,
    amounts: &[u64],
    remaining_accounts: &[Vec<Pubkey>],
    payer: Pubkey,
    executor_program: Pubkey,
    minimum_profit: u64,
    allowed_token2022: &AHashSet<Pubkey>,
) -> Result<Instruction> {
    if calculators.len() != 2 || amounts.len() < 3 || remaining_accounts.len() < 2 {
        bail!("executor V2 requiere exactamente dos patas y tres cantidades");
    }
    if amounts[0] == 0 || amounts[1] == 0 || amounts[2] == 0 {
        bail!("executor V2 no acepta cantidades cero");
    }
    if *starting_mint != WSOL {
        bail!("executor V2 requiere iniciar y terminar en WSOL");
    }
    let (pump, dlmm, direction) = match (calculators[0].as_ref(), calculators[1].as_ref()) {
        (CalculatorEnum::PumpAmm(pump), CalculatorEnum::MeteoraDlmm(dlmm)) => (pump, dlmm, Direction::PumpToDlmm),
        (CalculatorEnum::MeteoraDlmm(dlmm), CalculatorEnum::PumpAmm(pump)) => (pump, dlmm, Direction::DlmmToPump),
        _ => bail!("executor V2 solo soporta PumpSwap <-> Meteora DLMM"),
    };
    if pump.pool.quote_mint != WSOL {
        bail!("la pool PumpSwap no usa WSOL como quote mint");
    }
    if pump.pool.base_mint != dlmm.lb_pair.token_x_mint && pump.pool.base_mint != dlmm.lb_pair.token_y_mint {
        bail!("PumpSwap y DLMM no comparten el mint intermedio");
    }

    let vault = vault_pda(executor_program, payer, pump.pool.base_mint);
    let wsol_ata = get_associated_token_address_with_program_id(&payer, &WSOL, &TOKEN_PROGRAM);
    let pump_accounts = build_pump_accounts(
        pump,
        payer,
        vault,
        wsol_ata,
        direction == Direction::PumpToDlmm,
        allowed_token2022,
    )?;
    let dlmm_accounts = build_dlmm_accounts(
        dlmm,
        payer,
        if direction == Direction::PumpToDlmm {
            vault
        } else {
            wsol_ata
        },
        if direction == Direction::PumpToDlmm {
            wsol_ata
        } else {
            vault
        },
        &remaining_accounts[if direction == Direction::PumpToDlmm { 1 } else { 0 }],
        allowed_token2022,
    )?;
    if pump_accounts.len() > u8::MAX as usize || dlmm_accounts.len() > u8::MAX as usize {
        bail!("executor V2 excede el límite de cuentas");
    }

    let mut accounts = Vec::with_capacity(pump_accounts.len() + dlmm_accounts.len());
    let mut data = Vec::new();
    match direction {
        Direction::PumpToDlmm => {
            data.extend_from_slice(&PUMPSWAP_TO_DLMM_TAG);
            data.extend_from_slice(&amounts[0].to_le_bytes());
            data.extend_from_slice(&amounts[0].to_le_bytes());
            data.extend_from_slice(&minimum_profit.to_le_bytes());
            data.extend_from_slice(&amounts[2].to_le_bytes());
            data.extend_from_slice(&amounts[1].to_le_bytes());
            data.push(pump_accounts.len() as u8);
            data.push(dlmm_accounts.len() as u8);
            accounts.extend(pump_accounts);
            accounts.extend(dlmm_accounts);
        }
        Direction::DlmmToPump => {
            data.extend_from_slice(&DLMM_TO_PUMPSWAP_TAG);
            data.extend_from_slice(&amounts[0].to_le_bytes());
            data.extend_from_slice(&minimum_profit.to_le_bytes());
            data.extend_from_slice(&amounts[1].to_le_bytes());
            data.push(dlmm_accounts.len() as u8);
            data.push(pump_accounts.len() as u8);
            accounts.extend(dlmm_accounts);
            accounts.extend(pump_accounts);
        }
    }
    Ok(Instruction {
        program_id: executor_program,
        accounts,
        data,
    })
}

pub fn vault_pda(program: Pubkey, payer: Pubkey, mint: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", payer.as_ref(), mint.as_ref()], &program).0
}

fn build_dlmm_accounts(
    data: &MeteoraDlmmData,
    payer: Pubkey,
    input: Pubkey,
    output: Pubkey,
    bin_arrays: &[Pubkey],
    allowed_token2022: &AHashSet<Pubkey>,
) -> Result<Vec<AccountMeta>> {
    if bin_arrays.is_empty() || bin_arrays.len() > 16 {
        bail!("DLMM necesita entre 1 y 16 bin arrays");
    }
    let pair = &data.lb_pair;
    let x_program = token_program(pair.token_mint_x_program_flag, allowed_token2022, &pair.token_x_mint);
    let y_program = token_program(pair.token_mint_y_program_flag, allowed_token2022, &pair.token_y_mint);
    let bitmap = data.bitmap_extension_pubkey.unwrap_or(DLMM_PROGRAM);
    let event_authority = Pubkey::find_program_address(&[b"__event_authority"], &DLMM_PROGRAM).0;
    let mut accounts = vec![
        AccountMeta::new(data.lb_pair_pubkey, false),
        AccountMeta::new(bitmap, false),
        AccountMeta::new(pair.reserve_x, false),
        AccountMeta::new(pair.reserve_y, false),
        AccountMeta::new(input, false),
        AccountMeta::new(output, false),
        AccountMeta::new_readonly(pair.token_x_mint, false),
        AccountMeta::new_readonly(pair.token_y_mint, false),
        AccountMeta::new(pair.oracle, false),
        AccountMeta::new(DLMM_PROGRAM, false),
        AccountMeta::new_readonly(payer, true),
        AccountMeta::new_readonly(x_program, false),
        AccountMeta::new_readonly(y_program, false),
        AccountMeta::new_readonly(MEMO_PROGRAM, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(DLMM_PROGRAM, false),
    ];
    accounts.extend(bin_arrays.iter().copied().map(|key| AccountMeta::new(key, false)));
    Ok(accounts)
}

fn build_pump_accounts(
    data: &PumpAmmData,
    payer: Pubkey,
    user_base: Pubkey,
    user_quote: Pubkey,
    buy: bool,
    allowed_token2022: &AHashSet<Pubkey>,
) -> Result<Vec<AccountMeta>> {
    let pool = &data.pool;
    let base_program = token_program(0, allowed_token2022, &pool.base_mint);
    let quote_program = TOKEN_PROGRAM;
    let creator_authority = Pubkey::find_program_address(&[b"creator_vault", pool.coin_creator.as_ref()], &PUMP_AMM).0;
    let creator_ata =
        get_associated_token_address_with_program_id(&creator_authority, &pool.quote_mint, &quote_program);
    let recipients = if pool.is_mayhem {
        &data.config.reserved_protocol_fee_recipients
    } else {
        &data.config.protocol_fee_recipients
    };
    let protocol_recipient = recipients
        .iter()
        .copied()
        .find(|recipient| *recipient != Pubkey::default())
        .ok_or_else(|| anyhow::anyhow!("PumpSwap GlobalConfig sin protocol recipient válido"))?;
    let protocol_ata =
        get_associated_token_address_with_program_id(&protocol_recipient, &pool.quote_mint, &quote_program);
    let user_volume = Pubkey::find_program_address(&[b"user_volume_accumulator", payer.as_ref()], &PUMP_AMM).0;
    let fee_config = Pubkey::find_program_address(&[b"fee_config", PUMP_AMM.as_ref()], &PUMP_FEE_PROGRAM).0;
    let event_authority = dex::pump_amm::EVENT_AUTHORITY;
    let mut accounts = vec![
        AccountMeta::new(data.pool_pubkey, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(PUMP_GLOBAL_CONFIG, false),
        AccountMeta::new_readonly(pool.base_mint, false),
        AccountMeta::new_readonly(pool.quote_mint, false),
        AccountMeta::new(user_base, false),
        AccountMeta::new(user_quote, false),
        AccountMeta::new(pool.pool_base_token_account, false),
        AccountMeta::new(pool.pool_quote_token_account, false),
        AccountMeta::new_readonly(protocol_recipient, false),
        AccountMeta::new(protocol_ata, false),
        AccountMeta::new_readonly(base_program, false),
        AccountMeta::new_readonly(quote_program, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(PUMP_AMM, false),
        AccountMeta::new(creator_ata, false),
        AccountMeta::new_readonly(creator_authority, false),
    ];
    if buy {
        accounts.push(AccountMeta::new(dex::pump_amm::GLOBAL_VOLUME_ACUMULATOR, false));
        accounts.push(AccountMeta::new(user_volume, false));
        accounts.push(AccountMeta::new_readonly(fee_config, false));
        accounts.push(AccountMeta::new_readonly(PUMP_FEE_PROGRAM, false));
    } else {
        accounts.push(AccountMeta::new_readonly(fee_config, false));
        accounts.push(AccountMeta::new_readonly(PUMP_FEE_PROGRAM, false));
    }
    if pool.is_cashback {
        accounts.push(AccountMeta::new(
            get_associated_token_address_with_program_id(&user_volume, &pool.quote_mint, &quote_program),
            false,
        ));
        if !buy {
            accounts.push(AccountMeta::new(user_volume, false));
        }
    }
    if pool.coin_creator != Pubkey::default() {
        accounts.push(AccountMeta::new_readonly(
            Pubkey::find_program_address(&[b"pool-v2", pool.base_mint.as_ref()], &PUMP_AMM).0,
            false,
        ));
    }
    accounts.push(AccountMeta::new_readonly(BREAKING_RECIPIENT, false));
    accounts.push(AccountMeta::new(
        get_associated_token_address_with_program_id(&BREAKING_RECIPIENT, &WSOL, &quote_program),
        false,
    ));
    Ok(accounts)
}

fn token_program(flag: u8, allowed: &AHashSet<Pubkey>, mint: &Pubkey) -> Pubkey {
    if flag == 1 || allowed.contains(mint) {
        TOKEN_2022_PROGRAM
    } else {
        TOKEN_PROGRAM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_is_domain_separated() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        assert_eq!(
            vault_pda(EXECUTOR_V2_PROGRAM_ID, wallet, mint),
            vault_pda(EXECUTOR_V2_PROGRAM_ID, wallet, mint)
        );
        assert_ne!(
            vault_pda(EXECUTOR_V2_PROGRAM_ID, wallet, mint),
            vault_pda(EXECUTOR_V2_PROGRAM_ID, wallet, Pubkey::new_unique())
        );
    }

    #[test]
    fn abi_tags_and_lengths_are_fixed() {
        assert_eq!(&DLMM_TO_PUMPSWAP_TAG, b"arbv2d02");
        assert_eq!(&PUMPSWAP_TO_DLMM_TAG, b"arbv2p02");
    }
}
