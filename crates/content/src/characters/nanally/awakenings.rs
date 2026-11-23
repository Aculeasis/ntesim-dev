use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) clock: super::clock::Clock,
    pub(crate) riposte_ready: bool,
    pub(crate) authority_until_us: u64,
    pub(crate) authority_started_us: u64,
    pub(crate) authority_recast_unlock_us: u64,
    pub(crate) authority_command: Option<usize>,
    pub(crate) fair_duel_unlocked: bool,
    pub(crate) blossom_enhancement_unlocked: bool,
    pub(crate) out_of_combat: bool,
    pub(crate) fair_duel_ready_us: u64,
    pub(crate) underboss_until_us: u64,
    pub(crate) underboss_coordinate: Option<u32>,
    pub(crate) underboss_generation: u64,
    pub(crate) a1_ready_us: u64,
    pub(crate) a3_ready_us: u64,
    pub(crate) a5_stacks: u32,
    pub(crate) a5_until_us: u64,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) heavy_resume: Option<usize>,
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1010" {
        return Ok(dmg_bonus);
    }

    // Effect4 names Underboss, not the initiating Burst's direct Q packets.
    if actor_config.awakening.has_effect(4)
        && super::underboss::is_coordinated_packet(ctx.damage_effect_id)
    {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.a4_underboss_damage_bonus)?;
    }

    // per 1010AwakenEffect.json / DT_NanallyEffectFigure.json Nanally_LvExtraFarTarget = 0.15
    if actor_config.awakening.count() >= 6 {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.resonance_damage_bonus)?;
    }

    Ok(dmg_bonus)
}

/// per DT_NanallyEffectFigure.json Nanally_CritDamageUp = 0.30.
pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1010" {
        return ExactFraction::new(0, 1);
    }

    if matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Nanally(state) if state.authority_until_us > ctx.time_us
    ) {
        Ok(PARAMETERS.authority_crit_damage)
    } else {
        ExactFraction::new(0, 1)
    }
}
