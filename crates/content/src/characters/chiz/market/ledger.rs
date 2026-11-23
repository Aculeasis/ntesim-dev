use super::*;
use ntesim_core::kernel::state::{ExactPool, wide_fraction};

impl super::super::finance::State {
    pub(super) fn invest(
        &mut self,
        seed: u64,
        owner: usize,
        a3: bool,
        recorded: bool,
        ends_at_us: u64,
    ) -> Result<(), String> {
        let debt = self
            .debt
            .as_mut()
            .ok_or("Chiz market requires loan ledger")?;
        if self.market.is_none() {
            let cap = self.wallet.capacity();
            let cap = ExactFraction::new(
                cap.numer().to_u128().ok_or("Chiz cap numerator overflow")?,
                cap.denom()
                    .to_u128()
                    .ok_or("Chiz cap denominator overflow")?,
            )?;
            self.market = Some(Box::new(State {
                wallet: WidePool::from_exact(&ExactPool::new(
                    cap.mul(ExactFraction::new(2, 1)?)?,
                    ExactFraction::new(0, 1)?,
                )?)?,
                session: None,
                rng: prices::new(seed, owner)?,
                generation: 0,
                first_crit: true,
            }));
        }
        let market = self.market.as_mut().unwrap();
        if market.session.is_some() {
            return Err("Chiz investment overlaps an unclosed market".into());
        }
        let prices = prices::generate(&mut market.rng, a3, recorded)?;
        let remaining = debt.capacity() - debt.current();
        debt.grant(remaining.clone())?;
        let savings = self.wallet.spend_up_to(self.wallet.current().clone())?;
        market.wallet.grant(savings + remaining)?;
        market.generation = market
            .generation
            .checked_add(1)
            .ok_or("Chiz market generation overflow")?;
        market.first_crit = true;
        market.session = Some(Session {
            ends_at_us,
            prices,
            index: 0,
            pending: BTreeMap::new(),
            closing: false,
        });
        Ok(())
    }

    pub(super) fn withdraw(&mut self) -> Result<(), String> {
        let Some(market) = &mut self.market else {
            return Ok(());
        };
        if market.session.take().is_none() {
            return Ok(());
        }
        let capital = market.wallet.spend_up_to(market.wallet.current().clone())?;
        let repaid = self
            .debt
            .as_mut()
            .ok_or("Chiz withdrawal lacks debt")?
            .spend_up_to(capital.clone())?;
        self.wallet.grant(capital - repaid)?;
        Ok(())
    }

    pub(super) fn revalue(&mut self, price: i16) -> Result<u128, String> {
        let market = self.market.as_mut().ok_or("Chiz settlement lacks market")?;
        let before = market.wallet.current().clone();
        if price >= 0 {
            market
                .wallet
                .grant(&before * wide_fraction(ExactFraction::new(price as u128, 100)?)?)?;
        } else {
            market.wallet.spend_up_to(
                &before * wide_fraction(ExactFraction::new(price.unsigned_abs().into(), 100)?)?,
            )?;
        }
        market
            .wallet
            .current()
            .to_integer()
            .to_u128()
            .ok_or_else(|| "Chiz proceeds exceed damage-base range".into())
    }
}
