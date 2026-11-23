//! ETD-4 entry requires the outgoing owner's actual control state.
use crate::runtime_api::{CastContext, OwnerEntryAction, OwnerEntryAttack};
use crate::simulation::{GameScenarioRequest, state::GameSimulatorState};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::BaicangSkiaEntryV1;
pub(crate) const SKIA_PACKET: &str = "GE_Player_Cang_Skia_Damage";

fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(super) fn has_passive(config: &crate::simulation::GameActorConfig) -> bool {
    ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )
    .ok()
    .and_then(|value| value["breakthrough"].as_u64())
    .is_some_and(|level| level >= 4)
}

pub(crate) fn entry_action(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    old: usize,
    new: usize,
    qualified_support: bool,
) -> Result<Option<OwnerEntryAction>, String> {
    if let Some(entry) = super::entry_jump::select(request, state, old, new, qualified_support)? {
        return Ok(Some(entry));
    }
    if let Some(entry) = super::support::select(request, state, new, qualified_support, false)? {
        return Ok(Some(OwnerEntryAction::Support(entry)));
    }
    let now = state.current_time_us;
    if !has_passive(&request.actors[new])
        || request.actors[old].character_id != "character:1008"
        || !crate::characters::skia::entry_state::is_sneaking(&state.actors[old], now)
    {
        return Ok(None);
    }
    if !accepted(request) {
        return Err(
            "UNSUPPORTED_BAICANG_SKIA_ENTRY: live Skia state38 requires baicang.skia_entry_v1"
                .into(),
        );
    }
    if !super::words::accepted(request) {
        return Err("Baicang Skia entry requires baicang.power_words_v1".into());
    }
    let crate::characters::RuntimeState::Baicang(owner) = &state.actors[new].character else {
        return Err("Baicang entry state missing".into());
    };
    if owner.skia_assist_until_us > now {
        return Ok(None);
    }
    let coordinate =
        crate::simulation::inputs::normal_coordinate(&request.actors[new], "GA_Cang_Melee")?;
    Ok(Some(OwnerEntryAction::Attack(OwnerEntryAttack {
        ability: "GA_Cang_Melee",
        effect: SKIA_PACKET,
        coordinate,
        hit_delay_us: 578_030,
        recovery_us: 399_466,
        model: HYPOTHESIS_ID,
    })))
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(SKIA_PACKET) {
        return Ok(false);
    }
    if !accepted(ctx.request)
        || !super::words::accepted(ctx.request)
        || !has_passive(ctx.config)
        || !matches!(ctx.request.actions.get(ctx.command_index),
            Some(crate::simulation::GameActionCommand::SwitchActor { target_actor_index, .. }) if *target_actor_index == ctx.actor_index)
    {
        return Err("Baicang Skia assist requires an engine-selected ETD-4 entry switch".into());
    }
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let until = ctx
        .time_us
        .checked_add(30_000_000)
        .ok_or("Baicang Skia-assist cooldown overflow")?;
    if ctx.actor.character.baicang_mut()?.skia_assist_until_us > ctx.time_us {
        return Err("Baicang Skia-assist cooldown is active".into());
    }
    super::timing::primary(ctx, 578_030, 399_466)?;
    ctx.scope_primary_aoe(SKIA_PACKET)?;
    for offset in [658_030, 738_030] {
        ctx.schedule_native_aoe_packet(
            SKIA_PACKET,
            ctx.time_us
                .checked_add(offset)
                .ok_or("Baicang Skia pulse overflow")?,
        )?;
    }
    let state = ctx.actor.character.baicang_mut()?;
    state.skia_assist_until_us = until;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.riposte_ready = false;
    Ok(true)
}
