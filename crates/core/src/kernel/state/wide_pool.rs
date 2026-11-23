//! Bounded exact compounding for capped resources whose rational components
//! can exceed u128 before the balance reaches its ordinary capacity.
use super::{ExactPool, ExactPoolSummary};
use crate::combat::exact::ExactFraction;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Signed;
use serde::{Deserialize, Serialize};

const MAX_COMPONENT_BITS: u64 = 4096;

fn checked(value: BigRational) -> Result<BigRational, String> {
    if value.is_negative()
        || value.numer().bits() > MAX_COMPONENT_BITS
        || value.denom().bits() > MAX_COMPONENT_BITS
    {
        return Err(
            "wide resource requires nonnegative rational components within4096 bits".into(),
        );
    }
    Ok(value)
}

pub fn wide_fraction(value: ExactFraction) -> Result<BigRational, String> {
    let value = ExactFraction::new(value.numerator, value.denominator)?;
    checked(BigRational::new(
        BigInt::from(value.numerator),
        BigInt::from(value.denominator),
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WidePool {
    capacity: BigRational,
    initial: BigRational,
    current: BigRational,
    granted: BigRational,
    spent: BigRational,
    overflow: BigRational,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WideFraction {
    pub numerator: String,
    pub denominator: String,
}

impl Serialize for WideFraction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !crate::formula::is_canonical_decimal_uint(&self.numerator)
            || !crate::formula::is_canonical_decimal_uint(&self.denominator)
            || self.denominator == "0"
        {
            return Err(serde::ser::Error::custom(
                "wide fraction components must be canonical unsigned decimal integers with nonzero denominator",
            ));
        }
        serializer.serialize_str(&format!("{}/{}", self.numerator, self.denominator))
    }
}

impl<'de> Deserialize<'de> for WideFraction {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct WideFractionVisitor;
        impl serde::de::Visitor<'_> for WideFractionVisitor {
            type Value = WideFraction;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a canonical \"<numerator>/<denominator>\" rational string")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                let (n_str, d_str) =
                    crate::formula::parse_canonical_rational_str(value).map_err(E::custom)?;
                let n = n_str.parse::<BigInt>().map_err(E::custom)?;
                let d = d_str.parse::<BigInt>().map_err(E::custom)?;
                let rational = checked(BigRational::new(n, d)).map_err(E::custom)?;
                Ok(WideFraction::from(&rational))
            }
        }
        deserializer.deserialize_str(WideFractionVisitor)
    }
}

