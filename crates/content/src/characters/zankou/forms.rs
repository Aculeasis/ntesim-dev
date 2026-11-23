//! Current rolling source form selection. Real form time is independent of montage time.
use super::state::Form;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::state::ActorRuntime;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{GameActionCommand, GameActorInterruptReceipt};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ZankouFormsV1;
pub(crate) const SKILL: &str = "GA_Zankou_Skill";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Variant {
    Entry(bool),
    Return(bool),
}
impl Variant {
    pub(crate) fn recovery(self) -> u64 {
        match self {
            Self::Entry(_) => 998_950,
            Self::Return(false) => 2_267_783,
            Self::Return(true) => 2_267_928,
        }
    }
    fn transition(self) -> u64 {
        match self {
            Self::Entry(_) => 496_507,
            Self::Return(_) => 2_039_354,
        }
    }
    fn packets(self) -> &'static [(&'static str, u64)] {
        match self {
            Self::Entry(false) => &[
                ("GE_Player_Zankou_Skill2_Damage", 122_519),
                ("GE_Player_Zankou_Skill2_Damage", 226_651),
                ("GE_Player_Zankou_Skill2_Damage", 345_053),
                ("GE_Player_Zankou_Skill2_Damage", 453_684),
                ("GE_Player_Zankou_Skill2_1_Damage", 666_135),
            ],
            Self::Entry(true) => &[
                ("GE_Player_Zankou_Skill1_Damage", 122_519),
                ("GE_Player_Zankou_Skill1_Damage", 226_651),
                ("GE_Player_Zankou_Skill1_Damage", 345_053),
                ("GE_Player_Zankou_Skill1_Damage", 453_684),
                ("GE_Player_Zankou_Skill1_1_Damage", 666_135),
            ],
            Self::Return(false) => &[
                ("GE_Player_Zankou_Skill4_Damage", 90_019),
                ("GE_Player_Zankou_Skill4_Damage", 196_616),
                ("GE_Player_Zankou_Skill4_1_Damage", 601_651),
                ("GE_Player_Zankou_Skill4_1_Damage", 722_260),
                ("GE_Player_Zankou_Skill4_1_Damage", 870_492),
                ("GE_Player_Zankou_Skill4_1_Damage", 1_006_986),
                ("GE_Player_Zankou_Skill4_2_Damage", 1_860_000),
            ],
            Self::Return(true) => &[
                ("GE_Player_Zankou_Skill3_Damage", 90_019),
                ("GE_Player_Zankou_Skill3_Damage", 196_617),
                ("GE_Player_Zankou_Skill3_1_Damage", 601_651),
                ("GE_Player_Zankou_Skill3_1_Damage", 722_260),
                ("GE_Player_Zankou_Skill3_1_Damage", 870_492),
                ("GE_Player_Zankou_Skill3_1_Damage", 1_006_986),
                ("GE_Player_Zankou_Skill3_2_Damage", 1_860_000),
            ],
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct ActiveSkill {
    pub(crate) command: usize,
    variant: Variant,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    A5Impulse { owner: usize, command: usize },
    Transition { owner: usize, command: usize },
    Expire { owner: usize, generation: u64 },
    Bloodfeast { owner: usize, command: usize },
}
fn schedule(ctx: &mut CastContext<'_>, delay: u64, event: Event) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx
            .time_us
            .checked_add(delay)
            .ok_or("Zankou notify overflow")?,
        priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::ZankouForms(event)),
    });
    Ok(())
}
pub(crate) fn select(actor: &ActorRuntime, command: &mut GameActionCommand) {
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        ability_id,
        recovery_us,
        hit_delay_us,
        ..
    } = command
        && ability_id == "GA_Zankou_Melee"
        && let crate::characters::RuntimeState::Zankou(state) = &actor.character
    {
        let kind = super::normal::kind(state, *timestamp_us);
        *recovery_us = super::profile::recovery(kind);
        *hit_delay_us = super::profile::hits(kind)[0].1;
    }
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        ability_id,
        recovery_us,
        cooldown_us,
        hit_delay_us,
        ..
    } = command
        && ability_id == SKILL
        && let crate::characters::RuntimeState::Zankou(state) = &actor.character
    {
        let variant = state.skill_variant(*timestamp_us);
        *recovery_us = variant.recovery();
        *cooldown_us = 5_000_000;
        *hit_delay_us = variant.packets()[0].1;
    }
}
pub(crate) fn admission(
    req: &crate::simulation::GameScenarioRequest,
    command: usize,
    ability: &str,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if ability != SKILL {
        return None;
    }
    let input = req.actions.get(command)?;
    if matches!(
        input,
        GameActionCommand::InvokeAbility {
            energy_cost: 0,
            cooldown_us: 5_000_000,
            ..
        }
    ) {
        return None;
    }
    let message = "Zankou canonical Skill requires zero energy and its real five-second cooldown";
    Some(crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Other {
            message: message.into(),
        },
        message: message.into(),
    })
}
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    let variant = ctx.actor.character.zankou_mut()?.skill_variant(ctx.time_us);
    let kind = if matches!(variant, Variant::Entry(_)) {
        super::modifiers::Kind::Hunt
    } else {
        super::modifiers::Kind::Delusion
    };
    let modifier = super::clock::carries(ctx.actor, kind, ctx.time_us).then_some(kind);
    stop(ctx.actor, ctx.queue, ctx.buffs, ctx.time_us)?;
    let first_seq = *ctx.sequence;
    if ctx.config.awakening.has_effect(1)
        && let Some(delay) = super::profile::spread(super::clock::Kind::Skill(variant))
    {
        super::spread::schedule(ctx, delay)?;
    }
    if variant == Variant::Entry(true) && ctx.config.awakening.has_effect(5) {
        schedule(
            ctx,
            657_891,
            Event::A5Impulse {
                owner: ctx.actor_index,
                command: ctx.command_index,
            },
        )?;
    }
    ctx.cancel_primary_delivery()?;
    let mut groups = BTreeMap::new();
    for &(effect, delay) in variant.packets() {
        groups.insert(
            ctx.schedule_native_aoe_group(
                effect,
                ctx.time_us
                    .checked_add(delay)
                    .ok_or("Zankou Skill hit overflow")?,
            )?,
            delay,
        );
    }
    ctx.actor.locked_until_us = ctx
        .time_us
        .checked_add(variant.recovery())
        .ok_or("Zankou Skill recovery overflow")?;
    ctx.actor.switch_action_locked_until_us = ctx
        .time_us
        .checked_add(match variant {
            Variant::Entry(_) => 0,
            Variant::Return(false) => 2_250_017,
            Variant::Return(true) => 2_249_976,
        })
        .ok_or("Zankou Skill switch overflow")?;
    let state = ctx.actor.character.zankou_mut()?;
    state.pending_skill = Some(ActiveSkill {
        command: ctx.command_index,
        variant,
    });
    schedule(
        ctx,
        variant.transition(),
        Event::Transition {
            owner: ctx.actor_index,
            command: ctx.command_index,
        },
    )?;
    if let Variant::Entry(true) | Variant::Return(true) = variant {
        let delay = if matches!(variant, Variant::Entry(_)) {
            662_055
        } else {
            1_860_000
        };
        let at = ctx
            .time_us
            .checked_add(delay)
            .ok_or("Zankou Heartwrench notify overflow")?;
        for _ in 0..5 {
            super::vile_ash::apply_heartwrench(ctx, at)?;
        }
    }
    if variant == Variant::Return(true) {
        schedule(
            ctx,
            1_860_000,
            Event::Bloodfeast {
                owner: ctx.actor_index,
                command: ctx.command_index,
            },
        )?;
    }
    super::clock::start(
        ctx,
        super::clock::Kind::Skill(variant),
        groups,
        first_seq,
        modifier,
    )
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    match event {
        Event::A5Impulse { owner, command } => {
            super::combat_passives::impulse(ctx, owner, command)?
        }
        Event::Transition { owner, command } => {
            let actor = &mut ctx.state.actors[owner];
            if actor.hp == 0 || owner != ctx.state.active_actor_index {
                return Ok(());
            }
            let state = actor.character.zankou_mut()?;
            let Some(action) = state
                .pending_skill
                .as_ref()
                .filter(|a| a.command == command)
            else {
                return Ok(());
            };
            let entering = matches!(action.variant, Variant::Entry(_));
            state.form_generation = state
                .form_generation
                .checked_add(1)
                .ok_or("Zankou form generation overflow")?;
            state.form = if entering {
                Form::Illusion
            } else {
                Form::Reality
            };
            state.form_until_us = if entering {
                now.checked_add(8_000_000)
                    .ok_or("Zankou form expiry overflow")?
            } else {
                0
            };
            state.normal_stage = 0;
            state.normal_started_us = None;
            if entering {
                crate::runtime_api::increment_sequence(ctx.sequence)?;
                ctx.queue.push(ScheduledGameEvent {
                    time_us: state.form_until_us,
                    priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                    seq: *ctx.sequence,
                    data: GameEventData::Content(crate::callbacks::ContentEvent::ZankouForms(
                        Event::Expire {
                            owner,
                            generation: state.form_generation,
                        },
                    )),
                });
            }
            super::modifiers::transition(
                &mut ctx.state.actors[owner],
                owner,
                &mut ctx.state.buffs,
                entering,
                now,
            )?;
            trace_form(ctx, owner, "zankou_form_transition");
        }
        Event::Expire { owner, generation } => {
            let state = ctx.state.actors[owner].character.zankou_mut()?;
            if generation != state.form_generation || now < state.form_until_us {
                return Ok(());
            }
            state.form = Form::Reality;
            state.form_until_us = 0;
            state.normal_stage = 0;
            state.normal_started_us = None;
            trace_form(ctx, owner, "zankou_form_expired");
        }
        Event::Bloodfeast { owner, command } => {
            let state = ctx.state.actors[owner].character.zankou_mut()?;
            if state
                .pending_skill
                .as_ref()
                .is_some_and(|a| a.command == command)
            {
                state.bloodfeast_ready = true;
            }
        }
    }
    Ok(())
}
fn trace_form(ctx: &mut CallbackContext<'_>, owner: usize, kind: &str) {
    let crate::characters::RuntimeState::Zankou(state) = &ctx.state.actors[owner].character else {
        return;
    };
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "content".into(),
        event_kind: kind.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            condition: Some(
                if state.form == Form::Illusion {
                    "illusion"
                } else {
                    "reality"
                }
                .into(),
            ),
            expires_at_us: Some(state.form_until_us),
            ..Default::default()
        }),
    });
}
pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    now: u64,
) -> Result<(), String> {
    super::clock::stop(actor, queue, buffs, now)?;
    actor.character.zankou_mut()?.pending_skill = None;
    Ok(())
}
pub(crate) fn field_exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    stop(
        &mut ctx.state.actors[owner],
        ctx.queue,
        &mut ctx.state.buffs,
        ctx.state.current_time_us,
    )?;
    super::burst::clear(&mut ctx.state.actors[owner])?;
    super::modifiers::clear(
        &mut ctx.state.actors[owner],
        owner,
        &mut ctx.state.buffs,
        ctx.state.current_time_us,
    )?;
    let state = ctx.state.actors[owner].character.zankou_mut()?;
    state.form = Form::Reality;
    state.form_until_us = 0;
    state.form_generation = state
        .form_generation
        .checked_add(1)
        .ok_or("Zankou form generation overflow")?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.bloodfeast_ready = false;
    state.airborne = false;
    state.riposte_ready = false;
    state.hunt_until_us = 0;
    Ok(())
}
pub(crate) fn interrupt(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<GameActorInterruptReceipt, String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let cooldown = actor.ability_cooldowns.get(SKILL).copied();
    let mut receipt = GameActorInterruptReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        interrupted_command_index: None,
        cancelled_hit_deliveries: 0,
        recovery_before_us: actor.locked_until_us,
        recovery_after_us: actor.locked_until_us,
        skill_cooldown_before_us: cooldown,
        skill_cooldown_after_us: cooldown,
        field_cancelled: false,
    };
    if actor.hp == 0 || now >= actor.locked_until_us {
        return Ok(receipt);
    }
    receipt.interrupted_command_index = super::clock::command(actor);
    let before = ctx
        .queue
        .iter()
        .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
        .count();
    stop(actor, ctx.queue, &mut ctx.state.buffs, now)?;
    super::burst::clear(actor)?;
    actor.character.zankou_mut()?.bloodfeast_ready = false;
    let after = ctx
        .queue
        .iter()
        .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
        .count();
    receipt.cancelled_hit_deliveries = before - after;
    actor.locked_until_us = now;
    receipt.recovery_after_us = now;
    Ok(receipt)
}
