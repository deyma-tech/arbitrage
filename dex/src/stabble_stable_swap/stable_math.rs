#![allow(clippy::assign_op_pattern)]
#![allow(clippy::manual_div_ceil)]

use uint::construct_uint;

construct_uint! {
    pub struct U192(3);
}

#[macro_export]
macro_rules! uint192 {
    ($value:expr) => {
        U192::from($value)
    };
}

pub const AMP_PRECISION: u64 = 1_000;

pub const MIN_AMP: u16 = 1;
pub const MAX_AMP: u16 = 8000;

pub const MIN_SWAP_FEE: u64 = 10_000; // 0.001%
pub const MAX_SWAP_FEE: u64 = 10_000_000; // 1%

// Safe max balance supported by stable_math
pub const MAX_SAFE_BALANCE: u64 = 3_000_000_000_000_000_000; // 3B

pub const INV_THRESHOLD: u64 = 100;
pub const BALANCE_THRESHOLD: u64 = 1;

pub const MIN_TOKENS: usize = 2;
pub const MAX_TOKENS: usize = 5;

#[inline(always)]
pub fn amp_precision_u192() -> U192 {
    uint192!(AMP_PRECISION)
}

// StableMath._calculateInvariant
// Computes the invariant given the current balances, using the Newton-Raphson approximation.
// The amplification parameter equals: A n^(n-1)
// See: https://github.com/stabbleorg/balancer-v2-monorepo/blob/master/pkg/pool-stable/contracts/StableMath.sol#L57-L120
pub fn calc_invariant(amplification: u64, balances: &[u64]) -> Option<u64> {
    // invariant                                                                                 //
    // D = invariant                                                  D^(n+1)                    //
    // A = amplification coefficient      A  n^n S + D = A D n^n + -----------                   //
    // S = sum of balances                                             n^n P                     //
    // P = product of balances                                                                   //
    // n = number of tokens                                                                      //

    // Always round down, to match Vyper's arithmetic (which always truncates).
    let sum: u64 = balances.iter().sum(); // S in the Curve version

    if sum == 0 {
        return Some(0);
    }

    let num_tokens = balances.len();
    let num_tokens_u64 = num_tokens as u64;
    let num_tokens_u192 = uint192!(num_tokens_u64);

    let amp_times_total = amplification.checked_mul(num_tokens_u64)?; // Ann in the Curve version

    let sum = uint192!(sum);
    let mut invariant = sum; // D in the Curve version

    // Precompute balances[i] * num_tokens
    let mut balances_times: Vec<U192> = Vec::with_capacity(balances.len());
    for &balance in balances.iter() {
        balances_times.push(uint192!(balance.checked_mul(num_tokens_u64)?));
    }

    for _ in 0..255 {
        let mut p = invariant;

        for &balance_times in balances_times.iter() {
            // (p * invariant) / (balances[i] * num_tokens)
            p = p.checked_mul_div_down(invariant, balance_times)?;
        }

        let prev_invariant = invariant; // Dprev in the Curve version

        invariant = (uint192!(amp_times_total)
            .checked_mul_div_down(sum, amp_precision_u192())?
            .checked_add(p.checked_mul(num_tokens_u192)?))?
        .checked_mul_div_down(
            invariant,
            uint192!(amp_times_total.checked_sub(AMP_PRECISION)?)
                .checked_mul_div_down(invariant, amp_precision_u192())?
                .checked_add(uint192!(num_tokens.saturating_add(1)).checked_mul(p)?)?,
        )?;

        let invariant = invariant.as_u64()?;
        let prev_invariant = prev_invariant.as_u64()?;

        if invariant > prev_invariant {
            if invariant.saturating_sub(prev_invariant) <= INV_THRESHOLD {
                return Some(invariant);
            }
        } else if prev_invariant.saturating_sub(invariant) <= INV_THRESHOLD {
            return Some(invariant);
        }
    }

    None
}

