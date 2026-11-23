use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::DamageElement;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameActorConfig, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsDemonBladeV1;

pub(crate) const BUFF_ID: &str = "demon_blade.crit_damage";
pub(crate) const SOURCE: &str = "weapon:fork_DemonBlade";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) ready_at_us: u64,
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if matches!(proc, GearProcConfig::DemonBlade { .. }) {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
        }
    }
    Ok(())
}

pub(crate) fn crit_rate_bonus(config: &GameActorConfig) -> Result<ExactFraction, String> {
    let mut total = ExactFraction::new(0, 1)?;
    for proc in &config.gear_procs {
        if let GearProcConfig::DemonBlade {
            crit_rate_bonus, ..
        } = proc
        {
            total = total.add(ExactFraction::from_ratio(*crit_rate_bonus)?)?;
        }
    }
    Ok(total)
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let is_incantation = ctx.packet_metadata.as_ref().is_some_and(|p| {
        p.element == Some(DamageElement::Incantation)
            || p.native_damage_type_ex == "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION"
    });
    if !is_incantation {
        return Ok(());
    }

    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::DemonBlade {
            crit_damage_per_stack,
            duration_us,
            icd_us,
            ..
        } = proc
        {
            if ctx.time_us < ctx.actor.equipment.demon_blade.ready_at_us {
                continue;
            }
            ctx.actor.equipment.demon_blade.ready_at_us = ctx
                .time_us
                .checked_add(*icd_us)
                .ok_or("Demon Blade cooldown deadline overflow")?;

            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::DemonBladeCritStack {
                    actor_index: ctx.actor_index,
                    stack_amount: *crit_damage_per_stack,
                    duration_us: *duration_us,
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    stack_amount: Ratio,
    duration_us: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    state.apply_buff(
        actor_index,
        TimedBuffDeclaration {
            id: BUFF_ID.into(),
            source: SOURCE.into(),
            recipient: BuffRecipient::SpecificActor(actor_index),
            duration_us,
            stacking_scope: BuffStackingScope::PerSource,
            refresh_policy: RefreshPolicy::Reset,
            max_stacks: 7,
            decay_policy: DecayPolicy::AllAtOnce,
            stat_modifiers: std::collections::BTreeMap::from([(
                "crit_damage_bonus".into(),
                ExactFraction::from_ratio(stack_amount)?,
            )]),
        },
    )?;
    let deadline = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("Demon Blade expiry overflow")?;
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
        .filter(|b| b.id == BUFF_ID && b.source == SOURCE && b.owner_actor == actor_index)
        .map(|b| b.stacks)
        .sum::<u32>();
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "demon_blade_crit_stack".into(),
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
