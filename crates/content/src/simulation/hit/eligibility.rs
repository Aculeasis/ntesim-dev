use super::*;

pub(super) fn can_deliver(
    ctx: &mut EventContext<'_>,
    delivery: &HitDelivery,
) -> Result<bool, String> {
    let HitIdentity {
        actor_index,
        target_index,
        ..
    } = delivery.identity;
    let ability_id = &delivery.ability_id;
    let damage_effect_id = &delivery.damage_effect_id;
    let scaling_stat = delivery.scaling_stat;
    let metadata = &delivery.metadata;
    if metadata.action.ability_id != *ability_id
        || metadata.packet.damage_effect_id != *damage_effect_id
    {
        return Err("hit metadata identity does not match the delivered action and packet".into());
    }
    if ctx.state.actors[actor_index].hp == 0 {
        if metadata.binding.kind
            != crate::simulation::metadata::ActionPacketBindingKind::NativeDirect
            && !crate::registry::cancel_owned_packet_after_defeat(
                &ctx.request.actors[actor_index],
                damage_effect_id,
            )
            && !metadata.packet.is_arc_chain()
        {
            return Err(format!(
                "unsupported postmortem packet owner semantics for {} ({ability_id})",
                metadata.packet.damage_effect_id
            ));
        }
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "HitDelivery".into(),
            event_kind: "hit_owner_unavailable".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                ability_id: Some(ability_id.clone()),
                damage_effect_id: Some(damage_effect_id.clone()),
                target_index: Some(target_index),
                ..Default::default()
            }),
        });
        return Ok(false);
    }
    if scaling_stat.uses_hp()
        && !crate::build::supports_hp_packet(
            &ctx.request.actors[actor_index].character_id,
            ability_id,
            damage_effect_id,
        )
    {
        return Err("HP scaling is unsupported for this native owner/action/packet".into());
    }

    let target = &ctx.state.targets[target_index];
    let target_was_alive = target.hp_current > 0;
    crate::registry::validate_hit_target(
        ctx.request,
        &ctx.request.actors[actor_index],
        ability_id,
        &metadata.packet,
        target,
    )?;
    if !target_was_alive {
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "HitDelivery".into(),
            event_kind: "hit_target_unavailable".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                ability_id: Some(ability_id.clone()),
                damage_effect_id: Some(damage_effect_id.clone()),
                target_index: Some(target_index),
                ..Default::default()
            }),
        });
        return Ok(false);
    }
    Ok(true)
}

pub(super) struct PreparedHit {
    pub buffs: Option<ntesim_core::buffs::dispatch::BuffManager>,
    pub target_has_scorch: bool,
}

pub(super) fn prepare(
    ctx: &mut EventContext<'_>,
    delivery: &HitDelivery,
) -> Result<PreparedHit, String> {
    let HitIdentity {
        actor_index,
        target_index,
        ..
    } = delivery.identity;
    let ability_id = &delivery.ability_id;
    let damage_effect_id = &delivery.damage_effect_id;
    let atk_rate = delivery.atk_rate;
    let metadata = &delivery.metadata;
    let request = ctx.request;
    let state = &mut *ctx.state;
    let actor_config = &request.actors[actor_index];
    let scorch_stacks = crate::simulation::state::target_tagged_status_stacks(
        &state.buffs,
        target_index,
        crate::characters::packet_semantics::SCORCH_STATUS_TAG,
        state.mode_clock.relative_us(),
    )?;
    for (source_config, source_actor) in request.actors.iter().zip(&state.actors) {
        crate::registry::validate_packet_delivery(
            source_config,
            source_actor,
            &metadata.packet,
            scorch_stacks > 0,
            state
                .reactions
                .supports_dot_replacement(target_index, state.current_time_us),
            false,
        )?;
    }
    let speculative_buffs =
        if atk_rate.numerator != 0 && crate::registry::has_before_hit(&actor_config.character_id) {
            let mut speculative = state.buffs.clone();
            crate::registry::before_hit(&mut crate::runtime_api::HitContext {
                request,
                config: actor_config,
                actor_index,
                ability_id,
                damage_effect_id,
                world_time_us: state.mode_clock.relative_us(),

                target_index,
                buffs: &mut speculative,
            })?;
            Some(speculative)
        } else {
            None
        };
    Ok(PreparedHit {
        buffs: speculative_buffs,
        target_has_scorch: scorch_stacks > 0,
    })
}
