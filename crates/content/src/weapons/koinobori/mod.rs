use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::GearProcConfig;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsKoinoboriV2;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::KoinoboriStats { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn hit_modifiers(
    ctx: &DamageContext<'_>,
    modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    modifiers.add_def_percent(crate::gear::source_stats::def_percent(
        ctx.config,
        ctx.team_has_three_elements,
    )?)
}
