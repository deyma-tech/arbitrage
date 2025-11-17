use super::big_num::UnsafeMathTrait;
use super::big_num::U256;
use super::full_math::MulDiv;
//use super::unsafe_math::UnsafeMathTrait;
use super::fixed_point_64;

pub fn get_next_sqrt_price_from_amount_0_rounding_up(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount: u64,
    add: bool,
) -> anyhow::Result<u128> {
    if amount == 0 {
        return Ok(sqrt_price_x64);
    };
    let numerator_1 = (U256::from(liquidity)) << fixed_point_64::RESOLUTION;

    if add {
        if let Some(product) = U256::from(amount).checked_mul(U256::from(sqrt_price_x64)) {
            let denominator = numerator_1 + product;
            if denominator >= numerator_1 {
                if let Some(numerator_1) = numerator_1.mul_div_ceil(U256::from(sqrt_price_x64), denominator) {
                    return Ok(numerator_1.as_u128());
                }
                return Err(anyhow::anyhow!("RaydiumCLMMComputeSwap2"));
            };
        }

        // U256::div_rounding_up(
        //     numerator_1,
        //     (numerator_1 / U256::from(sqrt_price_x64))
        //         .checked_add(U256::from(amount))
        //         .un_wrap(),
        // )
        // .as_u128()

        if let Some(y) = (numerator_1 / U256::from(sqrt_price_x64)).checked_add(U256::from(amount)) {
            Ok(U256::div_rounding_up(numerator_1, y).as_u128())
        } else {
            Err(anyhow::anyhow!("RaydiumCLMMComputeSwap3"))
        }
    } else {
        // let product = U256::from(amount)
        //     .checked_mul(U256::from(sqrt_price_x64))
        //     .un_wrap();
        // let denominator = numerator_1.checked_sub(product).un_wrap();
        // numerator_1
        //     .mul_div_ceil(U256::from(sqrt_price_x64), denominator)
        //     .un_wrap()
        //     .as_u128()
        if let Some(product) = U256::from(amount).checked_mul(U256::from(sqrt_price_x64)) {
            if let Some(denominator) = numerator_1.checked_sub(product) {
                if let Some(numerator_1) = numerator_1.mul_div_ceil(U256::from(sqrt_price_x64), denominator) {
                    Ok(numerator_1.as_u128())
                } else {
                    Err(anyhow::anyhow!("RaydiumCLMMComputeSwap4"))
                }
            } else {
                Err(anyhow::anyhow!("RaydiumCLMMComputeSwap5"))
            }
        } else {
            Err(anyhow::anyhow!("RaydiumCLMMComputeSwap6"))
        }
    }
}

pub fn get_next_sqrt_price_from_amount_1_rounding_down(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount: u64,
    add: bool,
) -> anyhow::Result<u128> {
    if add {
        let quotient = U256::from(u128::from(amount) << fixed_point_64::RESOLUTION) / liquidity;
        sqrt_price_x64
            .checked_add(quotient.as_u128())
            .ok_or_else(|| anyhow::anyhow!("RaydiumCLMMGetNextSqrtPriceFromAmount1RoundDown1"))
    } else {
        let quotient = U256::div_rounding_up(
            U256::from(u128::from(amount) << fixed_point_64::RESOLUTION),
            U256::from(liquidity),
        );
        sqrt_price_x64
            .checked_sub(quotient.as_u128())
            .ok_or_else(|| anyhow::anyhow!("RaydiumCLMMGetNextSqrtPriceFromAmount1RoundDown2"))
    }
}

pub fn get_next_sqrt_price_from_input(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount_in: u64,
    zero_for_one: bool,
) -> anyhow::Result<u128> {
    assert!(sqrt_price_x64 > 0);
    assert!(liquidity > 0);

    // round to make sure that we don't pass the target price
    if zero_for_one {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price_x64, liquidity, amount_in, true)
    }
}

pub fn get_next_sqrt_price_from_output(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount_out: u64,
    zero_for_one: bool,
) -> anyhow::Result<u128> {
    assert!(sqrt_price_x64 > 0);
    assert!(liquidity > 0);

    if zero_for_one {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price_x64, liquidity, amount_out, false)
    } else {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_out, false)
    }
}
