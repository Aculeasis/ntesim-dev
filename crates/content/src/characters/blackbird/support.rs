//! Qualified entry owns QTE; its B4 cue is independent from damage connection.
use crate::runtime_api::{CastContext, SupportEntry};
use crate::simulation::{
    GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::{ActorRuntime, GameSimulatorState},
};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::BlackbirdQualifiedSupportV1;
pub(crate) const EVOLVED_MODEL: crate::HypothesisId =
    crate::HypothesisId::BlackbirdEvolvedSupportV1;
pub(crate) const ABILITY: &str = "GA_BlackBird_QTE";
pub(crate) const PACKET: &str = "GE_Player_BlackBird_QTE_Damage";
pub(crate) const EVOLVED_ABILITY: &str = "GA_BlackBird_Evolve_QTE";
pub(crate) const EVOLVED_PACKET: &str = "GE_Player_BlackBird_Evolve_QTE_Damage";
pub(crate) const EVOLVED_EXTRA_PACKET: &str = "GE_Player_BlackBird_Evolve_QTE_Extra_Damage";
const MALICE_MARKER_US: u64 = 1_002_488;
pub(crate) const ENTRY: SupportEntry = SupportEntry {
    ability_id: ABILITY,
    damage_effect_id: PACKET,
    hit_delay_us: 1_000_222,
    recovery_us: 1_333_181,
    dispatch_model: MODEL,
    packet_model: Some(MODEL),
    extra_damage_effects: &[],
};
pub(crate) const EVOLVED_ENTRY: SupportEntry = SupportEntry {
    ability_id: EVOLVED_ABILITY,
    damage_effect_id: EVOLVED_PACKET,
    hit_delay_us: 878_181,
    recovery_us: 1_167_929,
    dispatch_model: EVOLVED_MODEL,
    packet_model: Some(EVOLVED_MODEL),
    extra_damage_effects: &[],
};
pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn evolved_accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(EVOLVED_MODEL.as_str())
        .map(String::as_str)
        == Some(EVOLVED_MODEL.description())
}
pub(crate) fn select(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    new: usize,
    qualified_support: bool,
) -> Result<Option<&'static SupportEntry>, String> {
    if !accepted(request) {
        return Ok(None);
    }
    if state.actors[new].locked_until_us > state.current_time_us || !qualified_support {
        return Ok(None);
    }
    if !super::clock::accepted(request) || !super::malice::accepted(request) {
        return Err(
            "Blackbird qualified Support requires ordinary clock and exact Malice models".into(),
        );
    }
    if super::form::active(&state.actors[new]) {
        if !evolved_accepted(request) {
            return Err("UNSUPPORTED_BLACKBIRD_EVOLVED_SUPPORT: qualified entry requires the transformed QTE binding".into());
        }
        return Ok(Some(&EVOLVED_ENTRY));
    }
    if !accepted(request) {
        return Ok(None);
    }
    Ok(Some(&ENTRY))
}
pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let evolved = ctx.ability_id == EVOLVED_ABILITY;
    if ctx.ability_id != ABILITY && !evolved {
        return Ok(());
    }
    let (model, packet, accepted) = if evolved {
        (EVOLVED_MODEL, EVOLVED_PACKET, evolved_accepted(ctx.request))
    } else {
        (MODEL, PACKET, accepted(ctx.request))
    };
    if !accepted {
        return Ok(());
    }
    if !matches!(
        ctx.request.actions.get(ctx.command_index),
        Some(crate::simulation::GameActionCommand::SwitchActor { .. })
    ) {
        return Err(
            "Blackbird QTE is an engine-owned qualified entry, not an authored ability input"
                .into(),
        );
    }
    if evolved && !super::form::active(ctx.actor) {
        return Err("Blackbird evolved QTE requires active transformation form".into());
    }
    ctx.record_hypothesis(model)?;
    ctx.scope_primary_aoe(packet)?;
    if evolved {
        return Ok(());
    }
    let progression = ntesim_data::skill_levels::progression(
        &ctx.config.character_id,
        ctx.config.character_level,
        ctx.config.character_max_level,
    )?;
    if progression["breakthrough"]
        .as_u64()
        .ok_or("Blackbird breakthrough selection missing")?
        >= 4
    {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(MALICE_MARKER_US)
                .ok_or("Blackbird QTE marker overflow")?,
            priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::Blackbird(
                super::events::Event::SupportMalice {
                    actor_index: ctx.actor_index,
                    command_index: ctx.command_index,
                    progress_us: MALICE_MARKER_US,
                },
            )),
        });
    }
    Ok(())
}
pub(crate) fn switch_lock(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if ability != "switch_actor" {
        return None;
    }
    let model = if accepted(request) {
        MODEL
    } else if evolved_accepted(request) {
        EVOLVED_MODEL
    } else {
        return None;
    };
    let crate::characters::RuntimeState::Blackbird(state) = &actor.character else {
        return None;
    };
    (state.qte_switch_until_us > now)
        .then_some((actor.locked_until_us.max(state.qte_switch_until_us), model))
}
pub(crate) fn energy(
    ability: &str,
    packet: &str,
) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
    if ability == ABILITY && packet == PACKET {
        return Some(crate::runtime_api::DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: MODEL,
            event_kind: "blackbird_support_energy_grant",
        });
    }
    (ability == EVOLVED_ABILITY && packet == EVOLVED_PACKET).then_some(
        crate::runtime_api::DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: EVOLVED_MODEL,
            event_kind: "blackbird_evolved_support_energy_grant",
        },
    )
}
