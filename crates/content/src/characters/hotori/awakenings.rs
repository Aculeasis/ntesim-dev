use super::parameters::PARAMETERS;
use super::timepiece::Timepiece;
use crate::runtime_api::DamageContext;
use crate::runtime_api::{HitModifiers, RecipientContext};
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) timepiece: Timepiece,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
}

fn mode_records(actor: &crate::simulation::state::ActorRuntime, now_us: u64) -> Option<u32> {
    match &actor.character {
        crate::characters::RuntimeState::Hotori(state) => {
            let timepiece = &state.timepiece;
            if !timepiece.mode_active(now_us) {
                return None;
            }
            Some(timepiece.mode.as_ref().map_or(0, |mode| mode.records_n))
        }
        _ => None,
    }
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1052" {
        return Ok(dmg_bonus);
    }

    let Some(records_n) = mode_records(ctx.actor, ctx.time_us) else {
        return Ok(dmg_bonus);
    };

    // per 1052AwakenEffect.json / DT_JinEffectFigure.json Jin_lv1_Damup = 0.12
    if actor_config.awakening.has_effect(1) && records_n > 0 {
        let a1_bonus = PARAMETERS
            .a1_damage_per_recorded_skill
            .mul(ExactFraction::new(records_n as u128, 1)?)?;
        dmg_bonus = dmg_bonus.add_damage(a1_bonus)?;
    }

    // per 1052AwakenEffect.json / DT_JinEffectFigure.json Jin_lv2_TimeDamUp = 0.20
    if actor_config.awakening.has_effect(2) {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.a2_time_stop_damage_bonus)?;
    }

    Ok(dmg_bonus)
}

/// per 1052AwakenEffect.json / DT_JinEffectFigure.json Jin_lv6_TimeDef = 0.30.
pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1052" {
        return ExactFraction::new(0, 1);
    }

    if ctx.config.awakening.has_effect(6) && mode_records(ctx.actor, ctx.time_us).is_some() {
        Ok(PARAMETERS.a6_time_stop_defense_ignore)
    } else {
        ExactFraction::new(0, 1)
    }
}

pub(crate) fn recipient_modifiers(
    ctx: &RecipientContext<'_>,
    mut value: HitModifiers,
) -> Result<HitModifiers, String> {
    // Individual node Effect5: DT_JinEffectFigure.json Jin_lv5_SingleDamUp = 0.15.
    if ctx.source_config.awakening.has_effect(5) && ctx.target_count == 1 {
        value = value.add_damage(PARAMETERS.a5_single_target_damage_bonus)?;
    }
    // Milestone resonance_6 (Jin_lvExtra_AtkUp): activates at 6 total awakenings,
    // granting 0.30 of each recipient's own base ATK to the team.
    if ctx.source_config.awakening.has_resonance_6() {
        value = value.add_atk_percent(PARAMETERS.resonance_team_atk_percent)?;
    }
    Ok(value)
}
