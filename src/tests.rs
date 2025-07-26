use std::f64;

use crate::*;

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
}

#[test]
fn conversion() {
    // from f64
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

    assert_eq!(b(4) + b(-15), b(-11));
    assert_eq!(b(1) + Big::NaN, Big::NaN);
    assert_eq!(Big::Zero + b(0) + Big::Zero, Big::Zero);
    assert_eq!(b(0) + b(-0), Big::Zero);
    assert_eq!(b(1) + POS_INFINITY, POS_INFINITY);
    assert_eq!(
        Big::new(9.0, i64::MAX) + Big::new(9.0, i64::MAX),
        POS_INFINITY
    );
}

#[test]
fn substraction() {
    let mut a = b(1);
    a -= b(1);
    assert_eq!(a, b(0));

    assert_eq!(b(4) - b(-15), b(19));
    assert_eq!(b(1) - Big::NaN, Big::NaN);
    assert_eq!(Big::Zero - b(0) - Big::Zero, Big::Zero);
    assert_eq!(b(0) - b(-0), Big::Zero);
    assert_eq!(b(1) - POS_INFINITY, NEG_INFINITY);
    assert_eq!(
        Big::new(-9.0, i64::MAX) - Big::new(9.0, i64::MAX),
        NEG_INFINITY
    );
}

#[test]
fn multiplication() {
    let mut a = b(2);
    a *= b(2);
    assert_eq!(a, b(4));

    assert_eq!(b(7) * b(6), b(42));
    assert_eq!(b(7) * b(-6), b(-42));
    assert_eq!(POS_INFINITY * b(0), Big::NaN);
    assert_eq!(POS_INFINITY * NEG_INFINITY, NEG_INFINITY);
}

#[test]
fn division() {
    let mut a = b(4);
    a /= b(2);
    assert_eq!(a, b(2));

    assert_eq!(b(42) / b(6), b(7));
    assert_eq!(b(42) / b(-6), b(-7));
    assert_eq!(b(42) / b(0), Big::NaN);
    assert_eq!(POS_INFINITY / b(0), Big::NaN);
    assert_eq!(POS_INFINITY / NEG_INFINITY, Big::NaN);
}

#[test]
fn logarithms() {
    assert_eq!(b(f64::consts::E).ln(), 1.0);
    assert_eq!(b(f64::exp(5.0)).ln(), 5.0);
    assert_eq!(b(10.0).log10(), 1.0);
    assert_eq!(b(1.0).log10(), 0.0);
    assert!(b(0.0).log10().is_nan());
    assert!(b(-10.0).log10().is_nan());
}

#[test]
fn power() {
    assert_eq!(b(16.0).pow(0.5), b(4.0));
    assert_eq!(b(-4.0).pow(2.0), b(16.0));
    assert_eq!(b(0.25).pow(-1.0), b(4.0));
    assert_eq!(b(3454.0).pow(0.0), b(1.0));
    assert_eq!(b(0.0).pow(0.0), Big::NaN);
    assert_eq!(b(0.0).pow(1.0), Big::Zero);
    assert_eq!(Big::new(1.0, i64::MAX - 1).pow(2.0), POS_INFINITY);
    assert_eq!(Big::new(1.0, i64::MAX - 1).pow(-2.0), Big::Zero);
}
