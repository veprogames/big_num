use std::{f64, fmt::Display};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Big {
    Number(BigData),
    NaN,
    Infinity(InfinityKind),
    Zero,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InfinityKind {
    Positive,
    Negative,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BigData {
    m: f64,
    e: i64,
}

pub const POS_INFINITY: Big = Big::Infinity(InfinityKind::Positive);
pub const NEG_INFINITY: Big = Big::Infinity(InfinityKind::Negative);
// declared as i64 because it makes casting unneeded when working
// in the content of BigData::e, a i64
const SIG_DIGITS: i64 = 15;

impl Big {
    pub fn new(mantissa: f64, exponent: i64) -> Self {
        let data = BigData {
            m: mantissa,
            e: exponent,
        };
        Self::Number(data).normalized()
    }

    fn normalized(mut self) -> Self {
        match self {
            Self::Infinity(_) | Self::NaN | Self::Zero => self,
            Self::Number(ref mut data) => {
                // True for any exponent in m * 10 ^ e
                if data.m == 0.0 {
                    return Self::Zero;
                }

                if !data.m.is_normal() {
                    panic!("mantissa is not normal: {:?}", data.m);
                }

                let log = data.m.abs().log10() as i64;

                match log {
                    // might underflow to Zero
                    ..=0 => {
                        data.e = match data.e.checked_sub(-log) {
                            Some(e) => e,
                            None => return Self::Zero,
                        };
                    }
                    // might overflow to either positive or negative Infinity (+inf; -inf)
                    _positive => {
                        data.e = match data.e.checked_add(log) {
                            Some(e) => e,
                            None => {
                                let mantissa_is_positive = data.m.is_sign_positive();
                                let infinity_kind = if mantissa_is_positive {
                                    InfinityKind::Positive
                                } else {
                                    InfinityKind::Negative
                                };

                                return Self::Infinity(infinity_kind);
                            }
                        };
                    }
                }

                let log_i32: i32 = log.try_into().expect(
                    "abs(log) of mantissa should never exceed ~350
                    and therefore can be cast to i32",
                );
                data.m /= 10.0_f64.powi(log_i32);

                self
            }
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

    pub fn add(self, other: Self) -> Self {
        match (self, other) {
            // NaN
            (Self::NaN, _) | (_, Self::NaN) => Self::NaN,

            // Infinities

            // +inf + -inf and -inf + +inf are undefined
            (Self::Infinity(kind), Self::Infinity(kind2)) if kind != kind2 => Self::NaN,
            (Self::Infinity(kind), Self::Infinity(_)) => Self::Infinity(kind),

            (Self::Infinity(kind), Self::Number(_) | Self::Zero) => Self::Infinity(kind),
            (Self::Number(_) | Self::Zero, Self::Infinity(kind)) => Self::Infinity(kind),

            // Zero
            (Self::Zero, Self::Number(_)) => other,
            (Self::Number(_), Self::Zero) => self,
            (Self::Zero, Self::Zero) => Self::Zero,

            // a + b
            (Self::Number(self_data), Self::Number(other_data)) => {
                let delta = other_data.e - self_data.e;

                match delta {
                    // ..=-SIG_DIGITS produced a syntax error
                    _delta if delta <= -SIG_DIGITS => self,
                    _delta if delta >= SIG_DIGITS => other,
                    delta => {
                        let delta: i32 = delta.try_into().expect(
                            "exponent delta between a, b in a + b should never exceed 13
                            and can therefore be cast into i32",
                        );

                        let m = self_data.m + other_data.m * 10.0_f64.powi(delta);
                        let e = self_data.e;
                        Big::new(m, e)
                    }
                }
            }
        }
    }

    pub fn mul(self, other: Self) -> Self {
        match (self, other) {
            // NaN
            (Self::NaN, _) => Self::NaN,
            (_, Self::NaN) => Self::NaN,

            // Infinities

            // Let's say +inf * -inf is NaN
            (Self::Infinity(kind1), Self::Infinity(kind2)) if kind1 != kind2 => Self::NaN,
            // But is (-inf)² = +inf and (inf)² = inf? Let us say it is NaN also
            // To establish (-inf)² = (-inf)(inf) = NaN
            (Self::Infinity(_), Self::Infinity(_)) => Self::NaN,
            // obviously, multiplying inf with a normal number is inf
            (Self::Infinity(kind), Self::Number(_)) => Self::Infinity(kind),
            (Self::Number(_), Self::Infinity(kind)) => Self::Infinity(kind),

            // Zeroes
            (Self::Zero, _) => Self::Zero,
            (_, Self::Zero) => Self::Zero,

            // a * b
            (Self::Number(data_self), Self::Number(data_other)) => {
                let m = data_self.m * data_other.m;

                // This will need to be abstracted along with in normalized()
                // Cannot easily pass through here
                let possible_infinity_kind = if m.is_sign_positive() {
                    InfinityKind::Positive
                } else {
                    InfinityKind::Negative
                };
                let e = match data_other.e.is_positive() {
                    true => match data_self.e.checked_add(data_other.e) {
                        Some(e) => e,
                        None => return Big::Infinity(possible_infinity_kind),
                    },
                    false => match data_self.e.checked_sub(-data_other.e) {
                        Some(e) => e,
                        None => return Big::Zero,
                    },
                };

                Big::new(m, e)
            }
        }
    }

    pub fn log10(self) -> f64 {
        match self {
            Self::Number(BigData { m, e }) => m.log10() + e as f64,
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

impl Display for Big {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Big::Infinity(kind) => match kind {
                InfinityKind::Positive => write!(f, "+inf"),
                InfinityKind::Negative => write!(f, "-inf"),
            },
            Big::NaN => write!(f, "NaN"),
            Big::Zero => write!(f, "0"),
            Big::Number(BigData { m, e }) => write!(f, "{}e{}", m, e),
        }
    }
}

impl Into<Big> for f64 {
    fn into(self) -> Big {
        if self.is_nan() {
            return Big::NaN;
        }

        if self.is_infinite() {
            if self.is_sign_positive() {
                return Big::Infinity(InfinityKind::Positive);
            } else {
                return Big::Infinity(InfinityKind::Negative);
            }
        }

        if self == 0.0 {
            return Big::Zero;
        }

        // f64 log cannot be outside of i64::MIN..i64::MAX
        let log: i64 = self.log10() as i64;
        let mantissa = self / 10.0_f64.powi(log as i32);
        Big::new(mantissa, log)
    }
}

impl Into<Big> for f32 {
    fn into(self) -> Big {
        let float_64: f64 = self.into();
        let big: Big = float_64.into();
        big
    }
}

#[cfg(test)]
mod tests {
    use std::{f64, i64};

    use super::*;

    // methods for testing (mainly normalization)
    impl Big {
        fn m(&self) -> f64 {
            if let Self::Number(BigData { m, e: _ }) = self {
                *m
            } else {
                panic!("expected a valid mantissa but self is {:?}", self);
            }
        }

        fn e(&self) -> i64 {
            if let Self::Number(BigData { m: _, e }) = self {
                *e
            } else {
                panic!("expected a valid exponent but self is {:?}", self);
            }
        }
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
    }

    #[test]
    fn addition() {
        let a: Big = 5.0.into();
        let b: Big = 8.0.into();
        let b_neg: Big = (-8.0).into();
        let c: Big = 1.5e42.into();
        let d: Big = Big::new(1.0, i64::MAX - 1);

        assert_eq!(a.add(b), 13.0.into());
        assert_eq!(a.add(b_neg), (-3.0).into());
        assert_eq!(a.add(c), c);
        assert_eq!(c.add(a), c);
        assert_eq!(a.add(d), d);
        assert_eq!(d.add(a), d);
        assert_eq!(a.add(POS_INFINITY), POS_INFINITY);
        assert_eq!(a.add(NEG_INFINITY), NEG_INFINITY);
        assert_eq!(a.add(Big::NaN), Big::NaN);
        assert_eq!(a.add(Big::Zero), a);
        assert_eq!(Big::Zero.add(a), a);
        assert_eq!(a.add(Big::Zero).add(b).add(b), 21.0.into());
        assert_eq!(a.add(Big::Zero).add(b).add(Big::NaN).add(b), Big::NaN);
        assert_eq!(POS_INFINITY.add(NEG_INFINITY), Big::NaN);
    }

    #[test]
    fn multiplication() {
        let a: Big = 5.0.into();
        let b: Big = 8.0.into();
        let b_neg: Big = (-8.0).into();
        let c: Big = 1e42.into();
        let high: Big = Big::new(1.0, i64::MAX - 1);
        let low: Big = Big::new(1.0, i64::MIN + 1);

        assert_eq!(a.mul(b), 40.0.into());
        assert_eq!(a.mul(b_neg), (-40.0).into());
        assert_eq!(a.mul(c), 5.0e42.into());
        assert_eq!(a.mul(c), c.mul(a));

        assert_eq!(high.mul(1_000_000.0.into()), POS_INFINITY);
        assert_eq!(high.mul((-1_000_000.0).into()), NEG_INFINITY);
        assert_eq!(low.mul(0.000_001.into()), Big::Zero);

        assert!(a.mul(Big::NaN).is_nan());
        assert!(POS_INFINITY.mul(NEG_INFINITY).is_nan());
        assert!(a.mul(0.0.into()).is_zero());
        assert!(a.mul(POS_INFINITY).is_pos_inf());
        assert!(a.mul(NEG_INFINITY).is_neg_inf());
        assert!(POS_INFINITY.mul(POS_INFINITY).is_nan());
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
