use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::{CastContext, HitContext};

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) bullets: u8,
    pub(crate) pact_generation: u64,
    pub(crate) pacts: std::collections::BTreeMap<usize, super::pact::Pact>,
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::JiuyuanNormalChainV1;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::JiuyuanSkillBurstPacketsV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::JiuyuanAoeAllInRangeDummyV1;

fn handle_melee(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let packet_id = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str());
    if packet_id == Some("GE_Player_Kuhara_BranchPower_Damage") {
        let state = ctx.actor.character.jiuyuan_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        let count = state.bullets.min(super::resource::CAPACITY);
        state.bullets = 0;
        if count > 1 {
            for k in 1..count {
                ctx.schedule_native_child_packet(
                    "GE_Player_Kuhara_BranchPower_Damage",
                    ctx.target_index,
                    ctx.hit_time_us + 250_000 * (k as u64),
                )?;
            }
        }
    } else if packet_id == super::primary_packet(ctx.ability_id) {
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        let state = ctx.actor.character.jiuyuan_mut()?;
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
                ctx.replace_native_primary_packet("GE_Player_Kuhara_Melee2_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Kuhara_Melee2_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            2 => {
                let state = ctx.actor.character.jiuyuan_mut()?;
                state.bullets = state
                    .bullets
                    .saturating_add(1)
                    .min(super::resource::CAPACITY);
                ctx.replace_native_primary_packet("GE_Player_Kuhara_Melee3_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Kuhara_Melee3_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Kuhara_Melee3_2_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            3 => {
                ctx.replace_native_primary_packet("GE_Player_Kuhara_Melee4_Damage")?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Kuhara_Melee4_1_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Kuhara_Melee4_2_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
                ctx.schedule_native_child_packet(
                    "GE_Player_Kuhara_Melee4_3_Damage",
                    ctx.target_index,
                    ctx.hit_time_us,
                )?;
            }
            4 => {
                let state = ctx.actor.character.jiuyuan_mut()?;
                state.bullets = state
                    .bullets
                    .saturating_add(1)
                    .min(super::resource::CAPACITY);
                ctx.replace_native_primary_packet("GE_Player_Kuhara_Melee5_Damage")?;
            }
            _ => unreachable!(),
        }
    } else {
        let state = ctx.actor.character.jiuyuan_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    Ok(())
}

