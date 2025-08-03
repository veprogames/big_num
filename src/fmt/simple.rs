use crate::{Big, SIG_DIGITS};

impl Big {
    /// Formats the number in the format of number.places
    ///
    /// **Caution:** Large Numbers will not be truncated. Use this only on numbers
    /// that will not become very large. For large numbers, look at [Big::to_exponential] instead.
    ///
    /// # Example
    /// ```
    /// use bignum_ig::Big;
    ///
    /// assert_eq!(Big::from(1234.5678).to_fixed(2), "1234.57");
    /// ```
    pub fn to_fixed(&self, places: usize) -> String {
        match self {
            Self::Zero => format!("0.{}", "0".repeat(places)),
            Self::Number { m, e } => {
                if *e >= SIG_DIGITS {
                    let result = m.to_string().replace(".", "");
                    let remaining_places = (*e as usize) - result.len() + 1;
                    return format!(
                        "{result}{0}.{1}",
                        "0".repeat(remaining_places),
                        "0".repeat(places)
                    );
                }
                let m = m * 10f64.powi(*e as i32);
                format!("{m:.0$}", places)
            }
            slf => slf.to_string(),
        }
    }

    /// Formats the number in the format of mantissa.places**e**exponent
    ///
    /// # Example
    ///
    /// ```
    /// use bignum_ig::Big;
    ///
    /// assert_eq!(Big::from(1234.5678).to_exponential(2), "1.23e3");
    /// ```
    pub fn to_exponential(&self, places: usize) -> String {
        match self {
            Self::Zero => format!("0.{}", "0".repeat(places)),
            Self::Number { m, e } => {
                format!("{m:.0$}e{e}", places)
            }
            slf => slf.to_string(),
        }
    }
}
