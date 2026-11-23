//! Preserved partial cast behavior; no additional game applicability is inferred.
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::CastContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) const A1_HYPOTHESIS_ID: HypothesisId = HypothesisId::ZeroA1FirstHitV1;
pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::ZeroNormalChainV1;
pub(crate) const SKILL_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::ZeroSkillPacketsV1;
pub(crate) const BURST_PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::ZeroBurstPacketsV1;
pub(crate) const BURST_ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ZeroBurstAllInRangeV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::ZeroAoeAllInRangeDummyV1;
pub(crate) const BURST_A3_CRIT_HYPOTHESIS_ID: HypothesisId = HypothesisId::ZeroBurstA3CritRateV1;
pub(crate) const BURST_A4_DAMAGE_BONUS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ZeroBurstA4DamageBonusV1;

pub(crate) fn cancel_after_defeat(effect_id: &str) -> bool {
    effect_id.starts_with("GE_Player_Female046_")
}

fn extra_effect(awakening: &ntesim_data::AwakeningSelection) -> &'static str {
    if awakening.has_effect(6) {
        "GE_Player_Female046_Skill_Kill_Damage_lv2"
    } else {
        "GE_Player_Female046_Skill_Kill_Damage_lv1"
    }
}

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != Some("GE_Player_Female046_Melee1_Damage")
    {
        return Err("Zero Normal chain requires Melee1 as its admitted primary packet".into());
    }
    ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
    let state = ctx.actor.character.zero_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let stage = state.normal_stage;
    state.normal_stage = (stage + 1) % 5;
    state.normal_started_us = Some(ctx.time_us);
    match stage {
        0 => {}
        1 => {
            ctx.replace_native_primary_packet("GE_Player_Female046_Melee2_1_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Female046_Melee2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        2 => {
            ctx.replace_native_primary_packet("GE_Player_Female046_Melee3_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Female046_Melee4_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
            ctx.schedule_native_child_packet(
                "GE_Player_Female046_Melee5_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        3 => {
            ctx.replace_native_primary_packet("GE_Player_Female046_Melee6_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Female046_Melee7_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        4 => {
            ctx.replace_native_primary_packet("GE_Player_Female046_Melee8_Damage")?;
            ctx.schedule_native_child_packet(
                "GE_Player_Female046_Melee8_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let actor_idx = ctx.actor_index;
    let ability_id = ctx.ability_id;

    if actor_config.character_id != "character:1046" {
        return Ok(());
    }
    if ability_id == "GA_Female046_QTE" {
        super::support::on_cast(ctx)?;
    }
    if ability_id == "GA_Female046_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.zero_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ability_id == "GA_Female046_Skill" && actor_config.awakening.has_effect(1) {
        ctx.record_hypothesis(A1_HYPOTHESIS_ID)?;
        ctx.child_packet_metadata(extra_effect(&actor_config.awakening))?;
    }

    if ability_id == "GA_Female046_Skill" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != Some("GE_Player_Female046_Skill1_Damage")
        {
            return Err(
                "Zero ordinary Skill train requires Skill1 as its admitted primary packet".into(),
            );
        }
        ctx.record_hypothesis(SKILL_PACKETS_HYPOTHESIS_ID)?;
        // The selected Skill model fills this owner's Cycle once per admitted
        // cast, independently of recipient count and conventional energy.
        // Replay invokes the same semantic cast and retains the same ledger.
        ctx.actor.cycle_ledger.fill(&mut ctx.actor.cycle)?;
        for packet in [
            "GE_Player_Female046_Skill2_Damage",
            "GE_Player_Female046_Skill3_Damage",
            "GE_Player_Female046_Skill4_Damage",
        ] {
            ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
        }
    }
    if ability_id == "GA_Female046_UltraSkill" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != Some("GE_Player_Female046_UltraSkill1_Damage")
        {
            return Err(
                "Zero Burst train requires UltraSkill1 as its admitted primary packet".into(),
            );
        }
        ctx.record_hypothesis(BURST_PACKETS_HYPOTHESIS_ID)?;
        if ctx.request.targets.len() > 1
            && ctx
                .request
                .model_assumptions
                .contains_key(ALL_IN_RANGE_HYPOTHESIS_ID.as_str())
        {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        } else {
            ctx.record_hypothesis(BURST_ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        let packets = [
            "GE_Player_Female046_UltraSkill1_Damage",
            "GE_Player_Female046_UltraSkill2_Damage",
            "GE_Player_Female046_UltraSkill2_Damage",
            "GE_Player_Female046_UltraSkill2_Damage",
            "GE_Player_Female046_UltraSkill2_Damage",
            "GE_Player_Female046_UltraSkill3_Damage",
            "GE_Player_Female046_UltraSkill4_Damage",
        ];
        for packet in packets.iter().skip(1) {
            ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
        }
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            for packet in packets {
                ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
            }
        }
        if actor_config.awakening.has_effect(3) {
            ctx.record_hypothesis(BURST_A3_CRIT_HYPOTHESIS_ID)?;
        }
        if actor_config.awakening.has_effect(4) {
            ctx.record_hypothesis(BURST_A4_DAMAGE_BONUS_HYPOTHESIS_ID)?;
        }
    }
    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;

    if ability_id == "GA_Female046_Skill" && actor_config.awakening.has_effect(2) {
        crate::runtime_api::increment_sequence(seq)?;
        queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: CausalPriority::ProcCallback,
            seq: *seq,
            data: GameEventData::Content(ContentEvent::ZeroA2EnergyGrant {
                actor_index: actor_idx,
                amount: super::parameters::PARAMETERS.skill_energy,
            }),
        });
    }

    if ability_id == "GA_Female046_QTE" && actor_config.awakening.has_effect(5) {
        crate::runtime_api::increment_sequence(seq)?;
        queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: CausalPriority::ProcCallback,
            seq: *seq,
            data: GameEventData::Content(ContentEvent::ZeroA5AtkBuffGrant {
                actor_index: actor_idx,
                duration_us: super::parameters::PARAMETERS.support_atk_duration_us,
            }),
        });
    }

    Ok(())
}

/// Delivery owns first-hit eligibility; cast admission alone cannot trigger A1.
pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    if !ctx.config.awakening.has_effect(1)
        || ctx.ability_id != "GA_Female046_Skill"
        || matches!(
            ctx.damage_effect_id,
            "GE_Player_Female046_Skill_Kill_Damage_lv1"
                | "GE_Player_Female046_Skill_Kill_Damage_lv2"
        )
        || ctx.target_level > ctx.config.character_level
    {
        return Ok(());
    }
    let key = (ctx.command_index, ctx.target_index);
    if ctx.actor.character.zero_mut()?.a1_first_hits.contains(&key) {
        return Ok(());
    }
    let damage_effect_id = extra_effect(&ctx.config.awakening);
    let metadata = crate::simulation::metadata::resolve_packet_for_action(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.config.character_id,
        ctx.action_metadata,
        damage_effect_id,
        &ctx.request.model_assumptions,
    )?;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.actor.character.zero_mut()?.a1_first_hits.insert(key);
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: ctx.command_index,
            actor_index: ctx.actor_index,
            target_index: ctx.target_index,
            ability_id: ctx.ability_id.into(),
            damage_effect_id: damage_effect_id.into(),
            atk_rate: Ratio {
                numerator: if ctx.config.awakening.has_effect(6) {
                    3
                } else {
                    2
                },
                denominator: 1,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            metadata: Box::new(metadata),
        },
    });
    Ok(())
}
