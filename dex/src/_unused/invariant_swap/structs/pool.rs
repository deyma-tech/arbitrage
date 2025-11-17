use borsh::BorshDeserialize;
use bytemuck::{Pod, Zeroable};
use decimal::*;
use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;
use utils::pool::Pool;

use crate::invariant_swap::decimals::*;

pub const AUTHORITY_ID: Pubkey = pubkey!("J4uBbeoWpZE8fH58PM1Fp9n9K6f1aThyeVCyRdJbaXqt");
pub const POOL_ACCOUNT_DISCM: [u8; 8] = [241, 154, 109, 4, 17, 177, 109, 188];

#[repr(C, packed)]
#[derive(PartialEq, Default, Debug, Clone, Copy, Pod, Zeroable)]
pub struct PoolState {
    pub token_x: Pubkey,
    pub token_y: Pubkey,
    pub token_x_reserve: Pubkey,
    pub token_y_reserve: Pubkey,
    pub position_iterator: u128,
    pub tick_spacing: u16,
    pub fee: FixedPoint,
    pub protocol_fee: FixedPoint,
    pub liquidity: Liquidity,
    pub sqrt_price: Price,
    pub current_tick_index: i32, // nearest tick below the current price
    pub tickmap: Pubkey,
    pub fee_growth_global_x: FeeGrowth,
    pub fee_growth_global_y: FeeGrowth,
    pub fee_protocol_token_x: u64, // should be changed to TokenAmount when Armani implements tuple structs
    pub fee_protocol_token_y: u64,
    pub seconds_per_liquidity_global: FixedPoint,
    pub start_timestamp: u64,
    pub last_timestamp: u64,
    pub fee_receiver: Pubkey,
    pub oracle_address: Pubkey,
    //pub oracle_initialized: bool,
    pub oracle_initialized: u8,
    pub bump: u8,
}

impl PoolState {
    pub fn add_fee(
        &mut self,
        amount: TokenAmount,
        ref_percentage: FixedPoint,
        in_x: bool,
    ) -> TokenAmount {
        let protocol_fee = TokenAmount::from_decimal_up(amount.big_mul_up(self.protocol_fee));
        let ref_fee = match ref_percentage.is_zero() {
            true => TokenAmount(0),
            false => TokenAmount::from_decimal(amount.big_mul(ref_percentage)),
        };
        let pool_fee = amount - protocol_fee - ref_fee;

        if (pool_fee.is_zero() && protocol_fee.is_zero()) || self.liquidity.is_zero() {
            return ref_fee;
        }
        let fee_growth = FeeGrowth::from_fee(self.liquidity, pool_fee);

        if in_x {
            self.fee_growth_global_x = self.fee_growth_global_x.unchecked_add(fee_growth);
            self.fee_protocol_token_x = self
                .fee_protocol_token_x
                .checked_add(protocol_fee.0)
                .unwrap();
        } else {
            self.fee_growth_global_y = self.fee_growth_global_y.unchecked_add(fee_growth);
            self.fee_protocol_token_y = self
                .fee_protocol_token_y
                .checked_add(protocol_fee.0)
                .unwrap();
        }
        ref_fee
    }

    pub fn update_liquidity_safely(
        &mut self,
        liquidity_delta: Liquidity,
        add: bool,
    ) -> anyhow::Result<()> {
        // validate in decrease liquidity case
        if !add && { self.liquidity } < liquidity_delta {
            return Err(anyhow::format_err!("InvalidPoolLiquidity"));
        };
        // pool liquidity can cannot be negative
        self.liquidity = match add {
            true => self.liquidity + liquidity_delta,
            false => self.liquidity - liquidity_delta,
        };

        Ok(())
    }

    pub fn update_seconds_per_liquidity_global(&mut self, current_timestamp: u64) {
        self.seconds_per_liquidity_global = self.seconds_per_liquidity_global.unchecked_add(
            FixedPoint::from_integer(current_timestamp.checked_sub(self.last_timestamp).unwrap())
                / self.liquidity,
        );

        self.last_timestamp = current_timestamp;
    }

    pub fn set_oracle(&mut self, address: Pubkey) {
        self.oracle_address = address;
        self.oracle_initialized = 1u8;
    }

    pub fn deserialize(buf: &[u8]) -> anyhow::Result<Self> {        
        let mut buf = &buf[8..];
        let res = bytemuck::try_from_bytes::<PoolState>(&mut buf).map_err(|_| {
            anyhow::anyhow!("FailedToDeserializeInvariantSwapPool")
        })?;
        Ok(*res)
    }

}

#[derive(Debug)]
pub struct QuoteInput {
    // pub base: u64,
    // pub quote: u64, // coin = a
    // pub fee: u64,
}


impl Pool<QuoteInput> for PoolState {
    fn quote(
        &self,
        a_to_b: bool,
        exact_in: bool,
        amount_in: u64,
        input: &QuoteInput,
    ) -> anyhow::Result<utils::quote::QuoteResult> {
        todo!()
    }

