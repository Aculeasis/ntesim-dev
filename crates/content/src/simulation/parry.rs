//! Explicit successful parry events are separate from Critical Dodge and damage authorization.
use super::{GameActorParryReceipt, GameEventPayload, GameEventReceipt};
use crate::runtime_api::CallbackContext;

pub(super) fn validate(request: &super::GameScenarioRequest) -> Result<(), String> {
    let mut previous = 0;
    let mut after_actions = std::collections::BTreeSet::new();
    for (index, event) in request.parry_events.iter().enumerate() {
        let (timestamp_us, actor_index) = match event {
            super::GameActorParryEvent::At {
                timestamp_us,
                actor_index,
            } => {
                if *timestamp_us < previous {
                    return Err(format!("parry event {index} violates monotonicity"));
                }
                previous = *timestamp_us;
                (*timestamp_us, *actor_index)
            }
            super::GameActorParryEvent::AfterAction { command_index } => {
                if !after_actions.insert(*command_index) {
                    return Err("Duplicate action-bound parry.".into());
                }
                let command = request
                    .actions
                    .get(*command_index)
                    .ok_or("parry command index out of bounds")?;
                let (actor, input) = action_owner(command)?;
                let config = request
                    .actors
                    .get(actor)
                    .ok_or("parry actor index out of bounds")?;
                for (id, description) in
                    crate::characters::parry_inputs::assumptions(&config.character_id, input)?
                {
                    if request.model_assumptions.get(&id) != Some(&description) {
                        return Err(format!("Action-bound parry requires {id}."));
                    }
                }
                (command.timestamp_us(), actor)
            }
        };
        if timestamp_us > request.absolute_limit_us {
            return Err(format!(
                "parry event {index} timestamp violates monotonicity or horizon"
            ));
        }
        let config = request
            .actors
            .get(actor_index)
            .ok_or("parry actor index out of bounds")?;
        crate::registry::validate_parry(request, config)?;
    }
    Ok(())
}

pub(super) fn settle(
    ctx: &mut CallbackContext<'_>,
    event_index: usize,
) -> Result<GameActorParryReceipt, String> {
    let actor_index = match ctx.request.parry_events[event_index] {
        super::GameActorParryEvent::At { actor_index, .. } => actor_index,
        super::GameActorParryEvent::AfterAction { command_index } => {
            action_owner(&ctx.request.actions[command_index])?.0
        }
    };
    let receipt = crate::registry::parry_actor(ctx, actor_index, event_index)?;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "actor_parry".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            event_index: Some(event_index),
            exact_amount: (receipt.malice_granted.numerator != 0).then_some(receipt.malice_granted),
            expires_at_us: receipt.skill_permission_until_us,
            ..Default::default()
        }),
    });
    Ok(receipt)
}

fn action_owner(
    command: &super::GameActionCommand,
) -> Result<(usize, crate::characters::parry_inputs::Input), String> {
    use crate::characters::parry_inputs::Input;
    match command {
        super::GameActionCommand::SwitchActor {
            target_actor_index, ..
        } => Ok((*target_actor_index, Input::Entry)),
        super::GameActionCommand::PlayerInput {
            actor_index,
            input: super::PlayerInput::Dash,
            ..
        } => Ok((*actor_index, Input::Dodge)),
        _ => Err("Assumed action parry requires an eligible Dodge or entry switch.".into()),
    }
}
