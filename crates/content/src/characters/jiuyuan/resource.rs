//! Source-local integer PowerBullets, independent of Ultimate energy and Cycle.
use crate::simulation::{ExactFraction as F, GameActorConfig, state::ActorRuntime};

pub(crate) const CAPACITY: u8 = 6;

pub(crate) fn initial_value(initial: Option<crate::formula::Ratio>) -> Result<u8, String> {
    let value = initial.map(F::from_ratio).transpose()?.unwrap_or_default();
    if value.denominator != 1 || value.numerator > u128::from(CAPACITY) {
        return Err("Jiuyuan PowerBullets must be integral in 0..6.".into());
    }
    u8::try_from(value.numerator).map_err(|_| "Jiuyuan initial PowerBullets overflow".into())
}

pub(crate) fn initialize(config: &GameActorConfig, actor: &mut ActorRuntime) -> Result<(), String> {
    actor.character.jiuyuan_mut()?.bullets = initial_value(config.initial_resources.bullets)?;
    Ok(())
}

pub(crate) fn defeat(actor: &mut ActorRuntime) {
    if let crate::characters::RuntimeState::Jiuyuan(state) = &mut actor.character {
        state.bullets = 0;
    }
}
