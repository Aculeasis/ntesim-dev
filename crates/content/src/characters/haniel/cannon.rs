//! Charged Cannon and finite zero-travel ricochet instances on authored dummies.
use super::owned_packet::Source;
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::{
    GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
};
use ntesim_core::execution::CausalPriority;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HanielCannonRicochetV1;
pub(crate) const PACKET: &str = "GE_Player_Haniel_Branch_Damage1";
#[derive(Clone, Debug, Default)]
pub(super) struct Charge {
    fast: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Gate {
        owner: usize,
        source: Box<Source>,
    },
    Fire {
        owner: usize,
        source: Box<Source>,
        fast: bool,
    },
    Leg {
        owner: usize,
        source: Box<Source>,
        previous: usize,
        remaining: u8,
        ordinal: u8,
    },
}
impl Event {
    pub(super) fn belongs(&self, actor: usize, command: usize) -> bool {
        match self {
            Self::Gate { owner, source, .. } | Self::Fire { owner, source, .. } => {
                *owner == actor && source.command() == command
            }
            Self::Leg { .. } => false, // Already launched projectile lineage survives field exit.
        }
    }
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    super::secondary::accepted(req)
        || req
            .model_assumptions
            .get(MODEL.as_str())
            .map(String::as_str)
            == Some(MODEL.description())
}
pub(super) fn input(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    let Some(hold) = ctx.request.hold_inputs.get(&ctx.command_index) else {
        return Ok(false);
    };
    if !accepted(ctx.request)
        || ctx.ability_id != "GA_Haniel_Melee"
        || hold.moving
        || hold.reserved_stocks.is_some()
    {
        return Err(
            "Haniel Cannon requires cannon_ricochet_v1 and a stationary unreserved Normal hold"
                .into(),
        );
    }
    ctx.record_hypothesis(MODEL)?;
    Ok(hold.duration_us >= 200_000)
}
fn push(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    time_us: u64,
    event: Event,
    priority: CausalPriority,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority,
        seq: *sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::HanielCannon(event)),
    });
    Ok(())
}
pub(super) fn schedule(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let source = Box::new(Source::capture(ctx)?);
    let duration = ctx.request.hold_inputs[&ctx.command_index].duration_us;
    // The recovered zero-hold permission is strict .4 < progress < .5.
    // Early release waits for the first representable point after .4.
    let release = duration.saturating_sub(200_000).max(400_001);
    if release < 500_000 {
        push(
            ctx.queue,
            ctx.sequence,
            ctx.time_us
                .checked_add(200_000 + release)
                .ok_or("Haniel charge gate overflow")?,
            Event::Gate {
                owner: ctx.actor_index,
                source: source.clone(),
            },
            CausalPriority::ActionAdmission,
        )?;
    }
    push(
        ctx.queue,
        ctx.sequence,
        ctx.time_us
            .checked_add(1_061_263)
            .ok_or("Haniel Cannon emission overflow")?,
        Event::Fire {
            owner: ctx.actor_index,
            source,
            fast: false,
        },
        CausalPriority::ActionAdmission,
    )
}
pub(super) fn riposte(ctx: &mut CastContext<'_>) -> Result<(), String> {
    push(
        ctx.queue,
        ctx.sequence,
        ctx.time_us
            .checked_add(400_000)
            .ok_or("Haniel Riposte Cannon overflow")?,
        Event::Fire {
            owner: ctx.actor_index,
            source: Box::new(Source::capture(ctx)?),
            fast: false,
        },
        CausalPriority::ActionAdmission,
    )
}
fn trace(ctx: &mut CallbackContext<'_>, owner: usize, ordinal: u8, anchor: usize, remaining: u8) {
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "haniel_cannon_leg".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            ordinal: Some(ordinal),
            target_index: Some(anchor),
            remaining: Some(u32::from(remaining)),
            ..Default::default()
        }),
    });
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::Gate { owner, source } => {
            let now = ctx.state.current_time_us;
            let actor = &mut ctx.state.actors[owner];
            let ace = actor
                .character
                .haniel_mut()?
                .paranormal_ace_until_us
                .is_some_and(|until| until > now);
            if actor.hp == 0 || !ace {
                return Ok(());
            }
            let Some(charge) = super::clock::charge(actor, source.command())? else {
                return Ok(());
            };
            charge.fast = true;
            super::clock::accelerate(actor, source.command(), now)?;
            actor.locked_until_us = now
                .checked_add(195_077)
                .ok_or("Haniel accelerated recovery overflow")?;
            ctx.queue.retain(|event| !matches!(&event.data,
                GameEventData::Content(crate::callbacks::ContentEvent::HanielCannon(Event::Fire {owner: queued,source: parent,fast:false}))
                    if *queued == owner && parent.command() == source.command()));
            push(
                ctx.queue,
                ctx.sequence,
                now.checked_add(44_596)
                    .ok_or("Haniel accelerated emission overflow")?,
                Event::Fire {
                    owner,
                    source,
                    fast: true,
                },
                CausalPriority::ActionAdmission,
            )
        }
        Event::Fire {
            owner,
            source,
            fast,
        } => {
            let actor = &mut ctx.state.actors[owner];
            if actor.hp == 0 {
                return Ok(());
            }
            let Some(charge) = super::clock::charge(actor, source.command())? else {
                return Ok(());
            };
            if charge.fast != fast {
                return Ok(());
            }
            let plot = actor.character.haniel_mut()?.plot;
            let Some(anchor) = ctx.state.targets.iter().position(|t| t.hp_current > 0) else {
                return Ok(());
            };
            source.cannon(ctx, owner, Some(anchor))?;
            trace(ctx, owner, 0, anchor, plot);
            if plot > 0 {
                push(
                    ctx.queue,
                    ctx.sequence,
                    ctx.state.current_time_us,
                    Event::Leg {
                        owner,
                        source,
                        previous: anchor,
                        remaining: plot,
                        ordinal: 1,
                    },
                    CausalPriority::ProcCallback,
                )?;
            }
            Ok(())
        }
        Event::Leg {
            owner,
            source,
            previous,
            remaining,
            ordinal,
        } => {
            if ctx.state.actors[owner].hp == 0
                || ctx.state.actors[owner]
                    .character
                    .haniel_mut()?
                    .out_of_combat
            {
                return Ok(());
            }
            let Some(anchor) = ctx
                .state
                .targets
                .iter()
                .enumerate()
                .position(|(index, t)| index != previous && t.hp_current > 0)
            else {
                return Ok(());
            };
            source.cannon(ctx, owner, None)?;
            trace(ctx, owner, ordinal, anchor, remaining - 1);
            if remaining > 1 {
                push(
                    ctx.queue,
                    ctx.sequence,
                    ctx.state.current_time_us,
                    Event::Leg {
                        owner,
                        source,
                        previous: anchor,
                        remaining: remaining - 1,
                        ordinal: ordinal + 1,
                    },
                    CausalPriority::ProcCallback,
                )?;
            }
            Ok(())
        }
    }
}
pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.damage_effect_id != PACKET || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let Some(group) = ctx.packet_group_id else {
        return Err("Haniel Cannon requires a causal packet group".into());
    };
    super::clock::pause(ctx.actor, ctx.command_index, group, 20_000)
}