// Computes how many tokens can be taken out of a pool if `token_amount_in` are sent, given the current balances.
// The amplification parameter equals: A n^(n-1)
// See: https://github.com/stabbleorg/balancer-v2-monorepo/blob/master/pkg/pool-stable/contracts/StableMath.sol#L124-L159
pub fn calc_out_given_in(
    amplification: u64,
    balances: &[u64],
    token_index_in: usize,
    token_index_out: usize,
    token_amount_in: u64,
    invariant: u64,
) -> Option<u64> {
    /**************************************************************************************************************
    // outGivenIn token x for y - polynomial equation to solve                                                   //
    // ay = amount out to calculate                                                                              //
    // by = balance token out                                                                                    //
    // y = by - ay (finalBalanceOut)                                                                             //
    // D = invariant                                               D                     D^(n+1)                 //
    // A = amplification coefficient               y^2 + ( S + ----------  - D) * y -  ------------- = 0         //
    // n = number of tokens                                    (A * n^n)               A * n^2n * P              //
    // S = sum of final balances but y                                                                           //
    // P = product of final balances but y                                                                       //
     **************************************************************************************************************/
    // Amount out, so we round down overall.

    let mut new_balances = Vec::with_capacity(balances.len());
    for (i, balance) in balances.iter().enumerate() {
        if i == token_index_in {
            new_balances.push(balance.checked_add(token_amount_in)?);
        } else {
            new_balances.push(*balance);
        }
    }

    let balance_out = *balances.get(token_index_out)?;

    let final_balance_out =
        get_token_balance_given_invariant_n_all_other_balances(amplification, &new_balances, invariant, balance_out)?;

    balance_out.checked_sub(final_balance_out)?.checked_sub(1)
}

// Computes how many tokens must be sent to a pool if `token_amount_out` are sent given the
// current balances, using the Newton-Raphson approximation.
// The amplification parameter equals: A n^(n-1)
// See: https://github.com/stabbleorg/balancer-v2-monorepo/blob/master/pkg/pool-stable/contracts/StableMath.sol#L164-L199
pub fn calc_in_given_out(
    amplification: u64,
    balances: &[u64],
    token_index_in: usize,
    token_index_out: usize,
    token_amount_out: u64,
    invariant: u64,
) -> Option<u64> {
    /**************************************************************************************************************
    // inGivenOut token x for y - polynomial equation to solve                                                   //
    // ax = amount in to calculate                                                                               //
    // bx = balance token in                                                                                     //
    // x = bx + ax (finalBalanceIn)                                                                              //
    // D = invariant                                                D                     D^(n+1)                //
    // A = amplification coefficient               x^2 + ( S + ----------  - D) * x -  ------------- = 0         //
    // n = number of tokens                                     (A * n^n)               A * n^2n * P             //
    // S = sum of final balances but x                                                                           //
    // P = product of final balances but x                                                                       //
     **************************************************************************************************************/
    // Amount in, so we round up overall.
    let mut new_balances = Vec::with_capacity(balances.len());
    for (i, balance) in balances.iter().enumerate() {
        if i == token_index_out {
            new_balances.push(balance.checked_sub(token_amount_out)?);
        } else {
            new_balances.push(*balance);
        }
    }

    let balance_in = *balances.get(token_index_in)?;

    let final_balance_in =
        get_token_balance_given_invariant_n_all_other_balances(amplification, &new_balances, invariant, balance_in)?;

    final_balance_in.checked_sub(balance_in)?.checked_add(1)
}

// See: https://github.com/stabbleorg/balancer-v2-monorepo/blob/master/pkg/pool-stable/contracts/StableMath.sol#L201-L255
pub fn calc_pool_token_out_given_exact_tokens_in(
    amplification: u64,
    balances: &[u64],
    amounts_in: &[u64],
    pool_token_supply: u64,
    current_invariant: u64,
    swap_fee: u64,
) -> Option<u64> {
    // LP out, so we round down overall.

    // First loop calculates the sum of all token balances, which will be used to calculate
    // the current weights of each token, relative to this sum
    let sum: u64 = balances.iter().sum();

    // Calculate the weighted balance ratio without considering fees
    let mut balance_ratios_with_fee = Vec::with_capacity(balances.len());
    // The weighted sum of token balance ratios with fee
    let mut invariant_ratio_with_fees = 0;
    for i in 0..balances.len() {
        let current_weight = balances[i].div_down(sum)?;
        balance_ratios_with_fee.push(balances[i].checked_add(amounts_in[i])?.div_down(balances[i])?);
        invariant_ratio_with_fees = balance_ratios_with_fee[i]
            .mul_down(current_weight)?
            .checked_add(invariant_ratio_with_fees)?;
    }

    // Second loop calculates new amounts in, taking into account the fee on the percentage excess
    let mut new_balances = Vec::with_capacity(balances.len());
    for i in 0..balances.len() {
        let amount_in_without_fee = if balance_ratios_with_fee[i] > invariant_ratio_with_fees {
            let non_taxable_amount = balances[i].mul_down(invariant_ratio_with_fees.checked_sub(ONE)?)?;
            let taxable_amount = amounts_in[i].checked_sub(non_taxable_amount)?;

            taxable_amount
                .mul_down(swap_fee.complement())?
                .checked_add(non_taxable_amount)?
        } else {
            amounts_in[i]
        };

        new_balances.push(balances[i].checked_add(amount_in_without_fee)?);
    }

    let new_invariant = calc_invariant(amplification, &new_balances)?;
    let invariant_ratio = new_invariant.div_down(current_invariant)?;

    // If the invariant didn't increase for any reason, we simply don't mint LP
    if invariant_ratio > ONE {
        pool_token_supply.mul_down(invariant_ratio.saturating_sub(ONE))
    } else {
        Some(0)
    }
}

