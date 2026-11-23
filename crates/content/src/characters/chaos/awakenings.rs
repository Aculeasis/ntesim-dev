use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1071" {
        return Ok(dmg_bonus);
    }
    dmg_bonus = super::remora::damage_bonus(ctx, dmg_bonus)?;

    let has_warrant = ctx.has_status("chaos.warrant");

    // per DT_ChaosEffectFigure.json Chaos_HuntMarkDamageUp = 0.20
    if has_warrant {
        let cached = match &ctx.actor.character {
            crate::characters::RuntimeState::Chaos(state) => state.warrant_bonus,
            _ => None,
        };
        let bonus = match cached {
            Some(bonus) => bonus,
            None => super::parameters::warrant_bonus(actor_config)?,
        };
        dmg_bonus = dmg_bonus.add_damage(bonus)?;
    }

    // per 1071AwakenEffect.json / DT_ChaosEffectFigure.json Chaos_Star5DamageUp = 0.20
    if actor_config.awakening.has_effect(5)
        && has_warrant
        && ctx.owned_status_counts.get("chaos.warrant") == Some(&1)
    {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.a5_single_target_damage_bonus)?;
    }

    Ok(dmg_bonus)
}

pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1071" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

/// per 1071AwakenEffect.json / DT_ChaosEffectFigure.json Chaos_Star2DefIgnore = 0.20.
pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1071" {
        return ExactFraction::new(0, 1);
    }

    if ctx.config.awakening.has_effect(2) && ctx.has_status("chaos.warrant") {
        Ok(PARAMETERS.a2_defense_ignore)
    } else {
        ExactFraction::new(0, 1)
    }
}
