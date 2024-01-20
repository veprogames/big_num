use std::fmt::Display;

#[derive(Debug, PartialEq)]
struct Big {
    m: f64,
    e: i64,
}

impl Big {
    pub fn new(mantissa: f64, exponent: i64) -> Self {
        Self {
            m: mantissa,
            e: exponent,
        }.normalized()
    }

    fn normalized(mut self) -> Self {
        if self.is_nan() {
            return self;
        }

        if self.m == 0.0 {
            self.e = 0;
            return self;
        }

        let log = self.m.abs().log10() as i64;
        
        self.e += log;
        self.m *= 10.0_f64.powi(log as i32);

        self
    }

    pub fn is_nan(&self) -> bool {
        self.m.is_nan()
    }
}

impl Display for Big {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}e{}", self.m, self.e)
    }
}

impl Into<Big> for f64 {
    fn into(self) -> Big {
        if self.is_nan() {
            Big::new(f64::NAN, 0);
        }

        if self == 0.0 {
            Big::new(0.0, 0);
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

        // from f64
        let _: Big = 0.0_f64.into();
        let _: Big = f64::MIN_POSITIVE.into();
        let _: Big = f64::MAX.into();
        let nan: Big = f64::NAN.into();
        assert!(nan.is_nan())
    }

    #[test]
    fn normalization() {
        let norm = Big::new(1234.5, 0);
        assert_eq!(norm.e, 3);

        let norm = Big::new(-1234.5, 0);
        assert_eq!(norm.e, 3);

        let norm = Big::new(0.001, 0);
        assert_eq!(norm.e, -3);
    }
}