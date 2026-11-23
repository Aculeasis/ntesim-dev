//! Explicit combat transitions drive owner out-of-combat callbacks on the real axis.
use super::{GameActorCombatTransitionReceipt, GameEventPayload, GameEventReceipt};
use crate::runtime_api::CallbackContext;
use std::collections::BTreeMap;

pub(super) fn validate(request: &super::GameScenarioRequest) -> Result<(), String> {
    let mut previous = 0;
    let mut in_combat = BTreeMap::<usize, bool>::new();
    for (index, event) in request.combat_transition_events.iter().enumerate() {
        if event.timestamp_us < previous || event.timestamp_us > request.absolute_limit_us {
            return Err(format!(
                "combat transition event {index} timestamp violates monotonicity or horizon"
            ));
        }
        let config = request
            .actors
            .get(event.actor_index)
            .ok_or("combat transition actor index out of bounds")?;
        crate::registry::validate_combat_transition(request, config)?;
        let state = in_combat.entry(event.actor_index).or_insert(true);
        if event.leaving != *state {
            return Err(format!(
                "combat transition event {index} must alternate from the initial in-combat state"
            ));
        }
        *state = !event.leaving;
        previous = event.timestamp_us;
    }
    Ok(())
}

pub(super) fn settle(
    ctx: &mut CallbackContext<'_>,
    event_index: usize,
) -> Result<GameActorCombatTransitionReceipt, String> {
    let actor_index = ctx.request.combat_transition_events[event_index].actor_index;
    let receipt = crate::registry::combat_transition(ctx, actor_index, event_index)?;
    ctx.trace.push(GameEventReceipt {
        time_us: receipt.time_us,
        phase: "ProcCallback".into(),
        event_kind: "combat_transition".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            event_index: Some(event_index),
            leaving: Some(receipt.leaving),
            exact_amount: (receipt.malice_granted.numerator != 0).then_some(receipt.malice_granted),
            leave_grant_due_us: receipt.leave_grant_due_us,
            resource_deposits: (!receipt.resource_deposits.is_empty())
                .then(|| receipt.resource_deposits.clone()),
            ..Default::default()
        }),
    });
    Ok(receipt)
}
