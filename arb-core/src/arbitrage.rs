use crate::calculator::{
    CalculatorEnum, FusionAmmData, MeteoraDammV2Data, MeteoraDlmmData, OrcaData, OrcaSwapV2Data, PumpAmmData,
    RaydiumAmmV4Data, RaydiumClmmV3Data, RaydiumCpmmData, SarosAmmData, SarosDlmmData, StabbleStableSwapData,
    StabbleWeightedSwapData,
};
use crate::compute_units::CU_MAX_UNITS_PER_SWAP;
use crate::instruction::{ArbitrageCompressedInstructionInput, IxBuilder};
use ahash::{AHashSet, HashMap};
use anyhow::{format_err, Context};
use dex::fusion_amm::get_tick_array_address_fusion;
use dex::orca::get_tick_array_address;
use dex::orca::TICK_ARRAY_SIZE;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use utils::constants::{ALLOWED_TOKEN_2022, STATIC_ATA};
use utils::now;
use utils::pool::Pool;
use utils::pool::PoolType;
use utils::quote::QuoteResult;
use utils::rnd::rnd_index;
use utils::types::MintPair;

const MIN_MULTIPLICATOR: u64 = 10;
const MAX_MULTIPLICATOR: u64 = 130;

#[derive(Clone, Debug)]
pub struct OpportunityWithCalculators {
    pub slot: u64,
    pub diff: u64,
    pub calculators: Vec<Box<CalculatorEnum>>,
    pub mint_pair_route: Vec<MintPair>,
    pub timestamp: u128,
}

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct Opportunity {
    pub slot: u64,
    pub route: Vec<(Pubkey, PoolType, [u8; 8])>,
    pub diff: u64,
    pub mint_pair_route: Vec<MintPair>,
    pub timestamp: u128,
}

impl Hash for Opportunity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, _, price) in self.route.iter() {
            key.hash(state);
            price.hash(state);
        }
        self.diff.hash(state);
        let timestamp = now::as_secs() / 10;
        timestamp.hash(state);
    }
}

impl PartialOrd for Opportunity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Opportunity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.diff.cmp(&other.diff)
    }
}

impl Opportunity {
    // pub fn get_calculators(&self, data: &GPAResult) -> anyhow::Result<Vec<CalculatorEnum>> {
    //     let mut calculators = vec![];
    //     for (pubkey, pool_type, _) in self.route.iter() {
    //         let calculator = data.get_partial_data(pool_type, pubkey, self.slot);
    //         match calculator {
    //             None => Err(format_err!("DataNotFound"))?,
    //             Some(calculator) => calculators.push(calculator),
    //         }
    //     }
    //     Ok(calculators)
    // }

    pub fn to_opportunity_with_calculators(
        &self,
        pubkey_to_calculators: &HashMap<Pubkey, Box<CalculatorEnum>>,
    ) -> anyhow::Result<OpportunityWithCalculators> {
        let mut calculators = vec![];
        for (pubkey, _pool_type, _) in self.route.iter() {
            let calculator = pubkey_to_calculators.get(pubkey);
            match calculator {
                None => {
                    Err(format_err!("DataNotFound"))?;
                }
                Some(calculator) => calculators.push(calculator.clone()),
            }
        }
        Ok(OpportunityWithCalculators {
            slot: self.slot,
            diff: self.diff,
            calculators,
            mint_pair_route: self.mint_pair_route.clone(),
            timestamp: self.timestamp,
        })
    }
}

pub fn lookup_ata(mint_to_ata: &mut HashMap<Pubkey, Pubkey>, wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    if let Some(ata) = mint_to_ata.get(mint) {
        *ata
    } else {
        let ata = get_associated_token_address_with_program_id(
            wallet,
            mint,
            if ALLOWED_TOKEN_2022.contains(mint) {
                &spl_token_2022::ID
            } else {
                &spl_token::ID
            },
        );
        mint_to_ata.insert(*mint, ata);
        ata
    }
}

pub fn lookup_ata_v2(
    mint_to_ata: &mut HashMap<Pubkey, Pubkey>,
    wallet: &Pubkey,
    mint: &Pubkey,
    allowed_token2022: &AHashSet<Pubkey>,
) -> Pubkey {
    if let Some(ata) = mint_to_ata.get(mint) {
        *ata
    } else {
        let ata = get_associated_token_address_with_program_id(
            wallet,
            mint,
            if allowed_token2022.contains(mint) {
                &spl_token_2022::ID
            } else {
                &spl_token::ID
            },
        );
        mint_to_ata.insert(*mint, ata);
        ata
    }
}

