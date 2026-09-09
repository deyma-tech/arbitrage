#![no_std]
#![allow(unexpected_cfgs)]

extern crate alloc;

#[cfg(test)]
extern crate std;

use alloc::vec::Vec;
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    log::sol_log_compute_units,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

pub const DLMM_TO_PUMPSWAP_CPI_V2_TAG: [u8; 8] = *b"arbv2d02";
pub const PUMPSWAP_TO_DLMM_CPI_V2_TAG: [u8; 8] = *b"arbv2p02";

const PUMP_AMM: Pubkey = solana_program::pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
const PUMP_FEE_PROGRAM: Pubkey = solana_program::pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
const GLOBAL_CONFIG: Pubkey = solana_program::pubkey!("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");
const WSOL_MINT: Pubkey = solana_program::pubkey!("So11111111111111111111111111111111111111112");
const TOKEN_PROGRAM: Pubkey = solana_program::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_2022_PROGRAM: Pubkey = solana_program::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const SYSTEM_PROGRAM: Pubkey = solana_program::pubkey!("11111111111111111111111111111111");
// Token-2022 mint layout: the first 82 bytes are the base mint, bytes 82..165
// are reserved padding, byte 165 is AccountType::Mint, and TLV extensions
// begin at byte 166.  Keep this boundary identical to the legacy validator.
const TOKEN_2022_MINT_TLV_OFFSET: usize = 166;
const TOKEN_2022_TRANSFER_FEE_CONFIG_EXTENSION: u16 = 1;
const TOKEN_2022_METADATA_POINTER_EXTENSION: u16 = 18;
const TOKEN_2022_TOKEN_METADATA_EXTENSION: u16 = 19;
const TOKEN_2022_TRANSFER_FEE_CONFIG_LEN: usize = 116;
// PumpSwap's fixed account ABI still carries this readonly compatibility
// account. The executor never invokes it and never uses it to create a token
// account; the intermediate account is the executor-owned vault PDA below.
const ASSOCIATED_TOKEN_PROGRAM: Pubkey = solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const DLMM_PROGRAM: Pubkey = solana_program::pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
const MEMO_PROGRAM: Pubkey = solana_program::pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

const SWAP2: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];
const SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
const BITMAP_DISCRIMINATOR: [u8; 8] = [80, 111, 124, 113, 55, 237, 18, 5];
const TOKEN_AMOUNT_OFFSET: usize = 64;
const DLMM_PAIR_STATUS_OFFSET: usize = 82;
const DLMM_X_MINT_OFFSET: usize = 88;
const DLMM_Y_MINT_OFFSET: usize = 120;
const DLMM_RESERVE_X_OFFSET: usize = 152;
const DLMM_RESERVE_Y_OFFSET: usize = 184;
const DLMM_ORACLE_OFFSET: usize = 552;
const DLMM_X_PROGRAM_OFFSET: usize = 880;
const DLMM_Y_PROGRAM_OFFSET: usize = 881;
const MIN_DLMM_ACCOUNTS: usize = 17;
const MAX_DLMM_ACCOUNTS: usize = 32;
const MIN_PUMP_SELL_ACCOUNTS: usize = 23;
const MAX_PUMP_SELL_ACCOUNTS: usize = 26;
const MIN_PUMP_BUY_ACCOUNTS: usize = 25;
const MAX_PUMP_BUY_ACCOUNTS: usize = 28;
const INSTRUCTION_LEN: usize = 34;
const PUMPSWAP_TO_DLMM_INSTRUCTION_LEN: usize = 50;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutorV2Error {
    InvalidInstruction = 1,
    InvalidAccountCount = 2,
    InvalidRoute = 3,
    InvalidTokenAccount = 4,
    ZeroIntermediateOutput = 5,
    ProfitInsufficient = 6,
    CpiFailed = 7,
    BalanceOverflow = 8,
}

impl ExecutorV2Error {
    fn program_error(self) -> ProgramError {
        ProgramError::Custom(self as u32)
    }
}

entrypoint!(process_instruction);

fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8]) -> ProgramResult {
    if instruction_data.len() >= 8 && instruction_data[..8] == PUMPSWAP_TO_DLMM_CPI_V2_TAG {
        return process_pumpswap_to_dlmm(program_id, accounts, instruction_data);
    }
    let (q_best, minimum_profit, dlmm_min_out, dlmm_count, sell_count) = decode_instruction(instruction_data)?;
    let expected_count = usize::from(dlmm_count)
        .checked_add(usize::from(sell_count))
        .ok_or_else(|| ExecutorV2Error::InvalidAccountCount.program_error())?;
    if expected_count != accounts.len() || q_best == 0 || dlmm_min_out == 0 {
        return Err(ExecutorV2Error::InvalidInstruction.program_error());
    }
    let (dlmm, pump) = accounts.split_at(usize::from(dlmm_count));

    ensure_vault(program_id, pump, pump[3].key, pump[11].key)?;
    // The vault is also DLMM's output token account. On first use it does not
    // exist yet, so create/initialize it before DLMM validates token-account
    // ownership and mint metadata.
    let input_is_y = validate_dlmm(dlmm)?;
    validate_pump_sell(pump)?;
    validate_shared_route(dlmm, pump, input_is_y, false)?;

    let (input_program_index, output_program_index) = if input_is_y { (12, 11) } else { (11, 12) };
    let initial_wsol = read_token_amount(&dlmm[4], dlmm[input_program_index].key)?;
    let initial_intermediate = read_token_amount(&dlmm[5], dlmm[output_program_index].key)?;

    sol_log_compute_units();
    let dlmm_instruction = build_dlmm_instruction(dlmm, q_best, dlmm_min_out);
    invoke(&dlmm_instruction, dlmm).map_err(|_| ExecutorV2Error::CpiFailed.program_error())?;

    let post_intermediate = read_token_amount(&dlmm[5], dlmm[output_program_index].key)?;
    let actual_intermediate = post_intermediate
        .checked_sub(initial_intermediate)
        .ok_or_else(|| ExecutorV2Error::InvalidTokenAccount.program_error())?;
    if actual_intermediate == 0 {
        return Err(ExecutorV2Error::ZeroIntermediateOutput.program_error());
    }

    msg!("arbv2_dlmm_to_pump actual_intermediate={}", actual_intermediate);
    let pump_instruction = build_pump_sell_instruction(pump, actual_intermediate);
    let pre_pump_wsol = read_token_amount(&dlmm[4], dlmm[input_program_index].key)?;
    msg!(
        "arbv2_before_pumpswap initial_wsol={} initial_intermediate={} post_intermediate={} actual_intermediate={} pre_pump_wsol={} amount_in={}",
        initial_wsol,
        initial_intermediate,
        post_intermediate,
        actual_intermediate,
        pre_pump_wsol,
        actual_intermediate
    );
    invoke(&pump_instruction, pump).map_err(|_| ExecutorV2Error::CpiFailed.program_error())?;

    let final_wsol = read_token_amount(&dlmm[4], dlmm[input_program_index].key)?;
    msg!(
        "arbv2_after_pumpswap initial_wsol={} final_wsol={} post_intermediate={} actual_intermediate={}",
        initial_wsol,
        final_wsol,
        post_intermediate,
        actual_intermediate
    );
    let pumpswap_output = calculate_pumpswap_output(pre_pump_wsol, final_wsol)?;
    msg!("arbv2_pumpswap_output={}", pumpswap_output);
    let profit_signed = signed_profit(initial_wsol, final_wsol);
    msg!(
        "arbv2_profit initial_wsol={} final_wsol={} profit={} profit_signed={} minimum_profit={}",
        initial_wsol,
        final_wsol,
        final_wsol.saturating_sub(initial_wsol),
        profit_signed,
        minimum_profit
    );
    guard_profit(initial_wsol, final_wsol, minimum_profit)?;
    sol_log_compute_units();
    Ok(())
}

fn process_pumpswap_to_dlmm(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    instruction_data: &[u8],
) -> ProgramResult {
    let (q_best, max_quote_in, minimum_profit, dlmm_min_out, exact_base_out, pump_count, dlmm_count) =
        decode_pumpswap_to_dlmm_instruction(instruction_data)?;
    let expected_count = usize::from(pump_count)
        .checked_add(usize::from(dlmm_count))
        .ok_or_else(|| ExecutorV2Error::InvalidAccountCount.program_error())?;
    if expected_count != accounts.len() || q_best == 0 || exact_base_out == 0 || dlmm_min_out == 0 {
        return Err(ExecutorV2Error::InvalidInstruction.program_error());
    }
    let (pump, dlmm) = accounts.split_at(usize::from(pump_count));
    ensure_vault(program_id, pump, pump[3].key, pump[11].key)?;
    validate_pump_buy(pump, exact_base_out, max_quote_in)?;
    let input_is_y = validate_dlmm(dlmm)?;
    validate_shared_route(dlmm, pump, input_is_y, true)?;

    let initial_wsol = read_token_amount(&pump[6], pump[12].key)?;
    let initial_intermediate = read_token_amount(&pump[5], pump[11].key)?;

    sol_log_compute_units();
    let pump_instruction = build_pump_buy_instruction(pump, exact_base_out, max_quote_in);
    invoke(&pump_instruction, pump).map_err(|_| ExecutorV2Error::CpiFailed.program_error())?;

    let post_pump_wsol = read_token_amount(&pump[6], pump[12].key)?;
    let actual_pump_input = initial_wsol
        .checked_sub(post_pump_wsol)
        .ok_or_else(|| ExecutorV2Error::InvalidTokenAccount.program_error())?;
    let post_intermediate = read_token_amount(&pump[5], pump[11].key)?;
    let actual_intermediate = post_intermediate
        .checked_sub(initial_intermediate)
        .ok_or_else(|| ExecutorV2Error::InvalidTokenAccount.program_error())?;
    if actual_intermediate == 0 {
        return Err(ExecutorV2Error::ZeroIntermediateOutput.program_error());
    }

    msg!(
        "arbv2_pump_to_dlmm_after_pumpswap initial_wsol={} post_pump_wsol={} initial_intermediate={} post_intermediate={} actual_pump_input={} actual_intermediate={}",
        initial_wsol,
        post_pump_wsol,
        initial_intermediate,
        post_intermediate,
        actual_pump_input,
        actual_intermediate
    );
    let dlmm_instruction = build_dlmm_instruction(dlmm, actual_intermediate, dlmm_min_out);
    invoke(&dlmm_instruction, dlmm).map_err(|_| ExecutorV2Error::CpiFailed.program_error())?;

    let final_wsol = read_token_amount(&pump[6], pump[12].key)?;
    let dlmm_output = final_wsol
        .checked_sub(post_pump_wsol)
        .ok_or_else(|| ExecutorV2Error::BalanceOverflow.program_error())?;
    let profit_signed = signed_profit(initial_wsol, final_wsol);
    msg!(
        "arbv2_pump_to_dlmm_result initial_wsol={} final_wsol={} pump_input={} pump_output={} dlmm_output={} profit_signed={} minimum_profit={}",
        initial_wsol,
        final_wsol,
        actual_pump_input,
        actual_intermediate,
        dlmm_output,
        profit_signed,
        minimum_profit
    );
    guard_profit(initial_wsol, final_wsol, minimum_profit)?;
    sol_log_compute_units();
    Ok(())
}

