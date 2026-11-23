//! One qualified engine entry; its completed montage authorizes one authored Appear Normal.
use crate::runtime_api::{CallbackContext, CastContext, SupportEntry};
use crate::simulation::{GameScenarioRequest, state::GameSimulatorState};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChaosQualifiedSupportV1;
pub(crate) const ABILITY: &str = "GA_Chaos071_QTE";
pub(crate) const PACKET: &str = "GE_Player_Chaos_QTE1_Damage";
pub(crate) const RECOVERY: u64 = 1_376_352;
pub(crate) const ENTRY: SupportEntry = SupportEntry {
    ability_id: ABILITY,
    damage_effect_id: PACKET,
    hit_delay_us: 704_643,
    recovery_us: RECOVERY,
    dispatch_model: MODEL,
    packet_model: Some(MODEL),
    extra_damage_effects: &[],
};
pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(request) {
        return Ok(());
    }
    if !super::clock::accepted(request) {
        return Err("Chaos qualified Support requires ordinary_clock_v1".into());
    }
    for command in &request.actions {
        if let crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            ..
        } = command
            && request.actors[*actor_index].character_id == "character:1071"
            && ability_id == ABILITY
        {
            return Err(
                "Chaos QTE is engine-owned qualified entry, not an authored ability input".into(),
            );
        }
    }
    Ok(())
}
pub(crate) fn select(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    new: usize,
    qualified_support: bool,
) -> Result<Option<&'static SupportEntry>, String> {
    if !accepted(request)
        || state.actors[new].locked_until_us > state.current_time_us
        || !qualified_support
    {
        return Ok(None);
    }
    Ok(Some(&ENTRY))
}
pub(super) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if ctx.ability_id != ABILITY || !accepted(ctx.request) {
        return Ok(false);
    }
    if !matches!(ctx.request.actions.get(ctx.command_index),Some(crate::simulation::GameActionCommand::SwitchActor{target_actor_index,..}) if *target_actor_index==ctx.actor_index)
    {
        return Err("Chaos Support requires its actual qualified switch command".into());
    }
    ctx.record_hypothesis(MODEL)?;
    ctx.scope_primary_aoe(PACKET)?;
    let state = ctx.actor.character.chaos_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.entry_normal_ready = false;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(RECOVERY)
                .ok_or("Chaos Support completion overflow")?,
            priority: ntesim_core::execution::CausalPriority::ModePause,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::Chaos(super::events::Event::SupportFinish {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    progress: RECOVERY,
                }),
            ),
        });
    Ok(true)
}
pub(super) fn finish(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp > 0 && ctx.state.active_actor_index == owner {
        ctx.state.actors[owner]
            .character
            .chaos_mut()?
            .entry_normal_ready = true;
    }
    Ok(())
}
