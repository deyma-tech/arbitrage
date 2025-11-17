use crate::stabble_stable_swap::stable_math::{FixedDiv, FixedMul, BITS_ONE};
use fixed::traits::Fixed;
use fixed::types::extra::LeEqU64;
use fixed::types::U34F30;
use fixed::FixedU64;
use num_traits::{One, PrimInt, Zero};
use std::cmp::Ordering;
use typenum::{Bit, IsLessOrEqual, LeEq, True, Unsigned, U63};

pub trait FixedPow<RHS = Self> {
    /// Output type for the methods of this trait.
    type Output;

    fn pow_down(self, rhs: RHS) -> Option<Self::Output>;

    fn pow_up(self, rhs: RHS) -> Option<Self::Output>;
}

impl FixedPow for u64 {
    type Output = u64;
    // Optimize for when y equals 1.0, 2.0, 3.0 or 4.0, as those are very simple to implement and occur often in
    // 50/50, 80/20 and 60/20/20 Weighted Pools

    fn pow_down(self, rhs: Self) -> Option<Self::Output> {
        {
            let base = U34F30::from_bits(self.mul_down(BITS_ONE)?);
            let exp = U34F30::from_bits(rhs.mul_down(BITS_ONE)?);
            base.powf(exp)?.to_bits().div_down(BITS_ONE)
        }
    }

    fn pow_up(self, rhs: Self) -> Option<Self::Output> {
        {
            let base = U34F30::from_bits(self.mul_up(BITS_ONE)?);
            let exp = U34F30::from_bits(rhs.mul_up(BITS_ONE)?);
            base.powf(exp)?.to_bits().div_up(BITS_ONE)
        }
    }
}

/// Extension trait providing fixed-point exponentiation for fixed-point numbers.
pub trait FixedPowF: Fixed {
    /// Raises a number to a fixed-point power.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::types::U34F30;
    /// use fixed_exp::FixedPowF;
    ///
    /// let x = U34F30::from_num(4.0);
    /// assert_eq!(U34F30::from_num(8.0), x.powf(U34F30::from_num(1.5)).un_wrap());
    /// ```
    fn powf(self, n: Self) -> Option<Self>;
}

#[inline(always)]
fn powi<T: Fixed>(mut x: T, mut n: i32) -> Option<T> {
    // n cannot be zero

    let mut acc = x;
    n -= 1;

    while n > 0 {
        if n & 1 == 1 {
            acc = acc.checked_mul(x)?;
        }
        x = x.checked_mul(x)?;
        n >>= 1;
    }

    Some(acc)
}

#[inline(always)]
fn sqrt<T>(x: T) -> Option<T>
where
    T: Fixed + Helper,
    T::Bits: PrimInt,
{
    if x.is_zero() || x.is_one() {
        return Some(x);
    }

    let mut pow2 = T::one()?;
    let mut result;

    if x < T::one()? {
        while x <= pow2.checked_mul(pow2)? {
            pow2 >>= 1;
        }

        result = pow2;
    } else {
        // x >= T::one()
        while pow2.checked_mul(pow2)? <= x {
            pow2 <<= 1;
        }

        result = pow2 >> 1;
    }

    for _ in 0..T::NUM_BITS {
        pow2 >>= 1;
        let next_result = result.checked_add(pow2)?;
        if next_result.checked_mul(next_result)? <= x {
            result = next_result;
        }
    }

    Some(result)
}

#[inline(always)]
fn powf_01<T>(mut x: T, n: T) -> Option<T>
where
    T: Fixed + Helper,
    T::Bits: PrimInt + std::fmt::Debug,
{
    // n cannot be zero
    let mut n = n.to_bits();

    let top = T::Bits::one() << ((T::Frac::U32).checked_sub(1)? as usize);
    let mask = !(T::Bits::one() << ((T::Frac::U32) as usize));
    let mut acc = None;

    while !n.is_zero() {
        x = sqrt(x)?;
        if !(n & top).is_zero() {
            acc = match acc {
                Some(acc) => Some(acc * x),
                None => Some(x),
            };
        }
        n = (n << 1) & mask;
    }

    acc
}

