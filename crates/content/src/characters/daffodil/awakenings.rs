use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) clock: Option<Box<super::clock::Clock>>,
    pub(crate) riposte_ready: bool,
    pub(crate) resonance_stacks: u8,
    pub(crate) skill_captures: std::collections::BTreeMap<usize, u8>,
    pub(crate) insight_until_us: u64,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) phantom: super::phantom::State,
    pub(crate) discord_caps: std::collections::BTreeMap<usize, super::discord::CapState>,
    pub(crate) discord_generation: u64,
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1054" {
        return Ok(dmg_bonus);
    }

    if ability_id == "GA_Daffodill_Skill"
        && let crate::characters::RuntimeState::Daffodil(owner) = &ctx.actor.character
    {
        let stacks = owner
            .skill_captures
            .get(&ctx.command_index)
            .copied()
            .unwrap_or(0);
        dmg_bonus = dmg_bonus.add_damage(
            (if actor_config.awakening.has_effect(1) {
                PARAMETERS.a1_skill_damage_per_resonance
            } else {
                PARAMETERS.skill_damage_per_resonance
            })
            .mul(ExactFraction::new(u128::from(stacks), 1)?)?,
        )?;
    }

    // per DT_DaffodillEffectFigure.json Daffodill_UltraInsightDamUp = 0.1
    if matches!(&ctx.actor.character, crate::characters::RuntimeState::Daffodil(state) if state.insight_until_us > ctx.time_us)
    {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.insight_damage_bonus)?;
    }

    if super::phantom::damage_bonus_applies(ctx) {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.phantom_step_damage_bonus)?;
    }

    Ok(dmg_bonus)
}

/// per 1054AwakenEffect.json / DT_DaffodillEffectFigure.json Daffodill_DefIgnoreLv6 = 0.15.
pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id == "character:1054" && ctx.config.awakening.has_effect(6) {
        Ok(PARAMETERS.defense_ignore)
    } else {
        ExactFraction::new(0, 1)
    }
}
