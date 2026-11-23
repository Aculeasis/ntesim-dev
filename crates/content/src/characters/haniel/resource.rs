//! Persistent Plot Armor; preloading does not trigger Harmony or Ensemble.
use crate::simulation::{ExactFraction as F, GameActorConfig, state::ActorRuntime};

pub(crate) fn capacity(awakening: &ntesim_data::AwakeningSelection) -> u8 {
    if awakening.has_effect(1) { 6 } else { 4 }
}

pub(crate) fn initial_value(
    initial: Option<crate::formula::Ratio>,
    awakening: &ntesim_data::AwakeningSelection,
) -> Result<u8, String> {
    let value = initial.map(F::from_ratio).transpose()?.unwrap_or_default();
    if value.denominator != 1 || value.numerator > u128::from(capacity(awakening)) {
        return Err("Haniel Plot Armor must be integral in 0..4 (A1: 0..6).".into());
    }
    u8::try_from(value.numerator).map_err(|_| "Haniel initial Plot Armor overflow".into())
}

pub(crate) fn initialize(config: &GameActorConfig, actor: &mut ActorRuntime) -> Result<(), String> {
    let plot = initial_value(config.initial_resources.plot, &config.awakening)?;
    let state = actor.character.haniel_mut()?;
    state.plot = plot;
    state.plot_cap = capacity(&config.awakening);
    Ok(())
}
