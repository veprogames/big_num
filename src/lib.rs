//! # Big Number
//!
//! `bignum-ig` is a crate that supplies a Big Number Type [Big], which allows expressing
//! much larger or smaller numbers than [prim@f64] can do. This is done through an extra [prim@i64] exponent
//! field to increase the exponent range.
//!
//! [Big] has the same precision as a [prim@f64] and the same floating point arithmetic quirks.
//! The primary use of this crate is for [Incremental Games](https://en.wikipedia.org/wiki/Incremental_game),
//! a game genre which can feature very large numbers.

use std::{
    f64,
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

mod comparison;
mod conversion;
#[cfg(test)]
mod tests;

/// # The Big Number Type
///
/// A Number in the range of 10<sup>[i64::MIN]</sup>..10.0*10<sup>[i64::MAX]</sup> (exclusive).
///
/// # Basic Usage
///
/// Create Numbers using [Big::new()] and [Big::from()].
///
/// Operate on these numbers with regular operators: +, -, *, /
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Big {
    /// A normal number in the form of Mantissa * 10<sup>Exponent</sup>
    Number {
        /// Mantissa, ranging from 1.0 to 10.0 exclusively
        m: f64,
        /// Exponent
        e: i64,
    },
    /// Not a Number, never equal to itself
    NaN,
    /// Positive or Negative Infinity
    Infinity(InfinityKind),
    /// ± 0
    Zero,
}

/// This type is used to describe if an Infinity is positive or negative.
/// You will rarely use it yourself. You should look at [Big::is_pos_inf()] and [Big::is_neg_inf()] instead
/// There are also [crate::POS_INFINITY] and [crate::NEG_INFINITY] for ease of use.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InfinityKind {
    /// Positive Infinity, also referred to as +inf
    Positive,
    /// Negative Infinity, also referred to as -inf
    Negative,
}

/// A Constant Describing Positive Infinity
pub const POS_INFINITY: Big = Big::Infinity(InfinityKind::Positive);
/// A Constant Describing Negative Infinity
pub const NEG_INFINITY: Big = Big::Infinity(InfinityKind::Negative);
const SIG_DIGITS: i64 = 15;

impl Big {
    /// Create a new Instance. The Number is normalized automatically.
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// let number = Big::new(1.0, 1);
    /// assert_eq!(number, Big::new(10.0, 0));
    /// ```
    pub fn new(mantissa: f64, exponent: i64) -> Self {
        let mut number = Self::Number {
            m: mantissa,
            e: exponent,
        };
        number.normalize();
        number
    }

    /// Create a new Instance, skipping normalization
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call [Big::normalize()] yourself at some point to prevent bugs.
    ///
    /// You will most likely want to use Big::new or Big::From instead
    pub fn new_unnormalized(mantissa: f64, exponent: i64) -> Self {
        Self::Number {
            m: mantissa,
            e: exponent,
        }
    }

    /// Normalize the number so it is in a correct state.
    ///
    /// **Note:** Unless you used any `_unnormalized` method, you never need to call this manually.
    pub fn normalize(&mut self) {
        match *self {
            Self::Infinity(_) | Self::NaN | Self::Zero => return,
            Self::Number { m, .. } => match m {
                m if m == 0.0 => {
                    *self = Self::Zero;
                    return;
                }
                m if m.is_nan() => {
                    *self = Self::NaN;
                    return;
                }
                m if m == f64::INFINITY => {
                    *self = POS_INFINITY;
                    return;
                }
                m if m == f64::NEG_INFINITY => {
                    *self = NEG_INFINITY;
                    return;
                }
                // if the number is already normalized, we can skip everything below
                m if (1.0..10.0).contains(&m) => return,
                // see below
                _ => {}
            },
        };

        if let Self::Number {
            ref mut m,
            ref mut e,
        } = *self
        {
            let log = m.abs().log10() as i64;

            match log {
                // might underflow to Zero
                ..=0 => {
                    if e.checked_sub(-log).is_none() {
                        *self = Self::Zero;
                        return;
                    }
                }
                // might overflow to either positive or negative Infinity (+inf; -inf)
                _positive => {
                    if e.checked_add(log).is_none() {
                        match m.is_sign_positive() {
                            true => *self = POS_INFINITY,
                            false => *self = NEG_INFINITY,
                        };
                        return;
                    }
                }
            }

            let log = m.abs().log10().floor() as i64;
            *m /= 10.0_f64.powi(log as i32);
            *e += log;
        }
    }

