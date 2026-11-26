//! Awakenings A1–A6, R3, R6, Passive 2, and `Buff_Elyms_QAtkBuff` team modifiers.
use super::parameters::PARAMETERS;
use super::skill::Connection;
use crate::characters::packet_semantics::FOLLOW_UP_DAMAGE_TAG;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers, RecipientContext};
use crate::simulation::ExactFraction;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ElymsAwakeningsV1;
pub(crate) const Q_ATK_BUFF_ID: &str = "elyms.ultra.atk_add";
pub(crate) const A1_CRIT_BUFF_ID: &str = "elyms.a1.crit_dmg";

pub(crate) fn apply_q_buffs(
    ctx: &mut CastContext<'_>,
    consumed_before_q: u8,
) -> Result<(), String> {
    let base_atk = ExactFraction::new(u128::from(ctx.config.static_stats.base_atk), 1)?;
    let atk_add = base_atk.mul(PARAMETERS.q_atk_base_ratio)?;
    ctx.refresh_buff(
        Q_ATK_BUFF_ID,
        BuffRecipient::TeamAll,
        PARAMETERS.q_atk_buff_duration_us,
        BTreeMap::from([("atk_add".into(), atk_add)]),
    )?;
    let rel_now = ctx.world_time_us;
    let a6_until = rel_now
        .checked_add(PARAMETERS.a6_duration_us)
        .ok_or("Elyms A6 expiry overflow")?;
    let has_a1 = ctx.config.awakening.has_effect(1);
    let has_a6 = ctx.config.awakening.has_effect(6);
    if has_a1 {
        let stacks = consumed_before_q.min(PARAMETERS.total_consumed_cap);
        if stacks > 0 {
            ctx.record_hypothesis(MODEL)?;
            let bonus =
                ExactFraction::new(u128::from(stacks), 1)?.mul(PARAMETERS.a1_crit_dmg_per_stack)?;
            ctx.refresh_buff(
                A1_CRIT_BUFF_ID,
                BuffRecipient::TeamAll,
                PARAMETERS.a1_duration_us,
                BTreeMap::from([("crit_damage_bonus".into(), bonus)]),
            )?;
        } else {
            ctx.buffs
                .remove_owned_buff(A1_CRIT_BUFF_ID, "character:1090", ctx.actor_index);
        }
    }
    if has_a6 && consumed_before_q > 0 {
        ctx.record_hypothesis(MODEL)?;
    }
    let owner = ctx.actor_index;
    let queue = &mut *ctx.queue;
    let sequence = &mut *ctx.sequence;
    let abs_now = ctx.time_us;
    let generation = {
        let state = ctx.actor.character.elyms_mut()?;
        state.q_atk_buff_active = true;
        state.q_atk_buff_generation = state.q_atk_buff_generation.saturating_add(1);
        if has_a6 {
            state.a6_stacks = consumed_before_q.min(PARAMETERS.total_consumed_cap);
            state.a6_until_rel_us = a6_until;
        }
        state.total_consumed_stacks = 0;
        state.q_atk_buff_generation
    };
    queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(crate::callbacks::ContentEvent::Elyms(
                super::events::Event::ExpireQAtkBuff { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: abs_now
            .checked_add(PARAMETERS.q_atk_buff_duration_us)
            .ok_or("Elyms QAtkBuff expiry overflow")?,
        priority: crate::execution::CausalPriority::BuffTickOrExpiry,
        seq: *sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Elyms(
            super::events::Event::ExpireQAtkBuff { owner, generation },
        )),
    });
    Ok(())
}