#[allow(clippy::too_many_arguments)]
pub fn process_arbitrage_v5(
    slot: u64,
    amount: u64,
    partial_data: &[Box<CalculatorEnum>],
    starting_mint: &Pubkey,
    builder: &mut IxBuilder,
    mint_to_ata: &mut HashMap<Pubkey, Pubkey>,
    mint_pair_route: &[&MintPair],
    allowed_token2022: &AHashSet<Pubkey>,
) -> anyhow::Result<(i64, u32, ArbitrageCompressedInstructionInput)> {
    if partial_data.len() < 2 {
        return Err(format_err!("NotEnoughPools"));
    }
    let mut partial_data_deque = partial_data.iter().rev().collect::<Vec<_>>();
    let first_data = partial_data_deque.pop().context("NotExists")?;
    let mint_pair = mint_pair_route[0];

    // let (mut amount_out, mut _amount_remaining, optionals) =
    //     get_quote_result_for_ix_v2(starting_mint, amount, slot, first_data, mint_pair)?;

    let QuoteResultForIx {
        mut amount_out,
        remaining_amount_out: mut _amount_remaining,
        remaining_accounts: optionals,
        ..
    } = get_quote_result_for_ix_v3(starting_mint, amount, slot, first_data, mint_pair)?;

    //println!("amount: {:?}, amount_out: {:?}", amount, amount_out);

    let mut starting_mint = *starting_mint;
    let mut other_mint = mint_pair[1];

    // let mut source_ata = get_associated_token_address(&builder.owner, &starting_mint);
    let mut source_ata = lookup_ata_v2(mint_to_ata, &builder.owner, &starting_mint, allowed_token2022);

    let _source_ata_initial = source_ata;
    let mut destination_ata = lookup_ata_v2(mint_to_ata, &builder.owner, &other_mint, allowed_token2022);

    let mut preparation = ArbitrageCompressedInstructionInput::new(source_ata);

    let first_ix = get_ixs(
        first_data,
        amount,
        amount_out,
        &starting_mint,
        mint_pair[0] == starting_mint,
        source_ata,
        destination_ata,
        builder.owner,
        optionals,
        allowed_token2022,
    )?;
    // info!("starting mint: {} other mint: {}", starting_mint, other_mint);
    // info!("SATA: {} DATA: {}", source_ata, destination_ata);

    let mints = mint_pair_route.to_vec();
    let mut mints = mints
        .into_iter()
        .flatten()
        .filter(|x| !STATIC_ATA.contains(*x))
        .collect::<Vec<_>>();
    mints.sort();
    mints.dedup();
    for mint in mints.iter() {
        let c_ata_ix = builder.get_create_ata_if_not_exists_v2(mint, allowed_token2022);
        preparation.push(builder.get_simple_swap_ix(c_ata_ix.program_id, c_ata_ix.data, c_ata_ix.accounts));
    }
    // preparation.push(builder.get_simple_swap_ix(
    //     c_ata_ix.program_id,
    //     c_ata_ix.data,
    //     c_ata_ix.accounts,
    // ));

    preparation.push(builder.get_simple_swap_ix(first_ix.program_id, first_ix.data, first_ix.accounts));

    let mut final_result = 0_u64;
    let mut final_compute_units = 0_u32;
    let mut counter = 1;

    // wsol cant be deleted because flashloan cant be repayed
    // to_be_deleted.insert((source_ata, starting_mint));

    while let Some(calculator) = partial_data_deque.pop() {
        starting_mint = other_mint;
        let mint_pair = mint_pair_route[counter];
        other_mint = if mint_pair[0] == starting_mint {
            mint_pair[1]
        } else {
            mint_pair[0]
        };

        source_ata = destination_ata;
        destination_ata = lookup_ata_v2(mint_to_ata, &builder.owner, &other_mint, allowed_token2022);
        // if other_mint != WSOL {
        //     let c_ata_ix = builder.get_create_ata_if_not_exists(&starting_mint);
        //     builder.push_ix(c_ata_ix);
        //     // preparation.push(builder.get_simple_swap_ix(
        //     //     c_ata_ix.program_id,
        //     //     c_ata_ix.data,
        //     //     c_ata_ix.accounts,
        //     // ));
        // }
        // info!("starting mint: {} other mint: {}", starting_mint, other_mint);
        // info!("SATA: {} DATA: {}", source_ata, destination_ata);

        // let (amount_out2, _amount_remaining, optionals) =
        //     get_quote_result_for_ix_v2(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        let QuoteResultForIx {
            amount_out: amount_out2,
            remaining_amount_out: mut _amount_remaining,
            remaining_accounts: optionals,
            compute_units,
        } = get_quote_result_for_ix_v3(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        // println!(
        //     "amount_out: {:?}, amount_out2: {:?}",
        //     amount_out, amount_out2
        // );

        final_result = amount_out2;
        final_compute_units = compute_units;

        let next_ix = get_ixs(
            calculator,
            amount_out,
            amount_out2, //(amount_out2 / 100) * (100 - (counter * 10)),
            &starting_mint,
            mint_pair[0] == starting_mint,
            source_ata,
            destination_ata,
            builder.owner,
            optionals,
            allowed_token2022,
        )?;
        amount_out = amount_out2;
        let ix = builder.get_swap_ix(
            next_ix.program_id,
            next_ix.data,
            next_ix.accounts,
            source_ata,
            calculator.get_pool_type().to_offset(),
        );
        // in reality always close ata if possible ... - maybe check if not empty ...
        // let ix = builder.get_swap_with_close_ata_ix(
        //     next_ix.program_id,
        //     next_ix.data,
        //     next_ix.accounts,
        //     source_ata,
        //     calculator.get_pool_type().to_offset(),
        // );
        preparation.push(ix);
        counter += 1;
    }

    let diff = (final_result as i64) - (amount as i64);
    for mint in mints.iter() {
        let ata = lookup_ata_v2(mint_to_ata, &builder.owner, mint, allowed_token2022);
        let c_ata_ix = if !allowed_token2022.contains(mint) {
            builder.get_close_ata_ix(&ata)
        } else {
            builder.get_close_ata_2022_ix(&ata)
        };
        preparation.push(builder.get_simple_swap_ix(c_ata_ix.program_id, c_ata_ix.data, c_ata_ix.accounts));
    }

    Ok((diff, final_compute_units, preparation))
}

#[allow(clippy::too_many_arguments)]
pub fn process_arbitrage_v6(
    partial_data: &[Box<CalculatorEnum>],
    starting_mint: &Pubkey,
    builder: &mut IxBuilder,
    mint_to_ata: &mut HashMap<Pubkey, Pubkey>,
    mint_pair_route: &[&MintPair],
    allowed_token2022: &AHashSet<Pubkey>,
    amounts: Vec<u64>,
    remaining_accounts: Vec<Vec<Pubkey>>,
) -> anyhow::Result<ArbitrageCompressedInstructionInput> {
    let mut amounts_deque = amounts.into_iter().rev().collect::<Vec<_>>();
    let mut remaining_accounts_deque = remaining_accounts.into_iter().rev().collect::<Vec<_>>();
    let mut partial_data_deque = partial_data.iter().rev().collect::<Vec<_>>();
    let first_data = partial_data_deque.pop().context("Calculators:NotExists")?;
    let mint_pair = mint_pair_route[0];

    let mut starting_mint = *starting_mint;
    let mut other_mint = mint_pair[1];
    let mut source_ata = lookup_ata_v2(mint_to_ata, &builder.owner, &starting_mint, allowed_token2022);

    let mut destination_ata = lookup_ata_v2(mint_to_ata, &builder.owner, &other_mint, allowed_token2022);

    let mut preparation = ArbitrageCompressedInstructionInput::new(source_ata);

    let amount = amounts_deque.pop().context("Amount:NotExists")?;
    let mut amount_out = amounts_deque.pop().context("AmountOut:NotExists")?;
    let optionals = remaining_accounts_deque.pop().unwrap_or_default();
    let first_ix = get_ixs(
        first_data,
        amount,
        amount_out,
        &starting_mint,
        mint_pair[0] == starting_mint,
        source_ata,
        destination_ata,
        builder.owner,
        optionals,
        allowed_token2022,
    )?;

    let mints = mint_pair_route.to_vec();
    let mut mints = mints
        .into_iter()
        .flatten()
        .filter(|x| !STATIC_ATA.contains(*x))
        .collect::<Vec<_>>();
    mints.sort();
    mints.dedup();
    for mint in mints.iter() {
        let c_ata_ix = builder.get_create_ata_if_not_exists_v2(mint, allowed_token2022);
        preparation.push(builder.get_simple_swap_ix(c_ata_ix.program_id, c_ata_ix.data, c_ata_ix.accounts));
    }

    preparation.push(builder.get_simple_swap_ix(first_ix.program_id, first_ix.data, first_ix.accounts));

    let mut keep_mints = HashSet::new();

    for mint_pair in mint_pair_route.iter().skip(1) {
        starting_mint = other_mint;
        other_mint = if mint_pair[0] == starting_mint {
            mint_pair[1]
        } else {
            mint_pair[0]
        };
        source_ata = destination_ata;
        destination_ata = lookup_ata_v2(mint_to_ata, &builder.owner, &other_mint, allowed_token2022);

        let amount_out2 = amounts_deque.pop().context("AmountOut2:NotExists")?;

        let optionals = remaining_accounts_deque.pop().unwrap_or_default();

        let calculator = partial_data_deque.pop().context("Calculators:next:NotExists")?;

        let next_ix = get_ixs(
            calculator,
            amount_out,
            amount_out2,
            &starting_mint,
            mint_pair[0] == starting_mint,
            source_ata,
            destination_ata,
            builder.owner,
            optionals,
            allowed_token2022,
        )?;
        amount_out = amount_out2;

        let mut simple_swap = false;
        if calculator.get_pool_type() == PoolType::PumpAmm && next_ix.data.starts_with(&dex::pump_amm::BUY_IX_DISCM) {
            // starting mint sa nemôže close
            keep_mints.insert(starting_mint);
            simple_swap = true;
        }

        let ix = if !simple_swap {
            builder.get_swap_ix(
                next_ix.program_id,
                next_ix.data,
                next_ix.accounts,
                source_ata,
                calculator.get_pool_type().to_offset(),
            )
        } else {
            builder.get_simple_swap_ix(next_ix.program_id, next_ix.data, next_ix.accounts)
        };

        preparation.push(ix);
    }

    for mint in mints.iter() {
        if keep_mints.contains(mint) {
            continue;
        }
        let ata = lookup_ata_v2(mint_to_ata, &builder.owner, mint, allowed_token2022);
        let c_ata_ix = if !allowed_token2022.contains(mint) {
            builder.get_close_ata_ix(&ata)
        } else {
            builder.get_close_ata_2022_ix(&ata)
        };
        preparation.push(builder.get_simple_swap_ix(c_ata_ix.program_id, c_ata_ix.data, c_ata_ix.accounts));
    }

    Ok(preparation)
}