    /// This will invert the sign of `self`, modifying it in-place.
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// let mut number = Big::from(42);
    /// number.neg_mut();
    /// assert_eq!(number, Big::from(-42));
    /// ```
    pub fn neg_mut(&mut self) {
        // normalization should not be required here
        match self {
            Self::Infinity(InfinityKind::Positive) => *self = NEG_INFINITY,
            Self::Infinity(InfinityKind::Negative) => *self = POS_INFINITY,
            Self::Number { m, .. } => {
                *m *= -1.0;
            }
            _ => {}
        }
    }

    /// This will add `rhs` to `self` without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call [Big::normalize()] yourself at some point to prevent bugs.
    ///
    /// You will most likely want to use the += or + operator instead, which will normalize the result automatically.
    pub fn add_mut_unnormalized(&mut self, rhs: Self) {
        match (&self, &rhs) {
            // NaN
            (Self::NaN, _) | (_, Self::NaN) => *self = Self::NaN,

            // Infinities

            // +inf + -inf and -inf + +inf are undefined
            (Self::Infinity(kind), Self::Infinity(kind2)) if kind != kind2 => *self = Self::NaN,
            (Self::Infinity(_), Self::Infinity(_)) => return,

            (Self::Infinity(_), Self::Number { .. } | Self::Zero) => {
                return;
            }
            (Self::Number { .. } | Self::Zero, Self::Infinity(kind)) => {
                *self = Self::Infinity(kind.clone())
            }

            // Zero
            (Self::Zero, other) => {
                *self = other.clone();
            }
            (Self::Number { .. }, Self::Zero) => return,

            // see below
            (Self::Number { .. }, Self::Number { .. }) => {}
        }

        // a + b
        if let (
            Self::Number { m, e },
            Self::Number {
                m: other_m,
                e: other_e,
            },
        ) = (self, rhs)
        {
            let delta = other_e - *e;
            match delta {
                // ..=-SIG_DIGITS produced a syntax error
                _delta if delta <= -SIG_DIGITS => {}
                _delta if delta >= SIG_DIGITS => {
                    *m = other_m;
                    *e = other_e;
                }
                delta => {
                    let delta: i32 = delta.try_into().expect(
                        "exponent delta between a, b in a + b should never exceed 13
                                                        and can therefore be cast into i32",
                    );

                    *m += other_m * 10.0_f64.powi(delta);
                }
            }
        };
    }

    /// This will substract `rhs` from `self` without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call [Big::normalize()] yourself at some point to prevent bugs.
    ///
    /// You will most likely want to use the -= or - operator instead, which will normalize the result automatically.
    pub fn sub_mut_unnormalized(&mut self, rhs: Self) {
        match (&self, &rhs) {
            // NaN
            (Self::NaN, _) | (_, Self::NaN) => *self = Self::NaN,

            // Infinities

            // +inf - -inf and -inf - +inf are undefined
            (Self::Infinity(_), Self::Infinity(_)) => *self = Self::NaN,

            (Self::Infinity(_), Self::Number { .. } | Self::Zero) => {
                return;
            }
            (Self::Number { .. } | Self::Zero, Self::Infinity(InfinityKind::Positive)) => {
                *self = NEG_INFINITY;
            }
            (Self::Number { .. } | Self::Zero, Self::Infinity(InfinityKind::Negative)) => {
                *self = POS_INFINITY;
            }

            // Zero
            (Self::Zero, other) => {
                *self = other.clone();
                self.neg_mut();
            }
            (Self::Number { .. }, Self::Zero) => return,

            // see below
            (Self::Number { .. }, Self::Number { .. }) => {}
        }

        // a - b
        if let (
            Self::Number { m, e },
            Self::Number {
                m: other_m,
                e: other_e,
            },
        ) = (self, rhs)
        {
            let delta = other_e - *e;
            match delta {
                // ..=-SIG_DIGITS produced a syntax error
                _delta if delta <= -SIG_DIGITS => {}
                _delta if delta >= SIG_DIGITS => {
                    *m = other_m;
                    *e = other_e;
                }
                delta => {
                    let delta: i32 = delta.try_into().expect(
                        "exponent delta between a, b in a - b should never exceed 13
                                                            and can therefore be cast into i32",
                    );

                    *m -= other_m * 10.0_f64.powi(delta);
                }
            }
        };
    }