// See: https://github.com/stabbleorg/balancer-v2-monorepo/blob/master/pkg/pool-stable/contracts/StableMath.sol#L354-L395
pub fn calc_token_out_given_exact_pool_token_in(
    amplification: u64,
    balances: &[u64],
    token_index: usize,
    amount_in: u64,
    pool_token_supply: u64,
    current_invariant: u64,
    swap_fee: u64,
) -> Option<u64> {
    // Token out, so we round down overall.

    let new_invariant = pool_token_supply
        .checked_sub(amount_in)?
        .checked_mul_div_up(current_invariant, pool_token_supply)?;

    let balance = *balances.get(token_index)?;

    // Calculate amount out without fee
    let new_balance =
        get_token_balance_given_invariant_n_all_other_balances(amplification, balances, new_invariant, balance)?;
    let amount_out_without_fee = balance.checked_sub(new_balance)?;

    // First calculate the sum of all token balances, which will be used to calculate
    // the current weight of each token
    let sum: u64 = balances.iter().sum();

    // We can now compute how much excess balance is being withdrawn as a result of the virtual swaps, which result
    // in swap fees.
    let current_weight = balance.div_down(sum)?;
    let taxable_percentage = current_weight.complement();

    // Swap fees are typically charged on 'token in', but there is no 'token in' here, so we apply it
    // to 'token out'. This results in slightly larger price impact. Fees are rounded up.
    let taxable_amount = amount_out_without_fee.mul_up(taxable_percentage)?;
    let non_taxable_amount = amount_out_without_fee.saturating_sub(taxable_amount);

    taxable_amount
        .mul_down(swap_fee.complement())?
        .checked_add(non_taxable_amount)
}

// This function calculates the balance of a given token (token_index)
// given all the other balances and the invariant
// See: https://github.com/stabbleorg/balancer-v2-monorepo/blob/master/pkg/pool-stable/contracts/StableMath.sol#L399-L449
fn get_token_balance_given_invariant_n_all_other_balances(
    amplification: u64,
    balances: &[u64],
    invariant: u64,
    balance: u64, // balance of a given token (token_index)
) -> Option<u64> {
    // Rounds result up overall

    let num_tokens = balances.len() as u64;
    let amp_times_total = uint192!(amplification.checked_mul(num_tokens)?);

    let invariant = uint192!(invariant);

    let mut sum = balances[0];
    let mut p = uint192!(balances[0].checked_mul(num_tokens)?);
    for balance in balances.iter().skip(1) {
        let p_i = uint192!(balance.checked_mul(num_tokens)?);
        p = p.checked_mul_div_down(p_i, invariant)?;
        sum = sum.checked_add(*balance)?;
    }

    // No need to use safe math, based on the loop above `sum` is greater than or equal to `balances[token_index]`
    sum = sum.saturating_sub(balance);
    let sum = uint192!(sum);

    let invariant_2 = invariant.checked_mul(invariant)?;
    // We remove the balance from c by multiplying it
    let c = invariant_2
        .checked_mul_div_up(amp_precision_u192(), amp_times_total.checked_mul(p)?)?
        .checked_mul(uint192!(balance))?;
    let b = invariant
        .checked_mul_div_down(amp_precision_u192(), amp_times_total)?
        .checked_add(sum)?;

    // We iterate to find the balance
    // We multiply the first iteration outside the loop with the invariant to set the value of the
    // initial approximation.
    let mut token_balance = invariant_2.checked_add(c)?.checked_div_up(invariant.checked_add(b)?)?;

    for _ in 0..255 {
        let prev_token_balance = token_balance;

        token_balance = token_balance
            .checked_mul(token_balance)?
            .checked_add(c)?
            .checked_div_up(
                // No need to use checked arithmetic because max value of `token_balance` is u128::MAX
                (token_balance << 1).checked_add(b)?.checked_sub(invariant)?, // token_balance * 2 + b - invariant
            )?;

        let token_balance = token_balance.as_u64()?;
        let prev_token_balance = prev_token_balance.as_u64()?;

        if token_balance > prev_token_balance {
            if token_balance.saturating_sub(prev_token_balance) <= BALANCE_THRESHOLD {
                return Some(token_balance);
            }
        } else if prev_token_balance.saturating_sub(token_balance) <= BALANCE_THRESHOLD {
            return Some(token_balance);
        }
    }

    None
}

