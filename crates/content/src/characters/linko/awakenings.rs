use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) pet_active: bool,
    pub(crate) pulse_generation: u64,
    pub(crate) pulse_array_index: usize,
    pub(crate) pulse_command_index: Option<usize>,
    pub(crate) next_pulse_us: Option<u64>,
    pub(crate) oversync_until_us: Option<u64>,
    pub(crate) hexed_targets: std::collections::BTreeMap<usize, u64>,
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1072" {
        return Ok(dmg_bonus);
    }

    let is_ultra = ctx
        .action_metadata
        .categories
        .contains(&crate::simulation::metadata::ActionCategory::UltraSkill);
    let is_skill = ctx
        .action_metadata
        .categories
        .contains(&crate::simulation::metadata::ActionCategory::Skill);

    // Individual node Effect2: 1072AwakenEffect.json Effect2
    if actor_config.awakening.has_effect(2) && is_ultra {
        dmg_bonus = dmg_bonus.add_ratio(PARAMETERS.a2_burst_ratio_add)?;
    }

    // Individual node Effect5: 1072AwakenEffect.json / DT_LinkoEffectFigure.json LinkoLevel3_DamageUpGeneralBase = 0.50
    if actor_config.awakening.has_effect(5) && is_skill && !is_ultra {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.a5_skill_damage_bonus)?;
    }

    // Milestone resonance_3 (Linko_Extra1_AtkUp = 0.20): activates at 3 total selected awakenings
    if actor_config.awakening.has_resonance_3() {
        dmg_bonus = dmg_bonus.add_atk_percent(PARAMETERS.resonance_atk_percent)?;
    }

    Ok(dmg_bonus)
}

pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1072" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1072" {
        return ExactFraction::new(0, 1);
    }

    let is_skill = ctx
        .action_metadata
        .categories
        .contains(&crate::simulation::metadata::ActionCategory::Skill);
    let is_ultra = ctx
        .action_metadata
        .categories
        .contains(&crate::simulation::metadata::ActionCategory::UltraSkill);

    let is_lte_or_xiaozhen = matches!(
        ctx.damage_effect_id,
        "GE_Player_Radio072_LTE_Damage"
            | "GE_Player_Radio072_LTE_Xiaozhen_Damage"
            | "GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage"
            | "GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage2"
            | "GE_Xiaozhen_Skill1_Damage"
    );

    // per 1072AwakenEffect.json / DT_LinkoEffectFigure.json LinkoLevel6_CritBase = 0.25
    if ctx.config.awakening.has_effect(6) && ((is_skill && !is_ultra) || is_lte_or_xiaozhen) {
        Ok(PARAMETERS.a6_skill_crit_rate)
    } else {
        ExactFraction::new(0, 1)
    }
}

pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1072" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}
