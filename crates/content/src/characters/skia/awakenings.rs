use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1008" {
        return Ok(dmg_bonus);
    }

    // per 1008AwakenEffect.json / DT_SkiaEffectFigure.json Skia_Leve3DamageUpGeneralBase1/2 = 0.04
    if actor_config.awakening.has_effect(3) && ctx.has_status("skia.fang_thrust") {
        dmg_bonus = dmg_bonus.add_damage(
            if ctx.owned_status_counts.get("skia.fang_thrust") == Some(&1) {
                PARAMETERS.a3_single_mark_damage_bonus
            } else {
                PARAMETERS.a3_multiple_mark_damage_bonus
            },
        )?;
    }

    // per 1008AwakenEffect.json / DT_SkiaEffectFigure.json Skia_Leve1CoefModify = 0.60
    if actor_config.awakening.has_effect(1)
        && ctx.damage_effect_id == "GE_Player_Skia_SkillShadowAtk_Damage"
    {
        dmg_bonus = dmg_bonus.add_ratio(PARAMETERS.a1_shadow_skill_ratio_add)?;
    }

    Ok(dmg_bonus)
}
