//! Explicit airborne time, causal loop spending and one landing packet.
use super::clock::{Kind, Plan};
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{
    ExactFraction, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorAirattackV1;
pub(crate) const PACKET: &str = "GE_Player_Hathor_AirAttack_Damage";
#[derive(Clone, Debug)]
pub(crate) struct State {
    command: usize,
    origin: u64,
    landing: u64,
    /// Branch fallback hands its landing packet to the Branch owner state.
    fallback: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Boundary {
    Start,
    Cost(u64),
    Land,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(super) owner: usize,
    pub(super) command: usize,
    boundary: Boundary,
}
pub(crate) fn accepted(r: &GameScenarioRequest) -> bool {
    r.model_assumptions.get(MODEL.as_str()).map(String::as_str) == Some(MODEL.description())
}
pub(crate) fn validate(r: &GameScenarioRequest) -> Result<(), String> {
    if accepted(r) && (!super::secondary::accepted(r) || r.combat_stamina.is_none()) {
        return Err(
            "Hathor airattack_v1 requires secondary_actions_v1 and explicit stamina".into(),
        );
    }
    Ok(())
}
pub(crate) fn validate_plunge(
    c: &crate::runtime_api::PlungeInputContext<'_>,
) -> Result<(), String> {
    if !accepted(c.request) || c.metadata.packet.damage_effect_id != PACKET {
        return Err(
            "Hathor falling Normal requires airattack_v1 and its native AirAttack packet".into(),
        );
    }
    c.timestamp_us
        .checked_add(c.input.landing_delay_us)
        .and_then(|v| v.checked_add(584_348))
        .ok_or("Hathor AirAttack deadline overflow")?;
    Ok(())
}
pub(crate) fn admission(
    r: &GameScenarioRequest,
    command: usize,
    stamina: Option<&crate::simulation::CombatStaminaReport>,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if !accepted(r) || !r.plunge_inputs.contains_key(&command) {
        return None;
    }
    if stamina.is_some_and(|s| !s.current.is_zero()) {
        return None;
    }
    let message =
        "Hathor falling Normal requires positive Strength, without an admission fee".to_owned();
    Some(crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Other {
            message: message.clone(),
        },
        message,
    })
}
pub(super) fn select(ctx: &mut CastContext<'_>) -> Result<Option<Plan>, String> {
    if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(PACKET) {
        return Ok(None);
    }
    if !accepted(ctx.request) {
        return Err("Hathor AirAttack requires airattack_v1".into());
    }
    let input = ctx
        .request
        .plunge_inputs
        .get(&ctx.command_index)
        .ok_or("Hathor AirAttack needs declared landing/height")?;
    ctx.record_hypothesis(MODEL)?;
    ctx.scale_primary_coefficient(
        PACKET,
        ExactFraction::new(1, 1)?.add(ExactFraction::from_ratio(input.height_bonus)?)?,
    )?;
    Ok(Some(Plan {
        kind: Kind::Plunge,
        hits: vec![(PACKET, 116_417)],
        recovery: 584_348,
        jump: 484_688,
        switch: 584_348,
        notifies: vec![],
    }))
}
fn push(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    seq: &mut u64,
    event: Event,
    time: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(seq)?;
    queue.push(ScheduledGameEvent {
        time_us: time,
        priority: if matches!(event.boundary, Boundary::Cost(_)) {
            ntesim_core::execution::CausalPriority::ActionAdmission
        } else {
            ntesim_core::execution::CausalPriority::ModePause
        },
        seq: *seq,
        data: GameEventData::Content(crate::callbacks::ContentEvent::HathorAir(event)),
    });
    Ok(())
}
pub(super) fn begin(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let landing = ctx
        .time_us
        .checked_add(ctx.request.plunge_inputs[&ctx.command_index].landing_delay_us)
        .ok_or("Hathor AirAttack landing overflow")?;
    ctx.actor.character.hathor_mut()?.air = Some(State {
        command: ctx.command_index,
        origin: ctx.time_us,
        landing,
        fallback: false,
    });
    push(
        ctx.queue,
        ctx.sequence,
        Event {
            owner: ctx.actor_index,
            command: ctx.command_index,
            boundary: Boundary::Start,
        },
        ctx.time_us,
    )?;
    if landing <= ctx.request.absolute_limit_us {
        push(
            ctx.queue,
            ctx.sequence,
            Event {
                owner: ctx.actor_index,
                command: ctx.command_index,
                boundary: Boundary::Land,
            },
            landing,
        )?;
    }
    Ok(())
}
/// Enter the repeating section at the Branch fallback transition. The authored
/// landing remains the real deadline; only cost coordinates use the air clock.
pub(super) fn begin_loop(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    origin: u64,
    landing: u64,
) -> Result<(), String> {
    ctx.state.actors[owner].character.hathor_mut()?.air = Some(State {
        command,
        origin,
        landing,
        fallback: true,
    });
    push(
        ctx.queue,
        ctx.sequence,
        Event {
            owner,
            command,
            boundary: Boundary::Start,
        },
        origin,
    )?;
    if landing <= ctx.request.absolute_limit_us {
        push(
            ctx.queue,
            ctx.sequence,
            Event {
                owner,
                command,
                boundary: Boundary::Land,
            },
            landing,
        )?;
    }
    Ok(())
}
fn low(stamina: &crate::simulation::CombatStaminaReport) -> Result<bool, String> {
    match stamina.current.try_sub(ExactFraction::new(10, 1)?) {
        Ok(value) => Ok(value.is_zero()),
        Err(ntesim_core::combat::exact::ExactArithmeticError::SubtractionUnderflow) => Ok(true),
        Err(error) => Err(error.to_string()),
    }
}
fn land(ctx: &mut CallbackContext<'_>, event: &Event) -> Result<(), String> {
    let fallback = ctx.state.actors[event.owner]
        .character
        .hathor_mut()?
        .air
        .as_ref()
        .is_some_and(|s| s.fallback);
    ctx.state.actors[event.owner].character.hathor_mut()?.air = None;
    ctx.queue.retain(|e|!matches!(&e.data,GameEventData::Content(crate::callbacks::ContentEvent::HathorAir(a)) if a.owner==event.owner && a.command==event.command));
    if fallback {
        return super::branch::air_land(ctx, event.owner, event.command);
    }
    super::clock::land(
        &mut ctx.state.actors[event.owner],
        ctx.queue,
        ctx.state.current_time_us,
    )?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: if matches!(event.boundary, Boundary::Cost(_)) {
            "ActionAdmission"
        } else {
            "ModePause"
        }
        .into(),
        event_kind: "hathor_air_landing".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(event.owner),
        to_actor: Some(event.owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(event.command),
            ..Default::default()
        }),
    });
    Ok(())
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, e: Event) -> Result<(), String> {
    if ctx.state.actors[e.owner].hp == 0 {
        ctx.state.actors[e.owner].character.hathor_mut()?.air = None;
        return Ok(());
    }
    let Some(s) = ctx.state.actors[e.owner]
        .character
        .hathor_mut()?
        .air
        .clone()
        .filter(|s| s.command == e.command)
    else {
        return Ok(());
    };
    let now = ctx.state.current_time_us;
    let stamina = ctx
        .state
        .combat_stamina
        .as_mut()
        .ok_or("Hathor airborne stamina missing")?;
    stamina.advance(now)?;
    if e.boundary == Boundary::Land || low(stamina)? {
        return land(ctx, &e);
    }
    let progress = if let Boundary::Cost(progress) = e.boundary {
        stamina.spend_up_to(e.command, e.owner, now, 10)?;
        if low(stamina)? {
            return land(ctx, &e);
        }
        progress
            .checked_add(500_000)
            .ok_or("Hathor loop progress overflow")?
    } else {
        616_667
    };
    let due = super::clock::air_deadline(&ctx.state.actors[e.owner], e.owner, s.origin, progress)?;
    if due < s.landing && due <= ctx.request.absolute_limit_us {
        push(
            ctx.queue,
            ctx.sequence,
            Event {
                boundary: Boundary::Cost(progress),
                ..e
            },
            due,
        )?;
    }
    Ok(())
}

impl Event {
    pub(crate) fn timer_clock(&self) -> ntesim_core::kernel::timing::ClockDomain {
        crate::simulation::timer_policy::action(self.owner, self.command, false)
    }
}
