use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext, DamageContext, HitModifiers};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{ActionMetadata, InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsGoldRecordV2;

pub(crate) const BUFF_ID: &str = "gold_record.ultimate_crit_damage";
pub(crate) const SOURCE: &str = "weapon:fork_GoldRecord";

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if matches!(proc, GearProcConfig::GoldRecord { .. }) {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
        }
    }
    Ok(())
}

pub(crate) fn hit_modifiers(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::GoldRecord {
            qte_crit_damage_bonus,
            ..
        } = proc
        {
            if matches_role(ctx.action_metadata, packet, InputActionRole::Qte, "H") {
                modifiers = modifiers
                    .add_crit_damage(ExactFraction::from_ratio(*qte_crit_damage_bonus)?)?;
            }
            if matches_role(ctx.action_metadata, packet, InputActionRole::Ultimate, "Q")
                && let Some(ratio) = ctx.buff_stats.get("gold_record_ult_crit")
            {
                modifiers = modifiers.add_crit_damage(ExactFraction::from_ratio(*ratio)?)?;
            }
        }
    }
    Ok(modifiers)
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx
        .packet_metadata
        .is_some_and(|packet| matches_role(ctx.action_metadata, packet, InputActionRole::Qte, "H"))
    {
        return Ok(());
    }
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::GoldRecord {
            ult_crit_damage_per_stack,
            duration_us,
            ..
        } = proc
        {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::GoldRecordUltCritStack {
                    actor_index: ctx.actor_index,
                    stack_amount: *ult_crit_damage_per_stack,
                    duration_us: *duration_us,
                }),
            });
        }
    }
    Ok(())
}

fn matches_role(
    action: &ActionMetadata,
    packet: &PacketMetadata,
    role: InputActionRole,
    category: &str,
) -> bool {
    action.input_role == Some(role)
        && packet
            .native_damage_source_category
            .as_deref()
            .is_none_or(|native| {
                native.strip_prefix("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_")
                    == Some(category)
            })
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
            max_stacks: 3,
            decay_policy: DecayPolicy::AllAtOnce,
            stat_modifiers: std::collections::BTreeMap::from([(
                "gold_record_ult_crit".into(),
                ExactFraction::from_ratio(stack_amount)?,
            )]),
        },
    )?;
    let deadline = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("Voice of the Voyager expiry overflow")?;
    if deadline <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: deadline,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::BuffLifecycleExpiry),
        });
    }
    let fraction = ExactFraction::from_ratio(stack_amount)?;
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "gold_record_stack".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            crit_damage_bonus: Some(fraction),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}
