//! Neutral held Normal captures its row, but reads live Crime at stage1 completion.
use crate::runtime_api::{CallbackContext, CastContext, NormalActionOverride};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::{
    ExactFraction, GameActionCommand, GameHoldInput, GameScenarioRequest, state::ActorRuntime,
};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChaosNeutralVerdictV1;
const ABILITY: &str = "GA_Chaos071_Melee";
const FIRST: &str = "GE_Player_Chaos_Branch1_Damage";
const FIRST_ULTRA: &str = "GE_Player_Chaos_Branch1_Damage_Ultra";
const FULL: &str = "GE_Player_Chaos_Branch2_Damage";
const FULL_ULTRA: &str = "GE_Player_Chaos_Branch2_Damage_Ultra";

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    command: usize,
    transformed: bool,
    coordinate: usize,
}

fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn validate(
    request: &GameScenarioRequest,
    packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
) -> Result<(), String> {
    if accepted(request) && !super::clock::accepted(request) {
        return Err(
            "Chaos neutral Verdict requires ordinary_clock_v1 and ordinary_crime_v1".into(),
        );
    }
    if super::cancel::accepted(request, super::cancel::JUMP) && !super::clock::accepted(request) {
        return Err("Chaos Jump cancel requires ordinary_clock_v1".into());
    }
    if super::cancel::accepted(request, super::cancel::COMBO) && !accepted(request) {
        return Err("Chaos Normal combo Verdict requires neutral_verdict_v1".into());
    }
    if super::cancel::accepted(request, super::cancel::CONTINUATION) && !accepted(request) {
        return Err("Chaos Verdict continuation requires neutral_verdict_v1".into());
    }
    for (index, command) in request.actions.iter().enumerate() {
        let GameActionCommand::InvokeAbility {
            actor_index,
            timestamp_us,
            ..
        } = command
        else {
            continue;
        };
        if request.actors[*actor_index].character_id != "character:1071" {
            continue;
        }
        let Some(packet) = &packets[index] else {
            continue;
        };
        if matches!(
            packet.packet.damage_effect_id.as_str(),
            FIRST | FIRST_ULTRA | FULL | FULL_ULTRA
        ) {
            return Err("Chaos Verdict requires a held canonical Normal input, not a caller-selected branch row".into());
        }
        let Some(input) = request.hold_inputs.get(&index) else {
            continue;
        };
        if packet.action.ability_id != ABILITY
            || packet.packet.damage_effect_id != "GE_Player_Chaos_Melee1_Damage"
            || request.plunge_inputs.contains_key(&index)
            || input.moving
        {
            return Err(
                "Chaos neutral held input requires stationary canonical Normal without plunge"
                    .into(),
            );
        }
        if input.duration_us != 0
            && (!accepted(request)
                || (input.duration_us < 1_000_000
                    && !((super::cancel::accepted(request, super::cancel::COMBO)
                        && input.duration_us >= 200_000)
                        || (super::cancel::accepted(request, super::cancel::CONTINUATION)
                            && input.duration_us >= 100_000))))
        {
            return Err("Chaos held Verdict requires neutral_verdict_v1 and at least1s; short combo holds require a matching owner binding and duration threshold".into());
        }
        timestamp_us
            .checked_add(input.duration_us)
            .and_then(|time| time.checked_add(2_100_000))
            .ok_or("Chaos Verdict deadline overflow")?;
    }
    Ok(())
}

pub(crate) fn replacement(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: GameHoldInput,
    now: u64,
) -> Result<Option<NormalActionOverride>, String> {
    if input.duration_us == 0 {
        return Ok(None);
    }
    let combo = super::cancel::combo(request, actor, now, input.duration_us)?;
    let lead = if combo.is_some() {
        0
    } else {
        input.duration_us
    };
    if !accepted(request) || (combo.is_none() && input.duration_us < 1_000_000) {
        return Err(
            "Chaos neutral Verdict requires accepted model and at least1s held lead-in".into(),
        );
    }
    let transformed = super::form::active(actor);
    Ok(Some(NormalActionOverride {
        packet: if transformed { FIRST_ULTRA } else { FIRST },
        role: None,
        model: MODEL,
        timing: Some((
            lead.checked_add(if transformed { 899_451 } else { 898_334 })
                .ok_or("Chaos hold recovery overflow")?,
            lead.checked_add(if transformed { 167_196 } else { 166_691 })
                .ok_or("Chaos hold contact overflow")?,
        )),
    }))
}

