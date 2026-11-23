use super::*;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{
    ActionCategory, ActionMetadata, InputActionRole, MetadataProvenance,
};

pub(super) fn emit(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    command: usize,
    bonus: ExactFraction,
) -> Result<(), String> {
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: MODEL.into(),
        description: MODEL.description().into(),
    };
    let action = ActionMetadata {
        categories: [ActionCategory::Other].into(),
        gameplay_tags: Vec::new(),
        ability_id: ABILITY.into(),
        native_ability_index: None,
        input_role: Some(InputActionRole::Other),
        input_role_provenance: Some(provenance.clone()),
        primary_slot: None,
        source_evidence: None,
        identity_evidence: None,
        provenance,
    };
    let config = &ctx.request.actors[owner];
    let mut metadata = crate::simulation::metadata::resolve_packet_for_action(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        &action,
        PACKET,
        &ctx.request.model_assumptions,
    )?;
    let coordinate = 0; // All thirteen native coordinates are exactly8ATK.
    let (rate, scaling_stat) = crate::simulation::inputs::coefficient(
        ctx.request,
        config,
        "GA_Chaos071_UltraSkill",
        PACKET,
        coordinate,
    )?;
    let now = ctx.state.current_time_us;
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: ABILITY.into(),
            time_us: now,
            trigger: crate::simulation::GameCausalTrigger::FieldEntry {
                command_index: command,
            },
            recipient_healing: None,
        },
    )?;
    metadata.binding.energy_group_id = Some(crate::runtime_api::next_packet_group(
        owner,
        &mut ctx.state.actors[owner],
    )?);
    ctx.state.actors[owner]
        .character
        .chaos_mut()?
        .remora_bonuses
        .insert(key, bonus);
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: now,
        priority: crate::execution::CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: key,
            actor_index: owner,
            target_index: target,
            ability_id: ABILITY.into(),
            damage_effect_id: PACKET.into(),
            atk_rate: rate,
            scaling_stat,
            metadata: Box::new(metadata),
        },
    });
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: now,
        priority: crate::execution::CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Chaos(
            super::super::events::Event::RemoraCleanup { owner, key },
        )),
    });
    Ok(())
}