/// Prepares the fixed two-leg wallet-funded ABI for the isolated executor.
/// This path intentionally does not create intermediate ATAs or use the
/// legacy compressed wrapper program.
#[allow(clippy::too_many_arguments)]
pub fn process_executor_v2(
    partial_data: &[Box<CalculatorEnum>],
    starting_mint: &Pubkey,
    amounts: &[u64],
    remaining_accounts: &[Vec<Pubkey>],
    payer: Pubkey,
    executor_program: Pubkey,
    minimum_profit: u64,
    allowed_token2022: &AHashSet<Pubkey>,
) -> anyhow::Result<ArbitrageCompressedInstructionInput> {
    let instruction = crate::executor_v2::build_instruction(
        partial_data,
        starting_mint,
        amounts,
        remaining_accounts,
        payer,
        executor_program,
        minimum_profit,
        allowed_token2022,
    )?;
    Ok(ArbitrageCompressedInstructionInput::from_direct_instruction(
        instruction,
    ))
}

pub fn find_optimum(
    slot: u64,
    amount: u64,
    partial_data: &[CalculatorEnum],
    starting_mint: &Pubkey,
    mint_pair_route: &[&MintPair],
) -> anyhow::Result<(i64, Vec<Vec<Pubkey>>, Vec<u64>)> {
    if partial_data.len() < 2 {
        return Err(format_err!("NotEnoughPools"));
    }

    let mut optional_for_ix = vec![];
    let mut amount_our_ix = vec![];

    let mut partial_data_deque = partial_data.iter().rev().collect::<Vec<_>>();
    let first_data = partial_data_deque.pop().context("NotExists")?;
    let mint_pair = mint_pair_route[0];

    // let (mut amount_out, mut _amount_remaining, optionals) =
    //     get_quote_result_for_ix_v2(starting_mint, amount, slot, first_data, mint_pair)?;

    let QuoteResultForIx {
        mut amount_out,
        remaining_amount_out: mut _amount_remaining,
        remaining_accounts: optionals,
        ..
    } = get_quote_result_for_ix_v3(starting_mint, amount, slot, first_data, mint_pair)?;

    optional_for_ix.push(optionals);
    amount_our_ix.push(amount_out);

    let mut starting_mint;
    let mut other_mint = mint_pair[1];

    let mut final_result = 0_u64;

    let mut counter = 1;
    while let Some(calculator) = partial_data_deque.pop() {
        let mint_pair = mint_pair_route[counter];

        starting_mint = other_mint;
        other_mint = if mint_pair[0] == starting_mint {
            mint_pair[1]
        } else {
            mint_pair[0]
        };

        // other_mint = *calculator
        //     .get_sorted_mints()
        //     .into_iter()
        //     .find(|x| x != &&starting_mint)
        //     .context("NotExists")?;

        // let (amount_out2, _amount_remaining, optionals) =
        //     get_quote_result_for_ix_v2(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        let QuoteResultForIx {
            amount_out: amount_out2,
            remaining_amount_out: mut _amount_remaining,
            remaining_accounts: optionals,
            ..
        } = get_quote_result_for_ix_v3(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        optional_for_ix.push(optionals);
        amount_our_ix.push(amount_out2);

        final_result = amount_out2;

        amount_out = amount_out2;
        counter += 1;
    }

    Ok(((final_result as i64) - (amount as i64), optional_for_ix, amount_our_ix))
}

pub fn find_optimum_v2(
    slot: u64,
    amount: u64,
    partial_data: &[CalculatorEnum],
    starting_mint: &Pubkey,
    mint_pair_route: &[&MintPair],
) -> anyhow::Result<i64> {
    if partial_data.len() < 2 {
        return Err(format_err!("NotEnoughPools"));
    }

    let mut partial_data_deque = partial_data.iter().rev().collect::<Vec<_>>();
    let first_data = partial_data_deque.pop().context("NotExists")?;
    let mint_pair = mint_pair_route[0];

    // let (mut amount_out, mut _amount_remaining, _optionals) =
    //     get_quote_result_for_ix_v2(starting_mint, amount, slot, first_data, mint_pair)?;

    let QuoteResultForIx {
        mut amount_out,
        remaining_amount_out: mut _amount_remaining,
        remaining_accounts: _optionals,
        ..
    } = get_quote_result_for_ix_v3(starting_mint, amount, slot, first_data, mint_pair)?;

    let mut starting_mint;
    let mut other_mint = mint_pair[1];

    let mut final_result = 0_u64;

    let mut counter = 1;
    while let Some(calculator) = partial_data_deque.pop() {
        let mint_pair = mint_pair_route[counter];

        starting_mint = other_mint;
        other_mint = if mint_pair[0] == starting_mint {
            mint_pair[1]
        } else {
            mint_pair[0]
        };

        // other_mint = *calculator
        //     .get_sorted_mints()
        //     .into_iter()
        //     .find(|x| x != &&starting_mint)
        //     .context("NotExists")?;

        // let (amount_out2, _amount_remaining, _optionals) =
        //     get_quote_result_for_ix_v2(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        let QuoteResultForIx {
            amount_out: amount_out2,
            remaining_amount_out: mut _amount_remaining,
            remaining_accounts: _optionals,
            ..
        } = get_quote_result_for_ix_v3(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        final_result = amount_out2;

        amount_out = amount_out2;
        counter += 1;
    }

    Ok((final_result as i64) - (amount as i64))
}

