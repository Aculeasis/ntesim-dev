//! Explicit combat state selects the next Authority duration, never a refresh.
use crate::runtime_api::CallbackContext;
use crate::simulation::{ExactFraction, GameActorCombatTransitionReceipt, GameScenarioRequest};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::NanallyCombatLifecycleV1;

pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        != Some(MODEL.description())
    {
        return Err("Nanally combat transitions require combat_lifecycle_v1".into());
    }
    Ok(())
}

pub(crate) fn transition(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<GameActorCombatTransitionReceipt, String> {
    let leaving = ctx.request.combat_transition_events[event].leaving;
    let actor = &mut ctx.state.actors[owner];
    if actor.hp == 0 || actor.character.nanally_mut()?.out_of_combat == leaving {
        return Err(
            "Nanally combat transition requires a living actor and alternating state".into(),
        );
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let state = actor.character.nanally_mut()?;
    state.out_of_combat = leaving;
    if leaving {
        state.a5_stacks = 0;
        state.a5_until_us = 0;
        ctx.state
            .buffs
            .remove_owned_buff("Buff_Nanally010_Level5", "character:1010", owner);
    }
    Ok(GameActorCombatTransitionReceipt {
        event_index: event,
        actor_index: owner,
        time_us: ctx.state.current_time_us,
        leaving,
        malice_granted: ExactFraction::new(0, 1)?,
        leave_grant_due_us: None,
        resource_deposits: Default::default(),
    })
}
