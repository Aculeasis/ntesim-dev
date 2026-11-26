use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) suppression_inputs: std::collections::BTreeSet<(usize, usize)>,
    pub(crate) a3_kill_bonus_deadlines: Vec<u64>,
    pub(crate) riposte_ready: bool,
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;
    if actor_config.character_id != "character:1003" {
        return Ok(dmg_bonus);
    }

    // per 1003AwakenEffect.json / DT_SagiriEffectFigure.json Sagiri_Lv3_DamUp (0.06)
    if actor_config.awakening.has_effect(3)
        && (ability_id == "GA_Sagiri_Skill" || ability_id == "GA_Sagiri_UltraSkill")
    {
        match &ctx.actor.character {
            crate::characters::RuntimeState::Sakiri(state)
                if state
                    .a3_kill_bonus_deadlines
                    .iter()
                    .any(|until| *until > ctx.time_us) =>
            {
                let stacks = state
                    .a3_kill_bonus_deadlines
                    .iter()
                    .filter(|until| **until > ctx.time_us)
                    .count()
                    .min(10);
                dmg_bonus = dmg_bonus.add_damage(
                    PARAMETERS
                        .defeat_damage_per_stack
                        .mul(ExactFraction::new(stacks as u128, 1)?)?,
                )?;
            }
            _ => {}
        }
    }

    // per 1003AwakenEffect.json / DT_SagiriEffectFigure.json Sagiri_lvExtra_Damup (0.03), Sagiri_lvExtra_DamupMax (0.12)
    if actor_config.awakening.has_resonance_6() && ctx.debuff_count > 0 {
        let count = ctx.debuff_count.min(PARAMETERS.debuff_limit) as u128;
        let fog_bonus = PARAMETERS
            .damage_per_debuff
            .mul(ExactFraction::new(count, 1)?)?;
        dmg_bonus = dmg_bonus.add_damage(fog_bonus)?;
    }

    Ok(dmg_bonus)
}

pub(crate) fn on_team_defeat(
    config: &crate::simulation::GameActorConfig,
    actor: &mut crate::simulation::state::ActorRuntime,
    time_us: u64,
) -> Result<(), String> {
    if config.awakening.has_effect(3) && actor.hp > 0 {
        let state = actor.character.sakiri_mut()?;
        state
            .a3_kill_bonus_deadlines
            .retain(|deadline| *deadline > time_us);
        if state.a3_kill_bonus_deadlines.len() < 10 {
            state.a3_kill_bonus_deadlines.push(
                time_us
                    .checked_add(PARAMETERS.defeat_duration_us)
                    .ok_or("Sakiri team defeat buff deadline overflow")?,
            );
        }
    }
    Ok(())
}
