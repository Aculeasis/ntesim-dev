use crate::HypothesisId;
use crate::runtime_api::{DamageContext, HitModifiers};
use crate::simulation::{ExactFraction, GearProcConfig};

pub(crate) mod projection;
pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::SetsDevilsBloodV1;
pub(crate) const REACTION_TAGS: [&str; 2] = [
    "Ability.Player.Buff.Reaction.AnHun",
    "Ability.Player.Buff.Reaction.HunXiang",
];

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::DevilsBloodDamage {
            damage_bonus,
            affected_bonus,
        } = proc
        else {
            continue;
        };
        modifiers = modifiers.add_damage(ExactFraction::from_ratio(*damage_bonus)?)?;
        for tag in REACTION_TAGS {
            if crate::simulation::state::target_tagged_status_stacks(
                ctx.buffs,
                ctx.target_index,
                tag,
                ctx.world_time_us,
            )? > 0
            {
                modifiers = modifiers.add_damage(ExactFraction::from_ratio(*affected_bonus)?)?;
                break;
            }
        }
    }
    Ok(modifiers)
}

pub(crate) fn validate_config(
    proc: &GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let GearProcConfig::DevilsBloodDamage {
        damage_bonus,
        affected_bonus,
    } = proc
    else {
        return Err("Devil's Blood validator received the wrong proc kind".into());
    };
    if ctx.actor.set_id.as_deref() != Some("set:Suit3") {
        return Err("Devil's Blood requires equipped set:Suit3".into());
    }
    for ratio in [damage_bonus, affected_bonus] {
        let value = ExactFraction::from_ratio(*ratio)?;
        if value.is_zero() || value.numerator > value.denominator {
            return Err("Devil's Blood requires positive fractional damage bonuses <= 1".into());
        }
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "devils_blood",
        duration_us: 0,
    })
}
