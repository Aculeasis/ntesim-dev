//! B2 listens to Word creation, never to the refreshed reaction's ticks.
use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangModerateMischiefV1;

pub(crate) fn on_word(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let progression = ntesim_data::skill_levels::progression(
        &ctx.config.character_id,
        ctx.config.character_level,
        ctx.config.character_max_level,
    )?;
    if progression["breakthrough"].as_u64().unwrap_or(0) < 2 {
        return Ok(());
    }
    // Refresh the authored aim target. The marked Word recipient may differ
    // when a later area recipient first connects.
    let target = ctx.request.aim_target(ctx.command_index);
    if crate::simulation::state::target_tagged_status_stacks(
        ctx.buffs,
        target,
        crate::characters::packet_semantics::SCORCH_STATUS_TAG,
        ctx.world_time_us,
    )? == 0
    {
        return Ok(());
    }
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let word_generation = ctx.actor.character.baicang_mut()?.word_generation_counter;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.time_us,
        priority: crate::execution::CausalPriority::ReactionOrBreak,
        seq: *ctx.sequence,
        data: GameEventData::ReactionScorchRefresh {
            source_actor_index: ctx.actor_index,
            target_index: target,
            word_generation,
        },
    });
    Ok(())
}