pub(super) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let Some(packet) = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
    else {
        return Ok(false);
    };
    if !matches!(packet, FIRST | FIRST_ULTRA) {
        return Ok(false);
    }
    if !accepted(ctx.request) {
        return Err("Chaos neutral Verdict model missing".into());
    }
    let input = ctx
        .request
        .hold_inputs
        .get(&ctx.command_index)
        .ok_or("Chaos Verdict hold missing")?;
    let transformed = packet == FIRST_ULTRA;
    let coordinate = crate::simulation::inputs::normal_coordinate(ctx.config, ABILITY)?;
    ctx.record_hypothesis(MODEL)?;
    ctx.scope_primary_aoe(packet)?;
    ctx.schedule_native_aoe_packet(packet, ctx.hit_time_us)?;
    let state = ctx.actor.character.chaos_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.entry_normal_ready = false;
    state.verdict = Some(Pending {
        command: ctx.command_index,
        transformed,
        coordinate,
    });
    let lead = if super::cancel::combo(ctx.request, ctx.actor, ctx.time_us, input.duration_us)?
        .is_some()
    {
        0
    } else {
        input.duration_us
    };
    super::clock::verdict::start(ctx, transformed, lead)?;
    Ok(true)
}

pub(super) fn prepare_read(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    progress: u64,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0
        || !ctx.state.actors[owner]
            .character
            .chaos_mut()?
            .verdict
            .as_ref()
            .is_some_and(|pending| pending.command == command)
    {
        return Ok(());
    }
    crate::runtime_api::defer_current_admissions(ctx.queue, ctx.state.current_time_us);
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.state.current_time_us,
        priority: ntesim_core::execution::CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Chaos(
            super::events::Event::VerdictCommit {
                owner,
                command,
                progress,
            },
        )),
    });
    Ok(())
}

pub(super) fn read(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let state = ctx.state.actors[owner].character.chaos_mut()?;
    let Some(pending) = state
        .verdict
        .as_ref()
        .filter(|pending| pending.command == command)
        .cloned()
    else {
        return Ok(());
    };
    let full = state
        .crime
        .as_ref()
        .ok_or("Chaos Verdict Crime missing")?
        .current()
        == ExactFraction::new(1000, 1)?;
    if full {
        launch(ctx, owner, pending)?;
    }
    ctx.state.actors[owner].character.chaos_mut()?.verdict = None;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "chaos_verdict_read".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            full: Some(full),
            ..Default::default()
        }),
    });
    Ok(())
}

fn launch(ctx: &mut CallbackContext<'_>, owner: usize, pending: Pending) -> Result<(), String> {
    let effect = if pending.transformed {
        FULL_ULTRA
    } else {
        FULL
    };
    let config = &ctx.request.actors[owner];
    let (atk_rate, scaling_stat) = crate::simulation::inputs::coefficient(
        ctx.request,
        config,
        ABILITY,
        effect,
        pending.coordinate,
    )?;
    let base = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        ABILITY,
        effect,
        None,
        &ctx.request.model_assumptions,
    )?;
    let offsets = [
        65_847,
        if pending.transformed {
            431_182
        } else {
            466_857
        },
    ];
    let mut groups = BTreeMap::new();
    let mut events = Vec::new();
    for offset in offsets {
        let time_us = ctx
            .state
            .current_time_us
            .checked_add(offset)
            .ok_or("Chaos full Verdict contact overflow")?;
        let group = crate::runtime_api::next_packet_group(owner, &mut ctx.state.actors[owner])?;
        groups.insert(group.clone(), offset);
        let mut metadata = base.clone();
        metadata.binding.energy_group_id = Some(group);
        for target_index in 0..ctx.state.targets.len() {
            if ctx.state.targets[target_index].hp_current == 0 {
                continue;
            }
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            events.push(ScheduledGameEvent {
                time_us,
                priority: ntesim_core::execution::CausalPriority::HitDelivery,
                seq: *ctx.sequence,
                data: GameEventData::HitDelivery {
                    command_index: pending.command,
                    actor_index: owner,
                    target_index,
                    ability_id: ABILITY.into(),
                    damage_effect_id: effect.into(),
                    atk_rate,
                    scaling_stat,
                    metadata: Box::new(metadata.clone()),
                },
            });
        }
    }
    let pool = ctx.state.actors[owner]
        .character
        .chaos_mut()?
        .crime
        .as_mut()
        .ok_or("Chaos full Verdict Crime missing")?;
    pool.spend_up_to(ExactFraction::new(1000, 1)?)?;
    ctx.queue.extend(events);
    super::clock::verdict::launch(ctx, owner, pending.command, pending.transformed, groups)
}

pub(crate) fn recovery_override(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    command: usize,
    now: u64,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    let Some(input) = request.hold_inputs.get(&command) else {
        return Ok(None);
    };
    if input.duration_us == 0 {
        return Ok(None);
    }
    super::cancel::combo(request, actor, now, input.duration_us)
}
