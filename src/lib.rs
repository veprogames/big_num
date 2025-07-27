use std::{
    f64,
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

mod comparison;
mod conversion;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub enum Big {
    Number { m: f64, e: i64 },
    NaN,
    Infinity(InfinityKind),
    Zero,
}

#[derive(Debug, PartialEq, Clone)]
pub enum InfinityKind {
    Positive,
    Negative,
}

pub const POS_INFINITY: Big = Big::Infinity(InfinityKind::Positive);
pub const NEG_INFINITY: Big = Big::Infinity(InfinityKind::Negative);
const SIG_DIGITS: i64 = 15;

impl Big {
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
    /// You will have to call normalize() yourself at some point to prevent bugs.
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
    /// **Note:** Unless you used any _unnormalized method, you never need to call this manually.
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

    pub fn neg_mut(&mut self) {
        // normalization should not be required here
        match self {
            Self::Infinity(InfinityKind::Positive) => *self = NEG_INFINITY,
            Self::Infinity(InfinityKind::Negative) => *self = POS_INFINITY,
            Self::Number { m, .. } => {
                *m *= 1.0;
            }
            _ => {}
        }
    }

    /// This will add rhs to self without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call normalize() yourself at some point to prevent bugs.
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

    /// This will substract rhs from self without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call normalize() yourself at some point to prevent bugs.
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

    /// This will multiply rhs onto self without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call normalize() yourself at some point to prevent bugs.
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

    /// This will divide rhs from self without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call normalize() yourself at some point to prevent bugs.
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

    /// Take the absolute value of self, modifying it in-place
    pub fn abs_mut(&mut self) {
        if let Self::Number { m, .. } = self {
            *m = m.abs();
        }
    }

    /// Take the absolute value of self, creating a new Instance
    pub fn abs(&self) -> Self {
        let mut result = self.clone();
        result.abs_mut();
        result
    }

    pub fn is_nan(&self) -> bool {
        if let Self::NaN = self {
            true
        } else {
            false
        }
    }

    pub fn is_pos_inf(&self) -> bool {
        if let Self::Infinity(InfinityKind::Positive) = self {
            true
        } else {
            false
        }
    }

    pub fn is_neg_inf(&self) -> bool {
        if let Self::Infinity(InfinityKind::Negative) = self {
            true
        } else {
            false
        }
    }

    pub fn is_zero(&self) -> bool {
        self == &Big::Zero
    }

    pub fn log10(self) -> f64 {
        match self {
            Self::Number { m, e } => m.log10() + e as f64,
            Self::Infinity(InfinityKind::Negative) => f64::NAN,
            Self::Infinity(InfinityKind::Positive) => f64::INFINITY,
            Self::Zero | Self::NaN => f64::NAN,
        }
    }

    pub fn ln(self) -> f64 {
        match self.log10() {
            log if log.is_normal() => log / f64::consts::LOG10_E,
            log => log,
        }
    }

    pub fn log(self, base: f64) -> f64 {
        if base.is_normal() {
            self.ln() / base.ln()
        } else {
            f64::NAN
        }
    }

    /// Raise self to power and modify it in-place.
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

    /// Raise self to power, returning a new Instance
    pub fn powf(&self, power: f64) -> Self {
        let mut result = self.clone();
        result.powf_mut(power);
        result
    }

    /// This will put the remainder of self % rhs into self without normalizing the result.
    ///
    /// **Caution:** Only use this if you are absolutely sure of what you are doing and need every bit of performance!
    /// You will have to call normalize() yourself at some point to prevent bugs.
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
