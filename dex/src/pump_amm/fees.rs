use anyhow::Context;
use solana_sdk::pubkey::Pubkey;

use crate::pump_amm::{FeeConfig, Fees, PUMP_FUN_PROGRAM_ID};

impl FeeConfig {
    fn calculate_fee_tier(&self, market_cap: u128) -> Fees {
        let first_tier = &self.fee_tiers[0];
        if market_cap < first_tier.market_cap_lamports_threshold {
            return first_tier.fees;
        }
        for tier in self.fee_tiers.iter().rev() {
            if market_cap >= tier.market_cap_lamports_threshold {
                return tier.fees;
            }
        }
        first_tier.fees
    }

    pub fn get_fees(&self, is_pump_pool: bool, market_cap: u128) -> Fees {
        if is_pump_pool {
            self.calculate_fee_tier(market_cap)
        } else {
            self.flat_fees
        }
    }
}

// TODO: lazy pda
pub fn is_pump_pool(base_mint: &Pubkey, pool_creator: &Pubkey) -> bool {
    let (pda, _bump) = Pubkey::find_program_address(&[b"pool-authority", base_mint.as_ref()], &PUMP_FUN_PROGRAM_ID);
    pda == *pool_creator
}

pub fn pool_market_cap(base_mint_supply: u128, base_reserve: u128, quote_reserve: u128) -> anyhow::Result<u128> {
    if base_reserve == 0 {
        return Err(anyhow::format_err!(
            "Division by zero: pool base token reserves cannot be zero"
        ));
    }
    quote_reserve
        .checked_mul(base_mint_supply)
        .context("Overflow")?
        .checked_div(base_reserve)
        .context("DivisionByZero")
}
