use super::parameters::PARAMETERS;
use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;
use ntesim_core::kernel::state::ExactPool;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) breakthrough: u8,
    pub(crate) fading_reason: Option<ExactPool>,
    pub(crate) paid_burst_until_us: u64,
    pub(crate) surging_crimson_until_us: u64,
    pub(crate) skill_rage_ready_us: u64,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) scarlet_descent_casts: u32,
    pub(crate) crimson_judgment_stacks: u32,
    pub(crate) crimson_judgment_combo: bool,
    pub(crate) passive1_aoe_icd_until_us: u64,
    pub(crate) watch_generation: u64,
    pub(crate) menacing_gaze_active: bool,
    pub(crate) watch_stacks: u32,
    pub(crate) watch_next_tick_us: u64,
    pub(crate) granted_resource_groups: BTreeSet<String>,
}

impl State {
    pub(crate) fn fading_reason_amount(&self) -> ExactFraction {
        self.fading_reason
            .as_ref()
            .map(ExactPool::current)
            .unwrap_or(ExactFraction {
                numerator: 0,
                denominator: 1,
            })
    }

    pub(crate) fn has_full_fading_reason(&self) -> bool {
        self.fading_reason_amount()
            .try_cmp(ExactFraction {
                numerator: PARAMETERS.fading_reason_max as u128,
                denominator: 1,
            })
            .is_ok_and(|ord| ord.is_ge())
    }
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1076" {
        return Ok(dmg_bonus);
    }

    let (is_surging_crimson, passive2_unlocked) = match &ctx.actor.character {
        crate::characters::RuntimeState::Shinku(state) => (
            state.surging_crimson_until_us >= ctx.time_us,
            state.breakthrough >= PARAMETERS.passive2_min_breakthrough,
        ),
        _ => (false, false),
    };

    // Calc_Shinku_RageDmgUp: Watch, WatchEx, and ReactionAOE receive +0% base Rage DMG (only +30% from R3).
    let is_passive_proc = matches!(
        ctx.damage_effect_id,
        "GE_Player_Shinku_Watch_Damage"
            | "GE_Player_Shinku_WatchEx_Damage"
            | "GE_Player_Shinku_ReactionAOE_Damage"
    );

    // per DT_ShinkuEffectFigure.json Shinku_Rage_DmgUp = 0.30
    if is_surging_crimson && !is_passive_proc {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.rage_damage_bonus)?;
    }

    // per 1076AwakenEffect.json / DT_ShinkuEffectFigure.json Shinku_Rage_DmgUpEx_L3 = 0.30
    if actor_config.awakening.has_resonance_3() && is_surging_crimson {
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.resonance_rage_damage_bonus)?;
    }

    // Passive 2 (Wrathscale, Breakthrough >= 4): +50% UltraSkill DMG against non-boss targets.
    let is_ultra_packet = matches!(
        ctx.damage_effect_id,
        "GE_Player_Shinku_UltraSkill1_Damage"
            | "GE_Player_Shinku_UltraSkill2_Damage"
            | "GE_Player_Shinku_UltraSkill3_Damage"
            | "GE_Player_Shinku_UltraSkill4_Damage"
            | "GE_Player_Shinku_UltraSkill5_Damage"
            | "GE_Player_Shinku_UltraSkillPre_Rage_Damage"
            | "GE_Player_Shinku_UltraSkill_Rage_Damage"
            | "GE_Player_Shinku_UltraSkill2_Rage_Damage"
    );
    if passive2_unlocked && is_ultra_packet && !ctx.target.is_boss {
        dmg_bonus.record_hypothesis(None, super::passive::MODEL)?;
        dmg_bonus = dmg_bonus.add_damage(PARAMETERS.passive2_non_boss_ultra_damage)?;
    }

    // per 1076AwakenEffect.json / Calc_Shinku_RageSkillDmgCoef / DT_ShinkuEffectFigure.json Shinku_RageSkillDmgCoefL6 = 0.30
    // Applies only to Scarlet Descent, Crimson Judgment, and Dragonflame Verdict packets.
    let is_rage_skill_packet = matches!(
        ctx.damage_effect_id,
        "GE_Player_Shinku_Skill1_Rage_Damage"
            | "GE_Player_Shinku_Skill2_Rage_Damage"
            | "GE_Player_Shinku_UltraSkillPre_Rage_Damage"
            | "GE_Player_Shinku_UltraSkill_Rage_Damage"
            | "GE_Player_Shinku_UltraSkill2_Rage_Damage"
    );

    if actor_config.awakening.has_effect(6) && is_rage_skill_packet {
        dmg_bonus = dmg_bonus.add_ratio(PARAMETERS.a6_rage_skill_ratio_add)?;
    }

    Ok(dmg_bonus)
}

pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1076" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

/// per 1076AwakenEffect.json / DT_ShinkuEffectFigure.json Shinku_CritUp_L6Ex = 0.10.
pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1076" {
        return ExactFraction::new(0, 1);
    }

    if ctx.config.awakening.has_resonance_6() {
        Ok(PARAMETERS.resonance_crit_rate)
    } else {
        ExactFraction::new(0, 1)
    }
}

/// per 1076AwakenEffect.json / DT_ShinkuEffectFigure.json Shinku_DefIgnore_L1 = 0.12 and Shinku_DefIgnore_L6Extra = 0.15.
pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1076" {
        return ExactFraction::new(0, 1);
    }

    let effect_bonus = if ctx.config.awakening.has_effect(1) {
        PARAMETERS.a1_defense_ignore
    } else {
        ExactFraction::new(0, 1)?
    };
    let resonance_bonus = if ctx.config.awakening.has_resonance_6() {
        PARAMETERS.resonance_defense_ignore
    } else {
        ExactFraction::new(0, 1)?
    };
    effect_bonus.add(resonance_bonus)
}