fn calculate_pumpswap_output(pre_pump_wsol: u64, final_wsol: u64) -> Result<u64, ProgramError> {
    final_wsol
        .checked_sub(pre_pump_wsol)
        .ok_or_else(|| ExecutorV2Error::BalanceOverflow.program_error())
}

fn signed_profit(initial_wsol: u64, final_wsol: u64) -> i128 {
    i128::from(final_wsol) - i128::from(initial_wsol)
}

fn guard_profit(initial_wsol: u64, final_wsol: u64, minimum_profit: u64) -> Result<i128, ProgramError> {
    let minimum_final_wsol = initial_wsol
        .checked_add(minimum_profit)
        .ok_or_else(|| ExecutorV2Error::BalanceOverflow.program_error())?;
    let profit_signed = signed_profit(initial_wsol, final_wsol);
    if final_wsol < minimum_final_wsol {
        return Err(ExecutorV2Error::ProfitInsufficient.program_error());
    }
    Ok(profit_signed)
}

fn decode_instruction(data: &[u8]) -> Result<(u64, u64, u64, u8, u8), ProgramError> {
    if data.len() != INSTRUCTION_LEN || data[..8] != DLMM_TO_PUMPSWAP_CPI_V2_TAG {
        return Err(ExecutorV2Error::InvalidInstruction.program_error());
    }
    let q_best = read_u64(data, 8)?;
    let minimum_profit = read_u64(data, 16)?;
    let dlmm_min_out = read_u64(data, 24)?;
    Ok((q_best, minimum_profit, dlmm_min_out, data[32], data[33]))
}

fn decode_pumpswap_to_dlmm_instruction(data: &[u8]) -> Result<(u64, u64, u64, u64, u64, u8, u8), ProgramError> {
    if data.len() != PUMPSWAP_TO_DLMM_INSTRUCTION_LEN || data[..8] != PUMPSWAP_TO_DLMM_CPI_V2_TAG {
        return Err(ExecutorV2Error::InvalidInstruction.program_error());
    }
    Ok((
        read_u64(data, 8)?,
        read_u64(data, 16)?,
        read_u64(data, 24)?,
        read_u64(data, 32)?,
        read_u64(data, 40)?,
        data[48],
        data[49],
    ))
}

