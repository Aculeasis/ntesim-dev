use super::parameters::PARAMETERS;
#[allow(unused_imports)]
pub(crate) use super::state::State;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1036" {
        return Ok(dmg_bonus);
    }

    let kindling_active = matches!(&ctx.actor.character,crate::characters::RuntimeState::Zankou(state) if state.kindling_active_for_ultra);

    // DT_ZankouEffectFigure.json ZankouFinalUltraDamUpLv2 = 1.50
    let is_ultra = matches!(
        ctx.damage_effect_id,
        "GE_Player_Zankou_UltraSkill1_Damage"
            | "GE_Player_Zankou_UltraSkill2_Damage"
            | "GE_Player_Zankou_ForceUltraSkill1_Damage"
            | "GE_Player_Zankou_ForceUltraSkill2_Damage"
    );
    let is_magic_ultra = matches!(
        ctx.damage_effect_id,
        "GE_Player_Zankou_MagicUltraSkill1_Damage" | "GE_Player_Zankou_MagicUltraSkill2_Damage"
    );

    if actor_config.awakening.has_effect(2)
        && kindling_active
        && (is_ultra || (actor_config.awakening.has_effect(4) && is_magic_ultra))
    {
        dmg_bonus = dmg_bonus.add_final(PARAMETERS.a2_final_multiplier)?;
    }

    // Zankou A6 (Blood Stained Eyes): Oblivion final hit damage bonus:
    // +12% per DoT stack on target, up to +240% (DT_ZankouEffectFigure.json ZankouPerStackDamUpLv6 = 0.12, ZankouDamUpMaxLv6 = 2.40)
    if actor_config.awakening.has_effect(6)
        && ctx.damage_effect_id == "GE_Player_Zankou_Skill3_2_Damage"
    {
        let dot_stacks = crate::simulation::state::target_tagged_status_stacks(
            ctx.buffs,
            ctx.target_index,
            crate::characters::packet_semantics::DOT_DAMAGE_TAG,
            ctx.world_time_us,
        )?;
        let dot_stacks = dot_stacks
            .checked_add(ctx.foreign_dot_stacks)
            .ok_or("Zankou A6 stack count overflow")?;
        let effective_stacks = dot_stacks.min(PARAMETERS.a6_dot_stack_cap);
        if effective_stacks > 0 {
            let a6_bonus = PARAMETERS
                .a6_damage_per_dot_stack
                .mul(ExactFraction::new(effective_stacks as u128, 1)?)?;
            dmg_bonus = dmg_bonus.add_damage(a6_bonus)?;
        }
    }

    // DT_ZankouEffectFigure.json ZankouExtraCoefLv3 = 0.20
    if actor_config.awakening.count() >= 3 && (is_ultra || is_magic_ultra) {
        dmg_bonus = dmg_bonus.add_ratio(PARAMETERS.resonance_burst_ratio_add)?;
    }

    Ok(dmg_bonus)
}