fn schedule_packet_train(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let children: &[&str] = match ctx.ability_id {
        "GA_Kuhara_Skill" => &[
            "GE_Player_Kuhara_Skill2_Damage",
            "GE_Player_Kuhara_Skill2_Damage",
            "GE_Player_Kuhara_Skill2_Damage",
            "GE_Player_Kuhara_Skill2_Damage",
            "GE_Player_Kuhara_Skill3_Damage",
        ],
        "GA_Kuhara_UltraSkill" => &[
            "GE_Player_Kuhara_UltraSkill2_Damage",
            "GE_Player_Kuhara_UltraSkill2_Damage",
            "GE_Player_Kuhara_UltraSkill2_Damage",
            "GE_Player_Kuhara_UltraSkill3_Damage",
            "GE_Player_Kuhara_UltraSkill3_Damage",
            "GE_Player_Kuhara_UltraSkill3_Damage",
            "GE_Player_Kuhara_UltraSkill3_Damage",
            "GE_Player_Kuhara_UltraSkill3_Damage",
            "GE_Player_Kuhara_UltraSkill3_Damage",
            "GE_Player_Kuhara_UltraSkill3_Damage",
            "GE_Player_Kuhara_UltraSkill4_Damage",
        ],
        _ => &[],
    };
    if children.is_empty() {
        return Ok(());
    }
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err(
            "Jiuyuan packet train requires the modeled first packet as its admitted primary".into(),
        );
    }
    ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
    }
    for child in children {
        let hit_time = if *child == "GE_Player_Kuhara_UltraSkill4_Damage"
            && super::pact::accepted(ctx.request)
            && ctx
                .request
                .model_assumptions
                .contains_key(PACKETS_HYPOTHESIS_ID.as_str())
        {
            ctx.hit_time_us + 5_283_316
        } else {
            ctx.hit_time_us
        };
        ctx.schedule_native_child_packet(child, ctx.target_index, hit_time)?;
    }
    for target_index in 0..ctx.request.targets.len() {
        if target_index == ctx.target_index {
            continue;
        }
        let all_packets: &[&str] = match ctx.ability_id {
            "GA_Kuhara_Skill" => &[
                "GE_Player_Kuhara_Skill1_Damage",
                "GE_Player_Kuhara_Skill2_Damage",
                "GE_Player_Kuhara_Skill2_Damage",
                "GE_Player_Kuhara_Skill2_Damage",
                "GE_Player_Kuhara_Skill2_Damage",
                "GE_Player_Kuhara_Skill3_Damage",
            ],
            "GA_Kuhara_UltraSkill" => &[
                "GE_Player_Kuhara_UltraSkill1_Damage",
                "GE_Player_Kuhara_UltraSkill2_Damage",
                "GE_Player_Kuhara_UltraSkill2_Damage",
                "GE_Player_Kuhara_UltraSkill2_Damage",
                "GE_Player_Kuhara_UltraSkill3_Damage",
                "GE_Player_Kuhara_UltraSkill3_Damage",
                "GE_Player_Kuhara_UltraSkill3_Damage",
                "GE_Player_Kuhara_UltraSkill3_Damage",
                "GE_Player_Kuhara_UltraSkill3_Damage",
                "GE_Player_Kuhara_UltraSkill3_Damage",
                "GE_Player_Kuhara_UltraSkill3_Damage",
                "GE_Player_Kuhara_UltraSkill4_Damage",
            ],
            _ => &[],
        };
        for packet in all_packets {
            ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
        }
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.config.character_id != "character:1055" {
        return Ok(());
    }

    if ctx.ability_id == "GA_Kuhara_Melee" {
        handle_melee(ctx)?;
    } else {
        let state = ctx.actor.character.jiuyuan_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }

    if ctx.ability_id == "GA_Kuhara_Skill" {
        let state = ctx.actor.character.jiuyuan_mut()?;
        state.bullets = state
            .bullets
            .saturating_add(4)
            .min(super::resource::CAPACITY);
    }
    if ctx.ability_id == "GA_Kuhara_UltraSkill"
        && ctx
            .request
            .model_assumptions
            .contains_key(PACKETS_HYPOTHESIS_ID.as_str())
    {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: ctx.time_us + 999_811,
                priority: ntesim_core::execution::CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::JiuyuanForceGrow {
                        actor_index: ctx.actor_index,
                    },
                ),
            });
    }

    schedule_packet_train(ctx)?;
    super::pact::on_cast(ctx)
}

pub(crate) fn before_hit(ctx: &mut HitContext<'_>) -> Result<(), String> {
    // When Secrets Take Shape binds the target on its final hit. The shared
    // Melee action also owns earlier stages, aimed/branch/plunge and Pact packets;
    // matching the action alone would incorrectly apply or refresh their Pact.
    let natural = super::pact::accepted(ctx.request);
    let applies_pact = (ctx.ability_id == "GA_Kuhara_Skill"
        || (!natural && ctx.ability_id == "GA_Kuhara_UltraSkill"))
        || (ctx.ability_id == "GA_Kuhara_Melee"
            && ctx.damage_effect_id == "GE_Player_Kuhara_Melee5_Damage");
    if applies_pact
        && (!natural
            || !crate::simulation::state::target_has_status(
                ctx.buffs,
                ctx.actor_index,
                &ctx.config.character_id,
                ctx.target_index,
                "jiuyuan.rose_pact",
            ))
    {
        let reductions = if ctx.config.awakening.has_resonance_6() {
            std::collections::BTreeMap::from([(
                "ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE".into(),
                PARAMETERS.resonance_resistance_reduction,
            )])
        } else {
            Default::default()
        };
        let duration = if natural {
            super::pact::NATURAL_DURATION_US + 200_000
        } else {
            PARAMETERS.pact_duration_us
        };
        ctx.refresh_status("jiuyuan.rose_pact", duration, reductions)?;
    }
    Ok(())
}
