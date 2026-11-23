use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext, DamageContext, HitModifiers};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::PacketMetadata;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsOraoraV1;

const BUFF_ID: &str = "oraora.basic_attack_damage";
const SOURCE: &str = "weapon:fork_oulaquantao";
const CATEGORY_A: &str = "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A";

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::OraoraBasicAttackDamage { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .as_ref()
        .and_then(|packet| packet.native_damage_source_category.as_deref())
        != Some(CATEGORY_A)
    {
        return Ok(());
    }
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::OraoraBasicAttackDamage {
            damage_bonus_per_stack,
            duration_us,
        } = proc
        {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::OraoraBasicAttackStack {
                    actor_index: ctx.actor_index,
                    stack_amount: *damage_bonus_per_stack,
                    duration_us: *duration_us,
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if packet.native_damage_source_category.as_deref() != Some(CATEGORY_A)
        || !ctx
            .config
            .gear_procs
            .iter()
            .any(|proc| matches!(proc, GearProcConfig::OraoraBasicAttackDamage { .. }))
    {
        return Ok(modifiers);
    }
    let active = ctx.buff_stats;
    if let Some(bonus) = active.get("oraora_basic_attack_damage_bonus") {
        modifiers = modifiers.add_damage(ExactFraction::from_ratio(*bonus)?)?;
    }
    Ok(modifiers)
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    stack_amount: Ratio,
    duration_us: u64,
) -> Result<(), String> {
    if ctx
        .state
        .actors
        .get(actor_index)
        .is_none_or(|actor| actor.hp == 0)
    {
        return Ok(());
    }
    let state = &mut *ctx.state;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let active_stacks = state
        .buffs
        .instances()
        .iter()
        .filter(|buff| {
            buff.id == BUFF_ID && buff.source == SOURCE && buff.owner_actor == actor_index
        })
        .map(|buff| buff.stacks)
        .sum::<u32>();
    let admitted = active_stacks < 10;
    let deadline = admitted
        .then(|| {
            state
                .current_time_us
                .checked_add(duration_us)
                .ok_or("Oraora! expiry overflow")
        })
        .transpose()?;
    if admitted {
        state.apply_buff(
            actor_index,
            TimedBuffDeclaration {
                id: BUFF_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::SpecificActor(actor_index),
                duration_us,
                stacking_scope: BuffStackingScope::PerSource,
                refresh_policy: RefreshPolicy::Independent,
                max_stacks: 10,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: std::collections::BTreeMap::from([(
                    "oraora_basic_attack_damage_bonus".into(),
                    ExactFraction::from_ratio(stack_amount)?,
                )]),
            },
        )?;
        if deadline.is_some_and(|deadline| deadline <= ctx.request.absolute_limit_us) {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: deadline.unwrap(),
                priority: CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::BuffLifecycleExpiry),
            });
        }
    }
    let stacks = state
        .buffs
        .instances()
        .iter()
        .filter(|buff| {
            buff.id == BUFF_ID && buff.source == SOURCE && buff.owner_actor == actor_index
        })
        .map(|buff| buff.stacks)
        .sum::<u32>();
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: if admitted {
            "oraora_basic_attack_stack"
        } else {
            "oraora_stack_capped"
        }
        .into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            admitted: Some(admitted),
            stacks: Some(stacks),
            expires_at_us: deadline,
            ..Default::default()
        }),
    });
    Ok(())
}