pub trait CheckedMulDiv<RHS = Self> {
    /// Output type for the methods of this trait.
    type Output;

    /// Calculates `floor(val * num / denom)`, i.e. the largest integer less than or equal to the
    /// result of the division.
    fn checked_mul_div_down(self, num: RHS, denom: RHS) -> Option<Self::Output>;

    /// Calculates `ceil(val * num / denom)`, i.e. the the smallest integer greater than or equal to
    /// the result of the division.
    fn checked_mul_div_up(self, num: RHS, denom: RHS) -> Option<Self::Output>;
}

pub trait CheckedDivCeil<RHS = Self> {
    /// Output type for the methods of this trait.
    type Output;

    /// Calculates `ceil(val / denom)`, i.e. the the smallest integer greater than or equal to
    /// the result of the division.
    fn checked_div_up(self, denom: RHS) -> Option<Self::Output>;
}

pub trait CheckedDivFloor<RHS = Self> {
    /// Output type for the methods of this trait.
    type Output;

    /// Calculates `floor(val / denom)`, i.e. the largest integer less than or equal to the
    /// result of the division.
    fn checked_div_down(self, denom: RHS) -> Option<Self::Output>;
}

pub trait Upcast {
    #[allow(clippy::wrong_self_convention)]
    fn as_u192(self) -> U192;
}

pub trait Downcast {
    #[allow(clippy::wrong_self_convention)]
    fn as_u64(self) -> Option<u64>;
}

impl Upcast for u128 {
    fn as_u192(self) -> U192 {
        uint192!(self)
    }
}

impl Downcast for U192 {
    fn as_u64(self) -> Option<u64> {
        if !self.fits_word() {
            return None;
        }

        Some(self.0[0])
    }
}

impl CheckedMulDiv for u64 {
    type Output = u64;

    fn checked_mul_div_down(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom == 0 {
            return None;
        }

        let r = (self as u128).checked_mul(num as u128)?.checked_div(denom as u128)?;
        if r > u64::MAX as u128 {
            None
        } else {
            Some(r as u64)
        }
    }

    fn checked_mul_div_up(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom == 0 {
            return None;
        }

        let r = (self as u128)
            .checked_mul(num as u128)?
            .checked_add(denom.saturating_sub(1) as u128)?
            .checked_div(denom as u128)?;
        if r > u64::MAX as u128 {
            None
        } else {
            Some(r as u64)
        }
    }
}

impl CheckedDivCeil for u64 {
    type Output = u64;

    fn checked_div_up(self, denom: Self) -> Option<Self::Output> {
        if denom == 0 {
            return None;
        }

        let r = (self as u128)
            .checked_add(denom.saturating_sub(1) as u128)?
            .checked_div(denom as u128)?;
        if r > u64::MAX as u128 {
            None
        } else {
            Some(r as u64)
        }
    }
}

impl CheckedMulDiv for U192 {
    type Output = U192;

    fn checked_mul_div_down(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom == U192::default() {
            return None;
        }

        let r = self.checked_mul(num)?.checked_div(denom)?;
        if r > u128::MAX.as_u192() {
            None
        } else {
            Some(r)
        }
    }

    fn checked_mul_div_up(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom == U192::default() {
            return None;
        }

        let r = self.checked_mul(num)?.checked_add(denom - 1)?.checked_div(denom)?;
        if r > u128::MAX.as_u192() {
            None
        } else {
            Some(r)
        }
    }
}

impl CheckedDivCeil for U192 {
    type Output = U192;

