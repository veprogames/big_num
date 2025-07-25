use std::{
    f64,
    fmt::Display,
    ops::{Add, AddAssign},
};

#[derive(Debug, PartialEq, Clone)]
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
// declared as i64 because it makes casting unneeded when working
// in the content of BigData::e, a i64
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

    fn normalize_m_and_e(m: &mut f64, e: &mut i64) {
        let log = m.abs().log10() as i64;
        *m /= 10.0_f64.powi(log as i32);
        *e += log;
    }

    fn normalize(&mut self) {
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

            Self::normalize_m_and_e(m, e);
        }
    }

    pub fn is_nan(&self) -> bool {
        self == &Big::NaN
    }

    pub fn is_pos_inf(&self) -> bool {
        self == &Big::Infinity(InfinityKind::Positive)
    }

    pub fn is_neg_inf(&self) -> bool {
        self == &Big::Infinity(InfinityKind::Negative)
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

    pub fn pow(self, power: f64) -> Self {
        // handle 0 ^ power = 0, which cannot be determined using logarithm method below
        if let Self::Zero = self {
            if power.is_normal() {
                return Self::Zero;
            }
        }

        let result_log10 = self.log10() * power;

        match result_log10 {
            f64::NEG_INFINITY => Self::NaN,
            f64::INFINITY => POS_INFINITY,
            log if log.is_nan() => Self::NaN,
            // result_log10 may over/underflow as an i64, handle it
            log if log < i64::MIN as f64 => Self::Zero,
            log if log > i64::MAX as f64 => POS_INFINITY,
            log => Big::new(10.0_f64.powf(log % 1.0), log as i64),
        }
    }
}

impl AddAssign for Big {
    fn add_assign(&mut self, rhs: Self) {
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

                    Self::normalize_m_and_e(m, e);
                }
            }
        };
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

impl From<f64> for Big {
    fn from(value: f64) -> Self {
        Big::new(value, 0)
    }
}

impl From<f32> for Big {
    fn from(value: f32) -> Self {
        Big::new(value as f64, 0)
    }
}

impl From<i64> for Big {
    fn from(value: i64) -> Self {
        Big::new(value as f64, 0)
    }
}

impl From<i32> for Big {
    fn from(value: i32) -> Self {
        Big::new(value as f64, 0)
    }
}

#[cfg(test)]
mod tests {
    use std::f64;

    use super::*;

    // methods for testing (mainly normalization)
    impl Big {
        fn m(&self) -> f64 {
            if let Self::Number { m, e: _ } = self {
                *m
            } else {
                panic!("expected a valid mantissa but self is {:?}", self);
            }
        }

        fn e(&self) -> i64 {
            if let Self::Number { m: _, e } = self {
                *e
            } else {
                panic!("expected a valid exponent but self is {:?}", self);
            }
        }
    }

    fn b<T>(value: T) -> Big
    where
        Big: From<T>,
    {
        Big::from(value)
    }

    #[test]
    fn creation() {
        // from ::new
        Big::new(1.0, 0);
        Big::new(-1.0, 0);
        Big::new(1.0, i64::MAX);
        Big::new(1.0, i64::MIN);

        let pos_inf = Big::new(100.0, i64::MAX - 1);
        assert_eq!(pos_inf, POS_INFINITY);

        let neg_inf = Big::new(-100.0, i64::MAX - 1);
        assert_eq!(neg_inf, NEG_INFINITY);

        let zero = Big::new(0.01, i64::MIN + 1);
        assert_eq!(zero, Big::Zero);

        // from f64
        let _: Big = f64::MIN_POSITIVE.into();
        let _: Big = f64::MAX.into();
        let zero: Big = 0.0_f64.into();
        assert!(zero.is_zero());
        let nan: Big = f64::NAN.into();
        assert!(nan.is_nan());
        let inf: Big = f64::INFINITY.into();
        assert!(inf.is_pos_inf());
        let inf: Big = (-f64::INFINITY).into();
        assert!(inf.is_neg_inf());
    }

    #[test]
    // Note: this doesn't need thorough testing because Big::new in creation
    // implicitly calls normalized
    fn normalization() {
        let norm = Big::new(1234.5, 0);
        assert_eq!(norm.m(), 1.2345);
        assert_eq!(norm.e(), 3);

        let norm = Big::new(-1234.5, 0);
        assert_eq!(norm.m(), -1.2345);
        assert_eq!(norm.e(), 3);

        let norm = Big::new(0.001, 0);
        assert_eq!(norm.m(), 1.0);
        assert_eq!(norm.e(), -3);

        let norm = Big::new(0.0, 4);
        assert_eq!(norm, Big::Zero);
    }

    #[test]
    fn addition() {
        let mut a = b(1);
        a += b(1);
        assert_eq!(a, b(2));

        assert_eq!(b(4) + b(-5), b(-1));
        assert_eq!(b(1) + Big::NaN, Big::NaN);
        assert_eq!(Big::Zero + b(0) + Big::Zero, Big::Zero);
        assert_eq!(b(0) + b(-0), Big::Zero);
        assert_eq!(b(1) + POS_INFINITY, POS_INFINITY);
    }

    #[test]
    fn logarithms() {
        assert_eq!(Big::new(f64::consts::E, 0).ln(), 1.0);
        assert_eq!(Big::new(f64::consts::E.powf(5.0), 0).ln(), 5.0);
        assert_eq!(Big::new(10.0, 0).log10(), 1.0);
        assert_eq!(Big::new(1.0, 0).log10(), 0.0);
        assert!(Big::new(0.0, 0).log10().is_nan());
        assert!(Big::new(-10.0, 0).log10().is_nan());
    }

    #[test]
    fn power() {
        assert_eq!(Big::new(16.0, 0).pow(0.5), Big::new(4.0, 0));
        assert_eq!(Big::new(0.25, 0).pow(-1.0), Big::new(4.0, 0));
        assert_eq!(Big::new(3454.0, 0).pow(0.0), Big::new(1.0, 0));
        assert_eq!(Big::new(0.0, 0).pow(0.0), Big::NaN);
        assert_eq!(Big::new(0.0, 0).pow(1.0), Big::Zero);
        assert_eq!(Big::new(1.0, i64::MAX - 1).pow(2.0), POS_INFINITY);
        assert_eq!(Big::new(1.0, i64::MAX - 1).pow(-2.0), Big::Zero);
    }
}
