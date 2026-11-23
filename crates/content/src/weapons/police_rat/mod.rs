use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod arc;
pub(crate) mod projection;
use crate::runtime_api::{DamageContext, HitModifiers};
use crate::simulation::metadata::PacketMetadata;
use crate::simulation::{ExactFraction, GearProcConfig};
pub(crate) use arc::State;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsPoliceRatV3;

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    _packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::PoliceRatBoss {
            boss_damage_bonus, ..
        } = proc
        else {
            continue;
        };
        if ctx.target.is_boss {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*boss_damage_bonus)?)?;
        }
    }
    Ok(modifiers)
}
