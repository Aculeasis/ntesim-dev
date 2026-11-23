use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

pub(crate) fn max_hp_bonus(awakening: &ntesim_data::AwakeningSelection) -> crate::formula::Ratio {
    if awakening.has_effect(5) {
        crate::formula::Ratio {
            numerator: 1,
            denominator: 5,
        }
    } else {
        crate::formula::Ratio::zero()
    }
}

pub(crate) fn owned_max_hp_bonus(
    character_id: &str,
    awakening: &ntesim_data::AwakeningSelection,
) -> crate::formula::Ratio {
    if character_id == "character:1021" {
        max_hp_bonus(awakening)
    } else {
        crate::formula::Ratio::zero()
    }
}

pub(crate) fn project_max_hp(
    base_hp: u64,
    hp_percent: crate::formula::Ratio,
    hp_flat: crate::formula::Ratio,
) -> Result<u64, String> {
    let base = ExactFraction::new(u128::from(base_hp), 1)?;
    let percent = ExactFraction::from_ratio(hp_percent)?;
    base.mul(ExactFraction::new(1, 1)?.add(percent)?)?
        .add(ExactFraction::from_ratio(hp_flat)?)?
        .round_u64()
}

pub(crate) fn validate_max_hp(
    base_hp: Option<u64>,
    hp_percent: crate::formula::Ratio,
    hp_flat: crate::formula::Ratio,
    static_hp: u64,
    initial_hp_max: u64,
    runtime_hp_max: u64,
    awakening: &ntesim_data::AwakeningSelection,
) -> Result<(), String> {
    if max_hp_bonus(awakening).numerator == 0 {
        return Ok(());
    }
    let base_hp = base_hp.ok_or("Edgar A5 Burst requires base-HP provenance")?;
    ExactFraction::from_ratio(hp_percent)?
        .sub(ExactFraction::from_ratio(max_hp_bonus(awakening))?)
        .map_err(
            |_| "Edgar A5 MaxHP provenance mismatch: HP percent pool omits the A5 contribution",
        )?;
    let expected = project_max_hp(base_hp, hp_percent, hp_flat)?;
    if static_hp != expected || initial_hp_max != expected || runtime_hp_max != expected {
        return Err(format!(
            "Edgar A5 MaxHP provenance mismatch: expected {expected}, static HP {static_hp}, initial MaxHP {initial_hp_max}, runtime MaxHP {runtime_hp_max}"
        ));
    }
    Ok(())
}

pub(crate) fn validate_burst_admission(
    config: &crate::simulation::GameActorConfig,
    actor: &crate::simulation::state::ActorRuntime,
    ability_id: &str,
) -> Result<(), String> {
    if ability_id == "GA_Edgar_UltraSkill" && config.awakening.has_effect(5) {
        validate_max_hp(
            config.static_stats.base_hp,
            config.static_stats.hp_percent,
            config.static_stats.hp_flat,
            config.static_stats.hp,
            config.initial_hp_max,
            actor.hp_max,
            &config.awakening,
        )?;
    }
    Ok(())
}

pub(crate) const KEYS_CAPACITY: u32 = 3;

pub(crate) fn initial_keys(initial: Option<crate::formula::Ratio>) -> Result<u32, String> {
    let value = initial
        .map(ExactFraction::from_ratio)
        .transpose()?
        .unwrap_or_default();
    if value.denominator != 1 || value.numerator > u128::from(KEYS_CAPACITY) {
        return Err("Edgar Keys must be integral in 0..3.".into());
    }
    u32::try_from(value.numerator).map_err(|_| "Edgar initial Keys overflow".into())
}

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) clock: Option<Box<super::clock::Clock>>,
    pub(crate) riposte_ready: bool,
    pub(crate) out_of_combat: bool,
    pub(crate) key_ready_us: u64,
    pub(crate) field: Option<super::effects::Field>,
    pub(crate) pending_charge: Option<ntesim_core::kernel::state::ExactPool>,
    pub(crate) charge_guards: std::collections::BTreeMap<usize, u64>,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) keys_of_truth: u32,
    pub(crate) finnegan_wake_until_us: u64,
    pub(crate) healing_generation: u64,
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1021" {
        return Ok(dmg_bonus);
    }

    // per 1021AwakenEffect.json / DT_EdgarEffectFigure.json Buff_Edgar_Star4_SkillDamUP = 0.50
    if actor_config.awakening.has_effect(4)
        && (ctx.ability_id == "GA_Edgar_Skill" || ctx.ability_id == "GE_Player_Edgar_Skill_Damage")
    {
        dmg_bonus = dmg_bonus.add_damage(ExactFraction::new(1, 2)?)?;
    }

    Ok(dmg_bonus)
}