/// Ensure the intermediate token account is a persistent vault owned by this
/// executor. Creation is part of the same outer instruction, so any later CPI
/// or profit/slippage failure rolls the account creation back atomically.
fn ensure_vault(
    program_id: &Pubkey,
    pump: &[AccountInfo<'_>],
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<(), ProgramError> {
    let vault = &pump[5];
    let payer = &pump[1];
    let system = &pump[13];
    let (expected, bump) = Pubkey::find_program_address(&[b"vault", payer.key.as_ref(), mint.as_ref()], program_id);
    if vault.key != &expected
        || !payer.is_signer
        || !payer.is_writable
        || !vault.is_writable
        || system.key != &SYSTEM_PROGRAM
        || (token_program != &TOKEN_PROGRAM && token_program != &TOKEN_2022_PROGRAM)
    {
        return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
    }

    match classify_vault_account(vault.owner, vault.data_is_empty(), token_program) {
        Ok(VaultAction::Create) => {
            let space = token_account_space(&pump[3], token_program)? as u64;
            let lamports = Rent::get()
                .map_err(|_| ExecutorV2Error::InvalidTokenAccount.program_error())?
                .minimum_balance(space as usize);
            let create = solana_system_interface::instruction::create_account(
                payer.key,
                vault.key,
                lamports,
                space,
                token_program,
            );
            invoke_signed(
                &create,
                &[payer.clone(), vault.clone(), system.clone()],
                &[&[b"vault", payer.key.as_ref(), mint.as_ref(), &[bump]]],
            )
            .map_err(|_| ExecutorV2Error::CpiFailed.program_error())?;
            let mut initialize_data = Vec::with_capacity(33);
            initialize_data.push(18_u8); // InitializeAccount3
            initialize_data.extend_from_slice(&payer.key.to_bytes());
            let initialize = Instruction {
                program_id: *token_program,
                accounts: alloc::vec![
                    AccountMeta::new(*vault.key, false),
                    AccountMeta::new_readonly(*mint, false),
                ],
                data: initialize_data,
            };
            invoke(&initialize, &[vault.clone(), pump[3].clone(), pump[11].clone()])
                .map_err(|_| ExecutorV2Error::CpiFailed.program_error())?;
        }
        Ok(VaultAction::Reuse) => {}
        Err(()) => return Err(ExecutorV2Error::InvalidTokenAccount.program_error()),
    }
    let vault_data = vault
        .try_borrow_data()
        .map_err(|_| ExecutorV2Error::InvalidTokenAccount.program_error())?;
    if !validate_vault_data(vault.owner, &vault_data, mint, payer.key, token_program) {
        return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VaultAction {
    Create,
    Reuse,
}

fn classify_vault_account(owner: &Pubkey, data_is_empty: bool, token_program: &Pubkey) -> Result<VaultAction, ()> {
    if *owner == SYSTEM_PROGRAM && data_is_empty {
        Ok(VaultAction::Create)
    } else if *owner == *token_program {
        Ok(VaultAction::Reuse)
    } else {
        Err(())
    }
}

fn validate_vault_data(
    owner: &Pubkey,
    data: &[u8],
    expected_mint: &Pubkey,
    expected_authority: &Pubkey,
    token_program: &Pubkey,
) -> bool {
    *owner == *token_program
        && read_pubkey_from_slice(data, 0) == Some(*expected_mint)
        // Token accounts store the authority as a plain Pubkey immediately
        // after the mint. COption starts at the delegate field, not here.
        && read_pubkey_from_slice(data, 32) == Some(*expected_authority)
}

fn token_account_space(mint: &AccountInfo<'_>, token_program: &Pubkey) -> Result<usize, ProgramError> {
    let data = mint
        .try_borrow_data()
        .map_err(|_| ExecutorV2Error::InvalidTokenAccount.program_error())?;
    token_account_space_from_data(&data, token_program)
}

fn token_account_space_from_data(data: &[u8], token_program: &Pubkey) -> Result<usize, ProgramError> {
    if data.len() < 82 {
        return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
    }
    if token_program == &TOKEN_PROGRAM {
        return Ok(165);
    }
    if token_program != &TOKEN_2022_PROGRAM {
        return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
    }
    // These mint extensions do not require token-account extensions. Any
    // account extension we do not explicitly understand is rejected rather
    // than allocating an incorrectly sized account.
    if data.len() > 82 {
        if data.len() <= TOKEN_2022_MINT_TLV_OFFSET
            || data[82..165].iter().any(|byte| *byte != 0)
            || data[TOKEN_2022_MINT_TLV_OFFSET - 1] != 1
        {
            return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
        }
        let mut offset = TOKEN_2022_MINT_TLV_OFFSET;
        while offset + 4 <= data.len() {
            let ty = u16::from_le_bytes([data[offset], data[offset + 1]]);
            let len = usize::from(u16::from_le_bytes([data[offset + 2], data[offset + 3]]));
            // Type zero is the TLV padding/terminator used by Token-2022
            // account data.  It does not describe an extension.
            if ty == 0 {
                break;
            }
            offset = offset
                .checked_add(4 + len)
                .ok_or_else(|| ExecutorV2Error::InvalidTokenAccount.program_error())?;
            if offset > data.len() {
                return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
            }
            let allowed = match ty {
                TOKEN_2022_TRANSFER_FEE_CONFIG_EXTENSION => len == TOKEN_2022_TRANSFER_FEE_CONFIG_LEN,
                TOKEN_2022_METADATA_POINTER_EXTENSION => len == 64,
                TOKEN_2022_TOKEN_METADATA_EXTENSION => len > 0,
                _ => false,
            };
            if !allowed {
                return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
            }
        }
        if offset < data.len() && data[offset..].iter().any(|byte| *byte != 0) {
            return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
        }
    }
    Ok(165)
}

fn read_pubkey_from_slice(data: &[u8], offset: usize) -> Option<Pubkey> {
    data.get(offset..offset.checked_add(32)?)
        .and_then(|bytes| bytes.try_into().ok())
        .map(Pubkey::new_from_array)
}

fn read_coption_pubkey(data: &[u8], offset: usize) -> Option<Pubkey> {
    let tag = data.get(offset..offset.checked_add(4)?)?;
    if tag != [1, 0, 0, 0] {
        return None;
    }
    read_pubkey_from_slice(data, offset.checked_add(4)?)
}

fn validate_pump_buy(segment: &[AccountInfo<'_>], exact_base_out: u64, max_quote_in: u64) -> Result<(), ProgramError> {
    if !(MIN_PUMP_BUY_ACCOUNTS..=MAX_PUMP_BUY_ACCOUNTS).contains(&segment.len())
        || !segment[1].is_signer
        || segment[0].owner != &PUMP_AMM
        || segment[2].key != &GLOBAL_CONFIG
        || segment[4].key != &WSOL_MINT
        || segment[11].key != &TOKEN_PROGRAM && segment[11].key != &TOKEN_2022_PROGRAM
        || segment[12].key != &TOKEN_PROGRAM
        || segment[13].key != &SYSTEM_PROGRAM
        || segment[14].key != &ASSOCIATED_TOKEN_PROGRAM
        || segment[16].key != &PUMP_AMM
        || segment[21].key != &fee_config()
        || segment[22].key != &PUMP_FEE_PROGRAM
        || exact_base_out == 0
        || max_quote_in == 0
    {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    }
    let pool_data = segment[0]
        .try_borrow_data()
        .map_err(|_| ExecutorV2Error::InvalidRoute.program_error())?;
    let base_mint = read_pubkey(&pool_data, 43)?;
    let base_vault = read_pubkey(&pool_data, 139)?;
    let quote_vault = read_pubkey(&pool_data, 171)?;
    let coin_creator = read_pubkey(&pool_data, 211)?;
    drop(pool_data);
    let creator_authority = Pubkey::find_program_address(&[b"creator_vault", coin_creator.as_ref()], &PUMP_AMM).0;
    if segment[3].key != &base_mint
        || segment[7].key != &base_vault
        || segment[8].key != &quote_vault
        || segment[18].key != &creator_authority
    {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    }
    if segment[5].owner != segment[11].key
        || segment[6].owner != segment[12].key
        || read_token_mint(&segment[5])? != base_mint
        || read_token_mint(&segment[6])? != WSOL_MINT
    {
        return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
    }
    Ok(())
}

fn fee_config() -> Pubkey {
    Pubkey::find_program_address(&[b"fee_config", PUMP_AMM.as_ref()], &PUMP_FEE_PROGRAM).0
}

fn build_pump_buy_instruction(accounts: &[AccountInfo<'_>], exact_base_out: u64, max_quote_in: u64) -> Instruction {
    let mut data = Vec::with_capacity(25);
    data.extend_from_slice(&[102, 6, 61, 18, 1, 218, 235, 234]);
    data.extend_from_slice(&exact_base_out.to_le_bytes());
    data.extend_from_slice(&max_quote_in.to_le_bytes());
    data.push(0);
    Instruction {
        program_id: PUMP_AMM,
        accounts: account_metas(accounts),
        data,
    }
}

fn validate_dlmm(segment: &[AccountInfo<'_>]) -> Result<bool, ProgramError> {
    if !(MIN_DLMM_ACCOUNTS..=MAX_DLMM_ACCOUNTS).contains(&segment.len())
        || !segment[10].is_signer
        || segment[0].owner != &DLMM_PROGRAM
    {
        return Err(ExecutorV2Error::InvalidAccountCount.program_error());
    }
    let pair_data = segment[0]
        .try_borrow_data()
        .map_err(|_| ExecutorV2Error::InvalidRoute.program_error())?;
    if pair_data.len() <= DLMM_Y_PROGRAM_OFFSET || pair_data[DLMM_PAIR_STATUS_OFFSET] != 0 {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    }
    let x_mint = read_pubkey(&pair_data, DLMM_X_MINT_OFFSET)?;
    let y_mint = read_pubkey(&pair_data, DLMM_Y_MINT_OFFSET)?;
    let reserve_x = read_pubkey(&pair_data, DLMM_RESERVE_X_OFFSET)?;
    let reserve_y = read_pubkey(&pair_data, DLMM_RESERVE_Y_OFFSET)?;
    let oracle = read_pubkey(&pair_data, DLMM_ORACLE_OFFSET)?;
    let x_program = token_program_from_flag(pair_data[DLMM_X_PROGRAM_OFFSET])?;
    let y_program = token_program_from_flag(pair_data[DLMM_Y_PROGRAM_OFFSET])?;
    drop(pair_data);

    if segment[1].key != &DLMM_PROGRAM && !valid_bitmap_extension(&segment[1], segment[0].key)? {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    }
    if segment[2].key != &reserve_x
        || segment[3].key != &reserve_y
        || segment[6].key != &x_mint
        || segment[7].key != &y_mint
        || segment[8].key != &oracle
        || segment[9].key != &DLMM_PROGRAM
        || segment[11].key != &x_program
        || segment[12].key != &y_program
        || segment[13].key != &MEMO_PROGRAM
        || segment[14].key != &event_authority()
        || segment[15].key != &DLMM_PROGRAM
    {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    }

    let input_mint = read_token_mint(&segment[4])?;
    let input_is_y = if input_mint == y_mint {
        true
    } else if input_mint == x_mint {
        false
    } else {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    };
    let output_program = if input_is_y { &x_program } else { &y_program };
    let input_program = if input_is_y { &y_program } else { &x_program };
    if segment[4].owner != input_program || segment[5].owner != output_program || segment[4].key == segment[5].key {
        return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
    }
    for bin_array in &segment[16..] {
        let data = bin_array
            .try_borrow_data()
            .map_err(|_| ExecutorV2Error::InvalidRoute.program_error())?;
        if bin_array.owner != &DLMM_PROGRAM || data.len() < 40 {
            return Err(ExecutorV2Error::InvalidRoute.program_error());
        }
        let index = &data[8..16];
        let expected = Pubkey::find_program_address(&[b"bin_array", segment[0].key.as_ref(), index], &DLMM_PROGRAM).0;
        if bin_array.key != &expected
            || data[24..56.min(data.len())].len() != 32
            || data[24..56] != segment[0].key.to_bytes()
        {
            return Err(ExecutorV2Error::InvalidRoute.program_error());
        }
    }
    Ok(input_is_y)
}

fn validate_pump_sell(segment: &[AccountInfo<'_>]) -> Result<(), ProgramError> {
    if !(MIN_PUMP_SELL_ACCOUNTS..=MAX_PUMP_SELL_ACCOUNTS).contains(&segment.len())
        || !segment[1].is_signer
        || segment[0].owner != &PUMP_AMM
        || segment[2].key != &GLOBAL_CONFIG
        || segment[4].key != &WSOL_MINT
        || segment[11].key != &TOKEN_PROGRAM && segment[11].key != &TOKEN_2022_PROGRAM
        || segment[12].key != &TOKEN_PROGRAM
        || segment[13].key != &SYSTEM_PROGRAM
        || segment[14].key != &ASSOCIATED_TOKEN_PROGRAM
        || segment[16].key != &PUMP_AMM
        || segment[20].key != &PUMP_FEE_PROGRAM
    {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    }
    let pool_data = segment[0]
        .try_borrow_data()
        .map_err(|_| ExecutorV2Error::InvalidRoute.program_error())?;
    let base_mint = read_pubkey(&pool_data, 43)?;
    let base_vault = read_pubkey(&pool_data, 139)?;
    let quote_vault = read_pubkey(&pool_data, 171)?;
    let coin_creator = read_pubkey(&pool_data, 211)?;
    drop(pool_data);
    let creator_authority = Pubkey::find_program_address(&[b"creator_vault", coin_creator.as_ref()], &PUMP_AMM).0;
    let fee_config = Pubkey::find_program_address(&[b"fee_config", PUMP_AMM.as_ref()], &PUMP_FEE_PROGRAM).0;
    if segment[3].key != &base_mint
        || segment[7].key != &base_vault
        || segment[8].key != &quote_vault
        || segment[18].key != &creator_authority
        || segment[19].key != &fee_config
    {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    }
    if segment[5].owner != segment[11].key
        || segment[6].owner != segment[12].key
        || read_token_mint(&segment[5])? != base_mint
        || read_token_mint(&segment[6])? != WSOL_MINT
    {
        return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
    }
    Ok(())
}

fn validate_shared_route(
    dlmm: &[AccountInfo<'_>],
    pump: &[AccountInfo<'_>],
    input_is_y: bool,
    pump_to_dlmm: bool,
) -> Result<(), ProgramError> {
    let (dlmm_input_mint, dlmm_output_mint, dlmm_input_program, dlmm_output_program) = if input_is_y {
        (dlmm[7].key, dlmm[6].key, dlmm[12].key, dlmm[11].key)
    } else {
        (dlmm[6].key, dlmm[7].key, dlmm[11].key, dlmm[12].key)
    };
    let valid_mints_and_accounts = if pump_to_dlmm {
        dlmm_input_mint == pump[3].key
            && dlmm_output_mint == &WSOL_MINT
            && dlmm_input_program == pump[11].key
            && dlmm_output_program == pump[12].key
            && dlmm[4].key == pump[5].key
            && dlmm[5].key == pump[6].key
    } else {
        dlmm_input_mint == &WSOL_MINT
            && dlmm_output_mint == pump[3].key
            && dlmm_input_program == pump[12].key
            && dlmm_output_program == pump[11].key
            && dlmm[4].key == pump[6].key
            && dlmm[5].key == pump[5].key
    };
    if !valid_mints_and_accounts || dlmm[10].key != pump[1].key {
        return Err(ExecutorV2Error::InvalidRoute.program_error());
    }
    Ok(())
}

fn valid_bitmap_extension(account: &AccountInfo<'_>, pair: &Pubkey) -> Result<bool, ProgramError> {
    let expected = Pubkey::find_program_address(&[b"bitmap", pair.as_ref()], &DLMM_PROGRAM).0;
    let data = account
        .try_borrow_data()
        .map_err(|_| ExecutorV2Error::InvalidRoute.program_error())?;
    Ok(account.key == &expected
        && account.owner == &DLMM_PROGRAM
        && data.len() >= 40
        && data[..8] == BITMAP_DISCRIMINATOR
        && data[8..40] == pair.to_bytes())
}

fn build_dlmm_instruction(accounts: &[AccountInfo<'_>], amount: u64, min_out: u64) -> Instruction {
    let mut data = Vec::with_capacity(28);
    data.extend_from_slice(&SWAP2);
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&min_out.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    Instruction {
        program_id: DLMM_PROGRAM,
        accounts: account_metas(accounts),
        data,
    }
}

fn build_pump_sell_instruction(accounts: &[AccountInfo<'_>], amount: u64) -> Instruction {
    let mut data = Vec::with_capacity(24);
    data.extend_from_slice(&SELL);
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    Instruction {
        program_id: PUMP_AMM,
        accounts: account_metas(accounts),
        data,
    }
}

fn account_metas(accounts: &[AccountInfo<'_>]) -> Vec<AccountMeta> {
    accounts
        .iter()
        .map(|account| {
            if account.is_writable {
                AccountMeta::new(*account.key, account.is_signer)
            } else {
                AccountMeta::new_readonly(*account.key, account.is_signer)
            }
        })
        .collect()
}

fn read_token_amount(account: &AccountInfo<'_>, program: &Pubkey) -> Result<u64, ProgramError> {
    if account.owner != program {
        return Err(ExecutorV2Error::InvalidTokenAccount.program_error());
    }
    let data = account
        .try_borrow_data()
        .map_err(|_| ExecutorV2Error::InvalidTokenAccount.program_error())?;
    read_u64_at(&data, TOKEN_AMOUNT_OFFSET)
}

fn read_token_mint(account: &AccountInfo<'_>) -> Result<Pubkey, ProgramError> {
    let data = account
        .try_borrow_data()
        .map_err(|_| ExecutorV2Error::InvalidTokenAccount.program_error())?;
    read_pubkey(&data, 0)
}

fn token_program_from_flag(flag: u8) -> Result<Pubkey, ProgramError> {
    match flag {
        0 => Ok(TOKEN_PROGRAM),
        1 => Ok(TOKEN_2022_PROGRAM),
        _ => Err(ExecutorV2Error::InvalidRoute.program_error()),
    }
}

fn event_authority() -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], &DLMM_PROGRAM).0
}

fn read_pubkey(data: &[u8], offset: usize) -> Result<Pubkey, ProgramError> {
    let bytes = data
        .get(offset..offset + 32)
        .ok_or_else(|| ExecutorV2Error::InvalidRoute.program_error())?;
    Ok(Pubkey::new_from_array(bytes.try_into().expect("32-byte slice")))
}

fn read_u64(data: &[u8], offset: usize) -> Result<u64, ProgramError> {
    let bytes = data
        .get(offset..offset + 8)
        .ok_or_else(|| ExecutorV2Error::InvalidInstruction.program_error())?;
    Ok(u64::from_le_bytes(bytes.try_into().expect("8-byte slice")))
}

fn read_u64_at(data: &[u8], offset: usize) -> Result<u64, ProgramError> {
    read_u64(data, offset).map_err(|_| ExecutorV2Error::InvalidTokenAccount.program_error())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn v2_tag_is_distinct_from_legacy_tags() {
        assert_ne!(DLMM_TO_PUMPSWAP_CPI_V2_TAG, *b"arbexd02");
        assert_ne!(PUMPSWAP_TO_DLMM_CPI_V2_TAG, *b"arbexd01");
    }

    #[test]
    fn instruction_layout_is_fixed_and_little_endian() {
        let mut data = Vec::from(DLMM_TO_PUMPSWAP_CPI_V2_TAG);
        data.extend_from_slice(&7u64.to_le_bytes());
        data.extend_from_slice(&11u64.to_le_bytes());
        data.extend_from_slice(&13u64.to_le_bytes());
        data.extend_from_slice(&17u8.to_le_bytes());
        data.extend_from_slice(&23u8.to_le_bytes());
        assert_eq!(decode_instruction(&data).unwrap(), (7, 11, 13, 17, 23));
    }

    #[test]
    fn malformed_instruction_is_rejected() {
        assert!(decode_instruction(&DLMM_TO_PUMPSWAP_CPI_V2_TAG).is_err());
    }

    #[test]
    fn pumpswap_to_dlmm_instruction_layout_is_fixed() {
        let mut data = Vec::from(PUMPSWAP_TO_DLMM_CPI_V2_TAG);
        data.extend_from_slice(&7u64.to_le_bytes());
        data.extend_from_slice(&11u64.to_le_bytes());
        data.extend_from_slice(&13u64.to_le_bytes());
        data.extend_from_slice(&17u64.to_le_bytes());
        data.extend_from_slice(&19u64.to_le_bytes());
        data.push(23);
        data.push(29);
        assert_eq!(
            decode_pumpswap_to_dlmm_instruction(&data).unwrap(),
            (7, 11, 13, 17, 19, 23, 29)
        );
    }

    #[test]
    fn pumpswap_to_dlmm_layout_rejects_truncated_data() {
        let mut data = Vec::from(PUMPSWAP_TO_DLMM_CPI_V2_TAG);
        data.extend_from_slice(&[0; PUMPSWAP_TO_DLMM_INSTRUCTION_LEN - 9]);
        assert!(decode_pumpswap_to_dlmm_instruction(&data).is_err());
    }

    #[test]
    fn roundtrip_accounting_uses_wsol_delta_not_intermediate_balance() {
        let pre_pump_wsol = 4_000_000_000;
        let final_wsol = 5_100_000_000;
        let post_intermediate = 100_000_000_000;

        let result = calculate_pumpswap_output(pre_pump_wsol, final_wsol);

        assert_eq!(result.unwrap(), 1_100_000_000);
        assert!(final_wsol.checked_sub(post_intermediate).is_none());
    }

    #[test]
    fn pumpswap_output_fails_closed_on_impossible_wsol_decrease() {
        assert!(calculate_pumpswap_output(4_000, 3_999).is_err());
        assert!(calculate_pumpswap_output(6_000, 5_500).is_err());
    }

    #[test]
    fn profit_above_minimum_succeeds() {
        assert_eq!(guard_profit(100, 111, 10).unwrap(), 11);
    }

    #[test]
    fn profit_below_minimum_returns_profit_insufficient() {
        assert_eq!(
            guard_profit(100, 109, 10),
            Err(ExecutorV2Error::ProfitInsufficient.program_error())
        );
    }

    #[test]
    fn profit_exactly_at_minimum_succeeds() {
        assert_eq!(guard_profit(100, 110, 10).unwrap(), 10);
    }

    #[test]
    fn canary_minimum_of_one_accepts_any_strictly_positive_profit() {
        assert_eq!(guard_profit(100, 101, 1).unwrap(), 1);
        assert_eq!(
            guard_profit(100, 100, 1),
            Err(ExecutorV2Error::ProfitInsufficient.program_error())
        );
    }

    #[test]
    fn loss_returns_profit_insufficient_with_signed_profit() {
        assert_eq!(signed_profit(100, 96), -4);
        assert_eq!(
            guard_profit(100, 96, 1),
            Err(ExecutorV2Error::ProfitInsufficient.program_error())
        );
    }

    #[test]
    fn profit_threshold_overflow_fails_closed() {
        assert_eq!(
            guard_profit(u64::MAX, u64::MAX, 1),
            Err(ExecutorV2Error::BalanceOverflow.program_error())
        );
    }

    #[test]
    fn vault_pda_is_stable_and_domain_separated() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let program = Pubkey::new_unique();
        let a = Pubkey::find_program_address(&[b"vault", wallet.as_ref(), mint.as_ref()], &program).0;
        let b = Pubkey::find_program_address(&[b"vault", wallet.as_ref(), mint.as_ref()], &program).0;
        assert_eq!(a, b);
        assert_ne!(
            a,
            Pubkey::find_program_address(&[b"vault", mint.as_ref(), wallet.as_ref()], &program).0
        );
    }

    #[test]
    fn token_account_space_supports_spl_and_plain_token_2022() {
        assert_eq!(token_account_space_from_data(&[0; 82], &TOKEN_PROGRAM).unwrap(), 165);
        assert_eq!(
            token_account_space_from_data(&[0; 82], &TOKEN_2022_PROGRAM).unwrap(),
            165
        );
        // Extended Token-2022 mints have padding and AccountType::Mint before
        // the TLV stream; these profiles were already supported economically.
        let mut transfer_fee_mint = vec![0u8; TOKEN_2022_MINT_TLV_OFFSET + 4 + 116];
        transfer_fee_mint[165] = 1;
        transfer_fee_mint[166..168].copy_from_slice(&TOKEN_2022_TRANSFER_FEE_CONFIG_EXTENSION.to_le_bytes());
        transfer_fee_mint[168..170].copy_from_slice(&116u16.to_le_bytes());
        assert_eq!(
            token_account_space_from_data(&transfer_fee_mint, &TOKEN_2022_PROGRAM).unwrap(),
            165
        );
        let mut metadata_mint = vec![0u8; TOKEN_2022_MINT_TLV_OFFSET + 4 + 64];
        metadata_mint[165] = 1;
        metadata_mint[166..168].copy_from_slice(&TOKEN_2022_METADATA_POINTER_EXTENSION.to_le_bytes());
        metadata_mint[168..170].copy_from_slice(&64u16.to_le_bytes());
        assert_eq!(
            token_account_space_from_data(&metadata_mint, &TOKEN_2022_PROGRAM).unwrap(),
            165
        );
    }

    #[test]
    fn token_2022_unknown_extension_is_rejected() {
        let mut mint = vec![0; TOKEN_2022_MINT_TLV_OFFSET + 4 + 1];
        mint[165] = 1;
        mint[166..168].copy_from_slice(&99u16.to_le_bytes());
        mint[168..170].copy_from_slice(&1u16.to_le_bytes());
        assert!(token_account_space_from_data(&mint, &TOKEN_2022_PROGRAM).is_err());
    }

    #[test]
    fn first_use_is_create_and_later_use_is_reuse() {
        assert_eq!(
            classify_vault_account(&SYSTEM_PROGRAM, true, &TOKEN_PROGRAM),
            Ok(VaultAction::Create)
        );
        assert_eq!(
            classify_vault_account(&TOKEN_PROGRAM, false, &TOKEN_PROGRAM),
            Ok(VaultAction::Reuse)
        );
        assert_eq!(
            classify_vault_account(&TOKEN_2022_PROGRAM, false, &TOKEN_2022_PROGRAM),
            Ok(VaultAction::Reuse)
        );
    }

    #[test]
    fn invalid_existing_vault_fails_closed() {
        assert_eq!(classify_vault_account(&SYSTEM_PROGRAM, false, &TOKEN_PROGRAM), Err(()));
        assert_eq!(
            classify_vault_account(&TOKEN_PROGRAM, false, &TOKEN_2022_PROGRAM),
            Err(())
        );
    }

    #[test]
    fn existing_spl_and_token_2022_vaults_validate_mint_and_authority() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let mut account = vec![0u8; 165];
        account[..32].copy_from_slice(mint.as_ref());
        account[32..64].copy_from_slice(authority.as_ref());
        assert!(validate_vault_data(
            &TOKEN_PROGRAM,
            &account,
            &mint,
            &authority,
            &TOKEN_PROGRAM
        ));
        assert!(validate_vault_data(
            &TOKEN_2022_PROGRAM,
            &account,
            &mint,
            &authority,
            &TOKEN_2022_PROGRAM
        ));
        account[32] ^= 1;
        assert!(!validate_vault_data(
            &TOKEN_PROGRAM,
            &account,
            &mint,
            &authority,
            &TOKEN_PROGRAM
        ));
    }
}
