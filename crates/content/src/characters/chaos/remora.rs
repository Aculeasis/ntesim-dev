//! Justice in Time listens to real shared Remora generations.
use crate::runtime_api::{CallbackContext, DamageContext, HitModifiers};
use crate::simulation::{
    ExactFraction, GameActorConfig, GameScenarioRequest, state::GameSimulatorState,
};
use std::collections::BTreeMap;
mod packet;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChaosB2RemoraEndV1;
pub(crate) const PACKET: &str = "GE_Reaction_3_new_1071_Damage";
pub(crate) const ABILITY: &str = "GA_Chaos_Passive_1";

pub(crate) fn validate_inputs(request: &GameScenarioRequest) -> Result<(), String> {
    if request.actions.iter().any(|action| matches!(action,
        crate::simulation::GameActionCommand::InvokeAbility { damage_effect_id, .. } if damage_effect_id == PACKET)) {
        return Err("Chaos Remora-end packet requires a real target status removal, not an authored ability input".into());
    }
    Ok(())
}

fn eligible(config: &GameActorConfig, hp: u64) -> Result<bool, String> {
    if config.character_id != "character:1071" || hp == 0 {
        return Ok(false);
    }
    let progression = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?;
    Ok(progression["breakthrough"]
        .as_u64()
        .ok_or("Chaos breakthrough missing")?
        >= 2)
}
pub(crate) fn validate(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    for (owner, config) in request.actors.iter().enumerate() {
        if eligible(config, state.actors[owner].hp)? {
            crate::runtime_api::record_hypothesis_id(request, assumptions, MODEL)?;
        }
    }
    Ok(())
}
pub(crate) fn created(
    ctx: &mut CallbackContext<'_>,
    target: usize,
    generation: u64,
) -> Result<bool, String> {
    let mut assumptions = BTreeMap::new();
    let mut armed = false;
    validate(ctx.request, ctx.state, &mut assumptions)?;
    for (owner, config) in ctx.request.actors.iter().enumerate() {
        if eligible(config, ctx.state.actors[owner].hp)? {
            let state = ctx.state.actors[owner].character.chaos_mut()?;
            state.warrant_bonus = Some(super::parameters::warrant_bonus(config)?);
            state.remoras.insert(target, generation);
            armed = true;
        }
    }
    ctx.state.execution_model_assumptions.extend(assumptions);
    Ok(armed)
}

pub(crate) fn has_listener(state: &GameSimulatorState, target: usize, generation: u64) -> bool {
    state.actors.iter().any(|actor| actor.hp > 0 && matches!(&actor.character,
        crate::characters::RuntimeState::Chaos(chaos) if chaos.remoras.get(&target) == Some(&generation)))
}
pub(crate) fn removed(
    ctx: &mut CallbackContext<'_>,
    target: usize,
    generation: u64,
    elapsed: u64,
    command: usize,
) -> Result<(), String> {
    for owner in 0..ctx.request.actors.len() {
        if ctx.request.actors[owner].character_id != "character:1071" {
            continue;
        }
        let state = ctx.state.actors[owner].character.chaos_mut()?;
        if state.remoras.get(&target) != Some(&generation) {
            continue;
        }
        state.remoras.remove(&target);
        if !eligible(&ctx.request.actors[owner], ctx.state.actors[owner].hp)?
            || ctx.state.targets[target].hp_current == 0
        {
            continue;
        }
        // .45 per real second beyond5, capped at3; kept outside coefficient/bonus zones.
        let bonus = ExactFraction::new(
            u128::from(elapsed.saturating_sub(5_000_000)) * 9,
            20_000_000,
        )?;
        let bonus = if bonus.numerator > 3 * bonus.denominator {
            ExactFraction::new(3, 1)?
        } else {
            bonus
        };
        packet::emit(ctx, owner, target, command, bonus)?;
    }
    Ok(())
}
pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if ctx.damage_effect_id != PACKET {
        return Ok(modifiers);
    }
    let crate::characters::RuntimeState::Chaos(state) = &ctx.actor.character else {
        return Err("Chaos Remora packet has no owner state".into());
    };
    let bonus = state
        .remora_bonuses
        .get(&ctx.command_index)
        .copied()
        .ok_or("Chaos Remora packet has no real elapsed-generation factor")?;
    modifiers.add_final(bonus)
}