#[inline(always)]
fn powf<T>(x: T, n: T) -> Option<T>
where
    T: Fixed + Helper,
    T::Bits: PrimInt + std::fmt::Debug,
{
    if x.is_zero() {
        return Some(T::ZERO);
    }

    let int = n.int();
    let frac = n.frac();

    if int.is_zero() {
        if frac.is_zero() {
            T::one()
        } else {
            powf_01(x, frac)
        }
    } else {
        let powi = powi(x, int.to_num());

        if frac.is_zero() {
            powi
        } else {
            powi?.checked_mul(powf_01(x, frac)?)
        }
    }
}

macro_rules! impl_fixed_pow {
    ($fixed:ident, $le_eq:ident, $le_eq_one:ident) => {
        impl<Frac> FixedPowF for $fixed<Frac>
        where
            Frac: $le_eq + IsLessOrEqual<$le_eq_one, Output = True>,
        {
            fn powf(self, n: Self) -> Option<Self> {
                let zero = Self::from_bits(0);

                if !<LeEq<Frac, $le_eq_one>>::BOOL && n <= zero {
                    // panic!(
                    //     "cannot raise `{}` to the power of `{}` because numbers larger than or equal to `1` are not representable",
                    //     self, n
                    // );
                    return None;
                }

                match n.cmp(&zero) {
                    Ordering::Greater => powf(self, n),
                    Ordering::Equal => Some(Self::from_bits(1 << Frac::U32)),
                    Ordering::Less => powf(
                        Self::from_bits(1 << Frac::U32).checked_div(self)?,
                        Helper::neg(n)?,
                    ),
                }
            }
        }
    };
}

impl_fixed_pow!(FixedU64, LeEqU64, U63);

trait Helper {
    const NUM_BITS: u32;
    fn is_one(&self) -> bool;
    fn one() -> Option<Self>
    where
        Self: Sized;
    fn neg(self) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! impl_sign_helper {
    (signed, $fixed:ident, $le_eq:ident, $le_eq_one:ident) => {
        impl<Frac: $le_eq> Helper for $fixed<Frac>
        where
            Frac: $le_eq + IsLessOrEqual<$le_eq_one>,
        {
            const NUM_BITS: u32 = <Self as Fixed>::INT_NBITS + <Self as Fixed>::FRAC_NBITS;
            fn is_one(self) -> bool {
                <LeEq<Frac, $le_eq_one>>::BOOL && self.to_bits() == 1 << Frac::U32
            }
            fn one() -> Option<Self> {
                if <LeEq<Frac, $le_eq_one>>::BOOL {
                    Some(Self::from_bits(1 << Frac::U32))
                } else {
                    None
                }
            }
            fn neg(self) -> Option<Self> {
                Some(-self)
            }
        }
    };
    (unsigned, $fixed:ident, $le_eq:ident, $le_eq_one:ident) => {
        impl<Frac: $le_eq> Helper for $fixed<Frac>
        where
            Frac: $le_eq + IsLessOrEqual<$le_eq_one>,
        {
            const NUM_BITS: u32 = <Self as Fixed>::INT_NBITS + <Self as Fixed>::FRAC_NBITS;
            fn is_one(&self) -> bool {
                <LeEq<Frac, $le_eq_one>>::BOOL && self.to_bits() == 1 << Frac::U32
            }
            fn one() -> Option<Self> {
                if <LeEq<Frac, $le_eq_one>>::BOOL {
                    Some(Self::from_bits(1 << Frac::U32))
                } else {
                    None
                }
            }
            fn neg(self) -> Option<Self> {
                None
            }
        }
    };
}

impl_sign_helper!(unsigned, FixedU64, LeEqU64, U63);
