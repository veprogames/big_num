use std::{error::Error, fmt::Display, str::FromStr};

use crate::Big;

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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Parts,
    Mantissa(String),
    Exponent(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Parts => write!(f, "Invalid Parts"),
            Self::Mantissa(m) => write!(f, "Invalid Mantissa: {m}"),
            Self::Exponent(e) => write!(f, "Invalid Exponent: {e}"),
        }
    }
}

impl Error for ParseError {}

impl FromStr for Big {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "0" => return Ok(Big::Zero),
            "nan" => return Ok(Big::NaN),
            // see below
            _ => {}
        }

        if let Ok(number) = s.parse::<f64>() {
            return Ok(Big::from(number));
        }

        let mut iter = s.split("e");
        match (iter.next(), iter.next(), iter.next()) {
            (Some(m), Some(e), None) => match (m.parse(), e.parse()) {
                (Ok(m), Ok(e)) => Ok(Big::new(m, e)),
                (Err(_), Ok(_)) => Err(ParseError::Mantissa(m.to_string())),
                (Ok(_), Err(_)) => Err(ParseError::Exponent(e.to_string())),
                _ => Err(ParseError::Parts),
            },
            _ => Err(ParseError::Parts),
        }
    }
}
