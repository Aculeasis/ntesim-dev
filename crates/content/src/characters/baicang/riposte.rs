use crate::HypothesisId;
use crate::runtime_api::{CastContext, NormalActionOverride};
use crate::simulation::{metadata::InputActionRole, state::ActorRuntime};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangRiposteV1;
pub(crate) const PACKET_ID: &str = "GE_Player_Cang_PerfectEvadeAttack_Damage";

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character, crate::characters::RuntimeState::Baicang(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: PACKET_ID,
            role: Some(InputActionRole::CriticalRiposte),
            model: HYPOTHESIS_ID,
            timing: Some((1_141_373, 166_667)),
        })
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let packet_id = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();

    let is_riposte_packet = packet_id == PACKET_ID;
    let is_riposte_ready = matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Baicang(state) if state.riposte_ready
    );

    if !is_riposte_packet && !is_riposte_ready {
        return Ok(false);
    }

    ctx.record_hypothesis(HYPOTHESIS_ID)?;

    if is_riposte_ready && !is_riposte_packet {
        ctx.replace_native_primary_packet(PACKET_ID)?;
    }
    if !is_riposte_ready {
        return Err("Baicang Critical Riposte requires a successful Critical Dodge".into());
    }
    let recovery = if super::clock::accepted(ctx.request) {
        844_373
    } else {
        1_141_373
    };
    super::timing::primary(ctx, 166_667, recovery)?;
    ctx.scope_primary_aoe(PACKET_ID)?;

    let state = ctx.actor.character.baicang_mut()?;
    state.riposte_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;

    if super::words::accepted(ctx.request) {
        super::words::generate_word(ctx, super::awakenings::WordType::Bless, 0)?;
    }

    Ok(true)
}
