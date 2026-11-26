use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1004" {
        return Ok(dmg_bonus);
    }

    // per 1004AwakenEffect.json
    if actor_config.awakening.has_effect(1)
        && matches!(
            ctx.damage_effect_id,
            "GE_Player_Lacrimosa_Blood_Damage" | "GE_Player_Lacrimosa_Blood_Damage_LV6"
        )
    {
        dmg_bonus = dmg_bonus.add_ratio(PARAMETERS.a1_blood_ratio_add)?;
    }

    // Nightmare stack coefficient in the coefficient-addition zone
    // (k = k_native + sum(delta_k)): the tick carries the native per-stack
    // ratio and the live stack count adds ratio * clamp(count - 1, 0, 20).
    if matches!(
        ctx.damage_effect_id,
        "GE_Player_Lacrimosa_Blood_Damage" | "GE_Player_Lacrimosa_Blood_Damage_LV6"
    ) && let crate::characters::RuntimeState::Lacrimosa(state) = &ctx.actor.character
    {
        let add = super::dot::stack_ratio_add(&state.nightmare, ctx.target_index, ctx.time_us)?;
        if !add.is_zero() {
            dmg_bonus = dmg_bonus.add_ratio(add)?;
        }
    }

    Ok(dmg_bonus)
}

/// per 1004AwakenEffect.json / DT_LacrimosaEffectFigure.json Lacrimosa_lvExtra_DefIgnore = 0.20.
/// Talent A3 adds DEF-ignore stacks on Shatter. Total DEF ignore is capped at 100%.
pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1004" {
        return ExactFraction::new(0, 1);
    }
    let mut total = ExactFraction::new(0, 1)?;
    if ctx.config.awakening.has_resonance_6() {
        total = total.add(PARAMETERS.resonance_defense_ignore)?;
    }
    if ctx.config.awakening.has_resonance_3()
        && let crate::characters::RuntimeState::Lacrimosa(state) = &ctx.actor.character
        && state.a3_def_ignore_stacks > 0
    {
        let a3_bonus = ExactFraction::new(u128::from(state.a3_def_ignore_stacks) * 5, 100)?;
        total = total.add(a3_bonus)?;
    }
    let cap = ExactFraction::new(1, 1)?;
    if total.numerator * cap.denominator > cap.numerator * total.denominator {
        Ok(cap)
    } else {
        Ok(total)
    }
}
