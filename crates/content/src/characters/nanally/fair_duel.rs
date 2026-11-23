//! State-owned follow-ups from real settled party Cycle damage.
use super::parameters::PARAMETERS;
use crate::runtime_api::CallbackContext;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, ActionPacketBinding, ActionPacketBindingKind,
    ActionPacketMetadata, InputActionRole, MetadataProvenance,
};
use crate::simulation::{GameAutonomousEventReceipt, GameCausalTrigger, GameHitReceipt};
use ntesim_core::execution::CausalPriority;
use std::collections::BTreeSet;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::NanallyCycleFollowUpsV1;
pub(crate) const PACKET: &str = "GE_Nanally010_Lv1_Damage";
const ABILITY: &str = "passive:nanally.fair_duel";
const A3_ABILITY: &str = "passive:nanally.authority_a3";

fn active(ctx: &CallbackContext<'_>, owner: usize) -> bool {
    owner == ctx.state.active_actor_index
        && ctx.state.actors[owner].hp > 0
        && matches!(&ctx.state.actors[owner].character, crate::characters::RuntimeState::Nanally(state) if state.authority_until_us>ctx.state.current_time_us && state.authority_command.is_some())
}

pub(crate) fn observe_cycle(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    index: usize,
    hits: &[GameHitReceipt],
) -> Result<(), String> {
    let hit = hits
        .get(index)
        .ok_or("Nanally Cycle observer has no settled damage")?;
    if hit.applied_hp_loss == 0 || ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let metadata = ctx
        .state
        .metadata_catalog
        .get(hit.metadata_id)
        .ok_or("Nanally Cycle observer missing hit metadata")?;
    if !metadata
        .packet
        .semantic_tags
        .contains("State.Damage.Reaction")
    {
        return Ok(());
    }
    if hit.actor_index == owner {
        let is_hexed_followup = metadata.packet.semantic_tags.contains("hexed_followup");
        if is_hexed_followup
            && (ctx.request.actors[owner].awakening.has_effect(1)
                || ctx.request.actors[owner].awakening.has_effect(5))
        {
            record(ctx, MODEL)?;
            super::callback::schedule_awakenings(
                &ctx.request.actors[owner],
                owner,
                ctx.state.current_time_us,
                ctx.sequence,
                ctx.queue,
            )?;
        }
        a3(ctx, owner, index, hits)?;
    }
    if !active(ctx, owner) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner].character.nanally_mut()?;
    if !state.fair_duel_unlocked || now < state.fair_duel_ready_us {
        return Ok(());
    }
    let deadline = now
        .checked_add(PARAMETERS.fair_duel_interval_us)
        .ok_or("Nanally Fair Duel guard overflow")?;
    if emit(ctx, owner, index, hits, PACKET, ABILITY, MODEL)? {
        ctx.state.actors[owner]
            .character
            .nanally_mut()?
            .fair_duel_ready_us = deadline;
    }
    Ok(())
}

pub(crate) fn observe_direct(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    index: usize,
    hits: &[GameHitReceipt],
) -> Result<(), String> {
    let hit = hits
        .get(index)
        .ok_or("Nanally direct observer has no settled damage")?;
    if hit.actor_index != owner || hit.applied_hp_loss == 0 || hit.damage_effect_id != PACKET {
        return Ok(());
    }
    super::callback::schedule_awakenings(
        &ctx.request.actors[owner],
        owner,
        ctx.state.current_time_us,
        ctx.sequence,
        ctx.queue,
    )?;
    a3(ctx, owner, index, hits)
}