pub fn find_optimum_v3(
    slot: u64,
    amount: u64,
    partial_data: &[Box<CalculatorEnum>],
    starting_mint: &Pubkey,
    mint_pair_route: &[&MintPair],
) -> anyhow::Result<i64> {
    if partial_data.len() < 2 {
        return Err(format_err!("NotEnoughPools"));
    }
    let mut partial_data_deque = partial_data.iter().rev().collect::<Vec<_>>();
    let first_data = partial_data_deque.pop().context("NotExists")?;
    let mint_pair = mint_pair_route[0];

    // let (mut amount_out, mut _amount_remaining, _optionals) =
    //     get_quote_result_for_ix_v2(starting_mint, amount, slot, first_data, mint_pair)?;

    let QuoteResultForIx {
        mut amount_out,
        remaining_amount_out: mut _amount_remaining,
        remaining_accounts: _optionals,
        ..
    } = get_quote_result_for_ix_v3(starting_mint, amount, slot, first_data, mint_pair)?;

    //println!("amount: {:?}, amount_out: {:?}", amount, amount_out);

    let mut starting_mint: Pubkey; // = *starting_mint;
    let mut other_mint = mint_pair[1];

    let mut final_result = 0_u64;
    let mut counter = 1;

    while let Some(calculator) = partial_data_deque.pop() {
        starting_mint = other_mint;
        let mint_pair = mint_pair_route[counter];
        other_mint = if mint_pair[0] == starting_mint {
            mint_pair[1]
        } else {
            mint_pair[0]
        };

        // let (amount_out2, _amount_remaining, _optionals) =
        //     get_quote_result_for_ix_v2(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        let QuoteResultForIx {
            amount_out: amount_out2,
            remaining_amount_out: mut _amount_remaining,
            remaining_accounts: _optionals,
            ..
        } = get_quote_result_for_ix_v3(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        // println!(
        //     "amount_out: {:?}, amount_out2: {:?}",
        //     amount_out, amount_out2
        // );

        final_result = amount_out2;

        // let next_ix = get_ixs(
        //     calculator,
        //     amount_out,
        //     amount_out2, //(amount_out2 / 100) * (100 - (counter * 10)),
        //     &starting_mint,
        //     mint_pair[0] == starting_mint,
        //     source_ata,
        //     destination_ata,
        //     builder.owner,
        //     optionals,
        // )?;
        amount_out = amount_out2;
        counter += 1;
    }

    // for ata in to_be_deleted.iter() {
    //     let c_ata_ix = builder.get_close_ata_ix(ata);
    //     preparation.push(builder.get_simple_swap_ix(
    //         c_ata_ix.program_id,
    //         c_ata_ix.data,
    //         c_ata_ix.accounts,
    //     ));
    // }
    Ok((final_result as i64) - (amount as i64))
}

pub fn find_optimum_v4(
    slot: u64,
    amount: u64,
    partial_data: &[Box<CalculatorEnum>],
    starting_mint: &Pubkey,
    mint_pair_route: &[&MintPair],
) -> anyhow::Result<(i64, Vec<u64>, Vec<Vec<Pubkey>>)> {
    if partial_data.len() < 2 {
        return Err(format_err!("NotEnoughPools"));
    }
    let mut remmaining_accounts = vec![];
    let mut amounts = vec![];
    let mut partial_data_deque = partial_data.iter().rev().collect::<Vec<_>>();
    let first_data = partial_data_deque.pop().context("NotExists")?;
    let mint_pair = mint_pair_route[0];

    // let (mut amount_out, mut _amount_remaining, _optionals) =
    //     get_quote_result_for_ix_v2(starting_mint, amount, slot, first_data, mint_pair)?;

    let QuoteResultForIx {
        mut amount_out,
        remaining_amount_out: mut _amount_remaining,
        remaining_accounts: optionals,
        ..
    } = get_quote_result_for_ix_v3(starting_mint, amount, slot, first_data, mint_pair)?;
    remmaining_accounts.push(optionals);
    amounts.push(amount);
    amounts.push(amount_out);

    //println!("amount: {:?}, amount_out: {:?}", amount, amount_out);

    let mut starting_mint: Pubkey; // = *starting_mint;
    let mut other_mint = mint_pair[1];

    let mut final_result = 0_u64;
    let mut counter = 1;

    while let Some(calculator) = partial_data_deque.pop() {
        starting_mint = other_mint;
        let mint_pair = mint_pair_route[counter];
        other_mint = if mint_pair[0] == starting_mint {
            mint_pair[1]
        } else {
            mint_pair[0]
        };

        // let (amount_out2, _amount_remaining, _optionals) =
        //     get_quote_result_for_ix_v2(&starting_mint, amount_out, slot, calculator, mint_pair)?;

        let QuoteResultForIx {
            amount_out: amount_out2,
            remaining_amount_out: mut _amount_remaining,
            remaining_accounts: optionals,
            ..
        } = get_quote_result_for_ix_v3(&starting_mint, amount_out, slot, calculator, mint_pair)?;
        remmaining_accounts.push(optionals);
        amounts.push(amount_out2);

        // println!(
        //     "amount_out: {:?}, amount_out2: {:?}",
        //     amount_out, amount_out2
        // );

        final_result = amount_out2;

        // let next_ix = get_ixs(
        //     calculator,
        //     amount_out,
        //     amount_out2, //(amount_out2 / 100) * (100 - (counter * 10)),
        //     &starting_mint,
        //     mint_pair[0] == starting_mint,
        //     source_ata,
        //     destination_ata,
        //     builder.owner,
        //     optionals,
        // )?;
        amount_out = amount_out2;
        counter += 1;
    }

    // for ata in to_be_deleted.iter() {
    //     let c_ata_ix = builder.get_close_ata_ix(ata);
    //     preparation.push(builder.get_simple_swap_ix(
    //         c_ata_ix.program_id,
    //         c_ata_ix.data,
    //         c_ata_ix.accounts,
    //     ));
    // }
    Ok(((final_result as i64) - (amount as i64), amounts, remmaining_accounts))
}

