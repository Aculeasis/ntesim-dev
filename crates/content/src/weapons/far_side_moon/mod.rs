use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext, DamageContext, HitModifiers};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameActorConfig, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsFarSideMoonV2;

pub(crate) const MAX_SKILL_STACKS: u32 = 4;
pub(crate) const SKILL_STACK_DURATION_US: u64 = 5_000_000;
pub(crate) const ACQUISITION_GATE_US: u64 = 300_000;

pub(crate) const BUFF_ID: &str = "far_side_moon.redirect_window";
pub(crate) const STACK_BUFF_ID: &str = "far_side_moon.skill_stacks";
const SOURCE: &str = "weapon:fork_prism";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) ready_at_us: u64,
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::FarSideMoon {
            basic_attack_damage_bonus,
            lakshana_damage_bonus,
            duration_us,
            ..
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.action_metadata.input_role == Some(InputActionRole::RedirectSkill) {
                crate::gear::schedule_cast_buff(
                    ctx,
                    ContentEvent::FarSideMoonGrant {
                        actor_index: ctx.actor_index,
                        basic_attack_damage_bonus: *basic_attack_damage_bonus,
                        lakshana_damage_bonus: *lakshana_damage_bonus,
                        duration_us: *duration_us,
                    },
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn on_hp_loss_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let Some(packet) = ctx.packet_metadata else {
        return Ok(());
    };
    if packet.native_damage_source_category.as_deref()
        != Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A")
    {
        return Ok(());
    }
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::FarSideMoon {
            skill_damage_per_stack,
            ..
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.time_us < ctx.actor.equipment.far_side_moon.ready_at_us {
                continue;
            }
            ctx.actor.equipment.far_side_moon.ready_at_us = ctx
                .time_us
                .checked_add(ACQUISITION_GATE_US)
                .ok_or("Far Side of the Moon acquisition gate deadline overflow")?;

            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::FarSideMoonSkillStack {
                    actor_index: ctx.actor_index,
                    stack_amount: *skill_damage_per_stack,
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn crit_rate_bonus(config: &GameActorConfig) -> Result<ExactFraction, String> {
    let mut total = ExactFraction::new(0, 1)?;
    for proc in &config.gear_procs {
        if let GearProcConfig::FarSideMoon {
            crit_rate_bonus, ..
        } = proc
        {
            total = total.add(ExactFraction::from_ratio(*crit_rate_bonus)?)?;
        }
    }
    Ok(total)
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::FarSideMoon { .. } = proc else {
            continue;
        };
        let category = packet.native_damage_source_category.as_deref();
        let active = ctx.buff_stats;
        if category == Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E")
            && let Some(value) = active.get("far_side_moon_skill_damage")
        {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*value)?)?;
        }
        if category == Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A")
            && let Some(value) = active.get("far_side_moon_basic_attack_damage")
        {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*value)?)?;
        }
        if packet.element == Some(DamageElement::Lakshana)
            && active.get("far_side_moon_skill_damage").is_some()
            && let Some(value) = active.get("far_side_moon_lakshana_damage")
        {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*value)?)?;
        }
    }
    Ok(modifiers)
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    basic_attack_damage_bonus: Ratio,
    lakshana_damage_bonus: Ratio,
    duration_us: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    state.apply_buff(
        actor_index,
        TimedBuffDeclaration::reset(
            BUFF_ID,
            SOURCE,
            BuffRecipient::SelfActor,
            duration_us,
            BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([
                (
                    "far_side_moon_basic_attack_damage".into(),
                    ExactFraction::from_ratio(basic_attack_damage_bonus)?,
                ),
                (
                    "far_side_moon_lakshana_damage".into(),
                    ExactFraction::from_ratio(lakshana_damage_bonus)?,
                ),
            ]),
        ),
    )?;
    let expiry = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("Far Side of the Moon expiry overflow")?;
    if expiry <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: expiry,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::BuffLifecycleExpiry),
        });
    }
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "far_side_moon_redirect_window".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            expires_at_us: Some(expiry),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn apply_skill_stack(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    stack_amount: Ratio,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    state.apply_buff(
        actor_index,
        TimedBuffDeclaration {
            id: STACK_BUFF_ID.into(),
            source: SOURCE.into(),
            recipient: BuffRecipient::SelfActor,
            duration_us: SKILL_STACK_DURATION_US,
            stacking_scope: BuffStackingScope::PerSource,
            refresh_policy: RefreshPolicy::Reset,
            max_stacks: MAX_SKILL_STACKS,
            decay_policy: DecayPolicy::AllAtOnce,
            stat_modifiers: std::collections::BTreeMap::from([(
                "far_side_moon_skill_damage".into(),
                ExactFraction::from_ratio(stack_amount)?,
            )]),
        },
    )?;
    let deadline = state
        .current_time_us
        .checked_add(SKILL_STACK_DURATION_US)
        .ok_or("Far Side of the Moon skill stack expiry overflow")?;
    if deadline <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: deadline,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::BuffLifecycleExpiry),
        });
    }
    let stacks = state
        .buffs
        .instances()
        .iter()
        .filter(|b| b.id == STACK_BUFF_ID && b.source == SOURCE && b.owner_actor == actor_index)
        .map(|b| b.stacks)
        .sum::<u32>();
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "far_side_moon_skill_stack".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            stacks: Some(stacks),
            expires_at_us: Some(deadline),
            ..Default::default()
        }),
    });
    Ok(())
}
