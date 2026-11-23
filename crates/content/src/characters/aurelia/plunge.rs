use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaPlungeV1;
pub(crate) const PACKET: &str = "GE_Player_Mitsuki_AirAttack_Damage";

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if !accepted(ctx.request) {
        return Ok(false);
    }
    if ctx
        .packet_metadata
        .is_none_or(|packet| packet.damage_effect_id != PACKET)
    {
        return Ok(false);
    }
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let state = ctx.actor.character.aurelia_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    ctx.scope_primary_aoe(PACKET)?;
    Ok(true)
}
