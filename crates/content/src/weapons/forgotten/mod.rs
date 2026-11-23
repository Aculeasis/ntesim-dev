use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::{ExactFraction, GearProcConfig};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsForgottenHpThresholdV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::ForgottenHpStats { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn hit_modifiers(
    ctx: &DamageContext<'_>,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::ForgottenHpStats { def_bonus, .. } = proc else {
            continue;
        };
        let hp_twice = u128::from(ctx.actor.hp) * 2;
        let max_hp = u128::from(ctx.actor.hp_max);
        if hp_twice < max_hp {
            modifiers = modifiers.add_def_percent(ExactFraction::from_ratio(*def_bonus)?)?;
        }
    }
    Ok(modifiers)
}
