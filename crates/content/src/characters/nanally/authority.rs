//! A3 source-native Normal coefficient and live Authority damage trigger.
use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::execution::CausalPriority;

pub(crate) const PACKET: &str = "GE_Nanally010_Lv3_Damage";
pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::NanallyAuthorityA3FollowUpV1;

fn callbacks(ctx: &mut CastContext<'_>) -> Result<(), String> {
    super::callback::schedule_awakenings(
        ctx.config,
        ctx.actor_index,
        ctx.time_us,
        ctx.sequence,
        ctx.queue,
    )
}

pub(crate) fn on_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx.config.awakening.has_effect(3) {
        return Ok(());
    }
    let Some(packet) = ctx.packet_metadata else {
        return Ok(());
    };
    if packet.damage_effect_id == PACKET {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        return callbacks(ctx);
    }
    if !matches!(
        ctx.ability_id,
        "GA_Nanally_Melee" | "GA_Nanally_Skill" | "GA_Nanally_UltraSkill" | "GA_Nanally_QTE"
    ) || ctx.targets[ctx.target_index].hp_current == 0
    {
        return Ok(());
    }
    let state = ctx.actor.character.nanally_mut()?;
    if state.authority_until_us <= ctx.time_us || state.a3_ready_us > ctx.time_us {
        return Ok(());
    }
    if packet.source_evidence.is_none() {
        return Err(
            "unsupported Nanally A3 trigger: native packet source evidence is required".into(),
        );
    }
    state.a3_ready_us = ctx
        .time_us
        .checked_add(PARAMETERS.a3_interval_us)
        .ok_or("Nanally A3 deadline overflow")?;
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let coordinate = crate::simulation::inputs::normal_coordinate(ctx.config, "GA_Nanally_Melee")?;
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &ctx.request.content_id,
            character_id: &ctx.config.character_id,
            ability_id: "GA_Nanally_Melee",
            damage_effect_id: PACKET,
            array_index: coordinate
                .try_into()
                .map_err(|_| "Nanally A3 coordinate exceeds u32")?,
            research_metadata: None,
            model_assumptions: &ctx.request.model_assumptions,
        },
    )?;
    let metadata = ctx.child_packet_metadata(PACKET)?;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: ctx.command_index,
            actor_index: ctx.actor_index,
            target_index: ctx.target_index,
            ability_id: ctx.ability_id.to_owned(),
            damage_effect_id: PACKET.into(),
            atk_rate: serde_json::from_value(projection["compiled_rate"].clone())
                .map_err(|error| format!("Nanally A3 coefficient invalid: {error}"))?,
            scaling_stat: serde_json::from_value(projection["compiled_scaling_stat"].clone())
                .map_err(|error| format!("Nanally A3 scaling invalid: {error}"))?,
            metadata: Box::new(metadata),
        },
    });
    Ok(())
}
