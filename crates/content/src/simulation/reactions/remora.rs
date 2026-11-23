use super::{EventContext, ExactFraction, ReactionSnapshot, trace};
use crate::simulation::{GameEnergySource, GameEventPayload, GameEventReceipt};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffManager, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use std::collections::BTreeSet;

pub(super) const DURATION_US: u64 = 5_000_000;
const STATUS_ID: &str = "ntesim.reaction.remora";
pub(crate) const STATUS_TAG: &str = "ntesim.status.remora";
const CRIT_STAT: &str = "reaction_remora_crit_rate";

#[derive(Clone, Debug)]
pub(super) struct Generation {
    id: u64,
    applied_us: u64,
    pub(super) expires_us: u64,
    command: usize,
}

pub(super) fn apply(ctx: &mut EventContext<'_>, snapshot: ReactionSnapshot) -> Result<(), String> {
    let target = ctx.request.aim_target(snapshot.command_index);
    let mut assumptions = Default::default();
    let policy = crate::registry::remora_enhancement(ctx.request, ctx.state, &mut assumptions)?;
    let duration_us = policy
        .as_ref()
        .map_or(DURATION_US, |policy| policy.duration_us);
    let bonus = policy.map_or(crate::formula::Ratio::zero(), |policy| {
        policy.crit_rate_bonus
    });
    let now = ctx.state.current_time_us;
    let expires_us = now
        .checked_add(duration_us)
        .ok_or("Remora deadline overflow")?;
    let generation = ctx.state.reactions.next_id;
    ctx.state.reactions.next_id = generation
        .checked_add(1)
        .ok_or("reaction identity overflow")?;
    ctx.state.reactions.remora.insert(
        target,
        Generation {
            id: generation,
            applied_us: now,
            expires_us,
            command: snapshot.command_index,
        },
    );
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                source: super::REACTION_SOURCE.into(),
                id: STATUS_ID.into(),
                recipient: BuffRecipient::SpecificTarget(target.into()),
                stacking_scope: BuffStackingScope::Shared,
                source_lifetime: Default::default(),
                duration_us: Some(duration_us),
                max_stacks: 1,
                refresh_policy: RefreshPolicy::Reset,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: std::collections::BTreeMap::from([(CRIT_STAT.into(), bonus)]),
                semantic_tags: BTreeSet::from([STATUS_TAG.into()]),
            },
            snapshot.owner_index,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|error| format!("Remora application failed: {error:?}"))?;
    ctx.state.execution_model_assumptions.extend(assumptions);
    let armed = crate::registry::on_remora_created(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        target,
        generation,
    )?;
    // Refresh retires the old callback even when the new duration is shorter.
    ctx.queue.retain(|event| {
        !matches!(
            event.data,
            crate::simulation::events::GameEventData::ReactionRemoraEnd {
                target_index,
                ..
            } if target_index == target
        )
    });
    if armed {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: expires_us,
                priority: crate::execution::CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::ReactionRemoraEnd {
                    target_index: target,
                    generation,
                },
            });
    }
    trace(
        ctx,
        "reaction_remora_applied",
        Some(snapshot.owner_index),
        Some(GameEventPayload {
            target_index: Some(target),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    );
    Ok(())
}

pub(in crate::simulation) fn end(
    ctx: &mut EventContext<'_>,
    target: usize,
    generation: u64,
) -> Result<(), String> {
    let Some(current) = ctx.state.reactions.remora.get(&target) else {
        return Ok(());
    };
    if current.id != generation || ctx.state.current_time_us < current.expires_us {
        return Ok(());
    }
    remove(ctx, target)
}

/// The shared non-death removal boundary; refresh replaces the generation directly.
pub(in crate::simulation) fn remove(
    ctx: &mut EventContext<'_>,
    target: usize,
) -> Result<(), String> {
    let Some(current) = ctx.state.reactions.remora.remove(&target) else {
        return Ok(());
    };
    ctx.queue.retain(|event| !matches!(event.data,
        crate::simulation::events::GameEventData::ReactionRemoraEnd { target_index, .. } if target_index == target));
    clear_target(&mut ctx.state.buffs, target);
    let elapsed = ctx.state.current_time_us.saturating_sub(current.applied_us);
    crate::registry::on_remora_removed(
        &mut crate::runtime_api::CallbackContext {
            state: ctx.state,
            request: ctx.request,
            queue: ctx.queue,
            sequence: ctx.sequence,
            trace: ctx.trace,
        },
        target,
        current.id,
        elapsed,
        current.command,
    )
}

pub(crate) fn discard_unobserved(ctx: &mut crate::runtime_api::CallbackContext<'_>) {
    ctx.queue.retain(|event| match event.data {
        crate::simulation::events::GameEventData::ReactionRemoraEnd {
            target_index,
            generation,
        } => crate::registry::has_remora_listener(ctx.state, target_index, generation),
        _ => true,
    });
}

pub(crate) fn cancel_target(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target: usize,
) -> Result<(), String> {
    let Some(current) = ctx.state.reactions.remora.remove(&target) else {
        return Ok(());
    };
    clear_target(&mut ctx.state.buffs, target);
    ctx.queue.retain(|event| !matches!(event.data,
        crate::simulation::events::GameEventData::ReactionRemoraEnd { target_index, .. } if target_index == target));
    crate::registry::on_remora_removed(
        ctx,
        target,
        current.id,
        ctx.state.current_time_us.saturating_sub(current.applied_us),
        current.command,
    )
}

pub(crate) fn crit_rate_bonus(
    ctx: &crate::runtime_api::DamageContext<'_>,
) -> Result<ExactFraction, String> {
    if ctx.actor.hp == 0 {
        return ExactFraction::new(0, 1);
    }
    let stats = ctx
        .buffs
        .get_stat_modifiers_for_target(&ctx.target_index.to_string(), ctx.world_time_us)
        .map_err(|error| format!("Remora target query failed: {error:?}"))?;
    ExactFraction::from_ratio(
        stats
            .get(CRIT_STAT)
            .copied()
            .unwrap_or(crate::formula::Ratio::zero()),
    )
}

pub(super) fn is_active(ctx: &EventContext<'_>, target_index: usize) -> Result<bool, String> {
    Ok(crate::simulation::state::target_tagged_status_stacks(
        &ctx.state.buffs,
        target_index,
        STATUS_TAG,
        ctx.state.mode_clock.relative_us(),
    )? > 0)
}

pub(super) fn clear_target(buffs: &mut BuffManager, target_index: usize) {
    buffs.remove_target_buff(STATUS_ID, super::REACTION_SOURCE, &target_index.to_string());
}

/// Called once after a complete pistil emission, even if several targets qualify.
pub(super) fn charge(
    ctx: &mut EventContext<'_>,
    snapshot: &ReactionSnapshot,
    bud_id: u64,
) -> Result<(), String> {
    // Suppress the Charge reaction before owner overrides can consume an ICD
    // or create a receipt. Ordinary/direct packet energy keeps its own rules.
    if ctx.state.actors.iter().any(|actor| actor.hp > 0 && matches!(&actor.character,
        crate::characters::RuntimeState::Hotori(owner) if owner.timepiece.mode_active(ctx.state.current_time_us))) {
        return Ok(());
    }
    let recipient = ctx.state.active_actor_index;
    if ctx.state.actors[recipient].hp == 0 {
        return Ok(());
    }
    let mut assumptions = Default::default();
    let bonus = crate::registry::charge_reaction_bonus(ctx.request, ctx.state, &mut assumptions)?;
    let raw = crate::registry::charge_reaction_override(ctx.request, ctx.state, &mut assumptions)?
        .unwrap_or(10 + bonus);
    let actor = &mut ctx.state.actors[recipient];
    let before = crate::simulation::energy::EnergySnapshot::capture(actor);
    let gross = actor.grant_energy(raw)?;
    ctx.state
        .reactions
        .record_reaction(super::ReactionKind::Charge);
    ctx.state.execution_model_assumptions.extend(assumptions);
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "reaction_charge_energy".into(),
        active_actor: recipient,
        hit_index: None,
        from_actor: Some(snapshot.owner_index),
        to_actor: Some(recipient),
        energy_change: Some(before.receipt(
            actor,
            GameEnergySource::ReactionCharge,
            snapshot.owner_index,
            recipient,
            Some(ExactFraction::new(raw.into(), 1)?),
        )?),
        energy_fanout: None,
        data: Some(GameEventPayload {
            bud_id: Some(bud_id),
            gross: Some(gross),
            ..Default::default()
        }),
    });
    let mut cb = crate::runtime_api::CallbackContext {
        request: ctx.request,
        state: ctx.state,
        queue: ctx.queue,
        sequence: ctx.sequence,
        trace: ctx.trace,
    };
    crate::registry::on_charge_grant(&mut cb, recipient)?;
    Ok(())
}
