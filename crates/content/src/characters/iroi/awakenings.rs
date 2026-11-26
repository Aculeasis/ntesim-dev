use crate::HypothesisId;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) clock: Option<Box<super::clock::Clock>>,
    pub(crate) native_dream: Option<Box<super::clock::burst::Dream>>,
    pub(crate) a2_cooldown_until_us: u64,
    pub(crate) a2_rng: Option<ntesim_core::kernel::rng::Rng>,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) riposte_ready: bool,
    pub(crate) moon: ExactFraction,
    pub(crate) moon_overflow: ExactFraction,
    pub(crate) dream_active: bool,
    pub(crate) dream_is_ivory: bool,
    pub(crate) dream_generation: u64,
    pub(crate) dream_release_us: u64,
    pub(crate) dream_end_us: u64,
    pub(crate) revive_used_in_dream: bool,
    pub(crate) revived_ally: Option<usize>,
    pub(crate) last_off_field_us: Option<u64>,
    pub(crate) regressed_allies: std::collections::BTreeMap<usize, super::regression::LambType>,
    pub(crate) lamb_cooldowns: std::collections::BTreeMap<(usize, u8), u64>,
    pub(crate) bud_copy_ids: std::collections::VecDeque<u64>,
}

pub(crate) const R6_TEAM_CRIT_AURA_HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiR6TeamCritAuraV1;
pub(crate) const R3_TEAM_DEF_AURA_HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiR3TeamDefAuraV1;
pub(crate) const EFFECT3_HEALING_OUTGOING_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::IroiEffect3HealingOutgoingV1;
#[cfg(test)]
pub(crate) const R6_TEAM_CRIT_AURA_HYPOTHESIS: &str = R6_TEAM_CRIT_AURA_HYPOTHESIS_ID.description();
#[cfg(test)]
pub(crate) const R3_TEAM_DEF_AURA_HYPOTHESIS: &str = R3_TEAM_DEF_AURA_HYPOTHESIS_ID.description();
#[cfg(test)]
pub(crate) const EFFECT3_HEALING_OUTGOING_HYPOTHESIS: &str =
    EFFECT3_HEALING_OUTGOING_HYPOTHESIS_ID.description();

pub(crate) fn record_initial_hypotheses(
    config: &crate::simulation::GameActorConfig,
    request: &crate::simulation::GameScenarioRequest,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    if config.character_id != "character:1075" {
        return Ok(());
    }
    for (enabled, id) in [
        config.awakening.has_resonance_6(),
        config.awakening.has_resonance_3(),
        config.awakening.has_effect(3),
    ]
    .into_iter()
    .zip([
        R6_TEAM_CRIT_AURA_HYPOTHESIS_ID,
        R3_TEAM_DEF_AURA_HYPOTHESIS_ID,
        EFFECT3_HEALING_OUTGOING_HYPOTHESIS_ID,
    ]) {
        if enabled {
            crate::runtime_api::record_hypothesis_id(request, assumptions, id)?;
        }
    }
    Ok(())
}

pub(crate) fn initialize_team_effect(
    config: &crate::simulation::GameActorConfig,
    actor_index: usize,
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
) -> Result<(), String> {
    if config.character_id != "character:1075" {
        return Ok(());
    }
    let grant = |buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
                 id: &str,
                 recipient: ntesim_core::buffs::dispatch::BuffRecipient,
                 modifier: (&str, crate::formula::Ratio)|
     -> Result<(), String> {
        buffs
            .apply_buff(
                ntesim_core::buffs::dispatch::BuffDefinition {
                    stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    semantic_tags: Default::default(),
                    source_lifetime: ntesim_core::buffs::dispatch::BuffSourceLifetime::WhileAlive,
                    id: id.into(),
                    source: config.character_id.clone(),
                    recipient,
                    duration_us: None,
                    refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy::Disallow,
                    max_stacks: 1,
                    decay_policy: ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
                    stat_modifiers: std::collections::BTreeMap::from([(
                        modifier.0.into(),
                        modifier.1,
                    )]),
                },
                actor_index,
                0,
            )
            .map_err(|error| format!("Iroi team effect initialization failed: {error:?}"))
    };
    if config.awakening.has_resonance_6() {
        grant(
            buffs,
            "Buff_Oneiroi075_Level6_extra",
            ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
            (
                "crit_damage_bonus",
                crate::formula::Ratio {
                    numerator: 1,
                    denominator: 4,
                },
            ),
        )?;
    }
    if config.awakening.has_resonance_3() {
        grant(
            buffs,
            "Buff_Oneiroi075_Level2_Def",
            ntesim_core::buffs::dispatch::BuffRecipient::TeamAll,
            (
                "def_percent",
                crate::formula::Ratio {
                    numerator: 3,
                    denominator: 10,
                },
            ),
        )?;
    }
    if config.awakening.has_effect(3) {
        grant(
            buffs,
            "Buff_Oneiroi075_Level3",
            ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(actor_index),
            (
                "healing_bonus",
                crate::formula::Ratio {
                    numerator: 9,
                    denominator: 25,
                },
            ),
        )?;
    }
    Ok(())
}

pub(crate) fn is_lamb_ai_skill(effect: &str) -> bool {
    matches!(
        effect,
        "GE_Player_Oneiroi_GunAI_Skill1_Damage"
            | "GE_Player_Oneiroi_GunAI_Skill2_Damage"
            | "GE_Player_Oneiroi_SyringeAI_Skill1_Damage"
            | "GE_Player_Oneiroi_SyringeAI_Skill2_Damage"
            | "GE_Player_Oneiroi_TrumpetAI_Skill1_Damage"
            | "GE_Player_Oneiroi_TrumpetAI_Skill2_Damage"
    )
}

/// per 1075AwakenEffect.json / DT_OneiroiEffectFigure.json Buff_Oneiroi075_Level4_extra = 0.3.
pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    if ctx.config.character_id == "character:1075" {
        if ctx.config.awakening.has_effect(4) {
            if is_lamb_ai_skill(ctx.damage_effect_id) {
                dmg_bonus = dmg_bonus.modify_ratio(ExactFraction::new(3, 10)?)?;
            } else if ctx.ability_id == "GA_Oneiroi_Skill" {
                let lamb_bonus = ExactFraction::new(3, 10)?;
                dmg_bonus = dmg_bonus.add_damage(lamb_bonus)?;
            }
        }
        if ctx.config.awakening.has_resonance_6()
            && let Ok(state) = ctx.actor.character.iroi()
        {
            let count = state.regressed_allies.len().min(3) as u128;
            if count > 0 {
                let r6_bonus = ExactFraction::new(count * 15, 100)?;
                dmg_bonus = dmg_bonus.add_damage(r6_bonus)?;
            }
        }
    }
    Ok(dmg_bonus)
}

pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id == "character:1075"
        && ctx.config.awakening.has_resonance_6()
        && let Ok(state) = ctx.actor.character.iroi()
    {
        let count = state.regressed_allies.len().min(3) as u128;
        if count > 0 {
            return ExactFraction::new(count * 5, 100);
        }
    }
    ExactFraction::new(0, 1)
}

/// Iroi resonance_6's 25% Crit DMG is team-wide per frozen text and is granted by
/// initialize_team_effect as a TeamAll buff that also covers Iroi herself. The per-packet
/// registry hook returns zero so her own packets do not double-count the aura; the
/// registry arm stays as a no-op.
pub(crate) fn crit_bonus(_ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    ExactFraction::new(0, 1)
}
