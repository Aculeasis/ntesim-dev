//! Checked nonnegative rational arithmetic with explicit final quantization.
//! These are technical formula semantics, not an asserted NTE damage formula.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ratio {
    pub numerator: u64,
    pub denominator: u64,
}

pub(crate) fn is_canonical_decimal_uint(text: &str) -> bool {
    text == "0"
        || (!text.is_empty() && !text.starts_with('0') && text.bytes().all(|b| b.is_ascii_digit()))
}

pub(crate) fn parse_canonical_rational_str(text: &str) -> Result<(&str, &str), &'static str> {
    let (numerator, denominator) = text
        .split_once('/')
        .ok_or("rational must be a \"<numerator>/<denominator>\" string")?;
    if !is_canonical_decimal_uint(numerator) || !is_canonical_decimal_uint(denominator) {
        return Err("rational components must be canonical unsigned decimal integers");
    }
    if denominator == "0" {
        return Err("fraction denominator cannot be zero");
    }
    Ok((numerator, denominator))
}

impl Serialize for Ratio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.denominator == 0 {
            return Err(serde::ser::Error::custom(
                "fraction denominator cannot be zero",
            ));
        }
        serializer.serialize_str(&format!("{}/{}", self.numerator, self.denominator))
    }
}

impl<'de> Deserialize<'de> for Ratio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RatioVisitor;
        impl serde::de::Visitor<'_> for RatioVisitor {
            type Value = Ratio;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a canonical \"<numerator>/<denominator>\" rational string")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                let (n_str, d_str) = parse_canonical_rational_str(value).map_err(E::custom)?;
                let numerator = n_str.parse::<u64>().map_err(E::custom)?;
                let denominator = d_str.parse::<u64>().map_err(E::custom)?;
                Ok(Ratio {
                    numerator,
                    denominator,
                })
            }
        }
        deserializer.deserialize_str(RatioVisitor)
    }
}

impl Ratio {
    pub const fn zero() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }

    pub const fn one() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rounding {
    Floor,
    Ceil,
    NearestTiesEven,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Term {
    pub stat: String,
    pub coefficient: Ratio,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub id: String,
    pub multiplier: Ratio,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Formula {
    /// Caller-declared output quanta. Dimensional correctness needs an adapter.
    pub unit: String,
    pub flat: Ratio,
    pub terms: Vec<Term>,
    pub stages: Vec<Stage>,
    pub rounding: Rounding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormulaError {
    ZeroDenominator,
    Overflow,
    EmptyUnit,
    EmptyStat,
    UnknownStat(String),
    EmptyStage,
    DuplicateStage(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Exact {
    n: u128,
    d: u128,
}
pub(crate) fn gcd(mut a: u128, mut b: u128) -> u128 {
    if a == 1 || b == 1 {
        return 1;
    }
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    if (a | b) <= u128::from(u64::MAX) {
        return u128::from(gcd_binary(a as u64, b as u64));
    }
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn gcd_binary(mut a: u64, mut b: u64) -> u64 {
    let shared_shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    while b != 0 {
        b >>= b.trailing_zeros();
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        b -= a;
    }
    a << shared_shift
}
impl Exact {
    fn new(n: u128, d: u128) -> Result<Self, FormulaError> {
        if d == 0 {
            return Err(FormulaError::ZeroDenominator);
        }
        let common = gcd(n, d);
        Ok(Self {
            n: n / common,
            d: d / common,
        })
    }
    fn from_ratio(value: Ratio) -> Result<Self, FormulaError> {
        Self::new(u128::from(value.numerator), u128::from(value.denominator))
    }
    fn add(self, other: Self) -> Result<Self, FormulaError> {
        let common = gcd(self.d, other.d);
        let n = self
            .n
            .checked_mul(other.d / common)
            .and_then(|left| {
                other
                    .n
                    .checked_mul(self.d / common)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or(FormulaError::Overflow)?;
        let reduction = gcd(n, common);
        let d = (self.d / common)
            .checked_mul(other.d / reduction)
            .ok_or(FormulaError::Overflow)?;
        Self::new(n / reduction, d)
    }
    fn multiply(self, other: Self) -> Result<Self, FormulaError> {
        let left = gcd(self.n, other.d);
        let right = gcd(other.n, self.d);
        let n = (self.n / left)
            .checked_mul(other.n / right)
            .ok_or(FormulaError::Overflow)?;
        let d = (self.d / right)
            .checked_mul(other.d / left)
            .ok_or(FormulaError::Overflow)?;
        Self::new(n, d)
    }
    fn quantize(self, rounding: Rounding) -> Result<u64, FormulaError> {
        let quotient = self.n / self.d;
        let remainder = self.n % self.d;
        let increment = match rounding {
            Rounding::Floor => false,
            Rounding::Ceil => remainder != 0,
            Rounding::NearestTiesEven => {
                remainder > self.d - remainder
                    || (remainder == self.d - remainder && !quotient.is_multiple_of(2))
            }
        };
        let rounded = quotient
            .checked_add(u128::from(increment))
            .ok_or(FormulaError::Overflow)?;
        u64::try_from(rounded).map_err(|_| FormulaError::Overflow)
    }
    fn audit(self) -> ExactValue {
        ExactValue {
            numerator: self.n.to_string(),
            denominator: self.d.to_string(),
        }
    }
}

/// Decimal strings retain exact u128 intermediates in ordinary JSON consumers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExactValue {
    pub numerator: String,
    pub denominator: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TermResult {
    pub stat: String,
    pub value: ExactValue,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StageResult {
    pub id: String,
    pub value: ExactValue,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Evaluation {
    pub unit: String,
    pub rounding: Rounding,
    pub flat: ExactValue,
    pub terms: Vec<TermResult>,
    pub base: ExactValue,
    pub stages: Vec<StageResult>,
    pub exact: ExactValue,
    pub amount: u64,
}
impl Formula {
    pub fn evaluate(&self, stats: &BTreeMap<String, Ratio>) -> Result<Evaluation, FormulaError> {
        if self.unit.trim().is_empty() {
            return Err(FormulaError::EmptyUnit);
        }
        let flat = Exact::from_ratio(self.flat)?;
        let mut base = flat;
        let mut terms = Vec::new();
        for term in &self.terms {
            if term.stat.trim().is_empty() {
                return Err(FormulaError::EmptyStat);
            }
            let stat = stats
                .get(&term.stat)
                .ok_or_else(|| FormulaError::UnknownStat(term.stat.clone()))?;
            let value = Exact::from_ratio(*stat)?.multiply(Exact::from_ratio(term.coefficient)?)?;
            base = base.add(value)?;
            terms.push(TermResult {
                stat: term.stat.clone(),
                value: value.audit(),
            });
        }
        let mut exact = base;
        let mut stages = Vec::new();
        let mut seen = BTreeSet::new();
        for stage in &self.stages {
            if stage.id.trim().is_empty() {
                return Err(FormulaError::EmptyStage);
            }
            if !seen.insert(&stage.id) {
                return Err(FormulaError::DuplicateStage(stage.id.clone()));
            }
            exact = exact.multiply(Exact::from_ratio(stage.multiplier)?)?;
            stages.push(StageResult {
                id: stage.id.clone(),
                value: exact.audit(),
            });
        }
        Ok(Evaluation {
            unit: self.unit.clone(),
            rounding: self.rounding,
            flat: flat.audit(),
            terms,
            base: base.audit(),
            stages,
            exact: exact.audit(),
            amount: exact.quantize(self.rounding)?,
        })
    }
}

#[cfg(test)]
mod tests;
