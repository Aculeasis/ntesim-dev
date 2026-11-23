use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::{CastContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, PacketMetadata};
use crate::simulation::{ExactFraction, GameActorConfig, GearProcConfig};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsWorldrainV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::WorldrainDamage { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    if ctx.action_metadata.input_role
        == Some(crate::simulation::metadata::InputActionRole::RedirectSkill)
    {
        for proc in &ctx.config.gear_procs {
            let GearProcConfig::WorldrainDamage {
                cycle_intensity_bonus,
                duration_us,
                ..
            } = proc
            else {
                continue;
            };
            crate::simulation::state::apply_timed_buff(
                ctx.buffs,
                crate::simulation::state::TimedBuffDeclaration::reset(
                    "worldrain.cycle_intensity",
                    "weapon:fork_Worldrain",
                    ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
                    *duration_us,
                    ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([(
                        "cycle_intensity".into(),
                        ExactFraction::from_ratio(*cycle_intensity_bonus)?,
                    )]),
                ),
                ctx.actor_index,
                ctx.world_time_us,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    config: &GameActorConfig,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &config.gear_procs {
        let GearProcConfig::WorldrainDamage { damage_bonus, .. } = proc else {
            continue;
        };
        if packet.element != Some(DamageElement::Cosmos) {
            continue;
        }
        let source = packet
            .native_damage_source_category
            .as_deref()
            .ok_or_else(|| {
                format!(
                    "Worldrain requires damage source classification for Cosmos packet {}",
                    packet.damage_effect_id
                )
            })?;
        match source.strip_prefix("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_") {
            Some("E" | "Q") => {
                modifiers = modifiers.add_damage(ExactFraction::from_ratio(*damage_bonus)?)?;
            }
            Some("A" | "B" | "F" | "H" | "P" | "R" | "T" | "X" | "Z") => {}
            _ => {
                return Err(format!(
                    "unsupported Worldrain damage source classification {source} for {}",
                    packet.damage_effect_id
                ));
            }
        }
    }
    Ok(modifiers)
}