pub(crate) fn record_initial_hypotheses(
    config: &crate::simulation::GameActorConfig,
    request: &crate::simulation::GameScenarioRequest,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    if config.character_id != "character:1090" {
        return Ok(());
    }
    if config.awakening.has_effect(4)
        || config.awakening.has_effect(5)
        || config.awakening.count() >= 6
    {
        crate::runtime_api::record_hypothesis_id(request, assumptions, MODEL)?;
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if ctx.config.character_id != "character:1090" {
        return Ok(modifiers);
    }
    let state = ctx.actor.character.elyms()?;
    let rel_now = ctx.world_time_us;
    let effect = ctx.damage_effect_id;

    if matches!(
        effect,
        "GE_Player_Elyms_Skill3_Damage" | "GE_Player_Elyms_Skill3_1_Damage"
    ) && let Some(&consumed) = state.skill3_consumed_by_cmd.get(&ctx.command_index)
        && consumed > 0
    {
        let ratio = ExactFraction::new(u128::from(consumed), 1)?
            .mul(PARAMETERS.optical_illusion_coef_modify)?;
        modifiers = modifiers.modify_ratio(ratio)?;
    }

    if effect == "GE_Player_Elyms_Skill3Extra_Damage"
        && let Some(&consumed) = state.skill3_consumed_by_cmd.get(&ctx.command_index)
        && consumed > 1
    {
        modifiers.record_hypothesis(None, MODEL)?;
        modifiers = modifiers.modify_ratio(ExactFraction::new(u128::from(consumed - 1), 1)?)?;
    }

    if matches!(
        effect,
        "GE_Player_Elyms_UltraSkillA1_Damage"
            | "GE_Player_Elyms_UltraSkillA2_Damage"
            | "GE_Player_Elyms_UltraSkillB1_Damage"
            | "GE_Player_Elyms_UltraSkillB2_Damage"
            | "GE_Player_Elyms_UltraSkillB3_Damage"
            | super::on_the_line::XIETONG_A
            | super::on_the_line::XIETONG_B
    ) {
        if let Some(snap) = state.burst_snap_connection.get(&ctx.command_index) {
            match snap {
                Connection::Weak => {}
                Connection::Medium => {
                    modifiers = modifiers.modify_ratio(PARAMETERS.burst_med_coef_modify)?;
                }
                Connection::Strong => {
                    modifiers = modifiers.modify_ratio(PARAMETERS.burst_strong_coef_modify)?;
                }
            }
        }
        if !matches!(
            effect,
            super::on_the_line::XIETONG_A | super::on_the_line::XIETONG_B
        ) && ctx.config.awakening.has_effect(2)
        {
            modifiers.record_hypothesis(None, MODEL)?;
            modifiers = modifiers.modify_ratio(PARAMETERS.a2_burst_coef_modify)?;
        }
    }

    if ctx.config.awakening.has_effect(5)
        && matches!(
            effect,
            "GE_Player_Elyms_Skill1_Damage"
                | "GE_Player_Elyms_Skill1_1_Damage"
                | "GE_Player_Elyms_Skill1_2_Damage"
        )
    {
        modifiers.record_hypothesis(None, MODEL)?;
        modifiers = modifiers.modify_ratio(PARAMETERS.a5_skill1_coef_modify)?;
    }

    if ctx.config.awakening.has_effect(6)
        && state.a6_stacks > 0
        && state.a6_until_rel_us > rel_now
        && matches!(
            effect,
            super::on_the_line::XIETONG_A | super::on_the_line::XIETONG_B
        )
    {
        modifiers.record_hypothesis(None, MODEL)?;
        let bonus = ExactFraction::new(u128::from(state.a6_stacks), 1)?
            .mul(PARAMETERS.a6_followup_dmg_per_stack)?;
        modifiers = modifiers.add_damage(bonus)?;
    }

    Ok(modifiers)
}

pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1090" {
        return ExactFraction::new(0, 1);
    }
    Ok(super::passive::crit_damage_bonus(ctx.actor))
}

pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1090" || !ctx.config.awakening.has_effect(2) {
        return ExactFraction::new(0, 1);
    }
    if matches!(
        ctx.damage_effect_id,
        "GE_Player_Elyms_Skill3_Damage" | "GE_Player_Elyms_Skill3_1_Damage"
    ) && let Some(&consumed) = ctx
        .actor
        .character
        .elyms()?
        .skill3_consumed_by_cmd
        .get(&ctx.command_index)
        && consumed > 0
    {
        return ExactFraction::new(u128::from(consumed), 1)?
            .mul(PARAMETERS.a2_skill3_def_ignore_per_stack);
    }
    ExactFraction::new(0, 1)
}

pub(crate) fn recipient_modifiers(
    ctx: &RecipientContext<'_>,
    mut value: HitModifiers,
) -> Result<HitModifiers, String> {
    if ctx.source_actor.hp == 0 {
        return Ok(value);
    }
    let state = ctx.source_actor.character.elyms()?;
    if state.q_atk_buff_active
        && ctx
            .packet
            .is_some_and(|packet| packet.semantic_tags.contains(FOLLOW_UP_DAMAGE_TAG))
    {
        value = value.add_damage(PARAMETERS.q_followup_dmg_bonus)?;
    }
    if ctx.source_config.awakening.has_effect(4) {
        value.record_hypothesis(ctx.request, MODEL)?;
        value = value.add_crit_damage(PARAMETERS.a4_team_crit_dmg_bonus)?;
    }
    if ctx.source_config.awakening.count() >= 6 {
        value.record_hypothesis(ctx.request, MODEL)?;
        value = value.add_resistance_reduction(PARAMETERS.r6_all_res_ignore)?;
    }
    let _ = BuffStackingScope::PerSource;
    Ok(value)
}