    fn get_keys(&self) -> Vec<&'_ Pubkey> {
        vec![
            &self.token_x_reserve,
            &self.token_y_reserve,            
        ]
    }

    fn copy_keys(&self) -> Vec<Pubkey> {
        vec![
            self.token_x_reserve,
            self.token_y_reserve,
        ]
    }

    fn get_a_mint(&self) -> &'_ Pubkey {
        &self.token_x
    }

    fn get_b_mint(&self) -> &'_ Pubkey {
        &self.token_y
    }

    fn get_program_id(&self) -> Pubkey {
        crate::invariant_swap::ID
    }

    fn get_swap_data_ix(
        &self,
        amount: u64,
        amount_threshold: u64,
        a_to_b: bool,
        exact_in: bool,
    ) -> anyhow::Result<Vec<u8>> {
        todo!()
    }

    fn get_swap_keys_ix(
        &self,
        pool_pubkey: Pubkey,
        signer: Pubkey,
        source_token_account: Pubkey,
        destination_token_account: Pubkey,
        a_to_b: bool,
        optional_account: Option<Pubkey>,
        optional_accounts: Option<Vec<Pubkey>>,
    ) -> anyhow::Result<Vec<solana_sdk::instruction::AccountMeta>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_update_liquidity_safely_pool() {
        // Invalid pool liquidity
        {
            let mut pool = PoolState {
                liquidity: Liquidity::new(0),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(1);
            let add = false;

            let result = pool.update_liquidity_safely(liquidity_delta, add);

            assert!(result.is_err());
        }
        // adding liquidity
        {
            let mut pool = PoolState {
                liquidity: Liquidity::from_integer(1),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(2);
            let add = true;

            pool.update_liquidity_safely(liquidity_delta, add).unwrap();

            assert_eq!({ pool.liquidity }, Liquidity::from_integer(3));
        }
        // subtracting liquidity
        {
            let mut pool = PoolState {
                liquidity: Liquidity::from_integer(3),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(2);
            let add = false;

            pool.update_liquidity_safely(liquidity_delta, add).unwrap();

            assert_eq!({ pool.liquidity }, Liquidity::from_integer(1));
        }
    }

    #[test]
    fn test_add_fee() {
        let pool = PoolState {
            protocol_fee: FixedPoint::from_scale(2, 1),
            liquidity: Liquidity::from_integer(10),
            ..Default::default()
        };
        // without referral
        {
            let mut pool = pool.clone();
            let amount = TokenAmount::from_integer(6);
            let ref_fee = pool.add_fee(amount, FixedPoint::from_integer(0), true);
            assert_eq!({ pool.fee_growth_global_x }, FeeGrowth::from_scale(4, 1));
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::from_integer(0));
            assert_eq!({ pool.fee_protocol_token_x }, 2);
            assert_eq!({ pool.fee_protocol_token_y }, 0);
            assert!(ref_fee.is_zero())
        }
        // with referral
        {
            let mut pool = pool.clone();
            let amount = TokenAmount::from_integer(200);
            let ref_fee = pool.add_fee(amount, FixedPoint::from_scale(1, 2), false);

            assert_eq!({ pool.fee_growth_global_x }, FeeGrowth::from_integer(0));
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::from_scale(158, 1));
            assert_eq!({ pool.fee_protocol_token_x }, 0);
            assert_eq!({ pool.fee_protocol_token_y }, 40);
            assert_eq!(ref_fee, TokenAmount(2));
        }
        // all tokens go to protocol fee
        {
            let mut pool = pool.clone();
            let amount = TokenAmount::new(1);
            let ref_fee = pool.add_fee(amount, FixedPoint::from_scale(2, 1), true);

            assert_eq!({ pool.fee_growth_global_x }, FeeGrowth::new(0));
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::new(0));
            assert_eq!({ pool.fee_protocol_token_x }, 1);
            assert_eq!({ pool.fee_protocol_token_y }, 0);
            assert_eq!(ref_fee, TokenAmount(0));
        }
    }

    #[test]
    fn test_update_seconds_per_liquidity_global() {
        let mut test_pool;
        //without overflow
        {
            test_pool = PoolState {
                seconds_per_liquidity_global: FixedPoint { v: (34028) },
                liquidity: Liquidity { v: (15) },
                last_timestamp: 15,
                ..Default::default()
            };
            let current_timestamp: u64 = 18446;
            test_pool.update_seconds_per_liquidity_global(current_timestamp);
            let result = test_pool.seconds_per_liquidity_global;
            assert_eq!(
                result,
                FixedPoint {
                    v: (1228733333333333367361)
                }
            );
        }

        //with overflow
        {
            test_pool = PoolState {
                seconds_per_liquidity_global: FixedPoint { v: (u128::MAX) },
                liquidity: Liquidity { v: (1) },
                last_timestamp: 0,
                ..Default::default()
            };
            let current_timestamp: u64 = u64::MAX;
            test_pool.update_seconds_per_liquidity_global(current_timestamp);
            let result = test_pool.seconds_per_liquidity_global;
            assert_eq!(
                result,
                FixedPoint {
                    v: (18446744073709551614999999999999999999)
                }
            );
        }
    }
}
