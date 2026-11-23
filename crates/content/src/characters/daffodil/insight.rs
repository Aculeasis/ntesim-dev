//! Source-owned target Insight is distinct from the thirty-second self Eyes clock.
use super::parameters::PARAMETERS;
use crate::runtime_api::CastContext;
use crate::simulation::state::{TargetStatusDeclaration, apply_target_tagged_status};
use ntesim_core::buffs::dispatch::{DecayPolicy, RefreshPolicy};

pub(crate) const HYPOTHESIS: crate::HypothesisId =
    crate::HypothesisId::DaffodilInsightBreakCaptureV1;
const STATUS: &str = "ntesim.daffodil.insight";
const SOURCE: &str = "character:1054.burst";

fn tag(source_actor: usize) -> String {
    format!("{STATUS}.source:{source_actor}")
}

pub(crate) fn on_burst(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(HYPOTHESIS)?;
    let awakened = ctx.config.awakening.has_effect(3);
    for (index, target) in ctx.targets.iter().enumerate() {
        if target.hp_current == 0 {
            continue;
        }
        apply_target(
            ctx.buffs,
            ctx.actor_index,
            index,
            awakened,
            ctx.world_time_us,
        )?;
    }
    ctx.actor.character.daffodil_mut()?.insight_until_us = ctx
        .time_us
        .checked_add(PARAMETERS.self_insight_duration_us)
        .ok_or("Daffodil self Insight deadline overflow")?;
    Ok(())
}

pub(crate) fn apply_target(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    source_actor: usize,
    target_index: usize,
    awakened: bool,
    now_us: u64,
) -> Result<(), String> {
    apply_target_tagged_status(
        buffs,
        TargetStatusDeclaration {
            source_actor,
            source: SOURCE.into(),
            target_index,
            id: format!("{STATUS}.target:{target_index}"),
            now_us,
            duration_us: if awakened {
                PARAMETERS.a3_target_insight_duration_us
            } else {
                PARAMETERS.target_insight_duration_us
            },
            reductions: Default::default(),
        },
        std::collections::BTreeSet::from([tag(source_actor)]),
        if awakened { 2 } else { 1 },
        RefreshPolicy::Reset,
        DecayPolicy::AllAtOnce,
    )
}

pub(crate) fn clear_target(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
    team_size: usize,
) {
    let id = format!("{STATUS}.target:{target_index}");
    for source_actor in 0..team_size {
        buffs.remove_owned_buff(&id, SOURCE, source_actor);
    }
}

pub(crate) fn stacks(ctx: &crate::runtime_api::BreakContext<'_>) -> Result<u32, String> {
    let count = ctx
        .buffs
        .active_tagged_stacks_for_target_index(
            ctx.target_index,
            &tag(ctx.actor_index),
            ctx.world_time_us,
        )
        .map_err(|error| format!("Daffodil target Insight query failed: {error:?}"))?;
    if count > 2 {
        return Err("Daffodil target Insight exceeds source stack cap".into());
    }
    Ok(count)
}
