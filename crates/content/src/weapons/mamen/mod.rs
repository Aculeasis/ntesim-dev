use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod projection;
use crate::runtime_api::{DamageContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, PacketMetadata};
use crate::simulation::{ExactFraction, GearProcConfig};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsContemplativeCatV1;

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::MamenFons {
            cosmos_bonus_per_stack,
            fons_per_stack,
            max_stacks,
        } = proc
        else {
            continue;
        };
        let fons_held = ctx.start_fons.ok_or(
            "Contemplative Cat requires an explicit party start_fons value when its passive is reached",
        )?;
        if *fons_per_stack == 0 {
            return Err("Contemplative Cat requires a positive fons_per_stack".into());
        }
        if packet.element == Some(DamageElement::Cosmos) {
            let stacks = (fons_held / *fons_per_stack).min(u64::from(*max_stacks));
            let bonus = ExactFraction::from_ratio(*cosmos_bonus_per_stack)?
                .mul(ExactFraction::new(u128::from(stacks), 1)?)?;
            modifiers = modifiers.add_damage(bonus)?;
        }
    }
    Ok(modifiers)
}
