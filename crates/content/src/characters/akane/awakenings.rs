use super::parameters::{PARAMETERS, overtone_coef_add, upskill_short_coef_add};
use crate::runtime_api::{CallbackContext, CastContext, DamageContext, HitModifiers};
use crate::simulation::state::{
    TargetStatusDeclaration, TimedBuffDeclaration, apply_target_status,
};
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::AkaneAwakeningsV1;
pub(crate) const ANTHEM_BUFF_ID: &str = "akane.starlit_anthem";
pub(crate) const A1_BUFF_ID: &str = "akane.a1_def";
pub(crate) const A4_BUFF_ID: &str = "akane.a4_crit_dmg";
pub(crate) const R6_BUFF_ID: &str = "akane.r6_crit_dmg";
pub(crate) const OVERTONE_STATUS_ID: &str = "akane.overtone";
pub(crate) const REVERB_STATUS_ID: &str = "akane.reverb_field";
const LAKSHANA_KEY: &str = "ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA";

fn packet_array_index(
    config: &crate::simulation::GameActorConfig,
    ability: &str,
) -> Result<usize, String> {
    let slot = match ability {
        "GA_Akane_Melee" => Some(0usize),
        "GA_Akane_Skill" => Some(1usize),
        "GA_Akane_UltraSkill" => Some(2usize),
        _ => None,
    };
    let purchased = slot
        .and_then(|idx| config.talent_levels.map(|levels| levels[idx]))
        .unwrap_or(config.purchased_skill_level);
    let bonus = if slot.is_some() {
        crate::characters::resonance::bonus(
            ntesim_data::skill_levels::frozen::tables(),
            &config.character_id,
            ability,
            &config.awakening,
        )?
        .0
    } else {
        0
    };
    Ok((purchased + bonus).clamp(1, 13) as usize - 1)
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if ctx.config.character_id != "character:1057" {
        return Ok(modifiers);
    }
    let state = ctx.actor.character.akane()?;
    let now = ctx.world_time_us;
    let effect = ctx.damage_effect_id;

    modifiers = super::passive::damage_bonus(ctx, modifiers)?;

    let a5 = ctx.config.awakening.has_effect(5);
    if let Some(marked) = state.overtone_target(now)
        && ctx.target_index == marked
    {
        let coord = packet_array_index(ctx.config, ctx.ability_id)?;
        if let Some(add) = overtone_coef_add(effect, coord, a5)? {
            modifiers = modifiers.add_ratio(add)?;
        }
    }

    if effect == "GE_Player_Akane_UpgradeSkill_Damage" && state.overtone_active(now) {
        let coord = packet_array_index(ctx.config, "GA_Akane_Skill")?;
        modifiers = modifiers.add_ratio(upskill_short_coef_add(coord)?)?;
    }

    if state.reverb_active(now)
        && matches!(
            effect,
            "GE_Player_Akane_Skill2_Damage"
                | "GE_Player_Akane_UpgradeSkill_Damage"
                | "GE_Player_Akane_Melee1_Damage"
                | "GE_Player_Akane_Melee2_Damage"
                | "GE_Player_Akane_Melee3_Damage"
                | "GE_Player_Akane_Melee3_1_Damage"
                | "GE_Player_Akane_Melee4_Damage"
                | "GE_Player_Akane_PerfectEvadeAttack_Damage"
        )
    {
        modifiers = modifiers.add_damage(if a5 {
            PARAMETERS.reverb_a5_dmg_bonus
        } else {
            PARAMETERS.reverb_dmg_bonus
        })?;
    }

    if ctx.config.awakening.has_effect(3) && effect == "GE_Player_Akane_Skill_Damage" {
        modifiers = modifiers.add_ratio(PARAMETERS.a3_press_fixed_ratio)?;
    }

    if ctx.config.awakening.count() >= 3 {
        if effect == "GE_Player_Akane_UpgradeSkill_Damage" {
            modifiers = modifiers.add_ratio(PARAMETERS.r3_upskill_fixed_ratio)?;
        } else if effect == "GE_Player_Akane_UltraSkill2_Damage" {
            modifiers = modifiers.add_ratio(PARAMETERS.r3_ultra2_fixed_ratio)?;
        }
    }

    Ok(modifiers)
}

pub(crate) fn apply_starlit_anthem(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let mut stats = BTreeMap::from([("atk_percent".into(), PARAMETERS.anthem_atk_bonus)]);
    if ctx.config.awakening.has_effect(6) {
        ctx.record_hypothesis(MODEL)?;
        stats.insert("def_ignore".into(), PARAMETERS.a6_def_ignore);
    }
    ctx.refresh_buff(
        ANTHEM_BUFF_ID,
        BuffRecipient::SelfActor,
        PARAMETERS.anthem_duration_us,
        stats,
    )?;
    let state = ctx.actor.character.akane_mut()?;
    state.anthem_until_us = ctx
        .time_us
        .checked_add(PARAMETERS.anthem_duration_us)
        .ok_or("Akane Starlit Anthem expiry overflow")?;
    Ok(())
}