    /// This will multiply `rhs` onto `self` without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call [Big::normalize()] yourself at some point to prevent bugs.
    ///
    /// You will most likely want to use the *= or * operator instead, which will normalize the result automatically.
    pub fn mul_mut_unnormalized(&mut self, rhs: Self) {
        match (&self, &rhs) {
            // NaN
            (Self::NaN, _) | (_, Self::NaN) => *self = Self::NaN,

            // Infinities
            (Self::Infinity(_), Self::Infinity(InfinityKind::Positive)) => return,
            (_, Self::Infinity(InfinityKind::Negative)) => *self = NEG_INFINITY,
            (Self::Zero, Self::Infinity(_)) | (Self::Infinity(_), Self::Zero) => *self = Self::NaN,
            (Self::Number { .. }, Self::Infinity(kind)) => *self = Self::Infinity(kind.clone()),
            (Self::Infinity(_), Self::Number { .. }) => return,

            // Zero
            (Self::Zero, _) => return,
            (Self::Number { .. }, Self::Zero) => *self = Self::Zero,

            // see below
            (Self::Number { .. }, Self::Number { .. }) => {}
        }

        // a * b
        if let (
            Self::Number { m, e },
            Self::Number {
                m: other_m,
                e: other_e,
            },
        ) = (self, rhs)
        {
            *m *= other_m;
            *e += other_e;
        };
    }

    /// This will divide `rhs` from `self` without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call [Big::normalize()] yourself at some point to prevent bugs.
    ///
    /// You will most likely want to use the /= or / operator instead, which will normalize the result automatically.
    pub fn div_mut_unnormalized(&mut self, rhs: Self) {
        match (&self, &rhs) {
            // NaN
            (Self::NaN, _) | (_, Self::NaN) => *self = Self::NaN,

            // Infinities
            (Self::Infinity(_), Self::Infinity(_)) => *self = Self::NaN,
            (Self::Number { .. } | Self::Zero, Self::Infinity(_)) => *self = Self::Zero,
            (Self::Infinity(_), Self::Number { .. }) => return,

            // Zero
            (Self::Zero, _) => return,
            (_, Self::Zero) => *self = Self::NaN,

            // see below
            (Self::Number { .. }, Self::Number { .. }) => {}
        }

        // a / b
        if let (
            Self::Number { m, e },
            Self::Number {
                m: other_m,
                e: other_e,
            },
        ) = (self, rhs)
        {
            *m /= other_m;
            *e -= other_e;
        };
    }

    /// Take the absolute value of `self`, modifying it in-place
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// let mut number = Big::from(-42);
    /// number.abs_mut();
    /// assert_eq!(number, Big::from(42));
    /// ```
    pub fn abs_mut(&mut self) {
        if let Self::Number { m, .. } = self {
            *m = m.abs();
        }
    }

    /// Take the absolute value of `self`, creating a new Instance
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// assert_eq!(Big::from(-42).abs(), Big::from(42));
    /// assert_eq!(Big::from(42).abs(), Big::from(42));
    /// ```
    pub fn abs(&self) -> Self {
        let mut result = self.clone();
        result.abs_mut();
        result
    }

    /// Return true if `self` is NaN
    ///
    /// Use this method because [Big::NaN] != [Big::NaN]
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// assert!(Big::NaN.is_nan());
    /// ```
    pub fn is_nan(&self) -> bool {
        matches!(self, Self::NaN)
    }

    /// Return true if `self` is +inf
    ///
    /// Use this method because +inf != +inf
    ///
    /// # Example
    /// ```
    /// assert!(bignum_ig::POS_INFINITY.is_pos_inf());
    /// ```
    pub fn is_pos_inf(&self) -> bool {
        matches!(self, Self::Infinity(InfinityKind::Positive))
    }

    /// Return true if `self` is -inf
    ///
    /// Use this method because -inf != -inf
    ///
    /// # Example
    /// ```
    /// assert!(bignum_ig::NEG_INFINITY.is_neg_inf());
    /// ```
    pub fn is_neg_inf(&self) -> bool {
        matches!(self, Self::Infinity(InfinityKind::Negative))
    }

    /// Return true if `self` is Zero
    ///
    /// Comparing `self` to [Big::Zero] is also possible
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// assert!(Big::from(0).is_zero());
    /// ```
    pub fn is_zero(&self) -> bool {
        self == &Big::Zero
    }

    /// Return the logarithm to the base of 10 of `self`
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// let log = Big::from(100).log10();
    /// assert_eq!(log, 2.0);
    /// ```
    pub fn log10(self) -> f64 {
        match self {
            Self::Number { m, e } => m.log10() + e as f64,
            Self::Infinity(InfinityKind::Negative) => f64::NAN,
            Self::Infinity(InfinityKind::Positive) => f64::INFINITY,
            Self::Zero | Self::NaN => f64::NAN,
        }
    }

    /// Return the natural logarithm of `self`
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// let ln = Big::from(f64::exp(2.0)).ln();
    /// assert_eq!(ln, 2.0);
    /// ```
    pub fn ln(self) -> f64 {
        match self.log10() {
            log if log.is_normal() => log / f64::consts::LOG10_E,
            log => log,
        }
    }