#[allow(clippy::too_many_arguments)]
pub fn get_ixs(
    calculator: &CalculatorEnum,
    amount: u64,
    amount_out: u64,
    starting_mint: &Pubkey,
    a_to_b: bool,
    source_ata: Pubkey,
    destination_ata: Pubkey,
    signer: Pubkey,
    remaining_accounts: Vec<Pubkey>,
    allowed_token2022: &AHashSet<Pubkey>,
) -> anyhow::Result<Instruction> {
    match calculator {
        CalculatorEnum::OrcaSwapV2(data) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &data.market;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                a_to_b,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("OrcaSwapV2: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                pool.get_a_mint() == starting_mint,
                None,
                Some(remaining_accounts),
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("OrcaSwapV2: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::SarosAmm(data) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &data.market;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                a_to_b,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("SarosAmm: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                pool.get_a_mint() == starting_mint,
                None,
                Some(remaining_accounts),
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("SarosAmm: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::RaydiumCpmm(data) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &data.pool_state;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                pool.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("RaydiumCpmm: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                pool.get_a_mint() == starting_mint,
                None,
                Some(remaining_accounts),
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("RaydiumCpmm: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        // CalculatorEnum::Lifinity(data) => {
        //     let pubkey = *calculator.get_pubkey();
        //     let amm = &data.amm;
        //     let data = amm.get_swap_data_ix(
        //         amount,
        //         (amount_out / 100) * MIN_MULTIPLICATOR,
        //         amm.get_a_mint() == starting_mint,
        //         true,
        //         allowed_token2022,
        //     );
        //     let data = data.map_err(|e| format_err!("Lifinity: get_swap_data_ix failed. {}", e))?;
        //     let accounts = amm.get_swap_keys_ix(
        //         pubkey,
        //         signer,
        //         source_ata,
        //         destination_ata,
        //         amm.get_a_mint() == starting_mint,
        //         None,
        //         None, // best if the token account is already initialized
        //         allowed_token2022,
        //     );
        //     let accounts = accounts.map_err(|e| format_err!("Lifinity: get_swap_keys_ix failed. {}", e))?;
        //     Ok(Instruction {
        //         program_id: amm.get_program_id(),
        //         data,
        //         accounts,
        //     })
        // }
        CalculatorEnum::StabbleWeightedSwap(data) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &data.pool_swap;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                a_to_b, // pool.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("StabbleWeightedSwap: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                a_to_b,
                None,
                if remaining_accounts.len() == 2 {
                    Some(remaining_accounts)
                } else {
                    None
                }, // best if the token account is already initialized
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("StabbleWeightedSwap: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::StabbleStableSwap(data) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &data.pool_swap;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                a_to_b, //pool.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("StabbleStableSwap: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                a_to_b, //pool.get_a_mint() == starting_mint,
                None,
                if remaining_accounts.len() == 2 {
                    Some(remaining_accounts)
                } else {
                    None
                }, // best if the token account is already initialized
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("StabbleStableSwap: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::PumpAmm(amm) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &amm.pool;
            let a_to_b = pool.get_a_mint() == starting_mint;
            let (amount_out, amount) = if !a_to_b {
                (amount_out, (amount / 100) * MAX_MULTIPLICATOR)
            } else {
                ((amount_out / 100) * MIN_MULTIPLICATOR, amount)
            };
            let data = pool.get_swap_data_ix(amount, amount_out, a_to_b, true, allowed_token2022);
            let data = data.map_err(|e| format_err!("PumpAmm: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                a_to_b,
                Some(amm.config_pubkey),
                Some(vec![
                    amm.config.protocol_fee_recipients[rnd_index(&amm.config.protocol_fee_recipients)],
                ]), // best if the token account is already initialized
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("PumpAmm: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::RaydiumClmmV3(clmm) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &clmm.pool_state;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                pool.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("RaydiumClmmV3: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                pool.get_a_mint() == starting_mint,
                None,
                Some(remaining_accounts),
                allowed_token2022,
            );
            //let accounts = accounts
            //    .map_err(|e| format_err!("RaydiumClmmV3: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::RaydiumAmmV4(amm) => {
            let pubkey = *calculator.get_pubkey();
            let amm_info = &amm.amm_info;
            let data = amm_info.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                amm_info.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("RaydiumAmmV4: get_swap_data_ix failed. {}", e))?;
            let accounts = amm_info.get_swap_keys_ix(pubkey, signer, source_ata, destination_ata, None, None);
            Ok(Instruction {
                program_id: amm_info.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::Orca(orca) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &orca.whirlpool;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                pool.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("Orca: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                pool.get_a_mint() == starting_mint,
                None,
                Some(remaining_accounts),
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("Orca: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::MeteoraDlmm(dlmm) => {
            let pubkey = *calculator.get_pubkey();
            let lb_pair = &dlmm.lb_pair;
            let bitmap_extension = dlmm.bitmap_extension_pubkey;
            let data = lb_pair.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                lb_pair.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("MeteoraDlmm: get_swap_data_ix failed. {}", e))?;
            let accounts = lb_pair.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                lb_pair.get_a_mint() == starting_mint,
                bitmap_extension,
                Some(remaining_accounts),
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("MeteoraDlmm: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: lb_pair.get_program_id(),
                data,
                accounts,
            })
        }
        // CalculatorEnum::GooseGamma(gamma) => {
        //     let pubkey = *calculator.get_pubkey();
        //     let pool: &dex::goose_gamma::accounts::PoolState = &gamma.pool;
        //     let data = pool.get_swap_data_ix(
        //         amount,
        //         (amount_out / 100) * MIN_MULTIPLICATOR,
        //         pool.get_a_mint() == starting_mint,
        //         true,
        //         allowed_token2022,
        //     );
        //     let data = data.map_err(|e| format_err!("GooseGamma: get_swap_data_ix failed. {}", e))?;
        //     let accounts = pool.get_swap_keys_ix(
        //         pubkey,
        //         signer,
        //         source_ata,
        //         destination_ata,
        //         pool.get_a_mint() == starting_mint,
        //         None,
        //         None,
        //         allowed_token2022,
        //     );
        //     let accounts = accounts.map_err(|e| format_err!("GooseGamma: get_swap_keys_ix failed. {}", e))?;
        //     Ok(Instruction {
        //         program_id: pool.get_program_id(),
        //         data,
        //         accounts,
        //     })
        // }
        CalculatorEnum::MeteoraDammV2(damm) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &damm.pool;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                pool.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("MeteoraDammV2: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                pool.get_a_mint() == starting_mint,
                None,
                Some(remaining_accounts),
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("MeteoraDammV2: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::FusionAmm(fusion) => {
            let pubkey = *calculator.get_pubkey();
            let pool = &fusion.pool;
            let data = pool.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                pool.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("FUsionAmm: get_swap_data_ix failed. {}", e))?;
            let accounts = pool.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                pool.get_a_mint() == starting_mint,
                None,
                Some(remaining_accounts),
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("FusionAmm: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pool.get_program_id(),
                data,
                accounts,
            })
        }
        CalculatorEnum::SarosDlmm(dlmm) => {
            let pubkey = *calculator.get_pubkey();
            let pair = &dlmm.pair;
            let data = pair.get_swap_data_ix(
                amount,
                (amount_out / 100) * MIN_MULTIPLICATOR,
                pair.get_a_mint() == starting_mint,
                true,
                allowed_token2022,
            );
            let data = data.map_err(|e| format_err!("SarosDlmm: get_swap_data_ix failed. {}", e))?;
            let accounts = pair.get_swap_keys_ix(
                pubkey,
                signer,
                source_ata,
                destination_ata,
                pair.get_a_mint() == starting_mint,
                None,
                Some(remaining_accounts),
                allowed_token2022,
            );
            let accounts = accounts.map_err(|e| format_err!("SarosDlmm: get_swap_keys_ix failed. {}", e))?;
            Ok(Instruction {
                program_id: pair.get_program_id(),
                data,
                accounts,
            })
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct QuoteResultForIx {
    pub amount_out: u64,
    pub remaining_amount_out: u64,
    pub remaining_accounts: Vec<Pubkey>,
    pub compute_units: u32,
}

impl QuoteResultForIx {
    pub fn new(a_to_b: bool, quote_result: &QuoteResult, remaining_accounts: Vec<Pubkey>, compute_units: u32) -> Self {
        let amount_out = if a_to_b {
            quote_result.amount_b_out
        } else {
            quote_result.amount_a_out
        };
        let remaining_amount_out = if a_to_b {
            quote_result.amount_a_out
        } else {
            quote_result.amount_b_out
        };
        Self {
            amount_out,
            remaining_amount_out,
            remaining_accounts,
            compute_units,
        }
    }
}

// v3

pub fn get_quote_result_for_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    slot: u64,
    partial_data: &CalculatorEnum,
    mint_pair: &MintPair,
) -> anyhow::Result<QuoteResultForIx> {
    match partial_data {
        CalculatorEnum::OrcaSwapV2(data) => get_quote_result_for_orca_swap_v2_ix_v3(starting_mint, amount, data),
        CalculatorEnum::SarosAmm(data) => get_quote_result_for_saros_amm_ix_v3(starting_mint, amount, data),
        CalculatorEnum::RaydiumCpmm(data) => get_quote_result_for_raydium_cpmm_ix_v3(starting_mint, amount, data),
        CalculatorEnum::StabbleWeightedSwap(data) => {
            get_quote_result_for_stabble_weighted_swap_ix_v3(starting_mint, amount, data, *mint_pair)
        }
        CalculatorEnum::StabbleStableSwap(data) => {
            get_quote_result_for_stabble_stable_swap_ix_v3(starting_mint, amount, data, *mint_pair)
        }
        CalculatorEnum::PumpAmm(data) => get_quote_result_for_pump_amm_ix_v3(starting_mint, amount, data),
        CalculatorEnum::RaydiumClmmV3(data) => get_quote_result_for_raydium_clmm_ix_v3(starting_mint, amount, data),
        CalculatorEnum::RaydiumAmmV4(data) => get_quote_result_for_raydium_amm_ix_v3(starting_mint, amount, data),
        CalculatorEnum::Orca(data) => get_quote_result_for_orca_ix_v3(starting_mint, amount, data),
        CalculatorEnum::MeteoraDlmm(data) => get_quote_result_for_meteora_dlmm_ix_v3(starting_mint, amount, slot, data),
        //CalculatorEnum::Lifinity(data) => get_quote_result_for_lifinity_ix_v3(starting_mint, amount, slot, data),
        //CalculatorEnum::GooseGamma(data) => get_quote_result_for_goose_gamma_ix_v3(starting_mint, amount, data),
        CalculatorEnum::MeteoraDammV2(data) => {
            get_quote_result_for_meteora_damm_ix_v3(starting_mint, amount, slot, data)
        }
        CalculatorEnum::FusionAmm(data) => get_quote_result_for_fusion_amm_ix_v3(starting_mint, amount, data),
        CalculatorEnum::SarosDlmm(data) => get_quote_result_for_saros_dlmm_ix_v3(starting_mint, amount, data),
    }
}

fn get_quote_result_for_orca_swap_v2_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &OrcaSwapV2Data,
) -> anyhow::Result<QuoteResultForIx> {
    let market = &data.market;
    let input = dex::orca_swap_v2::accounts::QuoteInput {
        coin_a: data.total_coin,
        coin_b: data.total_pc,
    };
    let quote_result = market.quote(market.get_a_mint() == starting_mint, true, amount, &input)?;
    Ok(QuoteResultForIx::new(
        market.get_a_mint() == starting_mint,
        &quote_result,
        vec![],
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

fn get_quote_result_for_saros_amm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &SarosAmmData,
) -> anyhow::Result<QuoteResultForIx> {
    let market = &data.market;
    let input = dex::saros_amm::accounts::QuoteInput {
        coin_a: data.total_coin,
        coin_b: data.total_pc,
    };
    let quote_result = market.quote(market.get_a_mint() == starting_mint, true, amount, &input)?;
    Ok(QuoteResultForIx::new(
        market.get_a_mint() == starting_mint,
        &quote_result,
        vec![],
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

fn get_quote_result_for_raydium_cpmm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &RaydiumCpmmData,
) -> anyhow::Result<QuoteResultForIx> {
    let pool_state = &data.pool_state;
    let input = dex::raydium_cpmm::QuoteInput {
        coin_a: data.total_coin,
        coin_b: data.total_pc,
        config: &data.amm_config,
    };
    let quote_result = pool_state.quote(pool_state.get_a_mint() == starting_mint, true, amount, &input)?;
    Ok(QuoteResultForIx::new(
        pool_state.get_a_mint() == starting_mint,
        &quote_result,
        vec![],
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

// fn get_quote_result_for_lifinity_ix_v3(
//     starting_mint: &Pubkey,
//     amount: u64,
//     slot: u64,
//     data: &LifinityData,
// ) -> anyhow::Result<QuoteResultForIx> {
//     let amm = &data.amm;
//     let input = dex::lifinity::accounts::QuoteInput {
//         coin_a: data.total_coin,
//         coin_b: data.total_pc,
//         slot,
//         oracle_main: &data.oracle_main,
//         oracle_sub: &data.oracle_sub,
//         oracle_pc: &data.oracle_pc,
//     };
//     let quote_result = amm.quote(amm.get_a_mint() == starting_mint, true, amount, &input)?;
//     Ok(QuoteResultForIx::new(
//         amm.get_a_mint() == starting_mint,
//         &quote_result,
//         vec![],
//         quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
//     ))
// }

fn get_quote_result_for_stabble_stable_swap_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &StabbleStableSwapData,
    mint_pair: MintPair,
) -> anyhow::Result<QuoteResultForIx> {
    // if pool p
    let pool = &data.pool_swap;
    let quote_result = pool.quote(
        &mint_pair[0] == starting_mint,
        true,
        amount,
        &dex::stabble_stable_swap::QuoteInput {
            mint_pair: Some(mint_pair),
        },
    )?;
    Ok(QuoteResultForIx::new(
        &mint_pair[0] == starting_mint,
        &quote_result,
        mint_pair.to_vec(),
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

fn get_quote_result_for_stabble_weighted_swap_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &StabbleWeightedSwapData,
    mint_pair: MintPair,
) -> anyhow::Result<QuoteResultForIx> {
    let pool = &data.pool_swap;
    let quote_result = pool.quote(
        &mint_pair[0] == starting_mint,
        true,
        amount,
        &dex::stabble_weighted_swap::accounts::QuoteInput {
            mint_pair: Some(mint_pair),
        },
    )?;
    // ln!("Quote: {:?} {}", quote_result, amm_info.get_a_mint() == starting_mint);
    Ok(QuoteResultForIx::new(
        &mint_pair[0] == starting_mint,
        &quote_result,
        mint_pair.to_vec(),
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

pub fn get_quote_result_for_meteora_dlmm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    slot: u64,
    data: &MeteoraDlmmData,
) -> anyhow::Result<QuoteResultForIx> {
    let lb_pair = data.lb_pair;
    let bin_arrays = &data.bin_arrays_map;
    let quote_result = lb_pair.quote(
        lb_pair.get_a_mint() == starting_mint,
        true,
        amount,
        &dex::meteora_dlmm::quote::QuoteInput {
            bin_arrays,
            bitmap_extension: None,
            current_slot: slot,
        },
    )?;

    //println!("Quote CU coef: {:?}", quote_result.compute_units);

    let mut remaining_accounts = vec![];

    if let Some(indices) = quote_result.indices.clone() {
        for index in indices.iter() {
            let (pubkey, _) = bin_arrays.get(index).context("NotFound Index")?;
            remaining_accounts.push(*pubkey);
        }
        if indices.len() < 3 {
            if let Some(last) = indices.last() {
                let extend_keys = if lb_pair.get_a_mint() == starting_mint {
                    // a to b direction down
                    bin_arrays
                        .iter()
                        .rev()
                        .filter(|(_id, (_, bin_array))| {
                            !bin_array.is_zero_liquidity() && bin_array.index < *last as i64
                        })
                        .take(3 - indices.len())
                        .map(|(_, (pubkey, _))| *pubkey)
                        .collect::<Vec<_>>()
                } else {
                    // b to a direction up
                    bin_arrays
                        .iter()
                        .filter(|(_id, (_, bin_array))| {
                            !bin_array.is_zero_liquidity() && bin_array.index > *last as i64
                        })
                        .take(3 - indices.len())
                        .map(|(_, (pubkey, _))| *pubkey)
                        .collect::<Vec<_>>()
                };
                remaining_accounts.extend(extend_keys);
            }
        }
    }
    Ok(QuoteResultForIx::new(
        lb_pair.get_a_mint() == starting_mint,
        &quote_result,
        remaining_accounts,
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

pub fn get_quote_result_for_raydium_amm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &RaydiumAmmV4Data,
) -> anyhow::Result<QuoteResultForIx> {
    let amm_info = data.amm_info;
    let pc_amount = data.total_pc;
    let coin_amount = data.total_coin;
    let quote_result = amm_info.quote(
        amm_info.get_a_mint() == starting_mint,
        true,
        amount,
        &dex::raydium_amm::QuoteInput {
            total_pc: pc_amount,
            total_coin: coin_amount,
        },
    )?;
    Ok(QuoteResultForIx::new(
        amm_info.get_a_mint() == starting_mint,
        &quote_result,
        vec![],
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

pub fn get_quote_result_for_pump_amm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &PumpAmmData,
) -> anyhow::Result<QuoteResultForIx> {
    let pool = data.pool;
    let base = data.base;
    let quote = data.quote;
    // TODO: process fees
    //let is_pump_pool = dex::pump_amm::is_pump_pool(&pool.base_mint, &pool.creator);
    // TODO: process fees
    let fee = if dex::pump_amm::is_pump_pool(&pool.base_mint, &pool.creator) {
        data.config.protocol_fee_basis_points + data.config.lp_fee_basis_points
    } else {
        // TODO: flat fees
        2 + 93 + 30
    };
    let quote_result = pool.quote(
        pool.get_a_mint() == starting_mint,
        true,
        amount,
        &dex::pump_amm::QuoteInput {
            base,
            quote,
            fee,
            //fees: Fees::default(),
        },
    )?;
    Ok(QuoteResultForIx::new(
        pool.get_a_mint() == starting_mint,
        &quote_result,
        vec![],
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

pub fn get_quote_result_for_raydium_clmm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &RaydiumClmmV3Data,
) -> anyhow::Result<QuoteResultForIx> {
    let pool = &data.pool_state;
    let tick_arrays = &data.tick_arrays_map;
    let input = data.get_input();
    let quote_result = pool.quote(pool.get_a_mint() == starting_mint, true, amount, &input)?;

    let mut remaining_accounts = vec![];
    if let Some(indices) = quote_result.indices.clone() {
        //println!("RaydiumClmmV3 Quote indices: {:?}", indices);
        for index in indices.iter() {
            let (pubkey, _) = tick_arrays.get(index).context("NotFound Index")?;
            remaining_accounts.push(*pubkey);
        }
        if indices.len() < 5 {
            if let Some(last) = indices.last() {
                let extend_keys = if pool.get_a_mint() == starting_mint {
                    // a to b direction down
                    tick_arrays
                        .iter()
                        .rev()
                        .filter(|(_id, (_, tick_array))| {
                            tick_array.ticks.iter().any(|tick| tick.liquidity_gross != 0)
                                && tick_array.start_tick_index < *last
                        })
                        .take(5 - indices.len())
                        .map(|(_, (pubkey, _))| *pubkey)
                        .collect::<Vec<_>>()
                } else {
                    // b to a direction up
                    tick_arrays
                        .iter()
                        .rev()
                        .filter(|(_id, (_, tick_array))| {
                            tick_array.ticks.iter().any(|tick| tick.liquidity_gross != 0)
                                && tick_array.start_tick_index > *last
                        })
                        .take(5 - indices.len())
                        .map(|(_, (pubkey, _))| *pubkey)
                        .collect::<Vec<_>>()
                };
                remaining_accounts.extend(extend_keys);
            }
        }
    }

    //remaining_accounts.dedup();

    // add bitmap extension
    if let Some(bitmap_extension) = data.bitmap_extension {
        remaining_accounts.push(bitmap_extension);
    }
    Ok(QuoteResultForIx::new(
        pool.get_a_mint() == starting_mint,
        &quote_result,
        remaining_accounts,
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

pub fn get_quote_result_for_orca_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &OrcaData,
) -> anyhow::Result<QuoteResultForIx> {
    let pool_pubkey = &data.whirlpool_pubkey;
    let pool = &data.whirlpool;
    let tick_arrays = &&data.tick_arrays_map;

    let input = dex::orca::QuoteInput {
        map: tick_arrays,
        oracle: data.oracle.as_ref(),
    };

    let quote_result = pool.quote(pool.get_a_mint() == starting_mint, true, amount, &input)?;

    let mut remaining_accounts = vec![];
    if let Some(indices) = quote_result.indices.clone() {
        for index in indices.iter() {
            if let Some((pubkey, _)) = tick_arrays.get(index) {
                remaining_accounts.push(*pubkey);
            } else {
                remaining_accounts.push(get_tick_array_address(pool_pubkey, *index)?.0);
            }
        }
        if indices.len() < 3 {
            if let Some(last) = indices.last() {
                let offset = pool.tick_spacing * (TICK_ARRAY_SIZE as u16);
                if pool.get_a_mint() == starting_mint {
                    // a to b direction down
                    let tick_array1_index = *last - (offset as i32);
                    if let Some((pubkey, _)) = tick_arrays.get(&tick_array1_index) {
                        remaining_accounts.push(*pubkey);
                    } else {
                        remaining_accounts.push(get_tick_array_address(pool_pubkey, tick_array1_index)?.0);
                    }
                    if remaining_accounts.len() < 3 {
                        let tick_array2_index = *last - (offset * 2) as i32;
                        if let Some((pubkey, _)) = tick_arrays.get(&tick_array2_index) {
                            remaining_accounts.push(*pubkey);
                        } else {
                            remaining_accounts.push(get_tick_array_address(pool_pubkey, tick_array2_index)?.0);
                        }
                    }
                } else {
                    // b to a direction up
                    let tick_array1_index = *last + (offset as i32);
                    if let Some((pubkey, _)) = tick_arrays.get(&tick_array1_index) {
                        remaining_accounts.push(*pubkey);
                    } else {
                        remaining_accounts.push(get_tick_array_address(pool_pubkey, tick_array1_index)?.0);
                    }
                    if remaining_accounts.len() < 3 {
                        let tick_array2_index = *last + (offset * 2) as i32;
                        if let Some((pubkey, _)) = tick_arrays.get(&tick_array2_index) {
                            remaining_accounts.push(*pubkey);
                        } else {
                            remaining_accounts.push(get_tick_array_address(pool_pubkey, tick_array2_index)?.0);
                        }
                    }
                };
            }
        }
    }
    Ok(QuoteResultForIx::new(
        pool.get_a_mint() == starting_mint,
        &quote_result,
        remaining_accounts,
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

// pub fn get_quote_result_for_goose_gamma_ix_v3(
//     starting_mint: &Pubkey,
//     amount: u64,
//     data: &GooseGammaData,
// ) -> anyhow::Result<QuoteResultForIx> {
//     let pool = &data.pool;
//     let input = dex::goose_gamma::accounts::QuoteInput {
//         coin_a: data.coin_a,
//         coin_b: data.coin_b,
//         amm_config: &data.config,
//         observation_state: &data.observation_state,
//     };
//     let quote_result = pool.quote(pool.get_a_mint() == starting_mint, true, amount, &input)?;
//     Ok(QuoteResultForIx::new(
//         pool.get_a_mint() == starting_mint,
//         &quote_result,
//         vec![],
//         quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
//     ))
// }

pub fn get_quote_result_for_meteora_damm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    slot: u64,
    data: &MeteoraDammV2Data,
) -> anyhow::Result<QuoteResultForIx> {
    let pool = &data.pool;
    let input = dex::meteora_damm_v2::QuoteInput { slot };
    let quote_result = pool.quote(pool.get_a_mint() == starting_mint, true, amount, &input)?;
    Ok(QuoteResultForIx::new(
        pool.get_a_mint() == starting_mint,
        &quote_result,
        vec![],
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

pub fn get_quote_result_for_fusion_amm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &FusionAmmData,
) -> anyhow::Result<QuoteResultForIx> {
    let pool_pubkey = &data.pool_pubkey;
    let pool = &data.pool.whirlpool;
    let tick_arrays = &&data.tick_arrays_map;

    let input = dex::orca::QuoteInput {
        map: tick_arrays,
        oracle: None,
    };

    let quote_result = pool.quote(pool.get_a_mint() == starting_mint, true, amount, &input)?;

    let mut remaining_accounts = vec![];
    if let Some(indices) = quote_result.indices.clone() {
        for index in indices.iter() {
            if let Some((pubkey, _)) = tick_arrays.get(index) {
                remaining_accounts.push(*pubkey);
            } else {
                remaining_accounts.push(get_tick_array_address_fusion(pool_pubkey, *index)?.0);
            }
        }
        if indices.len() < 3 {
            if let Some(last) = indices.last() {
                let offset = pool.tick_spacing * (TICK_ARRAY_SIZE as u16);
                if pool.get_a_mint() == starting_mint {
                    // a to b direction down
                    let tick_array1_index = *last - (offset as i32);
                    if let Some((pubkey, _)) = tick_arrays.get(&tick_array1_index) {
                        remaining_accounts.push(*pubkey);
                    } else {
                        remaining_accounts.push(get_tick_array_address_fusion(pool_pubkey, tick_array1_index)?.0);
                    }
                    if remaining_accounts.len() < 3 {
                        let tick_array2_index = *last - (offset * 2) as i32;
                        if let Some((pubkey, _)) = tick_arrays.get(&tick_array2_index) {
                            remaining_accounts.push(*pubkey);
                        } else {
                            remaining_accounts.push(get_tick_array_address_fusion(pool_pubkey, tick_array2_index)?.0);
                        }
                    }
                } else {
                    // b to a direction up
                    let tick_array1_index = *last + (offset as i32);
                    if let Some((pubkey, _)) = tick_arrays.get(&tick_array1_index) {
                        remaining_accounts.push(*pubkey);
                    } else {
                        remaining_accounts.push(get_tick_array_address_fusion(pool_pubkey, tick_array1_index)?.0);
                    }
                    if remaining_accounts.len() < 3 {
                        let tick_array2_index = *last + (offset * 2) as i32;
                        if let Some((pubkey, _)) = tick_arrays.get(&tick_array2_index) {
                            remaining_accounts.push(*pubkey);
                        } else {
                            remaining_accounts.push(get_tick_array_address_fusion(pool_pubkey, tick_array2_index)?.0);
                        }
                    }
                };
            }
        }
    }
    Ok(QuoteResultForIx::new(
        pool.get_a_mint() == starting_mint,
        &quote_result,
        remaining_accounts,
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}

pub fn get_quote_result_for_saros_dlmm_ix_v3(
    starting_mint: &Pubkey,
    amount: u64,
    data: &SarosDlmmData,
) -> anyhow::Result<QuoteResultForIx> {
    let pair = data.pair;
    let index = data.pair.active_id / dex::saros_dlmm::BIN_ARRAY_SIZE;
    let (bin_array_lower_pubkey, bin_array_lower) =
        data.bin_arrays_map.get(&index).context("NotFound Active Bin Array")?;
    let (bin_array_upper_pubkey, bin_array_upper) = data
        .bin_arrays_map
        .get(&(index + 1))
        .context("NotFound Active Bin Array")?;
    let bin_array_pair = dex::saros_dlmm::BinArrayPair {
        bin_array_lower: *bin_array_lower,
        bin_array_upper: *bin_array_upper,
    };
    let quote_result = pair.quote(
        pair.get_a_mint() == starting_mint,
        true,
        amount,
        &dex::saros_dlmm::QuoteInput {
            bin_array_pair: &bin_array_pair,
        },
    )?;
    //println!("Quote CU coef: {:?}", quote_result.compute_units);
    let remaining_accounts = vec![*bin_array_lower_pubkey, *bin_array_upper_pubkey];
    Ok(QuoteResultForIx::new(
        pair.get_a_mint() == starting_mint,
        &quote_result,
        remaining_accounts,
        quote_result.compute_units.min(CU_MAX_UNITS_PER_SWAP as u32),
    ))
}
