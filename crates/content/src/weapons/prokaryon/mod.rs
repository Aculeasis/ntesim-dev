use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{CastContext, HitModifiers};
use crate::simulation::metadata::PacketMetadata;
use crate::simulation::{ExactFraction, GameActorConfig, GearProcConfig};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsProkaryonV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::ProkaryonDamage { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    config: &GameActorConfig,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &config.gear_procs {
        let GearProcConfig::ProkaryonDamage { damage_bonus } = proc else {
            continue;
        };
        let source = packet
            .native_damage_source_category
            .as_deref()
            .ok_or_else(|| {
                format!(
                    "Prokaryon requires damage source classification for packet {}",
                    packet.damage_effect_id
                )
            })?;
        match source.strip_prefix("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_") {
            Some("A") => {
                modifiers = modifiers.add_damage(ExactFraction::from_ratio(*damage_bonus)?)?
            }
            Some("B" | "E" | "F" | "H" | "P" | "Q" | "R" | "T" | "X" | "Z") => {}
            _ => {
                return Err(format!(
                    "unsupported Prokaryon damage source classification {source} for {}",
                    packet.damage_effect_id
                ));
            }
        }
    }
    Ok(modifiers)
}
