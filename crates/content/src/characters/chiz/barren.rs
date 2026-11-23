//! One captured lock, explicit secondary exclusion and independent tap limiter.
use super::{clock::Kind, source::Source};
use crate::runtime_api::{CallbackContext, CastContext, NormalActionOverride};
use crate::simulation::{
    GameActionCommand, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use std::collections::BTreeMap;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizBarrenV1;
const NORMAL: &str = "GA_Chiichan073_Melee";
pub(crate) const PRIMARY: &str = "GE_Player_Chiichan073_Branch2_Damage";
pub(crate) const SECONDARY: &str = "GE_Player_Chiichan073_Branch2_1_Damage";
#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    tap_until: u64,
    inputs: BTreeMap<usize, Input>,
}
#[derive(Clone, Debug)]
struct Input {
    source: Source,
    locked: usize,
    first_group: Option<String>,
    secondary_group: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(crate) owner: usize,
    pub(crate) command: usize,
    pub(crate) progress: u64,
    start: bool,
}
pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
fn selected(request: &GameScenarioRequest, command: usize) -> bool {
    request.hold_inputs.contains_key(&command)
        && matches!(request.actions.get(command),Some(GameActionCommand::InvokeAbility{ability_id,..}) if ability_id==NORMAL)
}
pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if accepted(request) && !super::secondary::accepted(request) {
        return Err("Chiz Barren requires secondary_actions_v1".into());
    }
    for (command, action) in request.actions.iter().enumerate() {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            damage_effect_id,
            ..
        } = action
            && request.actors[*actor_index].character_id == "character:1073"
        {
            if matches!(damage_effect_id.as_str(), PRIMARY | SECONDARY) {
                return Err("Chiz Barren uses canonical Normal plus explicit hold input; raw child rows cannot bypass selection".into());
            }
            if selected(request, command) {
                if !accepted(request) {
                    return Err("Chiz Normal hold/tap requires barren_v1".into());
                }
                let input = request.hold_inputs[&command];
                if input.reserved_stocks.is_some()
                    || input.moving
                    || request.plunge_inputs.contains_key(&command)
                {
                    return Err("Chiz Barren input has no stocks, moving or airborne branch".into());
                }
            }
        }
    }
    Ok(())
}
pub(crate) fn replacement(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    command: usize,
    now: u64,
) -> Result<Option<NormalActionOverride>, String> {
    if !selected(request, command) {
        return Ok(None);
    }
    if !accepted(request) {
        return Err("Chiz Barren requires barren_v1".into());
    }
    let held = request.hold_inputs[&command].duration_us > 0;
    if held {
        let gate = super::clock::normal_gate(actor)
            .ok_or("Chiz combo-held Barren requires a preceding Normal")?;
        if now > gate.saturating_add(2_000_000) {
            return Err("Chiz combo-held Barren requires a live Normal continuation".into());
        }
    }
    Ok(Some(NormalActionOverride {
        packet: PRIMARY,
        role: None,
        model: MODEL,
        timing: Some((
            if held { 1_912_478 } else { 2_412_493 },
            if held { 749_310 } else { 1_249_325 },
        )),
    }))
}
pub(crate) fn recovery(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    command: usize,
) -> Option<(u64, crate::HypothesisId)> {
    (selected(request, command) && request.hold_inputs[&command].duration_us > 0)
        .then(|| super::clock::normal_gate(actor))
        .flatten()
        .map(|gate| (gate, MODEL))
}
pub(crate) fn failure(
    request: &GameScenarioRequest,
    command: usize,
    actor: &ActorRuntime,
    ability: &str,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if ability != NORMAL
        || !selected(request, command)
        || request.hold_inputs[&command].duration_us > 0
    {
        return None;
    }
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return None;
    };
    let now = request.actions[command].timestamp_us();
    (now < state.barren.tap_until).then(|| crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Cooldown {
            until_us: state.barren.tap_until,
        },
        message: format!(
            "Chiz Barren tap limiter remains until{}",
            state.barren.tap_until
        ),
    })
}
fn enqueue(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    time: u64,
    event: Event,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: time,
        priority: if event.start {
            crate::execution::CausalPriority::ProcCallback
        } else {
            crate::execution::CausalPriority::ModePause
        },
        seq: *sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
            super::events::Event::Barren(event),
        )),
    });
    Ok(())
}
pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != Some(PRIMARY)
    {
        return Ok(false);
    }
    ctx.record_hypothesis(MODEL)?;
    let source = Source::capture(ctx)?;
    let held = ctx
        .request
        .hold_inputs
        .get(&ctx.command_index)
        .ok_or("Chiz Barren requires authored Normal hold/tap intent")?
        .duration_us
        > 0;
    ctx.cancel_primary_delivery()?;
    let state = ctx.actor.character.chiz_mut()?;
    state.riposte_ready = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    if !held {
        state.barren.tap_until = ctx
            .time_us
            .checked_add(2_500_000)
            .ok_or("Chiz Barren limiter overflow")?;
    }
    state.barren.inputs.insert(
        ctx.command_index,
        Input {
            source,
            locked: ctx.target_index,
            first_group: None,
            secondary_group: None,
        },
    );
    super::pressed::reset(ctx.actor)?;
    if held {
        emit(ctx)?;
    } else {
        enqueue(
            ctx.queue,
            ctx.sequence,
            ctx.time_us
                .checked_add(500_015)
                .ok_or("Chiz Barren cue overflow")?,
            Event {
                owner: ctx.actor_index,
                command: ctx.command_index,
                progress: 500_015,
                start: true,
            },
        )?;
        super::clock::start(ctx, Kind::BarrenCue, 0, BTreeMap::new())?;
    }
    Ok(true)
}
fn emit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    super::clock::stop(ctx.actor, ctx.queue, ctx.time_us)?;
    let first = ctx.schedule_native_aoe_group(PRIMARY, ctx.time_us)?;
    let secondary = ctx.schedule_native_aoe_group(SECONDARY, ctx.time_us)?;
    let final_group = ctx.schedule_native_aoe_group(SECONDARY, ctx.time_us)?;
    let input = ctx
        .actor
        .character
        .chiz_mut()?
        .barren
        .inputs
        .get_mut(&ctx.command_index)
        .ok_or("Chiz Barren source missing")?;
    input.first_group = Some(first.clone());
    input.secondary_group = Some(secondary.clone());
    enqueue(
        ctx.queue,
        ctx.sequence,
        ctx.time_us
            .checked_add(749_310)
            .ok_or("Chiz Barren lock overflow")?,
        Event {
            owner: ctx.actor_index,
            command: ctx.command_index,
            progress: 749_310,
            start: false,
        },
    )?;
    super::clock::start(
        ctx,
        Kind::Barren,
        0,
        BTreeMap::from([
            (first, (749_310, 0)),
            (secondary, (750_524, 0)),
            (final_group, (1_464_538, 0)),
        ]),
    )
}
pub(crate) fn apply(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    if ctx.state.actors[event.owner].hp == 0 || ctx.state.active_actor_index != event.owner {
        return Ok(());
    }
    let input = ctx.state.actors[event.owner]
        .character
        .chiz_mut()?
        .barren
        .inputs
        .get(&event.command)
        .cloned()
        .ok_or("Chiz Barren callback source missing")?;
    if event.start {
        return input.source.with_cast(ctx, event.owner, emit);
    }
    let locked = ctx
        .state
        .targets
        .get(input.locked)
        .filter(|target| target.hp_current > 0)
        .map(|_| input.locked)
        .or_else(|| {
            ctx.state
                .targets
                .iter()
                .position(|target| target.hp_current > 0)
        });
    ctx.queue.retain(|queued| {
        if let GameEventData::HitDelivery {
            metadata,
            target_index,
            ..
        } = &queued.data
        {
            if metadata.binding.energy_group_id.as_ref() == input.first_group.as_ref() {
                return locked == Some(*target_index);
            }
            if metadata.binding.energy_group_id.as_ref() == input.secondary_group.as_ref() {
                return locked != Some(*target_index);
            }
        }
        true
    });
    Ok(())
}
