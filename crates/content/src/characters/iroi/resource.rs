//! Moon is reserved by Dream; its live balance drains on the released clock.
use super::awakenings::State;
use crate::simulation::{ExactFraction as F, GameActorConfig, state::ActorRuntime};

pub(crate) fn initial_value(initial: Option<crate::formula::Ratio>) -> Result<F, String> {
    let value = initial.map(F::from_ratio).transpose()?.unwrap_or_default();
    if value
        .try_cmp(F::new(100, 1)?)
        .map_err(|e| e.to_string())?
        .is_gt()
    {
        return Err("Iroi initial Moon exceeds capacity100.".into());
    }
    Ok(value)
}

pub(crate) fn initialize(config: &GameActorConfig, actor: &mut ActorRuntime) -> Result<(), String> {
    let value = initial_value(config.initial_resources.moon)?;
    actor.character.iroi_mut()?.moon = if config.awakening.has_effect(1) {
        F::new(100, 1)?
    } else {
        value
    };
    Ok(())
}

pub(crate) fn duration_us(moon: F) -> Result<u64, String> {
    moon.mul(F::new(125_000, 1)?)?.round_u64()
}

pub(crate) fn remaining(state: &State, now: u64) -> Result<F, String> {
    if !state.dream_active || now <= state.dream_release_us {
        return Ok(state.moon);
    }
    if now >= state.dream_end_us {
        return Ok(F::default());
    }
    let spent = F::new(u128::from(now - state.dream_release_us), 125_000)?;
    if spent
        .try_cmp(state.moon)
        .map_err(|e| e.to_string())?
        .is_ge()
    {
        Ok(F::default())
    } else {
        state.moon.sub(spent)
    }
}

pub(crate) fn dream_active(state: &State, now: u64) -> bool {
    state.dream_active && now < state.dream_end_us
}

pub(crate) fn update_off_field_moon(
    config: &GameActorConfig,
    state: &mut State,
    now: u64,
    is_living: bool,
    is_on_field: bool,
) -> Result<(), String> {
    if !config.awakening.has_effect(1) || !is_living || is_on_field || state.dream_active {
        state.last_off_field_us = if !is_on_field && is_living {
            Some(now)
        } else {
            None
        };
        return Ok(());
    }
    if let Some(last) = state.last_off_field_us
        && now > last
    {
        let absolute_elapsed_us = now - last;
        let gain = F::new(u128::from(absolute_elapsed_us) * 3, 1_000_000)?;
        let cap = F::new(100, 1)?;
        let new_moon = state.moon.add(gain)?;
        if new_moon.numerator > 100 * new_moon.denominator {
            let overflow = new_moon.sub(cap)?;
            state.moon_overflow = state.moon_overflow.add(overflow)?;
            state.moon = cap;
        } else {
            state.moon = new_moon;
        }
    }
    state.last_off_field_us = Some(now);
    Ok(())
}
