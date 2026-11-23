use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) out_of_combat: bool,
    pub(crate) battle_generation: u64,
    pub(crate) riposte_ready_us: Option<u64>,
    pub(crate) riposte_reward_until_us: u64,
    pub(crate) air: Option<super::air::State>,
    pub(crate) branch: Option<super::branch::State>,
    pub(crate) gaze: Option<Box<super::gaze::State>>,
    pub(crate) burst: Option<Box<super::burst::State>>,
    pub(crate) clock: Option<Box<super::clock::Clock>>,
    pub(crate) power: Option<Box<super::power::State>>,
    pub(crate) express_delivery_power: u32,
    pub(crate) emergency_delivery_until_us: u64,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1025" {
        return Ok(dmg_bonus);
    }

    let is_emergency_delivery = super::burst::active(ctx.actor, ctx.time_us);

    let express_delivery_power = match &ctx.actor.character {
        crate::characters::RuntimeState::Hathor(state) => state.express_delivery_power,
        _ => 0,
    };

    // per DT_HathorEffectFigure.json Buff_HathorAngerActive_AtkUp = 0.15
    if matches!(&ctx.actor.character,crate::characters::RuntimeState::Hathor(s) if s.burst.is_some())
    {
        dmg_bonus =
            dmg_bonus.add_atk_percent(super::burst::atk_percent(ctx.actor, ctx.time_us)?)?;
    } else if is_emergency_delivery {
        dmg_bonus = dmg_bonus.add_atk_percent(PARAMETERS.emergency_atk_percent)?;
    }

    // per 1025AwakenEffect.json / DT_HathorEffectFigure.json Buff_Hathor025_Level3_DmgUp = 0.15
    if actor_config.awakening.has_effect(3) {
        let count = if matches!(&ctx.actor.character,crate::characters::RuntimeState::Hathor(s) if s.burst.is_some())
        {
            ctx.non_boss_target_count
        } else {
            1
        };
        dmg_bonus = dmg_bonus.add_damage(
            PARAMETERS
                .a3_damage_bonus
                .mul(ExactFraction::new(count as u128, 1)?)?,
        )?;
    }

    // per 1025AwakenEffect.json / DT_HathorEffectFigure.json Buff_Hathor_DmgUpLevel5 = 0.01
    if actor_config.awakening.has_effect(5) {
        let stacks = match &ctx.actor.character {
            crate::characters::RuntimeState::Hathor(state) => state
                .power
                .as_ref()
                .map_or(express_delivery_power.min(15), |power| {
                    u32::from(power.earned)
                }),
            _ => express_delivery_power.min(15),
        };
        let a5_bonus = PARAMETERS
            .a5_damage_per_power
            .mul(ExactFraction::new(stacks as u128, 1)?)?;
        dmg_bonus = dmg_bonus.add_damage(a5_bonus)?;
    }

    Ok(dmg_bonus)
}

/// per 1025AwakenEffect.json / DT_HathorEffectFigure.json Buff_HathorAngerActive_CritDmgUp = 0.30.
pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1025" {
        return ExactFraction::new(0, 1);
    }

    let is_emergency_delivery = super::burst::active(ctx.actor, ctx.time_us);

    if ctx.config.awakening.has_effect(6) && is_emergency_delivery {
        Ok(PARAMETERS.a6_emergency_crit_damage)
    } else {
        ExactFraction::new(0, 1)
    }
}

/// A4 snapshots the prior successful-stage count for the current Cyclone.
pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    super::cyclone::crit(ctx)
}

/// per 1025AwakenEffect.json / DT_HathorEffectFigure.json Buff_HathorAngerActive_DefIgnore = 0.10.
pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1025" {
        return ExactFraction::new(0, 1);
    }

    let is_emergency_delivery = super::burst::active(ctx.actor, ctx.time_us);

    if ctx.config.awakening.count() >= 6 && is_emergency_delivery {
        Ok(PARAMETERS.resonance_defense_ignore)
    } else {
        ExactFraction::new(0, 1)
    }
}
