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
