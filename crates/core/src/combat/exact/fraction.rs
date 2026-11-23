use crate::formula::Ratio;
use serde::{Deserialize, Serialize};

fn gcd(a: u128, b: u128) -> u128 {
    #[cfg(feature = "performance-diagnostics")]
    crate::performance::fraction_gcd(a, b);
    crate::formula::gcd(a, b)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExactFraction {
    pub numerator: u128,
    pub denominator: u128,
}

impl Serialize for ExactFraction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let normalized =
            Self::try_new(self.numerator, self.denominator).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&format!(
            "{}/{}",
            normalized.numerator, normalized.denominator
        ))
    }
}

impl<'de> Deserialize<'de> for ExactFraction {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ExactFractionVisitor;
        impl serde::de::Visitor<'_> for ExactFractionVisitor {
            type Value = ExactFraction;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a canonical \"<numerator>/<denominator>\" rational string")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                let (n_str, d_str) =
                    crate::formula::parse_canonical_rational_str(value).map_err(E::custom)?;
                let numerator = n_str.parse::<u128>().map_err(E::custom)?;
                let denominator = d_str.parse::<u128>().map_err(E::custom)?;
                ExactFraction::try_new(numerator, denominator).map_err(E::custom)
            }
        }
        deserializer.deserialize_str(ExactFractionVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExactArithmeticError {
    ZeroDenominator,
    DivisionByZero,
    AdditionOverflow,
    SubtractionUnderflow,
    MultiplicationOverflow,
    DamageRoundingOverflow,
    RoundedDamageExceedsU64,
    ZeroAttackerLevel,
    ShredOutOfRange,
    ResistanceComparisonOverflow,
}

impl std::fmt::Display for ExactArithmeticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDenominator => write!(f, "fraction denominator cannot be zero"),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::AdditionOverflow => write!(f, "addition overflow"),
            Self::SubtractionUnderflow => write!(f, "subtraction underflow"),
            Self::MultiplicationOverflow => write!(f, "multiplication overflow"),
            Self::DamageRoundingOverflow => write!(f, "damage rounding overflow"),
            Self::RoundedDamageExceedsU64 => write!(f, "rounded damage exceeds u64"),
            Self::ZeroAttackerLevel => write!(f, "attacker level cannot be zero"),
            Self::ShredOutOfRange => write!(f, "defense shred/ignore must be in [0,1]"),
            Self::ResistanceComparisonOverflow => write!(f, "resistance comparison overflow"),
        }
    }
}

impl std::error::Error for ExactArithmeticError {}

impl From<ExactArithmeticError> for String {
    fn from(err: ExactArithmeticError) -> Self {
        err.to_string()
    }
}

impl ExactFraction {
    /// Compare nonnegative fractions without overflowing cross products.
    pub fn try_cmp(self, other: Self) -> Result<std::cmp::Ordering, ExactArithmeticError> {
        if self.denominator == 0 || other.denominator == 0 {
            return Err(ExactArithmeticError::ZeroDenominator);
        }
        let (mut a, mut b) = (self, other);
        let mut inverted = false;
        loop {
            let order = (a.numerator / a.denominator).cmp(&(b.numerator / b.denominator));
            if order != std::cmp::Ordering::Equal {
                return Ok(if inverted { order.reverse() } else { order });
            }
            let (ar, br) = (a.numerator % a.denominator, b.numerator % b.denominator);
            if ar == 0 || br == 0 {
                let order = ar.cmp(&br);
                return Ok(if inverted { order.reverse() } else { order });
            }
            a = Self {
                numerator: a.denominator,
                denominator: ar,
            };
            b = Self {
                numerator: b.denominator,
                denominator: br,
            };
            inverted = !inverted;
        }
    }

    #[inline]
    pub fn try_new(n: u128, d: u128) -> Result<Self, ExactArithmeticError> {
        if d == 0 {
            return Err(ExactArithmeticError::ZeroDenominator);
        }
        if d == 1 {
            return Ok(Self {
                numerator: n,
                denominator: d,
            });
        }
        if n == 0 {
            return Ok(Self {
                numerator: 0,
                denominator: 1,
            });
        }
        let g = gcd(n, d);
        if g == 1 {
            return Ok(Self {
                numerator: n,
                denominator: d,
            });
        }
        if (n | d) <= u128::from(u64::MAX) {
            return Ok(Self {
                numerator: u128::from(n as u64 / g as u64),
                denominator: u128::from(d as u64 / g as u64),
            });
        }
        Ok(Self {
            numerator: n / g,
            denominator: d / g,
        })
    }

    #[inline]
    pub fn new(n: u128, d: u128) -> Result<Self, String> {
        Self::try_new(n, d).map_err(Into::into)
    }

    #[inline]
    pub fn try_from_ratio(r: Ratio) -> Result<Self, ExactArithmeticError> {
        Self::try_new(u128::from(r.numerator), u128::from(r.denominator))
    }

    #[inline]
    pub fn from_ratio(r: Ratio) -> Result<Self, String> {
        Self::try_from_ratio(r).map_err(Into::into)
    }

    #[inline]
    pub fn to_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    #[inline]
    pub fn try_mul(self, other: Self) -> Result<Self, ExactArithmeticError> {
        if self.denominator == 0 || other.denominator == 0 {
            return Err(ExactArithmeticError::ZeroDenominator);
        }
        if self.numerator == 0 || other.numerator == 0 {
            return Self::try_new(0, 1);
        }
        if self.numerator == 1 && self.denominator == 1 {
            return Ok(other);
        }
        if other.numerator == 1 && other.denominator == 1 {
            return Ok(self);
        }
        let g1 = gcd(self.numerator, other.denominator);
        let g2 = gcd(other.numerator, self.denominator);
        let n = (self.numerator / g1)
            .checked_mul(other.numerator / g2)
            .ok_or(ExactArithmeticError::MultiplicationOverflow)?;
        let d = (self.denominator / g2)
            .checked_mul(other.denominator / g1)
            .ok_or(ExactArithmeticError::MultiplicationOverflow)?;
        Ok(Self {
            numerator: n,
            denominator: d,
        })
    }

    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn mul(self, other: Self) -> Result<Self, String> {
        self.try_mul(other).map_err(Into::into)
    }

