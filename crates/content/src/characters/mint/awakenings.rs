use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[allow(dead_code)]
pub(crate) const A3_REACTION_TRIGGER_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::MintA3ReactionTriggerV1;
#[cfg(test)]
pub(crate) const A3_REACTION_TRIGGER_HYPOTHESIS: &str =
    A3_REACTION_TRIGGER_HYPOTHESIS_ID.description();
#[allow(dead_code)]
pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;
    let target = ctx.target;
    // per 1019AwakenEffect1.json / DT_MintEffectFigure.json Mint_Adcance_2_1
    if actor_config.character_id == "character:1019"
        && actor_config.awakening.has_effect(2)
        && ability_id == "GA_Mint019_Skill"
    {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.skill_damage_bonus)?;
    }

    // per 1019AwakenEffect1.json / DT_MintEffectFigure.json Mint_Adcance_6_1 (0.40), Mint_Adcance_6_2 (0.15)
    if actor_config.character_id == "character:1019"
        && actor_config.awakening.has_effect(6)
        && (target.hp_current as u128 * PARAMETERS.low_hp_threshold.denominator)
            <= (target.hp_max as u128 * PARAMETERS.low_hp_threshold.numerator)
    {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.low_hp_damage_bonus)?;
    }
    Ok(dmg_bonus)
}

/// per 1019AwakenEffect1.json / DT_MintEffectFigure.json Mint_Adcance_4_1 = 0.12.
pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id == "character:1019"
        && ctx.config.awakening.has_effect(4)
        && ctx.target_count <= 1
    {
        Ok(PARAMETERS.single_target_crit_rate)
    } else {
        ExactFraction::new(0, 1)
    }
}

#[allow(dead_code)]
pub(crate) fn apply_a3_attendance_bonus(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    actor_index: usize,
    now_us: u64,
) -> Result<(), String> {
    crate::simulation::state::apply_timed_buff(
        buffs,
        crate::simulation::state::TimedBuffDeclaration::reset(
            "mint.a3_attendance_bonus",
            "character:1019",
            ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
            15_000_000,
            ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([("atk_percent".into(), ExactFraction::new(3, 20)?)]),
        ),
        actor_index,
        now_us,
    )
    .map_err(|error| format!("Mint A3 buff application failed: {error}"))
}