    /// Return the logarithm of any base of `self`
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// let log = Big::from(256).log(16.0);
    /// assert_eq!(log, 2.0);
    /// ```
    pub fn log(self, base: f64) -> f64 {
        if base.is_normal() {
            self.ln() / base.ln()
        } else {
            f64::NAN
        }
    }

    /// Raise `self` to `power` and modify it in-place.
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// let mut number = Big::from(16);
    /// number.powf_mut(2.0);
    /// assert_eq!(number, Big::from(256));
    /// ```
    pub fn powf_mut(&mut self, power: f64) {
        if let Self::Zero = self {
            if power.is_normal() {
                return;
            }
        }

        let result_log10 = self.abs().log10() * power;

        match result_log10 {
            f64::NEG_INFINITY => *self = Self::NaN,
            f64::INFINITY => *self = POS_INFINITY,
            log if log.is_nan() => *self = Self::NaN,
            // result_log10 may over/underflow as an i64, handle it
            log if log < i64::MIN as f64 => *self = Self::Zero,
            log if log > i64::MAX as f64 => *self = POS_INFINITY,
            // normaliazion shouldn't be required here, since m will be between 1.0 and < 10.0
            log => {
                if let Self::Number { m, e } = self {
                    *m = 10.0_f64.powf(log % 1.0);
                    // minus times minus is plus
                    if log % 2.0 == 0.0 {
                        *m = m.abs();
                    }
                    *e = log as i64;
                }
            }
        };
    }

    /// Raise `self` to `power`, returning a new Instance
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// assert_eq!(Big::from(16).powf(2.0), Big::from(256));
    /// ```
    pub fn powf(&self, power: f64) -> Self {
        let mut result = self.clone();
        result.powf_mut(power);
        result
    }

    /// This will put the remainder of `self` % `rhs` into `self` without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call [Big::normalize()] yourself at some point to prevent bugs.
    ///
    /// You will most likely want to use the %= or % operator instead, which will normalize the result automatically.
    pub fn remainder_mut_unnormalized(&mut self, rhs: &Big) {
        match (&self, rhs) {
            (Self::NaN, _) | (_, Self::NaN) => return,
            (_, Self::Zero) => *self = Self::NaN,
            (Self::Infinity(_), Self::Infinity(_)) => *self = Self::NaN,
            (Self::Zero, _) => return,
            (Self::Number { .. }, Self::Infinity(_)) => return,
            (Self::Infinity(_), Self::Number { .. }) => *self = Self::NaN,
            // See below
            (Self::Number { .. }, Self::Number { .. }) => {}
        }

        if let (
            Self::Number { m, e },
            Self::Number {
                m: other_m,
                e: other_e,
            },
        ) = (self, rhs)
        {
            let other_m_normalized = other_m * 10_f64.powi((*other_e - *e) as i32);
            *m = match other_m_normalized {
                f64::INFINITY => *m,
                0.0 => 0.0,
                value => *m % value,
            }
        }
    }
}

impl AddAssign for Big {
    fn add_assign(&mut self, rhs: Self) {
        self.add_mut_unnormalized(rhs);
        self.normalize();
    }
}

impl Add for Big {
    type Output = Big;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result += rhs;
        result
    }
}

impl SubAssign for Big {
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_mut_unnormalized(rhs);
        self.normalize();
    }
}

impl Sub for Big {
    type Output = Big;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result -= rhs;
        result
    }
}

impl MulAssign for Big {
    fn mul_assign(&mut self, rhs: Self) {
        self.mul_mut_unnormalized(rhs);
        self.normalize();
    }
}

impl Mul for Big {
    type Output = Big;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result *= rhs;
        result
    }
}

impl DivAssign for Big {
    fn div_assign(&mut self, rhs: Self) {
        self.div_mut_unnormalized(rhs);
        self.normalize();
    }
}

impl Div for Big {
    type Output = Big;

    fn div(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result /= rhs;
        result
    }
}

impl RemAssign for Big {
    fn rem_assign(&mut self, rhs: Self) {
        self.remainder_mut_unnormalized(&rhs);
        self.normalize();
    }
}

impl Rem for Big {
    type Output = Big;

    fn rem(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result %= rhs;
        result
    }
}

impl Neg for Big {
    type Output = Big;

    fn neg(self) -> Self::Output {
        let mut result = self.clone();
        result.neg_mut();
        result
    }
}

impl Display for Big {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Big::Infinity(kind) => match kind {
                InfinityKind::Positive => write!(f, "+inf"),
                InfinityKind::Negative => write!(f, "-inf"),
            },
            Big::NaN => write!(f, "NaN"),
            Big::Zero => write!(f, "0"),
            Big::Number { m, e } => write!(f, "{}e{}", m, e),
        }
    }
}
