//! Exact bounded resource accounting; callers own units and gain/spend rules.
use crate::combat::exact::ExactFraction;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactPool {
    capacity: ExactFraction,
    initial: ExactFraction,
    current: ExactFraction,
    granted: ExactFraction,
    spent: ExactFraction,
    overflow: ExactFraction,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExactPoolSummary {
    pub capacity: ExactFraction,
    pub initial: ExactFraction,
    pub current: ExactFraction,
    pub granted: ExactFraction,
    pub spent: ExactFraction,
    pub overflow: ExactFraction,
    pub conserved: bool,
}

fn minimum(a: ExactFraction, b: ExactFraction) -> Result<ExactFraction, String> {
    Ok(if a.try_cmp(b).map_err(String::from)?.is_gt() {
        b
    } else {
        a
    })
}

impl ExactPool {
    pub fn new(capacity: ExactFraction, initial: ExactFraction) -> Result<Self, String> {
        if capacity.is_zero() || initial.try_cmp(capacity).map_err(String::from)?.is_gt() {
            return Err("exact resource initial value exceeds a positive capacity".into());
        }
        let zero = ExactFraction::new(0, 1)?;
        Ok(Self {
            capacity,
            initial,
            current: initial,
            granted: zero,
            spent: zero,
            overflow: zero,
        })
    }

    pub fn current(&self) -> ExactFraction {
        self.current
    }

    /// Return applied gain and excess; failures leave every field unchanged.
    pub fn grant(
        &mut self,
        amount: ExactFraction,
    ) -> Result<(ExactFraction, ExactFraction), String> {
        let applied = minimum(amount, self.capacity.sub(self.current)?)?;
        let excess = amount.sub(applied)?;
        let current = self.current.add(applied)?;
        let granted = self.granted.add(amount)?;
        self.initial.add(granted)?;
        let overflow = self.overflow.add(excess)?;
        self.current = current;
        self.granted = granted;
        self.overflow = overflow;
        Ok((applied, excess))
    }

    /// Partial stock spends its exact remainder, never an integer rounding.
    pub fn spend_up_to(&mut self, maximum: ExactFraction) -> Result<ExactFraction, String> {
        let actual = minimum(self.current, maximum)?;
        let current = self.current.sub(actual)?;
        let spent = self.spent.add(actual)?;
        self.current = current;
        self.spent = spent;
        Ok(actual)
    }

    pub fn summary(&self) -> Result<ExactPoolSummary, String> {
        let conserved =
            self.initial.add(self.granted)? == self.current.add(self.spent)?.add(self.overflow)?;
        if !conserved {
            return Err("exact resource conservation failed".into());
        }
        Ok(ExactPoolSummary {
            capacity: self.capacity,
            initial: self.initial,
            current: self.current,
            granted: self.granted,
            spent: self.spent,
            overflow: self.overflow,
            conserved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn f(n: u128, d: u128) -> ExactFraction {
        ExactFraction::new(n, d).unwrap()
    }

    #[test]
    fn exact_pool_preserves_native_fractional_gains_partial_spends_cap_and_continuation() {
        let mut pool = ExactPool::new(f(60, 1), f(0, 1)).unwrap();
        pool.grant(f(20, 1)).unwrap();
        pool.grant(f(40, 1)).unwrap();
        assert_eq!(
            pool.grant(f(20_000_003, 1_000_000)).unwrap(),
            (f(0, 1), f(20_000_003, 1_000_000))
        );
        assert_eq!(pool.spend_up_to(f(1, 10)).unwrap(), f(1, 10));
        assert_eq!(pool.spend_up_to(f(60, 1)).unwrap(), f(599, 10));
        assert_eq!(pool.spend_up_to(f(1, 1)).unwrap(), f(0, 1));
        pool.grant(f(1, 10)).unwrap();
        assert_eq!(pool.spend_up_to(f(1, 1)).unwrap(), f(1, 10));
        let summary = pool.summary().unwrap();
        assert_eq!(summary.spent, f(601, 10));
        assert_eq!(summary.current, f(0, 1));
        assert!(summary.conserved);
    }

    #[test]
    fn exact_pool_rejects_bad_fractions_and_accumulator_overflow_atomically() {
        let mut pool = ExactPool::new(f(60, 1), f(0, 1)).unwrap();
        let before = pool.clone();
        assert!(
            pool.grant(ExactFraction {
                numerator: 1,
                denominator: 0
            })
            .is_err()
        );
        assert_eq!(pool, before);
        pool.grant(f(u128::MAX, 1)).unwrap();
        let before = pool.clone();
        assert!(pool.grant(f(1, 1)).is_err());
        assert_eq!(pool, before);
        assert!(pool.summary().unwrap().conserved);
    }

    #[test]
    fn exact_comparison_handles_wide_unreduced_and_inverse_fractions_without_cross_products() {
        use std::cmp::Ordering::*;
        assert_eq!(
            f(u128::MAX, u128::MAX - 1)
                .try_cmp(f(u128::MAX - 1, u128::MAX))
                .unwrap(),
            Greater
        );
        assert_eq!(f(2, 3).try_cmp(f(3, 4)).unwrap(), Less);
        assert_eq!(
            ExactFraction {
                numerator: 2,
                denominator: 4
            }
            .try_cmp(f(1, 2))
            .unwrap(),
            Equal
        );
        assert_eq!(f(1, 7).try_cmp(f(0, 1)).unwrap(), Greater);
        assert!(
            f(1, 1)
                .try_cmp(ExactFraction {
                    numerator: 1,
                    denominator: 0
                })
                .is_err()
        );
    }
}
