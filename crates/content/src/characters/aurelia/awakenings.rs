use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) cadenza_until_us: u64,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) riposte_ready: bool,
    pub(crate) crescendo_stacks: u32,
    pub(crate) crescendo_expires_at_us: u64,
    pub(crate) crescendo_extended_once: bool,
    pub(crate) in_staccato: bool,
    pub(crate) staccato_entered_us: u64,
}

impl State {
    pub(crate) fn is_in_staccato(&self, time_us: u64) -> bool {
        self.in_staccato && time_us <= self.staccato_entered_us.saturating_add(2_000_000)
    }
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1070" {
        return Ok(dmg_bonus);
    }

    let has_cadenza = matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Aurelia(state) if state.cadenza_until_us > ctx.time_us
    );

    let is_ultra = ctx
        .action_metadata
        .categories
        .contains(&crate::simulation::metadata::ActionCategory::UltraSkill);

    // per 1070AwakenEffect.json / DT_MitsukiEffectFigure.json Mitsuki_Star3_DmgUp = 0.20
    if actor_config.awakening.has_effect(3) && has_cadenza && is_ultra {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.a3_burst_damage_bonus)?;
    }

    Ok(dmg_bonus)
}

pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1070" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

/// per 1070AwakenEffect.json / DT_MitsukiEffectFigure.json Mitsuki_Star1_CritUp = 0.15, Mitsuki_Star5_CritUp = 0.15.
pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1070" {
        return ExactFraction::new(0, 1);
    }

    let mut bonus = ExactFraction::new(0, 1)?;

    let has_cadenza = matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Aurelia(state) if state.cadenza_until_us > ctx.time_us
    );

    if ctx.config.awakening.has_effect(1) && has_cadenza {
        bonus = bonus.add(PARAMETERS.awakening_crit_rate_bonus)?;
    }

    let is_ultra = ctx
        .action_metadata
        .categories
        .contains(&crate::simulation::metadata::ActionCategory::UltraSkill);
    if ctx.config.awakening.has_effect(5) && is_ultra {
        bonus = bonus.add(PARAMETERS.awakening_crit_rate_bonus)?;
    }

    Ok(bonus)
}

pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1070" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}
