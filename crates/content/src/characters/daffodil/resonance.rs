//! Persistent Support pool and immutable per-Skill capture; no timed global buff.
use super::parameters::PARAMETERS;
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::{ActorRuntime, GameSimulatorState};
use crate::simulation::{ExactFraction as F, GameEventReceipt, GameScenarioRequest};

pub(crate) const HYPOTHESIS: crate::HypothesisId =
    crate::HypothesisId::DaffodilResonanceSkillCaptureV1;
pub(crate) const CAPACITY: u8 = 2;

pub(crate) fn initial_value(initial: Option<crate::formula::Ratio>) -> Result<u8, String> {
    let value = initial.map(F::from_ratio).transpose()?.unwrap_or_default();
    if value.denominator != 1 || value.numerator > u128::from(CAPACITY) {
        return Err("Daffodil Resonance must be integral in 0..2.".into());
    }
    u8::try_from(value.numerator).map_err(|_| "Daffodil initial Resonance overflow".into())
}

pub(crate) fn initialize(
    config: &crate::simulation::GameActorConfig,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    actor.character.daffodil_mut()?.resonance_stacks =
        initial_value(config.initial_resources.resonance)?;
    Ok(())
}

pub(crate) fn observe_admission(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    actor_index: usize,
    metadata: &crate::simulation::metadata::ActionMetadata,
    assumptions: &mut std::collections::BTreeMap<String, String>,
    trace: &mut Vec<GameEventReceipt>,
) -> Result<(), String> {
    if metadata.input_role != Some(InputActionRole::Qte)
        || metadata.native_ability_index != Some(4)
        || state.actors[actor_index].hp == 0
    {
        return Ok(());
    }
    for (index, config) in request.actors.iter().enumerate() {
        if config.character_id != "character:1054" || state.actors[index].hp == 0 {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(request, assumptions, HYPOTHESIS)?;
        let owner = state.actors[index].character.daffodil_mut()?;
        let before = owner.resonance_stacks;
        owner.resonance_stacks = before.saturating_add(1).min(CAPACITY);
        trace.push(GameEventReceipt {
            time_us: state.current_time_us,
            phase: "ActionAdmission".into(),
            event_kind: "daffodil_resonance_granted".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(index),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                stacks_before: Some(u32::from(before)),
                stacks: Some(u32::from(owner.resonance_stacks)),
                max_stacks: Some(u32::from(CAPACITY)),
                ..Default::default()
            }),
        });
    }
    Ok(())
}

pub(crate) fn capture(ctx: &mut crate::runtime_api::CastContext<'_>) -> Result<u8, String> {
    ctx.record_hypothesis(HYPOTHESIS)?;
    let owner = ctx.actor.character.daffodil_mut()?;
    let stacks = std::mem::take(&mut owner.resonance_stacks);
    owner.skill_captures.insert(ctx.command_index, stacks);
    Ok(stacks)
}

pub(crate) fn break_efficiency(
    config: &crate::simulation::GameActorConfig,
    actor: &ActorRuntime,
    command_index: usize,
) -> Result<F, String> {
    let stacks = match &actor.character {
        crate::characters::RuntimeState::Daffodil(owner) => owner
            .skill_captures
            .get(&command_index)
            .copied()
            .unwrap_or(0),
        _ => 0,
    };
    (if config.awakening.has_effect(1) {
        PARAMETERS.a1_skill_break_efficiency_per_resonance
    } else {
        PARAMETERS.skill_break_efficiency_per_resonance
    })
    .mul(F::new(u128::from(stacks), 1)?)
}
