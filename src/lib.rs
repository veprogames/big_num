use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
enum Big {
    Number(BigData),
    NaN,
    Infinity,
    Zero,
}

#[derive(Debug, PartialEq, Clone)]
struct BigData {
    m: f64,
    e: i64,
}

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
            Self::Infinity | Self::NaN | Self::Zero => self,
            Self::Number(ref mut data) => {
                if !data.m.is_normal() {
                    panic!("mantissa is not normal: {:?}", data.m);
                }

                if data.m == 0.0 {
                    return Self::Zero;
                }

                let log = data.m.abs().log10() as i64;

                match log {
                    ..=0 => {
                        data.e = match data.e.checked_sub(-log) {
                            Some(e) => e,
                            None => return Self::Zero,
                        };
                    },
                    _positive => {
                        data.e = match data.e.checked_add(log) {
                            Some(e) => e,
                            None => return Self::Infinity,
                        };
                    }
                }

                data.m /= 10.0_f64.powi(log as i32);

                self
            }
        }
    }

    fn m(&self) -> f64{
        if let Self::Number(BigData{m, e: _}) = self {
            *m
        } else {
            panic!("self is {:?}", self);
        }
    }

    fn e(&self) -> i64{
        if let Self::Number(BigData{m: _, e}) = self {
            *e
        } else {
            panic!("self is {:?}", self);
        }
    }

    pub fn is_nan(&self) -> bool {
        self == &Big::NaN
    }

    pub fn is_inf(&self) -> bool {
        self == &Big::Infinity
    }

    pub fn is_zero(&self) -> bool {
        self == &Big::Zero
    }
}

// arithmetic

impl Big {
    pub fn add(&self, other: &Self) -> Self {
        match (self, other) {
            // Infinity
            (Self::Infinity, Self::Infinity | Self::Number(_) | Self::Zero) => Self::Infinity,
            (Self::Number(_) | Self::Zero, Self::Infinity) => Self::Infinity,
            
            // NaN
            (Self::NaN, _) | (_, Self::NaN) => Self::NaN,
            
            // Zero
            (Self::Zero, Self::Number(_)) => other.clone(),
            (Self::Number(_), Self::Zero) => self.clone(),
            (Self::Zero, Self::Zero) => Self::Zero,

            // a + b
            (Self::Number(self_data), Self::Number(other_data)) => {
                let delta = other_data.e - self_data.e;

                match delta {
                    ..=-14 => self.clone(),
                    14.. => other.clone(),
                    delta => {
                        let m = self_data.m + other_data.m * 10.0_f64.powi(delta as i32);
                        let e = self_data.e;
                        Big::new(m, e)
                    }
                }
            }

        }
    }
}

impl Display for Big {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Big::Infinity => write!(f, "inf"),
            Big::NaN => write!(f, "NaN"),
            Big::Zero => write!(f, "0"),
            Big::Number(BigData {m, e}) => write!(f, "{}e{}", m, e),
        }
    }
}

impl Into<Big> for f64 {
    fn into(self) -> Big {
        if self.is_nan() {
            return Big::NaN
        }

        if self.is_infinite() {
            return Big::Infinity
        }

        if self == 0.0 {
            return Big::Zero
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
    use super::*;

    #[test]
    fn create_big() {
        // from ::new
        Big::new(1.0, 0);
        Big::new(-1.0, 0);
        Big::new(1.0, i64::MAX);
        Big::new(1.0, i64::MIN);

        let inf = Big::new(100.0, i64::MAX - 1);
        assert_eq!(inf, Big::Infinity);

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
        assert!(inf.is_inf());
    }

    #[test]
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

        assert_eq!(a.add(&b), 13.0.into());
        assert_eq!(a.add(&b_neg), (-3.0).into());
        assert_eq!(a.add(&c), c);
        assert_eq!(c.add(&a), c);
        assert_eq!(a.add(&Big::Infinity), Big::Infinity);
        assert_eq!(a.add(&Big::NaN), Big::NaN);
        assert_eq!(a.add(&Big::Zero), a);
        assert_eq!(a.add(&Big::Zero).add(&b).add(&b), 21.0.into());
        assert_eq!(a.add(&Big::Zero).add(&b).add(&Big::NaN).add(&b), Big::NaN);
    }
}