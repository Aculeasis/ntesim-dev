//! Authored dummy combat observations; these events never schedule attacks or damage.
use super::{GameDummyCombatEvent, GameEventPayload, GameEventReceipt, GameScenarioRequest};
use crate::{execution::ExecutionLimits, runtime_api::CallbackContext};

pub(super) fn validate(req: &GameScenarioRequest, limits: &ExecutionLimits) -> Result<(), String> {
    if req.dummy_combat_events.len() as u64 > limits.max_events {
        return Err("dummy combat declarations exceed event work limit".into());
    }
    for (&index, &target) in &req.incoming_dummy_sources {
        if index >= req.incoming_damage_events.len() || target >= req.targets.len() {
            return Err("incoming dummy source event/target index out of bounds".into());
        }
    }
    let mut previous = 0;
    for (index, event) in req.dummy_combat_events.iter().enumerate() {
        let time = event.timestamp_us();
        if time < previous || time > req.absolute_limit_us {
            return Err(format!(
                "dummy combat event {index} violates timestamp order or horizon"
            ));
        }
        if event.target_index() >= req.targets.len() {
            return Err(format!(
                "dummy combat event {index} target index out of bounds"
            ));
        }
        if let GameDummyCombatEvent::AttackStart { actor_index, .. } = event
            && *actor_index >= req.actors.len()
        {
            return Err(format!(
                "dummy combat event {index} recipient index out of bounds"
            ));
        }
        previous = time;
    }
    Ok(())
}

pub(super) fn settle(ctx: &mut CallbackContext<'_>, index: usize) -> Result<(), String> {
    let event = &ctx.request.dummy_combat_events[index];
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: match event {
            GameDummyCombatEvent::Proximity { .. } => "ModePause",
            GameDummyCombatEvent::AttackStart { .. } => "ProcCallback",
        }
        .into(),
        event_kind: "dummy_combat".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            event_index: Some(index),
            dummy_event: Some(event.clone()),
            ..Default::default()
        }),
    });
    crate::registry::observe_dummy_combat(ctx, index)
}

pub(super) fn contact(
    ctx: &mut CallbackContext<'_>,
    index: usize,
    receipt: &super::GameActorDamageReceipt,
) -> Result<(), String> {
    let Some(&target) = ctx.request.incoming_dummy_sources.get(&index) else {
        return Ok(());
    };
    let positive = std::iter::once(receipt)
        .chain(receipt.redirected.iter())
        .any(|r| r.actual_hp_loss > 0 || r.absorbed_by_shields.iter().any(|s| s.amount > 0));
    if positive {
        crate::registry::observe_dummy_contact(ctx, target)?;
    }
    Ok(())
}
