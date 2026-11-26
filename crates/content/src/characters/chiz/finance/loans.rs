//! Own savings and drawn principal remain distinct exact conserved ledgers.
use super::State;
use crate::simulation::{ExactFraction, GameActorConfig};
use ntesim_core::kernel::state::wide_fraction;
use num_rational::BigRational;
use num_traits::Zero;

impl State {
    pub(crate) fn in_surplus(&self) -> bool {
        !self.in_debt()
            && !self
                .market
                .as_ref()
                .and_then(|market| market.session.as_ref())
                .is_some_and(|session| session.closing)
    }

    pub(crate) fn balance(&self) -> &BigRational {
        if let Some(market) = &self.market
            && market.session.is_some()
        {
            market.wallet.current()
        } else {
            self.wallet.current()
        }
    }

    pub(crate) fn balance_mut(&mut self) -> &mut ntesim_core::kernel::state::WidePool {
        if let Some(market) = &mut self.market
            && market.session.is_some()
        {
            &mut market.wallet
        } else {
            &mut self.wallet
        }
    }

    pub(crate) fn in_debt(&self) -> bool {
        if self
            .market
            .as_ref()
            .is_some_and(|market| market.session.is_some())
        {
            return false;
        }
        self.debt
            .as_ref()
            .is_some_and(|debt| !debt.current().is_zero())
    }

    pub(crate) fn available(&self) -> BigRational {
        if self
            .market
            .as_ref()
            .is_some_and(|market| market.session.is_some())
        {
            return self.balance().clone();
        }
        match &self.debt {
            Some(debt) => self.wallet.current() + debt.capacity() - debt.current(),
            None => self.wallet.current().clone(),
        }
    }

    pub(crate) fn acquisition_multiplier(
        &self,
        config: &GameActorConfig,
    ) -> Result<ExactFraction, String> {
        let mut multiplier = if config.awakening.has_effect(4) {
            super::super::parameters::PARAMETERS.normal_acquisition_multiplier
        } else {
            ExactFraction::new(1, 1)?
        };
        if self.in_debt() && config.awakening.has_resonance_3() {
            multiplier =
                multiplier.add(super::super::parameters::PARAMETERS.debt_acquisition_bonus)?;
        }
        Ok(multiplier)
    }

    pub(crate) fn acquire(&mut self, amount: ExactFraction) -> Result<(), String> {
        if self
            .market
            .as_ref()
            .is_some_and(|market| market.session.is_some())
        {
            self.balance_mut().grant(wide_fraction(amount)?)?;
            return Ok(());
        }
        let mut own = self.wallet.clone();
        let mut debt = self.debt.clone();
        let amount = wide_fraction(amount)?;
        let repaid = if let Some(pool) = &mut debt {
            pool.spend_up_to(amount.clone())?
        } else {
            BigRational::zero()
        };
        own.grant(amount - repaid)?;
        self.wallet = own;
        self.debt = debt;
        Ok(())
    }

    pub(crate) fn spend(&mut self, maximum: BigRational) -> Result<BigRational, String> {
        if maximum < BigRational::zero() {
            return Err("Chiz spend cannot be negative".into());
        }
        let actual = maximum.min(self.available());
        if self
            .market
            .as_ref()
            .is_some_and(|market| market.session.is_some())
        {
            return self.balance_mut().spend_up_to(actual);
        }
        let mut own = self.wallet.clone();
        let mut debt = self.debt.clone();
        let own_spend = own.spend_up_to(actual.clone())?;
        let borrowed = &actual - own_spend;
        if !borrowed.is_zero() {
            let (_, excess) = debt
                .as_mut()
                .ok_or("Chiz spend has no admitted loan ledger")?
                .grant(borrowed)?;
            if !excess.is_zero() {
                return Err("Chiz loan exceeded admitted capacity".into());
            }
        }
        self.wallet = own;
        self.debt = debt;
        Ok(actual)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntesim_core::kernel::state::{ExactPool, WidePool};
    fn f(n: u128, d: u128) -> ExactFraction {
        ExactFraction::new(n, d).unwrap()
    }
    fn state(own: u128, debt: u128, cap: u128) -> State {
        State {
            wallet: WidePool::from_exact(&ExactPool::new(f(cap, 1), f(own, 1)).unwrap()).unwrap(),
            debt: Some(
                WidePool::from_exact(&ExactPool::new(f(cap, 1), f(debt, 1)).unwrap()).unwrap(),
            ),
            interest: None,
            market: None,
        }
    }
    #[test]
    fn source_intermediate_fields_read_remaining_credit_and_spend_own_before_drawn_principal() {
        let mut finance = state(20, 40, 100);
        assert_eq!(finance.available(), wide_fraction(f(80, 1)).unwrap());
        let captured = finance.available() / BigRational::from_integer(16.into());
        assert_eq!(
            finance.spend(captured).unwrap(),
            wide_fraction(f(5, 1)).unwrap()
        );
        assert_eq!(finance.wallet.current(), &wide_fraction(f(15, 1)).unwrap());
        assert_eq!(
            finance.debt.as_ref().unwrap().current(),
            &wide_fraction(f(40, 1)).unwrap()
        );
        let mut finance = state(0, 98, 100);
        assert_eq!(
            finance.spend(wide_fraction(f(5, 1)).unwrap()).unwrap(),
            wide_fraction(f(2, 1)).unwrap()
        );
        assert_eq!(
            finance.debt.as_ref().unwrap().current(),
            &wide_fraction(f(100, 1)).unwrap()
        );
        assert!(finance.wallet.summary().unwrap().conserved);
        assert!(finance.debt.as_ref().unwrap().summary().unwrap().conserved);
    }
    #[test]
    fn a4_and_r3_efficiency_add_before_repay_and_surplus_uses_only_a4_after_last_principal() {
        let mut config=crate::dsl::compiler::compile_scenario("options duration=1; chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=1000 def=1080 resist=0; active chiz;",Default::default()).unwrap().request.actors.remove(0);
        config.awakening = ntesim_data::AwakeningSelection::selected([2, 4, 6]).unwrap();
        let mut finance = state(0, 1000, 1000);
        let multiplier = finance.acquisition_multiplier(&config).unwrap();
        assert_eq!(multiplier, f(17, 10));
        finance.acquire(f(450, 1).mul(multiplier).unwrap()).unwrap();
        assert_eq!(
            finance.debt.as_ref().unwrap().current(),
            &wide_fraction(f(235, 1)).unwrap()
        );
        assert!(finance.wallet.current().is_zero());
        finance
            .acquire(
                f(450, 1)
                    .mul(finance.acquisition_multiplier(&config).unwrap())
                    .unwrap(),
            )
            .unwrap();
        assert!(!finance.in_debt());
        assert_eq!(finance.wallet.current(), &wide_fraction(f(530, 1)).unwrap());
        assert_eq!(finance.acquisition_multiplier(&config).unwrap(), f(6, 5));
    }
}
