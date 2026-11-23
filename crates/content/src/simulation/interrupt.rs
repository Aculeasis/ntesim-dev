//! Explicit scenario interruption is separate from incoming HP damage and input readiness.
use super::{GameActorInterruptReceipt, GameScenarioRequest};
use crate::runtime_api::CallbackContext;

pub(super) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    let mut previous = 0;
    for (index, event) in request.interrupt_events.iter().enumerate() {
        if event.timestamp_us < previous || event.timestamp_us > request.absolute_limit_us {
            return Err(format!(
                "interrupt event {index} timestamp violates monotonicity or horizon"
            ));
        }
        let config = request
            .actors
            .get(event.actor_index)
            .ok_or("interrupt actor index out of bounds")?;
        if !crate::gear::arc::Arc::validates_interrupt(
            request,
            event.actor_index,
            event.timestamp_us,
        )? {
            crate::registry::validate_interrupt(request, config)?;
        }
        if let Some(pressure) = event.armor_damage {
            if config.character_id != "character:1036" {
                return Err("armor pressure requires Zankou".into());
            }
            super::ExactFraction::from_ratio(pressure)?;
        }
        previous = event.timestamp_us;
    }
    Ok(())
}

pub(super) fn settle(
    ctx: &mut CallbackContext<'_>,
    event_index: usize,
) -> Result<GameActorInterruptReceipt, String> {
    let actor_index = ctx.request.interrupt_events[event_index].actor_index;
    if let Some(receipt) = crate::gear::arc::Arc::interrupt_startup(ctx, actor_index, event_index)?
    {
        ctx.trace.push(super::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "SettlementAndDeath".into(),
            event_kind: "actor_interrupt".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: None,
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(super::GameEventPayload {
                event_index: Some(event_index),
                command_index: receipt.interrupted_command_index,
                count: Some(receipt.cancelled_hit_deliveries),
                field_cancelled: receipt.field_cancelled.then_some(true),
                ..Default::default()
            }),
        });
        return Ok(receipt);
    }
    if let Some(pressure) = ctx.request.interrupt_events[event_index].armor_damage
        && !crate::characters::zankou::combat_passives::armor_pressure(ctx, actor_index, pressure)?
    {
        let actor = &ctx.state.actors[actor_index];
        let cooldown = actor.ability_cooldowns.get("GA_Zankou_Skill").copied();
        return Ok(GameActorInterruptReceipt {
            time_us: ctx.state.current_time_us,
            actor_index,
            event_index,
            interrupted_command_index: None,
            cancelled_hit_deliveries: 0,
            field_cancelled: false,
            recovery_before_us: actor.locked_until_us,
            recovery_after_us: actor.locked_until_us,
            skill_cooldown_before_us: cooldown,
            skill_cooldown_after_us: cooldown,
        });
    }
    let receipt = crate::registry::interrupt_actor(ctx, actor_index, event_index)?;
    if let Some(command) = receipt.interrupted_command_index {
        super::pause::cancel_cinematic(ctx.state, ctx.queue, actor_index, Some(command))?;
    }
    ctx.trace.push(super::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "SettlementAndDeath".into(),
        event_kind: "actor_interrupt".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(super::GameEventPayload {
            event_index: Some(event_index),
            command_index: receipt.interrupted_command_index,
            count: Some(receipt.cancelled_hit_deliveries),
            field_cancelled: receipt.field_cancelled.then_some(true),
            ..Default::default()
        }),
    });
    Ok(receipt)
}
