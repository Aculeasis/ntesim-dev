//! Jump cancels and Normal-to-Verdict holds use source progress, without retries.
use crate::simulation::{GameScenarioRequest, state::ActorRuntime};
pub(crate) const JUMP: crate::HypothesisId = crate::HypothesisId::ChaosJumpCancelV1;
pub(crate) const COMBO: crate::HypothesisId = crate::HypothesisId::ChaosNormalComboVerdictV1;
pub(crate) const CONTINUATION: crate::HypothesisId =
    crate::HypothesisId::ChaosVerdictContinuationV1;

pub(crate) fn accepted(request: &GameScenarioRequest, model: crate::HypothesisId) -> bool {
    request
        .model_assumptions
        .get(model.as_str())
        .map(String::as_str)
        == Some(model.description())
}

pub(crate) fn recovery(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    use super::clock::cancel::Family;
    let (family, model) = match ability {
        "jump" if accepted(request, JUMP) => (Family::Jump, JUMP),
        "dash" if super::secondary::accepted(request) => {
            (Family::Movement, super::secondary::MODEL)
        }
        "GA_Chaos071_Skill" | "GA_Chaos071_UltraSkill" if super::secondary::accepted(request) => {
            (Family::Skill, super::secondary::MODEL)
        }
        _ => return Ok(None),
    };
    Ok(
        super::clock::cancel::gate(actor, now, full_lifetime(request), family)?
            .map(|deadline| (deadline, model)),
    )
}

pub(crate) fn full_lifetime(request: &GameScenarioRequest) -> bool {
    accepted(request, CONTINUATION) || super::secondary::accepted(request)
}

/// Earliest source yield for the minimum hold of the actual native producer.
pub(crate) fn input_window(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    now: u64,
) -> Result<u64, String> {
    super::clock::cancel::input_window(
        actor,
        now,
        accepted(request, COMBO),
        accepted(request, CONTINUATION),
    )
}

pub(crate) fn combo(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    now: u64,
    hold: u64,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    let normal = accepted(request, COMBO);
    let continuation = accepted(request, CONTINUATION);
    if !normal && !continuation {
        return Ok(None);
    }
    super::clock::cancel::combo(actor, now, hold, normal, continuation)
}

pub(crate) fn on_jump(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    super::clock::exit(ctx, owner)?;
    let state = ctx.state.actors[owner].character.chaos_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    Ok(())
}
