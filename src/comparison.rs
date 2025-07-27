use crate::{Big, InfinityKind, SIG_DIGITS};
use std::cmp::Ordering;

impl PartialOrd for Big {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::NaN, _) | (_, Self::NaN) => None,
            (Self::Infinity(InfinityKind::Positive), Self::Infinity(InfinityKind::Positive)) => {
                None
            }
            (Self::Infinity(InfinityKind::Negative), Self::Infinity(InfinityKind::Negative)) => {
                None
            }
            (Self::Infinity(InfinityKind::Positive), Self::Infinity(InfinityKind::Negative)) => {
                Some(Ordering::Greater)
            }
            (Self::Infinity(InfinityKind::Negative), Self::Infinity(InfinityKind::Positive)) => {
                Some(Ordering::Less)
            }
            (Self::Infinity(InfinityKind::Positive), Self::Zero) => Some(Ordering::Greater),
            (Self::Infinity(InfinityKind::Negative), Self::Zero) => Some(Ordering::Less),
            (Self::Zero, Self::Infinity(InfinityKind::Positive)) => Some(Ordering::Less),
            (Self::Zero, Self::Infinity(InfinityKind::Negative)) => Some(Ordering::Greater),
            (Self::Zero, Self::Zero) => Some(Ordering::Equal),
            (Big::Number { .. }, Self::Infinity(InfinityKind::Positive)) => Some(Ordering::Less),
            (Big::Number { .. }, Self::Infinity(InfinityKind::Negative)) => Some(Ordering::Greater),
            (Self::Infinity(InfinityKind::Positive), Big::Number { .. }) => Some(Ordering::Greater),
            (Self::Infinity(InfinityKind::Negative), Big::Number { .. }) => Some(Ordering::Less),
            (Self::Number { m, .. }, Self::Zero) => {
                if m.is_sign_positive() {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Self::Zero, Self::Number { m, .. }) => {
                if m.is_sign_positive() {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
            (
                Self::Number { m, e },
                Self::Number {
                    m: other_m,
                    e: other_e,
                },
            ) => match other_e - e {
                delta if delta >= SIG_DIGITS => Some(Ordering::Greater),
                delta if delta <= -SIG_DIGITS => Some(Ordering::Less),
                delta => {
                    let m_normalized = other_m * 10_f64.powi(delta as i32);
                    if m_normalized == *m {
                        Some(Ordering::Equal)
                    } else if m_normalized > *m {
                        Some(Ordering::Less)
                    } else if m_normalized < *m {
                        Some(Ordering::Greater)
                    } else {
                        None
                    }
                }
            },
        }
    }
}

impl PartialEq for Big {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Number { m, e },
                Self::Number {
                    m: other_m,
                    e: other_e,
                },
            ) if m == other_m && e == other_e => true,
            (Self::Zero, Self::Zero) => true,
            _ => false,
        }
    }
}
