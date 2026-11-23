//! Burst commitment and the World-time Emergency Delivery snapshot.
use super::parameters::PARAMETERS;
use crate::runtime_api::CallbackContext;
use crate::simulation::{ExactFraction as F, GameScenarioRequest, state::ActorRuntime};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorBurstEmergencyV1;
pub(crate) const HYBRID: crate::HypothesisId = crate::HypothesisId::HathorHybridEmergencyV1;
pub(super) const RECOVERY: u64 = 4_530_561;
#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    generation: u64,
    delivery: Option<Delivery>,
}
#[derive(Clone, Debug)]
struct Delivery {
    generation: u64,
    until: u64,
    atk_until: u64,
    counter: u8,
    hidden_cooldown: Option<Option<u64>>,
    atk_percent: F,
    hybrid_burst_recovery: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Expiry {
    owner: usize,
    generation: u64,
    atk_only: bool,
}
fn selected(req: &GameScenarioRequest, model: crate::HypothesisId) -> bool {
    req.model_assumptions
        .get(model.as_str())
        .map(String::as_str)
        == Some(model.description())
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    selected(req, MODEL)
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if selected(req, HYBRID) && !accepted(req) {
        return Err("Hathor hybrid_emergency_v1 requires burst_emergency_v1".into());
    }
    if accepted(req) && !super::power::accepted(req) {
        return Err("Hathor burst_emergency_v1 requires tap_power_v1".into());
    }
    if accepted(req) {
        for action in &req.actions {
            if let crate::simulation::GameActionCommand::InvokeAbility {
                timestamp_us,
                actor_index,
                ability_id,
                ..
            } = action
                && req.actors[*actor_index].character_id == "character:1025"
                && ability_id == "GA_Hathor_UltraSkill"
            {
                timestamp_us
                    .checked_add(15_000_000)
                    .ok_or("Hathor Burst cooldown deadline overflow")?;
            }
        }
    }
    Ok(())
}
pub(crate) fn active(actor: &ActorRuntime, now: u64) -> bool {
    match &actor.character {
        crate::characters::RuntimeState::Hathor(s) => state_active(s, now),
        _ => false,
    }
}
pub(crate) fn state_active(s: &super::awakenings::State, now: u64) -> bool {
    s.burst
        .as_ref()
        .map_or(s.emergency_delivery_until_us > now, |b| {
            b.delivery.as_ref().is_some_and(|d| now < d.until)
        })
}
pub(crate) fn atk_percent(actor: &ActorRuntime, now: u64) -> Result<F, String> {
    if let crate::characters::RuntimeState::Hathor(s) = &actor.character
        && let Some(delivery) = s
            .burst
            .as_ref()
            .and_then(|b| b.delivery.as_ref())
            .filter(|d| now < d.atk_until)
    {
        return Ok(delivery.atk_percent);
    }
    F::new(0, 1)
}
fn schedule_kind(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    atk_only: bool,
) -> Result<(), String> {
    let d = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .burst
        .as_ref()
        .and_then(|b| b.delivery.as_ref())
        .ok_or("Hathor Emergency timer missing")?;
    let deadline = if atk_only { d.atk_until } else { d.until };
    if deadline <= ctx.request.absolute_limit_us {
        let event = Expiry {
            owner,
            generation: d.generation,
            atk_only,
        };
        let time = deadline;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: time,
                priority: if !atk_only && super::cyclone::accepted(ctx.request) {
                    ntesim_core::execution::CausalPriority::ModePause
                } else {
                    ntesim_core::execution::CausalPriority::BuffTickOrExpiry
                },
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::HathorEmergency(event),
                ),
            });
    }
    Ok(())
}
pub(crate) fn enter(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let hybrid = selected(ctx.request, HYBRID);
    let recovery = ctx.state.actors[owner].locked_until_us;
    let shared_release =
        crate::simulation::pause::cinematic_release_deadline(ctx.state, owner)?.unwrap_or(now);
    let power = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .express_delivery_power;
    let hidden_cooldown = super::cyclone::accepted(ctx.request).then(|| {
        ctx.state.actors[owner]
            .ability_cooldowns
            .remove("GA_Hathor_Skill")
    });
    let b = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .burst
        .as_mut()
        .ok_or("Hathor Burst binding missing")?;
    b.generation = b
        .generation
        .checked_add(1)
        .ok_or("Hathor Emergency generation overflow")?;
    let until = if hybrid {
        now.checked_add(5_000_000)
            .and_then(|until| until.checked_add(recovery.saturating_sub(shared_release.max(now))))
    } else {
        now.checked_add(8_000_000)
    }
    .ok_or("Hathor Emergency deadline overflow")?;
    let snapshot = PARAMETERS
        .emergency_atk_percent
        .mul(F::new(power.into(), 1)?)?;
    b.delivery = Some(Delivery {
        generation: b.generation,
        until,
        atk_until: until,
        counter: 0,
        hidden_cooldown,
        atk_percent: snapshot,
        hybrid_burst_recovery: hybrid.then_some(recovery),
    });
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    if super::cyclone::accepted(ctx.request) {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            super::cyclone::MODEL,
        )?;
    }
    if hybrid {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            HYBRID,
        )?;
    }
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ActionAdmission".into(),
        event_kind: "hathor_emergency".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            expires_at_us: Some(until),
            pool: Some(u64::from(power)),
            atk_percent: Some(snapshot),
            ..Default::default()
        }),
    });
    schedule_kind(ctx, owner, false)
}
pub(crate) fn clear(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let phase = if ctx.state.actors[owner].hp == 0 {
        "SettlementAndDeath"
    } else {
        "ActionAdmission"
    };
    clear_with_phase(ctx, owner, phase)
}
pub(super) fn clear_with_phase(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    phase: &str,
) -> Result<(), String> {
    restore_cooldown(&mut ctx.state.actors[owner])?;
    if let Some(b) = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .burst
        .as_mut()
        && b.delivery.take().is_some()
    {
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: phase.into(),
            event_kind: "hathor_emergency_end".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(owner),
            to_actor: Some(owner),
            energy_change: None,
            energy_fanout: None,
            data: None,
        });
    }
    Ok(())
}
pub(crate) fn expire(ctx: &mut CallbackContext<'_>, event: Expiry) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let current = ctx.state.actors[event.owner]
        .character
        .hathor_mut()?
        .burst
        .as_ref()
        .and_then(|b| b.delivery.as_ref());
    if current.is_none_or(|d| {
        d.generation != event.generation
            || (if event.atk_only { d.atk_until } else { d.until }) != now
    }) {
        return Ok(());
    }
    if !event.atk_only
        && super::cyclone::accepted(ctx.request)
        && let Some(finish) = super::clock::cyclone_finish(&ctx.state.actors[event.owner], now)
    {
        restore_cooldown(&mut ctx.state.actors[event.owner])?;
        let d = ctx.state.actors[event.owner]
            .character
            .hathor_mut()?
            .burst
            .as_mut()
            .and_then(|b| b.delivery.as_mut())
            .ok_or("Hathor Emergency missing")?;
        d.counter = 0;
        d.atk_until = finish.min(
            now.checked_add(2_000_000)
                .ok_or("Hathor pending ATK overflow")?,
        );
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: now,
            phase: "ModePause".into(),
            event_kind: "hathor_emergency_state_end".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(event.owner),
            to_actor: Some(event.owner),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                expires_at_us: Some(d.atk_until),
                ..Default::default()
            }),
        });
        return schedule_kind(ctx, event.owner, true);
    }
    let phase = if event.atk_only || !super::cyclone::accepted(ctx.request) {
        "BuffTickOrExpiry"
    } else {
        "ModePause"
    };
    clear_with_phase(ctx, event.owner, phase)
}
fn restore_cooldown(actor: &mut ActorRuntime) -> Result<(), String> {
    let hidden = actor
        .character
        .hathor_mut()?
        .burst
        .as_mut()
        .and_then(|b| b.delivery.as_mut())
        .and_then(|d| d.hidden_cooldown.take());
    if let Some(deadline) = hidden {
        actor.ability_cooldowns.remove("GA_Hathor_Skill");
        if let Some(deadline) = deadline {
            actor
                .ability_cooldowns
                .insert("GA_Hathor_Skill".into(), deadline);
        }
    }
    Ok(())
}
/// Connected pauses discovered after state expiry may extend the pending stage,
/// but never its original two-second safety bound or an already expired carrier.
pub(super) fn project_pending(
    actor: &mut ActorRuntime,
    owner: usize,
    now: u64,
    queue: &mut crate::simulation::timer_queue::EventQueue,
) -> Result<(), String> {
    let finish = actor.locked_until_us;
    let Some(d) = actor
        .character
        .hathor_mut()?
        .burst
        .as_mut()
        .and_then(|b| b.delivery.as_mut())
        .filter(|d| d.until <= now && now < d.atk_until)
    else {
        return Ok(());
    };
    let until = finish.min(
        d.until
            .checked_add(2_000_000)
            .ok_or("Hathor pending ATK safety overflow")?,
    );
    if until == d.atk_until {
        return Ok(());
    }
    d.atk_until = until;
    let generation = d.generation;
    let mut events = queue.drain().collect::<Vec<_>>();
    for e in &mut events {
        if matches!(&e.data,crate::simulation::events::GameEventData::Content(
            crate::callbacks::ContentEvent::HathorEmergency(x)) if x.owner==owner && x.generation==generation && x.atk_only)
        {
            e.time_us = until;
        }
    }
    queue.replace(events);
    Ok(())
}
pub(super) fn finish_cyclone(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    phase: &str,
) -> Result<(), String> {
    if !active(&ctx.state.actors[owner], ctx.state.current_time_us) {
        clear_with_phase(ctx, owner, phase)?;
    }
    Ok(())
}
pub(super) fn counter(actor: &ActorRuntime) -> u8 {
    if let crate::characters::RuntimeState::Hathor(s) = &actor.character {
        return s
            .burst
            .as_ref()
            .and_then(|b| b.delivery.as_ref())
            .map_or(0, |d| d.counter);
    }
    0
}
pub(super) fn increment_counter(actor: &mut ActorRuntime, now: u64) -> Result<(), String> {
    if active(actor, now)
        && let Some(d) = actor
            .character
            .hathor_mut()?
            .burst
            .as_mut()
            .and_then(|b| b.delivery.as_mut())
    {
        d.counter = d.counter.saturating_add(1).min(10);
    }
    Ok(())
}
pub(super) fn interrupt(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let Some(b) = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .burst
        .as_mut()
    else {
        return Ok(());
    };
    let Some(d) = b
        .delivery
        .as_mut()
        .filter(|d| d.hybrid_burst_recovery.is_some_and(|end| now < end))
    else {
        return Ok(());
    };
    d.until = now
        .checked_add(5_000_000)
        .ok_or("Hathor hybrid interrupted deadline overflow")?;
    d.atk_until = d.until;
    d.hybrid_burst_recovery = None;
    b.generation = b
        .generation
        .checked_add(1)
        .ok_or("Hathor hybrid generation overflow")?;
    d.generation = b.generation;
    schedule_kind(ctx, owner, false)
}
pub(crate) fn skill_failure(
    actor: &ActorRuntime,
    now: u64,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    active(actor, now).then(|| crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Other {
            message: "Hathor Cyclone admission remains unsupported by burst_emergency_v1".into(),
        },
        message: "Hathor Cyclone admission remains unsupported by burst_emergency_v1".into(),
    })
}

impl State {
    pub(crate) fn pause_world(&mut self, from: u64, delta: u64) -> Result<(), String> {
        if let Some(delivery) = &mut self.delivery {
            crate::simulation::timer_state::future(&mut delivery.until, from, delta)?;
            crate::simulation::timer_state::future(&mut delivery.atk_until, from, delta)?;
            if let Some(Some(deadline)) = &mut delivery.hidden_cooldown {
                crate::simulation::timer_state::future(deadline, from, delta)?;
            }
        }
        Ok(())
    }
}
