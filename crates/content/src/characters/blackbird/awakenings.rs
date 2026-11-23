use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::runtime_api::{HitModifiers, RecipientContext};
use crate::simulation::ExactFraction;

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1042" {
        return Ok(dmg_bonus);
    }

    // per 1042AwakenEffect.json / DT_BlackBirdEffectFigure.json Buff_BlackBird042_Level1_Skill_Coef = 1.5
    let is_skill = ctx.damage_effect_id == "GE_Player_BlackBird_Evolve_Skill_Damage";

    if actor_config.awakening.has_effect(1) && is_skill {
        dmg_bonus = dmg_bonus.modify_ratio(PARAMETERS.a1_evolved_skill_ratio_modify)?;
    }
    if actor_config.awakening.has_effect(2)
        && ctx.damage_effect_id == "GE_Player_BlackBird_Skill_Damage"
    {
        dmg_bonus = dmg_bonus.modify_ratio(PARAMETERS.a2_skill_ratio_modify)?;
    }

    // Milestone resonance_3: per 1042AwakenEffect.json / DT_BlackBirdEffectFigure.json Buff_BlackBird042_Level3_Extra_UltraSkill_CofeAdd = 1.0
    if actor_config.awakening.has_resonance_3() {
        if ctx.damage_effect_id == "GE_Player_BlackBird_UltraSkill_Damage" {
            dmg_bonus = dmg_bonus.modify_ratio(PARAMETERS.resonance_burst_ratio_modify)?;
        }
        if ctx.damage_effect_id == "GE_Player_BlackBird_Evolve_UltraSkill_Damage" {
            dmg_bonus = dmg_bonus.modify_ratio(PARAMETERS.resonance_evolved_burst_ratio_modify)?;
        }
    }

    Ok(dmg_bonus)
}

pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1042" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

pub(crate) fn recipient_modifiers(
    ctx: &RecipientContext<'_>,
    mut value: HitModifiers,
) -> Result<HitModifiers, String> {
    // Milestone resonance_6: team Crit DMG at 6 total awakenings
    if ctx.source_config.awakening.has_resonance_6() {
        value = value.add_crit_damage(PARAMETERS.resonance_team_crit_damage)?;
    }
    Ok(value)
}
