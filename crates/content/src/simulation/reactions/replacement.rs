use super::{
    EventContext, ExactFraction, GameSwitchOrigin, ReactionSnapshot, SCORCH_DURATION_US,
    apply_scorch_with_application, buff_stat, cycle_intensity, trace,
};
use crate::simulation::GameEventPayload;
use crate::simulation::metadata::PacketMetadata;

use std::collections::BTreeSet;

fn owner(
    ctx: &EventContext<'_>,
    target_index: usize,
    damage_effect_id: &str,
    semantic_tags: &BTreeSet<String>,
) -> Option<usize> {
    let current = ctx.state.reactions.scorch.get(&target_index)?;
    if current.expires_at_us <= ctx.state.current_time_us
        || ctx.state.targets[target_index].hp_current == 0
    {
        return None;
    }
    let policy = current.enhancement.as_ref()?;
    crate::registry::scorch_dot_eligible(
        ctx.request,
        ctx.state,
        policy.owner_actor_index,
        damage_effect_id,
        semantic_tags,
    )
    .then_some(policy.owner_actor_index)
}

pub(in crate::simulation) fn validate_dot_replacement(
    ctx: &EventContext<'_>,
    target_index: usize,
    packet: &PacketMetadata,
) -> Result<(), String> {
    if owner(
        ctx,
        target_index,
        &packet.damage_effect_id,
        &packet.semantic_tags,
    )
    .is_some()
    {
        ctx.state
            .current_time_us
            .checked_add(SCORCH_DURATION_US)
            .ok_or("Scorch replacement deadline overflow")?;
        // A connected eligible packet can create a fresh Scorch application
        // after damage lands, so validate any Discord reached by the replace
        // before the packet mutates HP or resources.
        super::validate_discord_pair_admission(
            ctx.request,
            ctx.state,
            ntesim_data::reaction_values::ReactionResult::Scorch,
            target_index,
        )?;
    }
    Ok(())
}

pub(in crate::simulation) fn on_dot_delivery(
    ctx: &mut EventContext<'_>,
    trigger_hit_index: usize,
) -> Result<(), String> {
    let trigger = &ctx.hits[trigger_hit_index];
    let target_index = trigger.target_index;
    let packet = &ctx
        .state
        .metadata_catalog
        .get(trigger.metadata_id)
        .ok_or_else(|| format!("missing metadata_catalog entry {}", trigger.metadata_id))?
        .packet;
    let Some(owner_index) = owner(
        ctx,
        target_index,
        &packet.damage_effect_id,
        &packet.semantic_tags,
    ) else {
        return Ok(());
    };
    let current = &ctx.state.reactions.scorch[&target_index];
    let (outgoing_actor_index, incoming_actor_index) = match current.snapshot.switch_origin {
        GameSwitchOrigin::EsperCycle {
            outgoing_actor_index,
            incoming_actor_index,
        }
        | GameSwitchOrigin::EsperCycleReplacement {
            outgoing_actor_index,
            incoming_actor_index,
            ..
        } => (outgoing_actor_index, incoming_actor_index),
        _ => return Err("Scorch replacement lacks its originating Cycle switch".into()),
    };
    let config = &ctx.request.actors[owner_index];
    let snapshot = ReactionSnapshot {
        command_index: current.snapshot.command_index,
        switch_origin: GameSwitchOrigin::EsperCycleReplacement {
            outgoing_actor_index,
            incoming_actor_index,
            owner_actor_index: owner_index,
            trigger_hit_index,
        },
        owner_index,
        level: config.character_level,
        cycle_intensity: cycle_intensity(ctx, owner_index)?,
        element_key: crate::registry::get_character_descriptor_by_id(&config.character_id)?.element,
        crit_damage: ExactFraction::from_ratio(config.static_stats.crit_damage_bonus)?
            .add(buff_stat(ctx, owner_index, "crit_damage_bonus")?)?
            .add(buff_stat(ctx, owner_index, "crit_damage_bonus_dot")?)?,
    };
    apply_scorch_with_application(
        ctx,
        ctx.state.current_time_us,
        target_index,
        snapshot,
        false,
    )?;
    trace(
        ctx,
        "reaction_scorch_replacement",
        Some(owner_index),
        Some(GameEventPayload {
            target_index: Some(target_index),
            parent_hit_index: Some(trigger_hit_index),
            ..Default::default()
        }),
    );
    Ok(())
}
