use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) paid_burst_until_us: u64,
    pub(crate) surging_crimson_until_us: u64,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) fading_reason: u32,
    pub(crate) scarlet_descent_casts: u32,
    pub(crate) crimson_judgment_stacks: u32,
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1076" {
        return Ok(dmg_bonus);
    }

    let is_surging_crimson = matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Shinku(state) if state.surging_crimson_until_us >= ctx.time_us
    );

    // per DT_ShinkuEffectFigure.json Shinku_Rage_DmgUp = 0.30
    if is_surging_crimson {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.rage_damage_bonus)?;
    }

    // per 1076AwakenEffect.json / DT_ShinkuEffectFigure.json Shinku_Rage_DmgUpEx_L3 = 0.30
    if actor_config.awakening.has_resonance_3() && is_surging_crimson {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.resonance_rage_damage_bonus)?;
    }

    // per 1076AwakenEffect.json / DT_ShinkuEffectFigure.json Shinku_RageSkillDmgCoefL6 = 0.30
    let is_rage_skill = ctx.action_metadata.categories.iter().any(|category| {
        matches!(
            category,
            crate::simulation::metadata::ActionCategory::Skill
                | crate::simulation::metadata::ActionCategory::UltraSkill
        )
    });

    if actor_config.awakening.has_effect(6) && is_surging_crimson && is_rage_skill {
        dmg_bonus = dmg_bonus.add_ratio(PARAMETERS.a6_rage_skill_ratio_add)?;
    }

    Ok(dmg_bonus)
}

pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1076" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

/// per 1076AwakenEffect.json / DT_ShinkuEffectFigure.json Shinku_CritUp_L6Ex = 0.10.
pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1076" {
        return ExactFraction::new(0, 1);
    }

    if ctx.config.awakening.has_resonance_6() {
        Ok(PARAMETERS.resonance_crit_rate)
    } else {
        ExactFraction::new(0, 1)
    }
}

/// per 1076AwakenEffect.json / DT_ShinkuEffectFigure.json Shinku_DefIgnore_L1 = 0.12 and Shinku_DefIgnore_L6Extra = 0.15.
pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1076" {
        return ExactFraction::new(0, 1);
    }

    let effect_bonus = if ctx.config.awakening.has_effect(1) {
        PARAMETERS.a1_defense_ignore
    } else {
        ExactFraction::new(0, 1)?
    };
    let resonance_bonus = if ctx.config.awakening.has_resonance_6() {
        PARAMETERS.resonance_defense_ignore
    } else {
        ExactFraction::new(0, 1)?
    };
    effect_bonus.add(resonance_bonus)
}
