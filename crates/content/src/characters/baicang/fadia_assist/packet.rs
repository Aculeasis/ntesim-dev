use super::*;
use crate::simulation::metadata::{ActionPacketMetadata, InputActionRole, MetadataProvenance};

pub(super) struct Prepared {
    rate: crate::formula::Ratio,
    scaling: crate::simulation::NativeScalingStat,
    metadata: ActionPacketMetadata,
}

pub(super) fn prepare(ctx: &CallbackContext<'_>, owner: usize) -> Result<Prepared, String> {
    let config = &ctx.request.actors[owner];
    let coordinate = crate::simulation::inputs::normal_coordinate(config, "GA_Cang_Melee")?;
    let (rate, scaling) = crate::simulation::inputs::coefficient(
        ctx.request,
        config,
        "GA_Cang_Melee",
        PACKET,
        coordinate,
    )?;
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: HYPOTHESIS_ID.as_str().into(),
        description: HYPOTHESIS_ID.description().into(),
    };
    let mut action = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        "GA_Cang_Melee",
        PACKET,
        None,
        &ctx.request.model_assumptions,
    )?
    .action;
    action.ability_id = ABILITY.into();
    action.native_ability_index = None;
    action.input_role = Some(InputActionRole::Other);
    action.input_role_provenance = Some(provenance.clone());
    action.primary_slot = None;
    action.gameplay_tags.clear();
    action.source_evidence = None;
    action.identity_evidence = None;
    action.provenance = provenance;
    let metadata = crate::simulation::metadata::resolve_packet_for_action(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        &action,
        PACKET,
        &ctx.request.model_assumptions,
    )?;
    Ok(Prepared {
        rate,
        scaling,
        metadata,
    })
}

pub(super) fn enqueue(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    key: usize,
    mut prepared: Prepared,
) -> Result<(), String> {
    prepared.metadata.binding.energy_group_id = Some(crate::runtime_api::next_packet_group(
        actor_index,
        &mut ctx.state.actors[actor_index],
    )?);
    for target_index in 0..ctx.state.targets.len() {
        if ctx.state.targets[target_index].hp_current == 0 {
            continue;
        }
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.state.current_time_us,
            priority: ntesim_core::execution::CausalPriority::HitDelivery,
            seq: *ctx.sequence,
            data: GameEventData::HitDelivery {
                command_index: key,
                actor_index,
                target_index,
                ability_id: ABILITY.into(),
                damage_effect_id: PACKET.into(),
                atk_rate: prepared.rate,
                scaling_stat: prepared.scaling,
                metadata: Box::new(prepared.metadata.clone()),
            },
        });
    }
    Ok(())
}
