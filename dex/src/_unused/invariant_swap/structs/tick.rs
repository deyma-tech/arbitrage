use crate::invariant_swap::decimals::*;

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

#[repr(packed)]
#[derive(PartialEq, Default, Debug, Copy, Clone)]
pub struct Tick {
    pub pool: Pubkey,
    pub index: i32,
    pub sign: bool, // true means positive
    pub liquidity_change: Liquidity,
    pub liquidity_gross: Liquidity,
    pub sqrt_price: Price,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    pub seconds_per_liquidity_outside: FixedPoint,
    pub seconds_outside: u64,
    pub bump: u8,
}

impl Tick {
    pub fn update(
        &mut self,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
        is_upper: bool,
        is_deposit: bool,
    ) -> Result<()> {
        self.liquidity_gross = self.calculate_new_liquidity_gross_safely(
            is_deposit,
            liquidity_delta,
            max_liquidity_per_tick,
        )?;

        self.update_liquidity_change(liquidity_delta, is_deposit ^ is_upper);
        Ok(())
    }

    fn update_liquidity_change(&mut self, liquidity_delta: Liquidity, add: bool) {
        if self.sign ^ add {
            if { self.liquidity_change } > liquidity_delta {
                self.liquidity_change = self.liquidity_change - liquidity_delta;
            } else {
                self.liquidity_change = liquidity_delta - self.liquidity_change;
                self.sign = !self.sign;
            }
        } else {
            self.liquidity_change = self.liquidity_change + liquidity_delta;
        }
    }

    fn calculate_new_liquidity_gross_safely(
        self,
        sign: bool,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
    ) -> Result<Liquidity> {
        // validate in decrease liquidity case
        if !sign && { self.liquidity_gross } < liquidity_delta {
            return Err(anyhow::format_err!("InvalidTickLiquidity"));
        }
        let new_liquidity = match sign {
            true => self.liquidity_gross + liquidity_delta,
            false => self.liquidity_gross - liquidity_delta,
        };
        // validate in increase liquidity case
        if sign && new_liquidity >= max_liquidity_per_tick {
            return Err(anyhow::format_err!("InvalidTickLiquidity"));
        }

        Ok(new_liquidity)
    }
}