    #[inline]
    pub fn try_add(self, other: Self) -> Result<Self, ExactArithmeticError> {
        if self.denominator == 0 || other.denominator == 0 {
            return Err(ExactArithmeticError::ZeroDenominator);
        }
        if self.numerator == 0 {
            return Ok(other);
        }
        if other.numerator == 0 {
            return Ok(self);
        }
        if self.denominator == other.denominator {
            let n = self
                .numerator
                .checked_add(other.numerator)
                .ok_or(ExactArithmeticError::AdditionOverflow)?;
            return Self::try_new(n, self.denominator);
        }
        let common = gcd(self.denominator, other.denominator);
        let n1 = self
            .numerator
            .checked_mul(other.denominator / common)
            .ok_or(ExactArithmeticError::AdditionOverflow)?;
        let n2 = other
            .numerator
            .checked_mul(self.denominator / common)
            .ok_or(ExactArithmeticError::AdditionOverflow)?;
        let n = n1
            .checked_add(n2)
            .ok_or(ExactArithmeticError::AdditionOverflow)?;
        let d = (self.denominator / common)
            .checked_mul(other.denominator)
            .ok_or(ExactArithmeticError::AdditionOverflow)?;
        Self::try_new(n, d)
    }

    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn add(self, other: Self) -> Result<Self, String> {
        self.try_add(other).map_err(Into::into)
    }

    #[inline]
    pub fn try_sub(self, other: Self) -> Result<Self, ExactArithmeticError> {
        if self.denominator == 0 || other.denominator == 0 {
            return Err(ExactArithmeticError::ZeroDenominator);
        }
        if other.numerator == 0 {
            return Ok(self);
        }
        if self.denominator == other.denominator {
            if self.numerator < other.numerator {
                return Err(ExactArithmeticError::SubtractionUnderflow);
            }
            let n = self.numerator - other.numerator;
            return Self::try_new(n, self.denominator);
        }
        let common = gcd(self.denominator, other.denominator);
        let n1 = self
            .numerator
            .checked_mul(other.denominator / common)
            .ok_or(ExactArithmeticError::SubtractionUnderflow)?;
        let n2 = other
            .numerator
            .checked_mul(self.denominator / common)
            .ok_or(ExactArithmeticError::SubtractionUnderflow)?;
        if n1 < n2 {
            return Err(ExactArithmeticError::SubtractionUnderflow);
        }
        let n = n1 - n2;
        let d = (self.denominator / common)
            .checked_mul(other.denominator)
            .ok_or(ExactArithmeticError::SubtractionUnderflow)?;
        Self::try_new(n, d)
    }

    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn sub(self, other: Self) -> Result<Self, String> {
        self.try_sub(other).map_err(Into::into)
    }

    #[inline]
    pub fn try_div(self, other: Self) -> Result<Self, ExactArithmeticError> {
        if self.denominator == 0 || other.denominator == 0 {
            return Err(ExactArithmeticError::ZeroDenominator);
        }
        if other.numerator == 0 {
            return Err(ExactArithmeticError::DivisionByZero);
        }
        if other.numerator == 1 && other.denominator == 1 {
            return Ok(self);
        }
        self.try_mul(Self {
            numerator: other.denominator,
            denominator: other.numerator,
        })
    }

    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn div(self, other: Self) -> Result<Self, String> {
        self.try_div(other).map_err(Into::into)
    }

    #[inline]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.try_add(other).ok()
    }

    #[inline]
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.try_sub(other).ok()
    }

    #[inline]
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.try_mul(other).ok()
    }

    #[inline]
    pub fn checked_div(self, other: Self) -> Option<Self> {
        self.try_div(other).ok()
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.numerator == 0
    }

    #[inline]
    pub fn try_round_u64(self) -> Result<u64, ExactArithmeticError> {
        if self.denominator == 0 {
            return Err(ExactArithmeticError::ZeroDenominator);
        }
        let whole = self.numerator / self.denominator;
        let remainder = self.numerator % self.denominator;
        let increment = u128::from(remainder >= self.denominator / 2 + self.denominator % 2);
        let rounded = whole
            .checked_add(increment)
            .ok_or(ExactArithmeticError::DamageRoundingOverflow)?;
        u64::try_from(rounded).map_err(|_| ExactArithmeticError::RoundedDamageExceedsU64)
    }

    /// Preview settlement policy: nearest integer, ties upward, without floats.
    #[inline]
    pub fn round_u64(self) -> Result<u64, String> {
        self.try_round_u64().map_err(Into::into)
    }
}

impl Default for ExactFraction {
    #[inline]
    fn default() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }
}

impl std::ops::Add for ExactFraction {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.try_add(rhs).expect("fraction addition overflow")
    }
}

impl std::ops::Sub for ExactFraction {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.try_sub(rhs).expect("fraction subtraction underflow")
    }
}

impl std::ops::Mul for ExactFraction {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.try_mul(rhs).expect("fraction multiplication overflow")
    }
}

impl std::ops::Div for ExactFraction {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.try_div(rhs)
            .expect("fraction division by zero or overflow")
    }
}