pub(crate) fn apply_mode_status_debuff(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    overtone_target: Option<usize>,
) -> Result<(), String> {
    let char_id = &ctx.request.actors[owner].character_id;
    let target_count = ctx.request.targets.len();
    for target in 0..target_count {
        ctx.state
            .buffs
            .remove_target_buff(OVERTONE_STATUS_ID, char_id, &target.to_string());
        ctx.state
            .buffs
            .remove_target_buff(REVERB_STATUS_ID, char_id, &target.to_string());
    }
    let a2 = ctx.request.actors[owner].awakening.has_effect(2);
    if a2 {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
    }
    let reductions = if a2 {
        BTreeMap::from([(LAKSHANA_KEY.into(), PARAMETERS.a2_res_reduction)])
    } else {
        BTreeMap::new()
    };
    let rel_now = ctx.state.mode_clock.relative_us();
    match overtone_target {
        Some(target) => {
            if target < target_count && ctx.state.targets[target].hp_current > 0 {
                apply_target_status(
                    &mut ctx.state.buffs,
                    TargetStatusDeclaration {
                        source_actor: owner,
                        source: char_id.clone(),
                        target_index: target,
                        id: OVERTONE_STATUS_ID.into(),
                        now_us: rel_now,
                        duration_us: PARAMETERS.mode_duration_us,
                        reductions,
                    },
                )?;
            }
        }
        None => {
            for target in 0..target_count {
                if ctx.state.targets[target].hp_current == 0 {
                    continue;
                }
                apply_target_status(
                    &mut ctx.state.buffs,
                    TargetStatusDeclaration {
                        source_actor: owner,
                        source: char_id.clone(),
                        target_index: target,
                        id: REVERB_STATUS_ID.into(),
                        now_us: rel_now,
                        duration_us: PARAMETERS.mode_duration_us,
                        reductions: reductions.clone(),
                    },
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn apply_synced_buffs(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let rel_now = ctx.state.mode_clock.relative_us();
    let config = &ctx.request.actors[owner];
    let a1 = config.awakening.has_effect(1);
    let a4 = config.awakening.has_effect(4);
    let r6 = config.awakening.count() >= 6;
    let base_atk_raw = config.static_stats.base_atk;
    let char_id = config.character_id.clone();

    if a1 {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
        let base_atk = ExactFraction::new(u128::from(base_atk_raw), 1)?;
        let raw_heal = base_atk.mul(PARAMETERS.a1_heal_base_atk_ratio)?;
        let mut recipients: Vec<usize> = (0..ctx.request.actors.len()).collect();
        recipients.sort_by_key(|&idx| (ctx.request.actors[idx].slot, idx));
        for recipient in recipients {
            if ctx.state.actors[recipient].hp == 0 {
                continue;
            }
            let settlement = ctx.settle_healing(owner, recipient, raw_heal)?;
            ctx.trace.push(GameEventReceipt {
                time_us: now,
                phase: "ProcCallback".into(),
                event_kind: "akane_a1_heal".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: Some(owner),
                to_actor: Some(recipient),
                energy_change: None,
                energy_fanout: None,
                data: Some(GameEventPayload {
                    input: Some("skill".into()),
                    exact_raw: Some(base_atk),
                    atk_ratio: Some(PARAMETERS.a1_heal_base_atk_ratio),
                    gross: Some(settlement.gross),
                    applied: Some(settlement.applied),
                    overflow: Some(settlement.excess),
                    hp_before: Some(settlement.before_hp),
                    hp_after: Some(settlement.after_hp),
                    ..Default::default()
                }),
            });
        }
        ctx.state.apply_buff(
            owner,
            TimedBuffDeclaration::reset(
                A1_BUFF_ID,
                &char_id,
                BuffRecipient::SelfActor,
                PARAMETERS.a1_duration_us,
                BuffStackingScope::PerSource,
                BTreeMap::from([("def_percent".into(), PARAMETERS.a1_def_bonus)]),
            ),
        )?;
        let state = ctx.state.actors[owner].character.akane_mut()?;
        state.a1_until_us = now
            .checked_add(PARAMETERS.a1_duration_us)
            .ok_or("Akane A1 expiry overflow")?;
    }

    if a4 {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
        ctx.state.apply_buff(
            owner,
            TimedBuffDeclaration::reset(
                A4_BUFF_ID,
                &char_id,
                BuffRecipient::SelfActor,
                PARAMETERS.a4_duration_us,
                BuffStackingScope::PerSource,
                BTreeMap::from([("crit_damage_bonus".into(), PARAMETERS.a4_crit_dmg_bonus)]),
            ),
        )?;
        let state = ctx.state.actors[owner].character.akane_mut()?;
        state.a4_until_us = now
            .checked_add(PARAMETERS.a4_duration_us)
            .ok_or("Akane A4 expiry overflow")?;
    }

    if r6 {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
        crate::simulation::state::apply_timed_buff(
            &mut ctx.state.buffs,
            TimedBuffDeclaration::reset(
                R6_BUFF_ID,
                &char_id,
                BuffRecipient::SelfActor,
                PARAMETERS.r6_duration_us,
                BuffStackingScope::PerSource,
                BTreeMap::from([("crit_damage_bonus".into(), PARAMETERS.r6_crit_dmg_bonus)]),
            ),
            owner,
            rel_now,
        )
        .map_err(|error| format!("Akane R6 CritDMG buff failed: {error}"))?;
        let state = ctx.state.actors[owner].character.akane_mut()?;
        state.r6_until_us = now
            .checked_add(PARAMETERS.r6_duration_us)
            .ok_or("Akane R6 expiry overflow")?;
    }

    Ok(())
}
