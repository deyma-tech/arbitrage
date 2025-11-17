use super::state::{AmmInfo, AmmInstruction, SwapInstructionBaseIn, SwapInstructionBaseOut};
use super::swap::calculate_swap_info;
use super::ID;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

#[allow(clippy::too_many_arguments)]
pub fn swap_base_in(
    amm_program: &Pubkey,
    amm_pool: &Pubkey,
    amm_authority: &Pubkey,
    amm_open_orders: &Pubkey,
    amm_coin_vault: &Pubkey,
    amm_pc_vault: &Pubkey,
    market_program: &Pubkey,
    market: &Pubkey,
    market_bids: &Pubkey,
    market_asks: &Pubkey,
    market_event_queue: &Pubkey,
    market_coin_vault: &Pubkey,
    market_pc_vault: &Pubkey,
    market_vault_signer: &Pubkey,
    user_token_source: &Pubkey,
    user_token_destination: &Pubkey,
    user_source_owner: &Pubkey,

    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<Instruction, ProgramError> {
    let data = AmmInstruction::SwapBaseIn(SwapInstructionBaseIn {
        amount_in,
        minimum_amount_out,
    })
    .pack()?;

    let accounts = vec![
        // spl token
        AccountMeta::new_readonly(spl_token::id(), false),
        // amm
        AccountMeta::new(*amm_pool, false),
        AccountMeta::new_readonly(*amm_authority, false),
        AccountMeta::new(*amm_open_orders, false),
        // AccountMeta::new(*amm_target_orders, false),
        AccountMeta::new(*amm_coin_vault, false),
        AccountMeta::new(*amm_pc_vault, false),
        // market
        AccountMeta::new_readonly(*market_program, false),
        AccountMeta::new(*market, false),
        AccountMeta::new(*market_bids, false),
        AccountMeta::new(*market_asks, false),
        AccountMeta::new(*market_event_queue, false),
        AccountMeta::new(*market_coin_vault, false),
        AccountMeta::new(*market_pc_vault, false),
        AccountMeta::new_readonly(*market_vault_signer, false),
        // user
        AccountMeta::new(*user_token_source, false),
        AccountMeta::new(*user_token_destination, false),
        AccountMeta::new_readonly(*user_source_owner, true),
    ];

    Ok(Instruction {
        program_id: *amm_program,
        accounts,
        data,
    })
}

/// Creates a 'swap base out' instruction.
#[allow(clippy::too_many_arguments)]
pub fn swap_base_out(
    amm_program: &Pubkey,
    amm_pool: &Pubkey,
    amm_authority: &Pubkey,
    amm_open_orders: &Pubkey,
    amm_coin_vault: &Pubkey,
    amm_pc_vault: &Pubkey,
    market_program: &Pubkey,
    market: &Pubkey,
    market_bids: &Pubkey,
    market_asks: &Pubkey,
    market_event_queue: &Pubkey,
    market_coin_vault: &Pubkey,
    market_pc_vault: &Pubkey,
    market_vault_signer: &Pubkey,
    user_token_source: &Pubkey,
    user_token_destination: &Pubkey,
    user_source_owner: &Pubkey,

    max_amount_in: u64,
    amount_out: u64,
) -> Result<Instruction, ProgramError> {
    let data = AmmInstruction::SwapBaseOut(SwapInstructionBaseOut {
        max_amount_in,
        amount_out,
    })
    .pack()?;

    let accounts = vec![
        // spl token
        AccountMeta::new_readonly(spl_token::id(), false),
        // amm
        AccountMeta::new(*amm_pool, false),
        AccountMeta::new_readonly(*amm_authority, false),
        AccountMeta::new(*amm_open_orders, false),
        // AccountMeta::new(*amm_target_orders, false),
        AccountMeta::new(*amm_coin_vault, false),
        AccountMeta::new(*amm_pc_vault, false),
        // market
        AccountMeta::new_readonly(*market_program, false),
        AccountMeta::new(*market, false),
        AccountMeta::new(*market_bids, false),
        AccountMeta::new(*market_asks, false),
        AccountMeta::new(*market_event_queue, false),
        AccountMeta::new(*market_coin_vault, false),
        AccountMeta::new(*market_pc_vault, false),
        AccountMeta::new_readonly(*market_vault_signer, false),
        // user
        AccountMeta::new(*user_token_source, false),
        AccountMeta::new(*user_token_destination, false),
        AccountMeta::new_readonly(*user_source_owner, true),
    ];

    Ok(Instruction {
        program_id: *amm_program,
        accounts,
        data,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn swap_ix(
    payer_pubkey: Pubkey,
    amm_info_id: Pubkey,
    amm_info: &AmmInfo,
    pc_vault_amount: u64,   // &TokenAccount,
    coin_vault_amount: u64, //  &TokenAccount,
    input_token: Pubkey,
    amount_specified: u64,
    slippage_bps: u64,
    base_in: bool,
) -> anyhow::Result<Option<Vec<Instruction>>> {
    let result = calculate_swap_info(
        amm_info_id,
        amm_info,
        pc_vault_amount,
        coin_vault_amount,
        input_token,
        amount_specified,
        slippage_bps,
        base_in,
    )?;
    let mut instructions = vec![];
    let create_user_output_token_instr =
        utils::instructions::create_ata_token_or_not(&payer_pubkey, &result.output_mint, &payer_pubkey, None);
    instructions.extend(vec![create_user_output_token_instr]);

    let user_input_token = spl_associated_token_account::get_associated_token_address(&payer_pubkey, &input_token);

    let user_output_token =
        spl_associated_token_account::get_associated_token_address(&payer_pubkey, &result.output_mint);

    let swap_instruction = if base_in {
        swap_base_in(
            &ID,
            &result.pool_id,
            &result.amm_authority,
            &result.amm_open_orders,
            &result.amm_coin_vault,
            &result.amm_pc_vault,
            &result.market_program,
            &result.market,
            &result.market_bids,
            &result.market_asks,
            &result.market_event_queue,
            &result.market_coin_vault,
            &result.market_pc_vault,
            &result.market_vault_signer,
            &user_input_token,
            &user_output_token,
            &payer_pubkey,
            result.amount_specified,
            result.other_amount_threshold,
        )?
    } else {
        swap_base_out(
            &ID,
            &result.pool_id,
            &result.amm_authority,
            &result.amm_open_orders,
            &result.amm_coin_vault,
            &result.amm_pc_vault,
            &result.market_program,
            &result.market,
            &result.market_bids,
            &result.market_asks,
            &result.market_event_queue,
            &result.market_coin_vault,
            &result.market_pc_vault,
            &result.market_vault_signer,
            &user_input_token,
            &user_output_token,
            &payer_pubkey,
            result.other_amount_threshold,
            result.amount_specified,
        )?
    };
    instructions.extend(vec![swap_instruction]);
    Ok(Some(instructions))
}
