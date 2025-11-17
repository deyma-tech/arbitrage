use crate::invariant_swap::decimals::*;
use crate::invariant_swap::math::{
    calculate_amount_delta, calculate_fee_growth_inside, calculate_max_liquidity_per_tick,
};
use crate::invariant_swap::structs::pool::PoolState;
use crate::invariant_swap::structs::tick::Tick;

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

#[repr(packed)]
#[derive(PartialEq, Default, Debug)]
pub struct Position {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub id: u128, // unique inside pool
    pub liquidity: Liquidity,
    pub lower_tick_index: i32,
    pub upper_tick_index: i32,
    pub fee_growth_inside_x: FeeGrowth,
    pub fee_growth_inside_y: FeeGrowth,
    pub seconds_per_liquidity_inside: FixedPoint,
    pub last_slot: u64,
    pub tokens_owed_x: FixedPoint,
    pub tokens_owed_y: FixedPoint,
    pub bump: u8,
}

impl Position {
    pub fn modify(
        &mut self,
        pool: &mut PoolState,
        upper_tick: &mut Tick,
        lower_tick: &mut Tick,
        liquidity_delta: Liquidity,
        add: bool,
        current_timestamp: u64,
    ) -> Result<(TokenAmount, TokenAmount)> {
        if !pool.liquidity.is_zero() {
            pool.update_seconds_per_liquidity_global(current_timestamp);
        } else {
            pool.last_timestamp = current_timestamp;
        }

        // calculate dynamically limit allows easy modification
        let max_liquidity_per_tick = calculate_max_liquidity_per_tick(pool.tick_spacing);

        // update initialized tick
        lower_tick.update(liquidity_delta, max_liquidity_per_tick, false, add)?;

        upper_tick.update(liquidity_delta, max_liquidity_per_tick, true, add)?;

        // update fee inside position
        let (fee_growth_inside_x, fee_growth_inside_y) = calculate_fee_growth_inside(
            *lower_tick,
            *upper_tick,
            pool.current_tick_index,
            pool.fee_growth_global_x,
            pool.fee_growth_global_y,
        );

        self.update(
            add,
            liquidity_delta,
            fee_growth_inside_x,
            fee_growth_inside_y,
        )?;

        // calculate tokens amounts and update pool liquidity
        calculate_amount_delta(
            pool,
            liquidity_delta,
            add,
            upper_tick.index,
            lower_tick.index,
        )
    }

    pub fn update(
        &mut self,
        sign: bool,
        liquidity_delta: Liquidity,
        fee_growth_inside_x: FeeGrowth,
        fee_growth_inside_y: FeeGrowth,
    ) -> Result<()> {
        if !(liquidity_delta.v != 0 || self.liquidity.v != 0) {
            return Err(anyhow::anyhow!("ErrorCode::EmptyPositionPokes"));
        };

        // calculate accumulated fee
        let tokens_owed_x = fee_growth_inside_x
            .unchecked_sub(self.fee_growth_inside_x)
            .to_fee(self.liquidity);
        let tokens_owed_y = fee_growth_inside_y
            .unchecked_sub(self.fee_growth_inside_y)
            .to_fee(self.liquidity);

        self.liquidity = self.calculate_new_liquidity_safely(sign, liquidity_delta)?;
        self.fee_growth_inside_x = fee_growth_inside_x;
        self.fee_growth_inside_y = fee_growth_inside_y;
        // trunk-ignore(clippy/assign_op_pattern)
        self.tokens_owed_x = self.tokens_owed_x + tokens_owed_x;
        // trunk-ignore(clippy/assign_op_pattern)
        self.tokens_owed_y = self.tokens_owed_y + tokens_owed_y;

        Ok(())
    }

    pub fn initialized_id(&mut self, pool: &mut PoolState) {
        self.id = pool.position_iterator;
        pool.position_iterator = pool.position_iterator.checked_add(1).unwrap();
    }

    // for future use
    pub fn get_id(self) -> String {
        let mut id = self.pool.to_string();
        id.push_str({ self.id }.to_string().as_str());
        id
    }

    fn calculate_new_liquidity_safely(
        &mut self,
        sign: bool,
        liquidity_delta: Liquidity,
    ) -> Result<Liquidity> {
        // validate in decrease liquidity case
        if !sign && { self.liquidity } < liquidity_delta {
            return Err(anyhow::format_err!("InvalidPositionLiquidity"));
        }

        Ok(match sign {
            true => self.liquidity + liquidity_delta,
            false => self.liquidity - liquidity_delta,
        })
    }
}