    fn checked_div_up(self, denom: Self) -> Option<Self::Output> {
        if denom == U192::default() {
            return None;
        }

        let r = self.checked_add(denom - 1)?.checked_div(denom)?;
        if r > u128::MAX.as_u192() {
            None
        } else {
            Some(r)
        }
    }
}

impl CheckedDivFloor for U192 {
    type Output = U192;

    fn checked_div_down(self, denom: Self) -> Option<Self::Output> {
        if denom == U192::default() {
            return None;
        }

        let r = self.checked_div(denom)?;
        if r > u128::MAX.as_u192() {
            None
        } else {
            Some(r)
        }
    }
}

pub const ZERO: u64 = 0;

pub const ONE: u64 = 1_000_000_000;

pub const TWO: u64 = 2_000_000_000;

pub const THREE: u64 = 3_000_000_000;

pub const FOUR: u64 = 4_000_000_000;

pub const SCALE: u32 = 9;

pub const BITS_ONE: u64 = 1073741824; // 1 << 30

pub trait FixedPow<RHS = Self> {
    /// Output type for the methods of this trait.
    type Output;

    fn pow_down(self, rhs: RHS) -> Option<Self::Output>;

    fn pow_up(self, rhs: RHS) -> Option<Self::Output>;
}

pub trait FixedMul<RHS = Self> {
    /// Output type for the methods of this trait.
    type Output;

    fn mul_down(self, rhs: RHS) -> Option<Self::Output>;

    fn mul_up(self, rhs: RHS) -> Option<Self::Output>;
}

pub trait FixedDiv<RHS = Self> {
    /// Output type for the methods of this trait.
    type Output;

    fn div_down(self, rhs: RHS) -> Option<Self::Output>;

    fn div_up(self, rhs: RHS) -> Option<Self::Output>;
}

pub trait FixedComplement<RHS = Self> {
    /// Output type for the methods of this trait.
    type Output;

    fn complement(self) -> Self::Output;
}

// impl FixedPow for u64 {
//     type Output = u64;
//     // Optimize for when y equals 1.0, 2.0, 3.0 or 4.0, as those are very simple to implement and occur often in
//     // 50/50, 80/20 and 60/20/20 Weighted Pools
//
//     fn pow_down(self, rhs: Self) -> Option<Self::Output> {
//         match rhs {
//             ZERO => Some(ONE),
//             ONE => Some(self),
//             TWO => self.mul_down(self),
//             THREE => self.mul_down(self)?.mul_down(self),
//             FOUR => {
//                 let square = self.mul_down(self)?;
//                 square.mul_down(square)
//             }
//             _ => {
//                 let base = U34F30::from_bits(self.mul_down(BITS_ONE)?);
//                 let exp = U34F30::from_bits(rhs.mul_down(BITS_ONE)?);
//                 base.powf(exp)?.to_bits().div_down(BITS_ONE)
//             }
//         }
//     }
//
//     // fn pow_up(self, rhs: Self) -> Option<Self::Output> {
//     //     match rhs {
//     //         ZERO => Some(ONE),
//     //         ONE => Some(self),
//     //         TWO => self.mul_up(self),
//     //         THREE => self.mul_up(self)?.mul_up(self),
//     //         FOUR => {
//     //             let square = self.mul_up(self)?;
//     //             square.mul_up(square)
//     //         }
//     //         _ => {
//     //             let base = U34F30::from_bits(self.mul_up(BITS_ONE)?);
//     //             let exp = U34F30::from_bits(rhs.mul_up(BITS_ONE)?);
//     //             base.powf(exp)?.to_bits().div_up(BITS_ONE)
//     //         }
//     //     }
//     // }
// }

impl FixedMul for u64 {
    type Output = u64;

    fn mul_down(self, rhs: Self) -> Option<Self::Output> {
        self.checked_mul_div_down(rhs, ONE)
    }

    fn mul_up(self, rhs: Self) -> Option<Self::Output> {
        self.checked_mul_div_up(rhs, ONE)
    }
}

impl FixedDiv for u64 {
    type Output = u64;

    fn div_down(self, rhs: Self) -> Option<Self::Output> {
        self.checked_mul_div_down(ONE, rhs)
    }

    fn div_up(self, rhs: Self) -> Option<Self::Output> {
        self.checked_mul_div_up(ONE, rhs)
    }
}

impl FixedComplement for u64 {
    type Output = u64;

    fn complement(self) -> Self::Output {
        ONE.saturating_sub(self)
    }
}
