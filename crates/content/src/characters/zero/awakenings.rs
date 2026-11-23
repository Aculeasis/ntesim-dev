use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

fn is_burst_packet(effect_id: &str) -> bool {
    matches!(
        effect_id,
        "GE_Player_Female046_UltraSkill1_Damage"
            | "GE_Player_Female046_UltraSkill2_Damage"
            | "GE_Player_Female046_UltraSkill3_Damage"
            | "GE_Player_Female046_UltraSkill4_Damage"
    )
}

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) a1_first_hits: std::collections::BTreeSet<(usize, usize)>,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
}

/// per the character-owned 1046AwakenEffect.json A1 description.
pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id == "character:1046"
        && (ctx.damage_effect_id == "GE_Player_Female046_Skill_Kill_Damage_lv1"
            || ctx.damage_effect_id == "GE_Player_Female046_Skill_Kill_Damage_lv2")
    {
        Ok(PARAMETERS.execution_defense_ignore)
    } else {
        ExactFraction::new(0, 1)
    }
}

/// per the character-owned 1046AwakenEffect.json A3 description.
pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id == "character:1046"
        && ctx.config.awakening.has_effect(3)
        && ctx.ability_id == "GA_Female046_UltraSkill"
        && is_burst_packet(ctx.damage_effect_id)
    {
        Ok(PARAMETERS.burst_crit_rate)
    } else {
        ExactFraction::new(0, 1)
    }
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    if ctx.config.character_id == "character:1046"
        && ctx.config.awakening.has_effect(4)
        && ctx.ability_id == "GA_Female046_UltraSkill"
        && is_burst_packet(ctx.damage_effect_id)
    {
        let base_atk = ctx.config.static_stats.base_atk as u128;
        let add = ExactFraction::new(base_atk, 1)?.mul(PARAMETERS.burst_damage_per_base_atk)?;
        let cap = PARAMETERS.burst_damage_cap;
        let add = if add.numerator * cap.denominator >= cap.numerator * add.denominator {
            cap
        } else {
            add
        };
        dmg_bonus = dmg_bonus.add_damage(add)?;
    }
    Ok(dmg_bonus)
}
