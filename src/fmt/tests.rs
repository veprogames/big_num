use crate::Big;

fn b<T>(value: T) -> Big
where
    Big: From<T>,
{
    Big::from(value)
}

#[test]
fn to_fixed() {
    assert_eq!(b(-6789.6799).to_fixed(3), "-6789.680");
    assert_eq!(Big::new(6.799, -500).to_fixed(2), "0.00");
    assert_eq!(
        Big::new(1.234, 500).to_fixed(200),
        format!("1234{}.{}", "0".repeat(497), "0".repeat(200))
    );
    assert_eq!(Big::NaN.to_fixed(2), "NaN");
}

#[test]
fn to_exponential() {
    assert_eq!(b(-6789.6789).to_exponential(2), "-6.79e3");
    assert_eq!(b(0).to_exponential(2), "0.00");
    assert_eq!(Big::new(1.23, -1234).to_exponential(2), "1.23e-1234");
    assert_eq!(Big::NaN.to_exponential(2), "NaN");
}
