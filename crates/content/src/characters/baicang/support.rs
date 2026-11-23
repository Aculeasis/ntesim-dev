//! Ordinary Support requires the real outgoing full Cycle and supported pair.
use crate::runtime_api::SupportEntry;
use crate::simulation::{GameScenarioRequest, state::GameSimulatorState};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId =
    crate::HypothesisId::BaicangQualifiedSupportV1;
pub(crate) const ENTRY: SupportEntry = SupportEntry {
    ability_id: "GA_Cang_QTE",
    damage_effect_id: "GE_Player_Cang_QTE_Damage",
    hit_delay_us: 1_000_358,
    recovery_us: 1_566_945,
    dispatch_model: HYPOTHESIS_ID,
    packet_model: Some(HYPOTHESIS_ID),
    extra_damage_effects: &[],
};

pub(crate) fn on_cast(ctx: &mut crate::runtime_api::CastContext<'_>) -> Result<(), String> {
    if ctx.ability_id == ENTRY.ability_id
        && matches!(
            ctx.request.actions.get(ctx.command_index),
            Some(crate::simulation::GameActionCommand::SwitchActor { .. })
        )
        && ctx
            .request
            .model_assumptions
            .get(HYPOTHESIS_ID.as_str())
            .map(String::as_str)
            == Some(HYPOTHESIS_ID.description())
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn select(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    new: usize,
    qualified_support: bool,
    required: bool,
) -> Result<Option<&'static SupportEntry>, String> {
    let accepted = request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description());
    if !required && !accepted {
        return Ok(None);
    }
    if state.actors[new].locked_until_us > state.current_time_us || !qualified_support {
        return Ok(None);
    }
    if !accepted || !super::words::accepted(request) {
        return Err("UNSUPPORTED_BAICANG_QUALIFIED_SUPPORT: qualified entry requires baicang.qualified_support_v1 and the Word model".into());
    }
    Ok(Some(&ENTRY))
}
