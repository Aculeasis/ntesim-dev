//! Preserved partial cast behavior; no additional game applicability is inferred.
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::CastContext;
use crate::simulation::{
    ExactFraction, GameEventReceipt,
    events::{GameEventData, ScheduledGameEvent},
};
use ntesim_core::execution::CausalPriority;

pub(crate) const PACKETS_HYPOTHESIS_ID: HypothesisId = HypothesisId::SakiriSkillBurstPacketsV1;
pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::SakiriNormalChainV1;
pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::SakiriAoeAllInRangeDummyV1;
pub(crate) const DIRECT_PACKET_INPUTS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::SakiriDirectPlungePacketV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let actor_config = ctx.config;
    let actor_idx = ctx.actor_index;
    let ability_id = ctx.ability_id;
    if actor_config.character_id != "character:1003" {
        return Ok(());
    }
    if ability_id == "GA_Sagiri_QTE" {
        super::support::on_cast(ctx)?;
    }
    if ability_id == "GA_Sagiri_Melee" {
        if super::riposte::on_cast(ctx)? {
            return Ok(());
        }
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());
        if packet_id == Some("GE_Player_Sagiri_AirAttack1_Damage") {
            ctx.record_hypothesis(DIRECT_PACKET_INPUTS_HYPOTHESIS_ID)?;
            ctx.scope_primary_aoe("GE_Player_Sagiri_AirAttack1_Damage")?;
            return Ok(());
        } else {
            if packet_id != super::primary_packet(ability_id) {
                return Err(
                    "Sakiri normal chain requires the modeled first packet as its admitted input"
                        .into(),
                );
            }
            ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
            let state = ctx.actor.character.sakiri_mut()?;
            state.normal_stage = crate::characters::normal::after_gap(
                state.normal_stage,
                state.normal_started_us,
                ctx.time_us,
            );
            let stage = state.normal_stage;
            state.normal_stage = (stage + 1) % 5;
            state.normal_started_us = Some(ctx.time_us);
            let packets: &[&str] = match stage {
                0 => &["GE_Player_Sagiri_Melee1_Damage"],
                1 => &[
                    "GE_Player_Sagiri_Melee2_Damage",
                    "GE_Player_Sagiri_Melee3_Damage",
                ],
                2 => &[
                    "GE_Player_Sagiri_Melee8_Damage",
                    "GE_Player_Sagiri_Melee4_Damage",
                ],
                3 => &[
                    "GE_Player_Sagiri_Melee5_Damage",
                    "GE_Player_Sagiri_Melee5_Damage",
                    "GE_Player_Sagiri_Melee5_Damage",
                ],
                _ => &[
                    "GE_Player_Sagiri_Melee6_Damage",
                    "GE_Player_Sagiri_Melee7_Damage",
                ],
            };
            if stage != 0 {
                ctx.replace_native_primary_packet(packets[0])?;
            }
            for packet in &packets[1..] {
                ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
            }
        }
    } else {
        let state = ctx.actor.character.sakiri_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    let (children, all_packets): (&[&str], &[&str]) = match ctx.ability_id {
        "GA_Sagiri_Skill" => {
            let packet_id = ctx
                .packet_metadata
                .map(|packet| packet.damage_effect_id.as_str());
            if packet_id == Some("GE_Player_Sagiri_Skill3_Damage") {
                (
                    &[
                        "GE_Player_Sagiri_Skill4_Damage",
                        "GE_Player_Sagiri_Skill5_Damage",
                    ],
                    &[
                        "GE_Player_Sagiri_Skill3_Damage",
                        "GE_Player_Sagiri_Skill4_Damage",
                        "GE_Player_Sagiri_Skill5_Damage",
                    ],
                )
            } else {
                (
                    &["GE_Player_Sagiri_Skill2_Damage"],
                    &[
                        "GE_Player_Sagiri_Skill1_Damage",
                        "GE_Player_Sagiri_Skill2_Damage",
                    ],
                )
            }
        }
        "GA_Sagiri_UltraSkill" => (
            &[
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill2_Damage",
            ],
            &[
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill1_Damage",
                "GE_Player_Sagiri_UltraSkill2_Damage",
            ],
        ),
        _ => (&[], &[]),
    };
    if !children.is_empty() {
        let packet_id = ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str());
        let expected_primary = if ctx.ability_id == "GA_Sagiri_Skill"
            && packet_id == Some("GE_Player_Sagiri_Skill3_Damage")
        {
            Some("GE_Player_Sagiri_Skill3_Damage")
        } else {
            super::primary_packet(ctx.ability_id)
        };
        if packet_id != expected_primary {
            return Err(
                "Sakiri packet train requires the modeled first packet as its admitted primary"
                    .into(),
            );
        }
        ctx.record_hypothesis(PACKETS_HYPOTHESIS_ID)?;
        super::control::preflight(ctx)?;
        if ctx.request.targets.len() > 1 {
            ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        for child in children {
            ctx.schedule_native_child_packet(child, ctx.target_index, ctx.hit_time_us)?;
        }
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            for packet in all_packets {
                ctx.schedule_native_child_packet(packet, target_index, ctx.hit_time_us)?;
            }
        }
    }

    if ability_id == "GA_Sagiri_UltraSkill" {
        ctx.record_hypothesis(super::BURST_ATK_MODEL)?;
        // Separate native ordinary and A4 carriers both use GetAtkBase.
        for a4 in [false, true] {
            if a4 && !actor_config.awakening.has_effect(4) {
                continue;
            }
            let grant =
                ExactFraction::new(actor_config.static_stats.base_atk.into(), 1)?.mul(if a4 {
                    super::parameters::PARAMETERS.a4_base_atk_ratio
                } else {
                    super::parameters::PARAMETERS.team_base_atk_ratio
                })?;
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(ContentEvent::SakiriAtkBuffGrant {
                    a4,
                    source_actor_index: actor_idx,
                    atk_add: grant,
                    duration_us: super::parameters::PARAMETERS.team_atk_duration_us,
                }),
            });
        }
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
    super::control::on_hit(ctx)?;
    if !ctx.config.awakening.has_effect(5)
        || ctx.ability_id != "GA_Sagiri_Skill"
        || (!matches!(
            ctx.damage_effect_id,
            "GE_Player_Sagiri_Skill1_Damage" | "GE_Player_Sagiri_Skill4_Damage"
        ))
        || !ctx.target_airborne
    {
        return Ok(());
    }

    let before = crate::simulation::energy::EnergySnapshot::capture(ctx.actor);
    let needed = ctx
        .actor
        .energy
        .capacity()
        .saturating_sub(ctx.actor.energy.current());
    if needed > 0 {
        ctx.actor.restore_energy(needed)?;
    }
    ctx.actor.ability_cooldowns.remove("GA_Sagiri_UltraSkill");
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.time_us,
        phase: "ProcCallback".into(),
        event_kind: "sakiri_a5_reset".into(),
        active_actor: ctx.active_actor_index,
        hit_index: None,
        from_actor: Some(ctx.actor_index),
        to_actor: Some(ctx.actor_index),
        energy_change: Some(before.receipt(
            ctx.actor,
            crate::simulation::GameEnergySource::SakiriA5,
            ctx.actor_index,
            ctx.actor_index,
            None,
        )?),
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            ability_id: Some("GA_Sagiri_UltraSkill".into()),
            target_index: Some(ctx.target_index),
            amount: Some(needed),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn cancel_after_defeat(effect: &str) -> bool {
    effect.starts_with("GE_Player_Sagiri_")
}
