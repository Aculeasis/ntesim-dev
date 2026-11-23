use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::CastContext;
use crate::simulation::GearProcConfig;
use crate::simulation::metadata::PacketMetadata;
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
};
use ntesim_core::formula::Ratio;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsNestBirdV2;

const MARK_ID: &str = "nest_bird.warning_gaze";
const SOURCE: &str = "weapon:fork_NestBird";
const STAT_KEY: &str = "outgoing_damage_reduction";
const SEMANTIC_TAG: &str = "nest_bird.warning_gaze";

pub(crate) fn on_hit(ctx: &mut CastContext<'_>, target_index: usize) -> Result<(), String> {
    let procs: Vec<(Ratio, u64)> = ctx
        .config
        .gear_procs
        .iter()
        .filter_map(|proc| match proc {
            GearProcConfig::NestBirdGaze {
                damage_reduction,
                duration_us,
            } => Some((*damage_reduction, *duration_us)),
            _ => None,
        })
        .collect();
    let packet = ctx
        .packet_metadata
        .ok_or("Tears Beneath the Mask on_hit requires packet metadata")?;
    for (damage_reduction, duration_us) in procs {
        if is_ultimate_hit(packet)? {
            mark(ctx, target_index, damage_reduction, duration_us)?;
        }
    }
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::NestBirdGaze { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

fn is_ultimate_hit(packet: &PacketMetadata) -> Result<bool, String> {
    let source = packet
        .native_damage_source_category
        .as_deref()
        .ok_or_else(|| {
            format!(
                "Tears Beneath the Mask requires damage source classification for Ultimate gating on {}",
                packet.damage_effect_id
            )
        })?;
    match source.strip_prefix("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_") {
        Some("Q") => Ok(true),
        Some("A" | "B" | "E" | "F" | "H" | "P" | "R" | "T" | "X" | "Z") => Ok(false),
        _ => Err(format!(
            "unsupported Tears Beneath the Mask damage source classification {source} for {}",
            packet.damage_effect_id
        )),
    }
}

fn mark(
    ctx: &mut CastContext<'_>,
    target_index: usize,
    damage_reduction: Ratio,
    duration_us: u64,
) -> Result<(), String> {
    let definition = BuffDefinition {
        stacking_scope: BuffStackingScope::PerSource,
        semantic_tags: BTreeSet::from([SEMANTIC_TAG.into()]),
        source_lifetime: Default::default(),
        id: MARK_ID.into(),
        source: SOURCE.into(),
        recipient: BuffRecipient::SpecificTarget(target_index.into()),
        duration_us: Some(duration_us),
        refresh_policy: RefreshPolicy::Reset,
        max_stacks: 1,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: BTreeMap::from([(STAT_KEY.into(), damage_reduction)]),
    };
    ctx.buffs
        .apply_buff(definition, ctx.actor_index, ctx.world_time_us)
        .map_err(|error| format!("Tears Beneath the Mask mark application failed: {error:?}"))?;
    Ok(())
}
