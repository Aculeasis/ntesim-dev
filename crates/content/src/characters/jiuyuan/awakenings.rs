use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1055" {
        return Ok(dmg_bonus);
    }

    let has_rose_pact = ctx.has_status("jiuyuan.rose_pact");

    // per 1055AwakenEffect.json / DT_KuharaEffectFigure.json Buff_Kuhara_Seed_AtkUp = 0.05
    if actor_config.awakening.has_effect(1) && has_rose_pact {
        let bound_targets = ctx
            .owned_status_counts
            .get("jiuyuan.rose_pact")
            .copied()
            .unwrap_or(0);
        dmg_bonus = dmg_bonus.add_atk_percent(
            PARAMETERS
                .a1_atk_per_bound_target
                .mul(ExactFraction::new(bound_targets.min(3) as u128, 1)?)?,
        )?;
    }

    // per 1055AwakenEffect.json / DT_KuharaEffectFigure.json Kuhara_DmgUp_L5 = 0.08
    if actor_config.awakening.has_effect(5) && has_rose_pact {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.a5_pact_damage_bonus)?;
    }

    Ok(dmg_bonus)
}

pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1055" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1055" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1055" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}
