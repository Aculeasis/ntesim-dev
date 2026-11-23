use super::*;

pub(super) fn enqueue(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    key: usize,
    coordinate: usize,
) -> Result<(), String> {
    let config = &ctx.request.actors[owner];
    let (atk_rate, scaling_stat) =
        crate::simulation::inputs::coefficient(ctx.request, config, ABILITY, PACKET, coordinate)?;
    let mut metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        ABILITY,
        PACKET,
        None,
        &ctx.request.model_assumptions,
    )?;
    metadata.binding.energy_group_id = Some(crate::runtime_api::next_packet_group(
        owner,
        &mut ctx.state.actors[owner],
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
                actor_index: owner,
                target_index,
                ability_id: ABILITY.into(),
                damage_effect_id: PACKET.into(),
                atk_rate,
                scaling_stat,
                metadata: Box::new(metadata.clone()),
            },
        });
    }
    Ok(())
}
