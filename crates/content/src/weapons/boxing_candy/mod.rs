use crate::HypothesisId;
/// Live target HP selects one source-bound damage bonus before each hit.
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::{ExactFraction, GearProcConfig};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsBoxingCandyV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::BoxingCandyDamage { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::BoxingCandyDamage {
            damage_bonus,
            low_hp_damage_bonus,
        } = proc
        else {
            continue;
        };
        let bonus = if u128::from(ctx.target.hp_current) * 2 < u128::from(ctx.target.hp_max) {
            *low_hp_damage_bonus
        } else {
            *damage_bonus
        };
        modifiers = modifiers.add_damage(ExactFraction::from_ratio(bonus)?)?;
    }
    Ok(modifiers)
}
