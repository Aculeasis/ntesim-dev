use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsRealityRefugeV2;

pub(crate) const ATTACHMENT_DAMAGE_TAG: &str = "ntesim.damage.attachment";
const ULT_ATTACHMENT_BUFF_ID: &str = "butterfly.ult_attachment";
const ULT_ATTACHMENT_MODIFIER: &str = "butterfly.ult_attachment_bonus";

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::ButterflyDamage {
            ult_attachment_bonus,
            ult_duration_us,
            ..
        } = proc
        else {
            continue;
        };
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        if ctx.action_metadata.input_role == Some(InputActionRole::Ultimate) {
            let bonus = ExactFraction::from_ratio(*ult_attachment_bonus)?;
            crate::simulation::state::apply_timed_buff(
                ctx.buffs,
                TimedBuffDeclaration::reset(
                    ULT_ATTACHMENT_BUFF_ID,
                    "weapon:fork_Butterfly",
                    BuffRecipient::SelfActor,
                    *ult_duration_us,
                    BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([(ULT_ATTACHMENT_MODIFIER.into(), bonus)]),
                ),
                ctx.actor_index,
                ctx.world_time_us,
            )
            .map_err(|error| format!("Reality Refuge buff grant failed: {error}"))?;
        }
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::ButterflyDamage {
            anima_bonus,
            attachment_bonus,
            ..
        } = proc
        else {
            continue;
        };
        if packet.element == Some(DamageElement::Nature) {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*anima_bonus)?)?;
        }
        if packet.semantic_tags.contains(ATTACHMENT_DAMAGE_TAG) {
            let timed = ctx
                .buff_stats
                .get(ULT_ATTACHMENT_MODIFIER)
                .copied()
                .map(ExactFraction::from_ratio)
                .transpose()?
                .filter(|value| !value.is_zero());
            match timed {
                Some(active) => modifiers = modifiers.add_damage(active)?,
                None => {
                    modifiers =
                        modifiers.add_damage(ExactFraction::from_ratio(*attachment_bonus)?)?;
                }
            }
        }
    }
    Ok(modifiers)
}
