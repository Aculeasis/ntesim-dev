use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::ShinkuNormalChainV1;
pub(crate) const SKILL_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::ShinkuSkillPacketsV1;
pub(crate) const BURST_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::ShinkuBurstPacketsV1;
pub(crate) const DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ShinkuDirectUltimateFollowupsV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ShinkuAoeAllInRangeDummyV1;

pub(crate) const PRE_DAWN_REFRESH_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ShinkuPreDawnRefreshV1;

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        == super::primary_packet("GA_Shinku_Melee")
    {
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        let state = ctx.actor.character.shinku_mut()?;
        state.normal_stage = crate::characters::normal::after_gap(
            state.normal_stage,
            state.normal_started_us,
            ctx.time_us,
        );
        let stage = state.normal_stage;
        state.normal_stage = (stage + 1) % 5;
        state.normal_started_us = Some(ctx.time_us);
        if state.surging_crimson_until_us > ctx.time_us {
            state.fading_reason = (state.fading_reason + 6).min(12);
        }
        match stage {
            0 => {}
            1 => {
                ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_2_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_Melee1_2_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            2 => {
                ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_3_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_Melee1_3_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_Melee1_3_2_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_Melee1_3_3_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            3 => {
                ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_4_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_Melee1_4_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            4 => {
                ctx.replace_native_primary_packet("GE_Player_Shinku_Melee1_5_Damage")?;
            }
            _ => unreachable!(),
        }
    } else {
        let state = ctx.actor.character.shinku_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1076" {
        return Ok(());
    }

    if ability_id == "GA_Shinku_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.shinku_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    // GA_Shinku_Skill activates Surging Crimson
    // Duration: 13.0s (13_000_000 us) per DT_ShinkuEffectFigure.json Shinku_Rage_Duration = 13.0
    if ability_id == "GA_Shinku_Skill" {
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());
        if packet_id == Some("GE_Player_Shinku_Skill1_Rage_Damage") {
            ctx.record_hypothesis(SKILL_PACKETS_HYPOTHESIS_ID)?;
            if ctx.request.targets.len() > 1 {
                ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Shinku_Skill2_Rage_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            for target_index in 0..ctx.request.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                for packet in [
                    "GE_Player_Shinku_Skill1_Rage_Damage",
                    "GE_Player_Shinku_Skill2_Rage_Damage",
                ] {
                    ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
                }
            }
            let shinku = ctx.actor.character.shinku_mut()?;
            shinku.fading_reason = shinku.fading_reason.saturating_sub(12);
            shinku.scarlet_descent_casts += 1;
            shinku.crimson_judgment_stacks = (shinku.crimson_judgment_stacks + 1).min(3);
        } else if packet_id == Some("GE_Player_Shinku_Skill1_1_Damage") {
            ctx.record_hypothesis(SKILL_PACKETS_HYPOTHESIS_ID)?;
            if ctx.request.targets.len() > 1 {
                ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            for packet in [
                "GE_Player_Shinku_Skill1_2_Damage",
                "GE_Player_Shinku_Skill1_3_Damage",
            ] {
                ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
            }
            for target_index in 0..ctx.request.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                for packet in [
                    "GE_Player_Shinku_Skill1_1_Damage",
                    "GE_Player_Shinku_Skill1_2_Damage",
                    "GE_Player_Shinku_Skill1_3_Damage",
                ] {
                    ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
                }
            }
        } else {
            return Err("Shinku ordinary Skill train requires the modeled first packet as its admitted primary".into());
        }
    }

    if ability_id == "GA_Shinku_UltraSkill" {
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());
        if packet_id == Some("GE_Player_Shinku_UltraSkillPre_Rage_Damage") {
            ctx.record_hypothesis(DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID)?;

            let is_judgment_dash = ctx
                .actor
                .character
                .shinku()
                .map(|s| {
                    s.surging_crimson_until_us > ctx.time_us
                        && s.surging_crimson_until_us.saturating_sub(ctx.time_us) <= 4_000_000
                        && s.crimson_judgment_stacks > 0
                })
                .unwrap_or(false);
            if is_judgment_dash {
                if ctx.request.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                for _ in 0..3 {
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_UltraSkillPre_Rage_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                for target_index in 0..ctx.request.targets.len() {
                    if target_index == ctx.target_index {
                        continue;
                    }
                    for _ in 0..4 {
                        ctx.schedule_native_child_packet(
                            "GE_Player_Shinku_UltraSkillPre_Rage_Damage",
                            target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                }
                let shinku = ctx.actor.character.shinku_mut()?;
                shinku.crimson_judgment_stacks = shinku.crimson_judgment_stacks.saturating_sub(1);
            }
        } else if packet_id == Some("GE_Player_Shinku_UltraSkill_Rage_Damage") {
            ctx.record_hypothesis(DIRECT_FOLLOWUP_INPUTS_HYPOTHESIS_ID)?;
            let is_verdict = ctx
                .actor
                .character
                .shinku()
                .map(|s| {
                    s.surging_crimson_until_us > ctx.time_us
                        && s.surging_crimson_until_us.saturating_sub(ctx.time_us) <= 4_000_000
                        && s.crimson_judgment_stacks == 0
                })
                .unwrap_or(false);
            if is_verdict {
                if ctx.request.targets.len() > 1 {
                    ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
                }
                for _ in 0..9 {
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_UltraSkill_Rage_Damage",
                        ctx.target_index,
                        ctx.hit_time_us,
                    )?;
                }
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill2_Rage_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                for target_index in 0..ctx.request.targets.len() {
                    if target_index == ctx.target_index {
                        continue;
                    }
                    for _ in 0..10 {
                        ctx.schedule_native_child_packet(
                            "GE_Player_Shinku_UltraSkill_Rage_Damage",
                            target_index,
                            ctx.hit_time_us,
                        )?;
                    }
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_UltraSkill2_Rage_Damage",
                        target_index,
                        ctx.hit_time_us,
                    )?;
                }
                let shinku = ctx.actor.character.shinku_mut()?;
                shinku.surging_crimson_until_us = 0;
                shinku.crimson_judgment_stacks = 0;
                shinku.fading_reason = 0;
                shinku.scarlet_descent_casts = 0;
                if shinku.paid_burst_until_us > ctx.time_us {
                    ctx.actor
                        .ability_cooldowns
                        .insert("GA_Shinku_UltraSkill".into(), shinku.paid_burst_until_us);
                }
            }
        } else {
            if packet_id != super::primary_packet(ability_id) {
                return Err(
                    "Shinku Burst train requires the modeled first packet as its admitted primary"
                        .into(),
                );
            }
            ctx.record_hypothesis(BURST_PACKETS_HYPOTHESIS_ID)?;
            if ctx.request.targets.len() > 1 {
                ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Shinku_UltraSkill2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            for _ in 0..5 {
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill3_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Shinku_UltraSkill4_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Shinku_UltraSkill5_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            for target_index in 0..ctx.request.targets.len() {
                if target_index == ctx.target_index {
                    continue;
                }
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill1_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill2_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
                for _ in 0..5 {
                    ctx.schedule_native_child_packet(
                        "GE_Player_Shinku_UltraSkill3_Damage",
                        target_index,
                        ctx.hit_time_us,
                    )?;
                }
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill4_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Shinku_UltraSkill5_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
            }
            let shinku = ctx.actor.character.shinku_mut()?;
            shinku.paid_burst_until_us = ctx.time_us + 20_000_000;
            shinku.surging_crimson_until_us = ctx
                .time_us
                .checked_add(PARAMETERS.rage_duration_us)
                .ok_or("Shinku surging_crimson deadline overflow")?;
            shinku.fading_reason = 0;
            shinku.scarlet_descent_casts = 0;
            shinku.crimson_judgment_stacks = 0;
            if ctx
                .actor
                .ability_cooldowns
                .get("GA_Shinku_UltraSkill")
                .copied()
                .unwrap_or(0)
                > ctx.time_us
            {
                ctx.actor
                    .ability_cooldowns
                    .insert("GA_Shinku_UltraSkill".into(), ctx.time_us + 9_000_000);
            }
            ctx.actor.ability_cooldowns.remove("GA_Shinku_Skill");
        }
    }

    Ok(())
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    if ctx.config.awakening.has_effect(2) && ctx.ability_id == "GA_Shinku_UltraSkill" {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            ctx.execution_assumptions,
            PRE_DAWN_REFRESH_HYPOTHESIS_ID,
        )?;
        let reductions = std::collections::BTreeMap::from([(
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS".into(),
            PARAMETERS.a2_cosmos_resistance_reduction,
        )]);
        ctx.refresh_status(
            "shinku.pre_dawn",
            PARAMETERS.a2_resistance_duration_us,
            reductions,
        )?;
    }
    Ok(())
}