fn a3(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    index: usize,
    hits: &[GameHitReceipt],
) -> Result<(), String> {
    if !ctx.request.actors[owner].awakening.has_effect(3) || !active(ctx, owner) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    if now < ctx.state.actors[owner].character.nanally_mut()?.a3_ready_us {
        return Ok(());
    }
    let deadline = now
        .checked_add(PARAMETERS.a3_interval_us)
        .ok_or("Nanally reaction A3 guard overflow")?;
    record(ctx, MODEL)?;
    if emit(
        ctx,
        owner,
        index,
        hits,
        super::authority::PACKET,
        A3_ABILITY,
        super::authority::HYPOTHESIS_ID,
    )? {
        ctx.state.actors[owner].character.nanally_mut()?.a3_ready_us = deadline;
    }
    Ok(())
}

fn record(ctx: &mut CallbackContext<'_>, model: crate::HypothesisId) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        model,
    )
}

fn emit(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    index: usize,
    hits: &[GameHitReceipt],
    effect: &str,
    ability: &str,
    model: crate::HypothesisId,
) -> Result<bool, String> {
    let hit = &hits[index];
    let target = if ctx.state.targets[hit.target_index].hp_current > 0 {
        Some(hit.target_index)
    } else {
        ctx.state
            .targets
            .iter()
            .position(|target| target.hp_current > 0)
    };
    let Some(target) = target else {
        return Ok(false);
    };
    let command = ctx.state.actors[owner]
        .character
        .nanally_mut()?
        .authority_command
        .ok_or("Nanally follow-up has no Authority source command")?;
    let config = &ctx.request.actors[owner];
    let coordinate = crate::simulation::inputs::normal_coordinate(config, "GA_Nanally_Melee")?;
    let projection = crate::build::project_native_packet_coefficient(
        ntesim_data::skill_levels::frozen::tables(),
        &crate::build::PacketCoefficientRequest {
            content_id: &ctx.request.content_id,
            character_id: &config.character_id,
            ability_id: "GA_Nanally_Melee",
            damage_effect_id: effect,
            array_index: coordinate
                .try_into()
                .map_err(|_| "Nanally Normal coordinate exceeds u32")?,
            research_metadata: None,
            model_assumptions: &ctx.request.model_assumptions,
        },
    )?;
    let native = ntesim_data::native_action_metadata::resolve_damage_effect(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        effect,
    )?;
    let packet = crate::simulation::metadata::resolve_packet_metadata(
        &native,
        &ctx.request.model_assumptions,
    )?;
    record(ctx, model)?;
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: model.as_str().into(),
        description: model.description().into(),
    };
    let metadata = ActionPacketMetadata {
        action: ActionMetadata {
            categories: BTreeSet::from([ActionCategory::Other]),
            gameplay_tags: Vec::new(),
            ability_id: ability.into(),
            native_ability_index: None,
            input_role: Some(InputActionRole::Other),
            input_role_provenance: Some(provenance.clone()),
            primary_slot: None,
            source_evidence: None,
            identity_evidence: None,
            provenance: provenance.clone(),
        },
        packet,
        binding: ActionPacketBinding {
            kind: ActionPacketBindingKind::OwnerChild,
            initiating_ability_id: ability.into(),
            packet_ga_name: native.ga_name.clone(),
            provenance: Some(provenance),
            energy_group_id: None,
        },
    };
    let key = ctx.state.origins.register_from_hit(
        ctx.request,
        GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: ability.into(),
            time_us: ctx.state.current_time_us,
            trigger: GameCausalTrigger::CoordinatedHit {
                cast_command_index: command,
                hit_index: index,
            },
            recipient_healing: None,
        },
        hits,
    )?;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.state.current_time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: key,
            actor_index: owner,
            target_index: target,
            ability_id: ability.into(),
            damage_effect_id: effect.into(),
            atk_rate: serde_json::from_value(projection["compiled_rate"].clone())
                .map_err(|error| format!("Nanally follow-up coefficient: {error}"))?,
            scaling_stat: serde_json::from_value(projection["compiled_scaling_stat"].clone())
                .map_err(|error| format!("Nanally follow-up scaling: {error}"))?,
            metadata: Box::new(metadata),
        },
    });
    Ok(true)
}
