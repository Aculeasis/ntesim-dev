//! World-time Dread Echo and finite active-owner Crime ticks; no off-field banking.
use super::events::Event;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{ExactFraction, GameScenarioRequest, state::ActorRuntime};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChaosPrimaryFormV1;
const ENTRY_US: u64 = 2_718_888;
const INTERVAL_US: u64 = 200_000;
const TICKS: u8 = 60;
pub(super) const SWITCH_LOCK_US: u64 = 4_944_578;
pub(super) const IMMUNITY_END_US: u64 = 4_802_493;

#[derive(Clone, Debug)]
pub(crate) struct Form {
    started_us: u64,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(super) fn active(actor: &ActorRuntime) -> bool {
    matches!(&actor.character, crate::characters::RuntimeState::Chaos(state) if state.form.is_some())
}

pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(request) {
        return Ok(());
    }
    if !super::clock::accepted(request) || !super::resources::accepted(request) {
        return Err("Chaos primary form requires ordinary_clock_v1 and ordinary_crime_v1".into());
    }
    if request
        .actors
        .iter()
        .any(|actor| actor.character_id == "character:1071" && actor.energy_capacity != 100)
    {
        return Err("Chaos primary form requires native100 Ultimate energy capacity".into());
    }
    for command in &request.actions {
        if let crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            energy_cost,
            timestamp_us,
            ..
        } = command
            && request.actors[*actor_index].character_id == "character:1071"
            && ability_id == "GA_Chaos071_UltraSkill"
        {
            if *energy_cost != 100 {
                return Err("Chaos Burst requires its native100 Ultimate energy cost".into());
            }
            timestamp_us
                .checked_add(ENTRY_US + u64::from(TICKS) * INTERVAL_US)
                .ok_or("Chaos form deadline overflow")?;
        }
    }
    Ok(())
}

pub(super) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.ability_id != "GA_Chaos071_UltraSkill" || !accepted(ctx.request) {
        return Ok(());
    }
    ctx.record_hypothesis(MODEL)?;
    let state = ctx.actor.character.chaos_mut()?;
    let generation = state
        .form_generation
        .checked_add(1)
        .ok_or("Chaos form generation overflow")?;
    state.form_generation = generation;
    state.form = None;
    if ctx.config.awakening.has_effect(6) {
        let pool = state.crime.as_mut().ok_or("Chaos A6 Crime pool missing")?;
        pool.grant(ExactFraction::new(1000, 1)?.sub(pool.current())?)?;
    }
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(ENTRY_US)
                .ok_or("Chaos entry overflow")?,
            priority: ntesim_core::execution::CausalPriority::ModePause,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::Chaos(Event::EnterForm {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    generation,
                    progress: ENTRY_US,
                }),
            ),
        });
    Ok(())
}

fn schedule(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
    tick: u8,
    start: u64,
) -> Result<(), String> {
    let time_us = start
        .checked_add(u64::from(tick) * INTERVAL_US)
        .ok_or("Chaos auto Crime deadline overflow")?;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us,
            priority: ntesim_core::execution::CausalPriority::ModePause,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::Chaos(Event::AutoCrime {
                    owner,
                    generation,
                    tick,
                }),
            ),
        });
    Ok(())
}

pub(super) fn enter(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let actor = &mut ctx.state.actors[owner];
    if actor.hp == 0 {
        return Ok(());
    }
    let state = actor.character.chaos_mut()?;
    if state.form_generation != generation {
        return Ok(());
    }
    let start = ctx.state.current_time_us;
    state.form = Some(Form { started_us: start });
    state.normal_stage = 0;
    state.normal_started_us = None;
    schedule(ctx, owner, generation, 1, start)?;
    trace(ctx, "chaos_form_entry", owner, None);
    Ok(())
}

pub(super) fn tick(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
    tick: u8,
) -> Result<(), String> {
    let actor = &mut ctx.state.actors[owner];
    if actor.hp == 0 {
        return Ok(());
    }
    let state = actor.character.chaos_mut()?;
    if state.form_generation != generation {
        return Ok(());
    }
    let Some(form) = &state.form else {
        return Ok(());
    };
    let start = form.started_us;
    if ctx.state.active_actor_index == owner {
        let (gain, overflow) = state
            .crime
            .as_mut()
            .ok_or("Chaos auto Crime pool missing")?
            .grant(ExactFraction::new(30, 1)?)?;
        trace(
            ctx,
            "chaos_auto_crime",
            owner,
            Some(crate::simulation::GameEventPayload {
                ordinal: Some(tick),
                exact_amount: Some(gain),
                exact_overflow: Some(overflow),
                ..Default::default()
            }),
        );
    }
    if tick == TICKS {
        end(&mut ctx.state.actors[owner]);
        trace(ctx, "chaos_form_end", owner, None);
    } else {
        schedule(ctx, owner, generation, tick + 1, start)?;
    }
    Ok(())
}

pub(super) fn end(actor: &mut ActorRuntime) {
    if let crate::characters::RuntimeState::Chaos(state) = &mut actor.character {
        state.form = None;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
}

pub(crate) fn defeat(actor: &mut ActorRuntime) {
    end(actor);
    if let crate::characters::RuntimeState::Chaos(state) = &mut actor.character {
        state.burst_immunity_until_us = 0;
        state.verdict_immunity_until_us = 0;
        state.verdict = None;
        state.entry_normal_ready = false;
        state.riposte_ready = false;
        state.remoras.clear();
        state.remora_bonuses.clear();
        if let Some(clock) = &mut state.clock {
            clock.end_action();
        }
    }
}

fn trace(
    ctx: &mut CallbackContext<'_>,
    kind: &str,
    owner: usize,
    data: Option<crate::simulation::GameEventPayload>,
) {
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ModePause".into(),
        event_kind: kind.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data,
    });
}

pub(crate) fn switch_lock(
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    let crate::characters::RuntimeState::Chaos(state) = &actor.character else {
        return None;
    };
    (ability == "switch_actor" && state.burst_switch_until_us > now).then_some((
        actor.locked_until_us.max(state.burst_switch_until_us),
        MODEL,
    ))
}

pub(crate) fn immune(actor: &ActorRuntime, now: u64) -> bool {
    matches!(&actor.character, crate::characters::RuntimeState::Chaos(state) if state.burst_immunity_until_us > now || state.verdict_immunity_until_us > now)
}

impl Form {
    pub(crate) fn pause_world(&mut self, delta: u64) -> Result<(), String> {
        crate::simulation::timer_state::anchor(&mut self.started_us, delta)
    }
}
