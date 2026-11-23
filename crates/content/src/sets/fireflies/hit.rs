use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::CallbackContext;
use crate::simulation::{
    GearProcConfig,
    events::{GameEventData, ScheduledGameEvent},
};
use ntesim_core::execution::CausalPriority;

pub(crate) const TEAM_HIT_HYPOTHESIS_ID: HypothesisId = HypothesisId::FirefliesTeamAnimaHitV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::FirefliesAllInRangeV1;

pub(crate) fn on_research_hit(ctx: &mut crate::runtime_api::CastContext<'_>) -> Result<(), String> {
    let Some(packet) = ctx.packet_metadata else {
        return Ok(());
    };
    if packet.damage_kind != ntesim_core::combat::DamageKind::Elemental
        || packet.element != Some(crate::simulation::metadata::DamageElement::Nature)
    {
        return Ok(());
    }
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::FirefliesCritDamage {
            crit_damage_per_stack,
            max_stacks,
            duration_us,
        } = proc
        {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::FirefliesCritStack {
                    actor_index: ctx.actor_index,
                    stack_amount: *crit_damage_per_stack,
                    max_stacks: *max_stacks,
                    duration_us: *duration_us,
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn on_team_hit(
    ctx: &mut CallbackContext<'_>,
    source_actor_index: usize,
    packet: &crate::simulation::metadata::PacketMetadata,
    time_us: u64,
    execution_assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    let is_anima = packet.damage_kind == ntesim_core::combat::DamageKind::Elemental
        && packet.element == Some(crate::simulation::metadata::DamageElement::Nature);
    if !is_anima {
        return Ok(());
    }
    if ctx.state.actors[source_actor_index].hp == 0 {
        return Ok(());
    }
    let wearers = ctx
        .request
        .actors
        .iter()
        .enumerate()
        .filter(|(index, actor)| {
            ctx.state.actors[*index].hp > 0
                && actor
                    .gear_procs
                    .iter()
                    .any(|proc| matches!(proc, GearProcConfig::FirefliesCritDamage { .. }))
        })
        .collect::<Vec<_>>();
    if wearers.is_empty() {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        execution_assumptions,
        TEAM_HIT_HYPOTHESIS_ID,
    )?;
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        execution_assumptions,
        ALL_IN_RANGE_HYPOTHESIS_ID,
    )?;

    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;
    for (actor_index, actor_config) in wearers {
        for proc in &actor_config.gear_procs {
            if let GearProcConfig::FirefliesCritDamage {
                crit_damage_per_stack,
                max_stacks,
                duration_us,
            } = proc
            {
                crate::runtime_api::increment_sequence(seq)?;
                queue.push(ScheduledGameEvent {
                    time_us,
                    priority: CausalPriority::ProcCallback,
                    seq: *seq,
                    data: GameEventData::Content(ContentEvent::FirefliesCritStack {
                        actor_index,
                        stack_amount: *crit_damage_per_stack,
                        max_stacks: *max_stacks,
                        duration_us: *duration_us,
                    }),
                });
            }
        }
    }
    Ok(())
}
