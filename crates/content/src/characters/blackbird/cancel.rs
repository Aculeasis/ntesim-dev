//! Documented earliest owner cancel gates, applied before full montage recovery.
use crate::simulation::{GameScenarioRequest, state::ActorRuntime};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::BlackbirdCancelGatesV1;

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
        && super::clock::accepted(request)
        && super::malice::accepted(request)
}

fn family(ability: &str) -> Option<super::clock::Cancel> {
    match ability {
        "jump" => Some(super::clock::Cancel::Jump),
        "dash" => Some(super::clock::Cancel::Movement),
        "GA_BlackBird_Skill" | "GA_BlackBird_Evolve_Skill" => Some(super::clock::Cancel::Skill),
        _ => None,
    }
}

pub(crate) fn recovery_override(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    if !accepted(request) {
        return Ok(None);
    }
    let Some(family) = family(ability) else {
        return Ok(None);
    };
    Ok(super::clock::gate(actor, now, family)?.map(|deadline| (deadline, MODEL)))
}
