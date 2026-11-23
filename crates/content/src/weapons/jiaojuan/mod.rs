use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{DamageContext, HitModifiers};
use crate::simulation::metadata::PacketMetadata;
use crate::simulation::{ExactFraction, GearProcConfig};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsJiaojuanV2;

pub(crate) const BROKEN_STATUS_TAG: &str = "ntesim.status.broken";

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    _packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::JiaojuanBreak {
            broken_damage_bonus,
            ..
        } = proc
        else {
            continue;
        };
        let broken = crate::simulation::state::target_tagged_status_stacks(
            ctx.buffs,
            ctx.target_index,
            BROKEN_STATUS_TAG,
            ctx.world_time_us,
        )?;
        if broken > 0 {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*broken_damage_bonus)?)?;
        }
    }
    Ok(modifiers)
}