impl From<&BigRational> for WideFraction {
    fn from(value: &BigRational) -> Self {
        Self {
            numerator: value.numer().to_string(),
            denominator: value.denom().to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WidePoolSummary {
    pub capacity: WideFraction,
    pub initial: WideFraction,
    pub current: WideFraction,
    pub granted: WideFraction,
    pub spent: WideFraction,
    pub overflow: WideFraction,
    pub conserved: bool,
}

/// Wide goes first so numeric strings retain their exact wire representation
/// even when a legacy ExactFraction deserializer could accept their values.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ResourcePoolSummary {
    Wide(WidePoolSummary),
    Exact(ExactPoolSummary),
}

impl From<ExactPoolSummary> for ResourcePoolSummary {
    fn from(value: ExactPoolSummary) -> Self {
        Self::Exact(value)
    }
}

impl From<WidePoolSummary> for ResourcePoolSummary {
    fn from(value: WidePoolSummary) -> Self {
        const SAFE_COMPONENT: u128 = 9_007_199_254_740_991;
        let compact = |value: &WideFraction| -> Option<ExactFraction> {
            let n = value.numerator.parse::<u128>().ok()?;
            let d = value.denominator.parse::<u128>().ok()?;
            if n > SAFE_COMPONENT || d > SAFE_COMPONENT {
                return None;
            }
            ExactFraction::new(n, d).ok()
        };
        let small = (|| {
            Some(ExactPoolSummary {
                capacity: compact(&value.capacity)?,
                initial: compact(&value.initial)?,
                current: compact(&value.current)?,
                granted: compact(&value.granted)?,
                spent: compact(&value.spent)?,
                overflow: compact(&value.overflow)?,
                conserved: value.conserved,
            })
        })();
        small.map_or_else(|| Self::Wide(value), Self::Exact)
    }
}

impl WidePool {
    pub fn from_exact(pool: &ExactPool) -> Result<Self, String> {
        let value = pool.summary()?;
        Ok(Self {
            capacity: wide_fraction(value.capacity)?,
            initial: wide_fraction(value.initial)?,
            current: wide_fraction(value.current)?,
            granted: wide_fraction(value.granted)?,
            spent: wide_fraction(value.spent)?,
            overflow: wide_fraction(value.overflow)?,
        })
    }

    pub fn current(&self) -> &BigRational {
        &self.current
    }

    pub fn capacity(&self) -> &BigRational {
        &self.capacity
    }

    /// Spend the available part of an exact requested amount. Resource owners
    /// decide whether an unpaid remainder may draw a separate credit line.
    pub fn spend_up_to(&mut self, maximum: BigRational) -> Result<BigRational, String> {
        let actual = checked(maximum)?.min(self.current.clone());
        let current = checked(&self.current - &actual)?;
        let spent = checked(&self.spent + &actual)?;
        self.current = current;
        self.spent = spent;
        Ok(actual)
    }

    /// Record the full gain and discarded excess without rounding a component.
    /// A precision-limit failure leaves every ledger field unchanged.
    pub fn grant(&mut self, amount: BigRational) -> Result<(BigRational, BigRational), String> {
        let amount = checked(amount)?;
        let applied = amount.clone().min(&self.capacity - &self.current);
        let excess = checked(&amount - &applied)?;
        let current = checked(&self.current + &applied)?;
        let granted = checked(&self.granted + amount)?;
        checked(&self.initial + &granted)?;
        let overflow = checked(&self.overflow + &excess)?;
        self.current = current;
        self.granted = granted;
        self.overflow = overflow;
        Ok((applied, excess))
    }

    pub fn summary(&self) -> Result<WidePoolSummary, String> {
        let conserved =
            &self.initial + &self.granted == &self.current + &self.spent + &self.overflow;
        if !conserved {
            return Err("wide resource conservation failed".into());
        }
        Ok(WidePoolSummary {
            capacity: (&self.capacity).into(),
            initial: (&self.initial).into(),
            current: (&self.current).into(),
            granted: (&self.granted).into(),
            spent: (&self.spent).into(),
            overflow: (&self.overflow).into(),
            conserved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;
    fn f(n: u128, d: u128) -> ExactFraction {
        ExactFraction::new(n, d).unwrap()
    }

    #[test]
    fn exact_partial_spend_retains_fractional_remainder_and_conserved_cap_overflow() {
        let mut pool = WidePool::from_exact(&ExactPool::new(f(100, 1), f(20, 1)).unwrap()).unwrap();
        pool.grant(wide_fraction(f(90, 1)).unwrap()).unwrap();
        assert_eq!(
            pool.spend_up_to(wide_fraction(f(29, 10)).unwrap()).unwrap(),
            wide_fraction(f(29, 10)).unwrap()
        );
        assert_eq!(pool.current(), &wide_fraction(f(971, 10)).unwrap());
        assert_eq!(
            pool.spend_up_to(wide_fraction(f(150, 1)).unwrap()).unwrap(),
            wide_fraction(f(971, 10)).unwrap()
        );
        let summary = pool.summary().unwrap();
        assert_eq!(
            summary.spent,
            WideFraction::from(&wide_fraction(f(100, 1)).unwrap())
        );
        assert_eq!(
            summary.overflow,
            WideFraction::from(&wide_fraction(f(10, 1)).unwrap())
        );
        assert!(summary.conserved);
        let before = pool.clone();
        assert!(
            pool.spend_up_to(BigRational::from_integer(BigInt::from(-1)))
                .is_err()
        );
        assert_eq!(pool, before);
    }

    #[test]
    fn compounding_crosses_u128_before_capacity_and_retains_exact_cap_overflow() {
        let mut small = ExactPool::new(f(16500, 1), f(0, 1)).unwrap();
        small.grant(f(450, 1)).unwrap();
        let mut pool = WidePool::from_exact(&small).unwrap();
        let tenth = wide_fraction(f(1, 10)).unwrap();
        for _ in 0..37 {
            pool.grant(pool.current() * &tenth).unwrap();
        }
        let expected = BigRational::new(
            BigInt::from(9) * BigInt::from(11).pow(37),
            BigInt::from(10).pow(37) / 50,
        );
        assert_eq!(pool.current(), &expected);
        assert!(pool.current().numer().bits() > 128);
        for _ in 37..150 {
            pool.grant(pool.current() * &tenth).unwrap();
        }
        assert_eq!(pool.current(), &BigRational::from_integer(16500.into()));
        let summary = pool.summary().unwrap();
        assert!(summary.conserved);
        let wire = serde_json::to_value(ResourcePoolSummary::from(summary)).unwrap();
        let granted = wire["granted"].as_str().unwrap();
        let (gn, gd) = granted.split_once('/').unwrap();
        assert!(gn.parse::<u128>().is_err());
        assert!(gd.chars().all(|c| c.is_ascii_digit()));
        let decoded: ResourcePoolSummary = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
        for invalid in [
            r#""1/0""#,
            r#""01/2""#,
            r#""-1/2""#,
            r#"{"numerator":"1","denominator":"2"}"#,
        ] {
            assert!(serde_json::from_str::<WideFraction>(invalid).is_err());
        }
    }

    #[test]
    fn small_wide_ledger_preserves_existing_numeric_summary_and_prior_grants() {
        let mut small = ExactPool::new(f(5775, 10), f(0, 1)).unwrap();
        small.grant(f(5775, 10)).unwrap();
        let pool = WidePool::from_exact(&small).unwrap();
        assert_eq!(
            serde_json::to_value(ResourcePoolSummary::from(pool.summary().unwrap())).unwrap(),
            serde_json::to_value(small.summary().unwrap()).unwrap()
        );
    }

    #[test]
    fn oversized_or_negative_gain_rejects_atomically() {
        let mut pool = WidePool::from_exact(&ExactPool::new(f(500, 1), f(0, 1)).unwrap()).unwrap();
        let before = pool.clone();
        assert!(
            pool.grant(BigRational::from_integer(BigInt::from(-1)))
                .is_err()
        );
        assert_eq!(pool, before);
        assert!(
            pool.grant(BigRational::from_integer(BigInt::from(1) << 4096))
                .is_err()
        );
        assert_eq!(pool, before);
        assert!(
            wide_fraction(ExactFraction {
                numerator: 1,
                denominator: 0
            })
            .is_err()
        );
        assert!(pool.current().is_zero());
    }
}
