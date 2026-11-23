//! One canonical Ultimate input, live grant priority and a real paid cooldown.
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{
    GameActionCommand, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
pub(crate) const ABILITY: &str = "GA_Zankou_UltraSkill";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Variant {
    Inferno,
    Bloodfeast,
    FreeInferno,
}
impl Variant {
    pub(crate) fn recovery(self) -> u64 {
        if self == Self::Bloodfeast {
            5_697_196
        } else {
            4_818_122
        }
    }
    fn packets(self) -> &'static [(&'static str, u64)] {
        match self {
            Self::Inferno => &[
                ("GE_Player_Zankou_UltraSkill1_Damage", 3_166_667),
                ("GE_Player_Zankou_UltraSkill2_Damage", 3_783_333),
            ],
            Self::FreeInferno => &[
                ("GE_Player_Zankou_ForceUltraSkill1_Damage", 3_166_667),
                ("GE_Player_Zankou_ForceUltraSkill2_Damage", 3_783_333),
            ],
            Self::Bloodfeast => &[
                ("GE_Player_Zankou_MagicUltraSkill1_Damage", 2_991_323),
                ("GE_Player_Zankou_MagicUltraSkill1_Damage", 3_338_388),
                ("GE_Player_Zankou_MagicUltraSkill1_Damage", 3_664_250),
                ("GE_Player_Zankou_MagicUltraSkill2_Damage", 4_216_667),
            ],
        }
    }
}
fn variant(state: &super::state::State, now: u64, energy: u64) -> Variant {
    if state.bloodfeast_ready || (state.a4 && energy >= 120 && now >= state.paid_burst_until_us) {
        Variant::Bloodfeast
    } else if now < state.free_inferno_until_us {
        Variant::FreeInferno
    } else {
        Variant::Inferno
    }
}
pub(crate) fn select(actor: &ActorRuntime, command: &mut GameActionCommand) {
    if let GameActionCommand::InvokeAbility {
        ability_id,
        timestamp_us,
        recovery_us,
        cooldown_us,
        energy_cost,
        hit_delay_us,
        ..
    } = command
        && ability_id == ABILITY
        && let crate::characters::RuntimeState::Zankou(state) = &actor.character
    {
        let selected = variant(state, *timestamp_us, actor.energy.current());
        *recovery_us = selected.recovery();
        *hit_delay_us = selected.packets()[0].1;
        *energy_cost = if selected == Variant::FreeInferno {
            0
        } else {
            120
        };
        *cooldown_us = if selected == Variant::FreeInferno {
            0
        } else {
            20_000_000
        };
    }
}
pub(crate) fn admission(
    req: &GameScenarioRequest,
    command: usize,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if ability != ABILITY {
        return None;
    }
    let crate::characters::RuntimeState::Zankou(state) = &actor.character else {
        return None;
    };
    let free = variant(state, now, actor.energy.current()) == Variant::FreeInferno;
    if !free && now < state.paid_burst_until_us {
        return Some(crate::simulation::rejection::AdmissionFailure {
            reason: crate::simulation::GameAdmissionReason::Cooldown {
                until_us: state.paid_burst_until_us,
            },
            message: "Zankou paid Ultimate cooldown is continuing".into(),
        });
    }
    if req.actions.get(command).is_some()
        && !matches!(req.actions.get(command),Some(GameActionCommand::InvokeAbility{energy_cost,cooldown_us,damage_effect_id,..}) if *energy_cost==(if free {0}else{120}) && *cooldown_us==(if free {0}else{20_000_000}) && damage_effect_id=="GE_Player_Zankou_UltraSkill1_Damage")
    {
        let message = "Zankou canonical Ultimate cost/cooldown must match its earned live variant";
        return Some(crate::simulation::rejection::AdmissionFailure {
            reason: crate::simulation::GameAdmissionReason::Other {
                message: message.into(),
            },
            message: message.into(),
        });
    }
    None
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Grant {
        owner: usize,
        command: usize,
    },
    Expire {
        owner: usize,
        generation: u64,
    },
    Finish {
        owner: usize,
        command: usize,
        variant: Variant,
    },
}
fn schedule(ctx: &mut CastContext<'_>, delay: u64, event: Event) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx
            .time_us
            .checked_add(delay)
            .ok_or("Zankou Burst notify overflow")?,
        priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::ZankouBurst(event)),
    });
    Ok(())
}
pub(crate) fn handle_burst(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(super::forms::MODEL)?;
    let cost = match ctx.request.actions.get(ctx.command_index) {
        Some(GameActionCommand::InvokeAbility { energy_cost, .. }) => *energy_cost,
        _ => return Err("Zankou Ultimate requires its canonical input".into()),
    };
    let energy = ctx
        .actor
        .energy
        .current()
        .checked_add(cost)
        .ok_or("Zankou pre-cost energy overflow")?;
    let selected = variant(ctx.actor.character.zankou_mut()?, ctx.time_us, energy);
    super::forms::stop(ctx.actor, ctx.queue, ctx.buffs, ctx.time_us)?;
    ctx.cancel_primary_delivery()?;
    let first_seq = *ctx.sequence;
    let mut groups = std::collections::BTreeMap::new();
    for &(effect, delay) in selected.packets() {
        groups.insert(
            ctx.schedule_native_aoe_group(
                effect,
                ctx.time_us
                    .checked_add(delay)
                    .ok_or("Zankou Ultimate hit overflow")?,
            )?,
            delay,
        );
    }
    let state = ctx.actor.character.zankou_mut()?;
    if selected == Variant::FreeInferno {
        state.free_inferno_until_us = 0;
        ctx.actor
            .ability_cooldowns
            .insert(ABILITY.into(), state.paid_burst_until_us);
    } else {
        state.paid_burst_until_us = ctx
            .time_us
            .checked_add(20_000_000)
            .ok_or("Zankou paid cooldown overflow")?;
        state.bloodfeast_ready = false;
    }
    let state = ctx.actor.character.zankou_mut()?;
    if selected == Variant::Bloodfeast && state.form == super::state::Form::Illusion {
        state.form = super::state::Form::Reality;
        state.form_until_us = 0;
        state.form_generation = state
            .form_generation
            .checked_add(1)
            .ok_or("Zankou Bloodfeast form generation overflow")?;
    }
    let capture = selected == Variant::Inferno
        || (selected == Variant::Bloodfeast && state.a4)
        || (selected == Variant::FreeInferno && !state.a4);
    if capture {
        state.kindling_active_for_ultra = state.kindling_stacks > 0;
        state.kindling_stacks = 0;
    }
    if selected == Variant::Bloodfeast {
        let at = ctx
            .time_us
            .checked_add(4_216_667)
            .ok_or("Zankou Bloodfeast application overflow")?;
        super::vile_ash::apply_vile_ash(ctx, at)?;
        schedule(
            ctx,
            5_675_956,
            Event::Grant {
                owner: ctx.actor_index,
                command: ctx.command_index,
            },
        )?;
    }
    schedule(
        ctx,
        selected.recovery(),
        Event::Finish {
            owner: ctx.actor_index,
            command: ctx.command_index,
            variant: selected,
        },
    )?;
    ctx.actor.switch_action_locked_until_us = ctx
        .time_us
        .checked_add(if selected == Variant::Bloodfeast {
            5_649_767
        } else {
            4_799_427
        })
        .ok_or("Zankou Ultimate switch overflow")?;
    super::clock::start(
        ctx,
        super::clock::Kind::Burst(selected),
        groups,
        first_seq,
        None,
    )
}
pub(crate) fn clear(actor: &mut ActorRuntime) -> Result<(), String> {
    let state = actor.character.zankou_mut()?;
    state.free_inferno_until_us = 0;
    state.kindling_active_for_ultra = false;
    state.free_inferno_generation = state
        .free_inferno_generation
        .checked_add(1)
        .ok_or("Zankou free grant generation overflow")?;
    if state.paid_burst_until_us > 0 {
        actor
            .ability_cooldowns
            .insert(ABILITY.into(), state.paid_burst_until_us);
    }
    Ok(())
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    match event {
        Event::Grant { owner, command } => {
            let actor = &mut ctx.state.actors[owner];
            if actor.hp == 0
                || ctx.state.active_actor_index != owner
                || super::clock::command(actor) != Some(command)
            {
                return Ok(());
            }
            let state = actor.character.zankou_mut()?;
            state.free_inferno_until_us = now
                .checked_add(5_000_000)
                .ok_or("Zankou free grant expiry overflow")?;
            state.free_inferno_generation = state
                .free_inferno_generation
                .checked_add(1)
                .ok_or("Zankou free grant generation overflow")?;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: state.free_inferno_until_us,
                priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: GameEventData::Content(crate::callbacks::ContentEvent::ZankouBurst(
                    Event::Expire {
                        owner,
                        generation: state.free_inferno_generation,
                    },
                )),
            });
            actor.ability_cooldowns.remove(ABILITY);
        }
        Event::Expire { owner, generation } => {
            let actor = &mut ctx.state.actors[owner];
            let state = actor.character.zankou_mut()?;
            if generation == state.free_inferno_generation
                && state.free_inferno_until_us > 0
                && now >= state.free_inferno_until_us
            {
                clear(actor)?;
            }
        }
        Event::Finish {
            owner,
            command,
            variant,
        } => {
            let actor = &mut ctx.state.actors[owner];
            if super::clock::command(actor) == Some(command) && variant != Variant::Bloodfeast {
                actor.character.zankou_mut()?.kindling_active_for_ultra = false;
            }
        }
    }
    Ok(())
}
