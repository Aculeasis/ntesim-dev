//! Explicit battle reset, distinct from field exit and source defeat.
use crate::runtime_api::CallbackContext;
use crate::simulation::{ExactFraction as F, GameScenarioRequest};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorCombatLifecycleV1;
pub(crate) fn accepted(r: &GameScenarioRequest) -> bool {
    r.model_assumptions.get(MODEL.as_str()).map(String::as_str) == Some(MODEL.description())
}
pub(crate) fn validate(r: &GameScenarioRequest) -> Result<(), String> {
    if accepted(r) && (!super::gaze::accepted(r) || !super::burst::accepted(r)) {
        return Err("Hathor combat_lifecycle_v1 requires gaze_v1 and burst_emergency_v1".into());
    }
    Ok(())
}
pub(crate) fn admission(actor: &crate::simulation::state::ActorRuntime) -> Result<(), String> {
    if matches!(&actor.character,crate::characters::RuntimeState::Hathor(s) if s.out_of_combat) {
        return Err("Hathor cannot act outside combat".into());
    }
    Ok(())
}
pub(crate) fn transition(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<crate::simulation::GameActorCombatTransitionReceipt, String> {
    let leaving = ctx.request.combat_transition_events[event].leaving;
    let actor = &mut ctx.state.actors[owner];
    if actor.hp == 0 || actor.character.hathor_mut()?.out_of_combat == leaving {
        return Err(
            "Hathor combat transition requires a living actor and alternating state".into(),
        );
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    if leaving {
        let s = ctx.state.actors[owner].character.hathor_mut()?;
        s.battle_generation = s
            .battle_generation
            .checked_add(1)
            .ok_or("Hathor battle generation overflow")?;
        s.riposte_ready_us = None;
        s.riposte_reward_until_us = 0;
        super::burst::clear_with_phase(ctx, owner, "ProcCallback")?;
        ctx.state
            .buffs
            .remove_owned_buff(super::cyclone::DEF, super::cyclone::SOURCE, owner);
        super::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
        )?;
        super::gaze::defeat(ctx, owner)?;
        let actor = &mut ctx.state.actors[owner];
        actor.locked_until_us = ctx.state.current_time_us;
        actor.switch_action_locked_until_us = ctx.state.current_time_us;
        let s = actor.character.hathor_mut()?;
        if let Some(power) = &mut s.power {
            let lost = power.pool.spend_up_to(power.pool.current())?;
            power.earned = 0;
            power.a1_used = false;
            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "ProcCallback".into(),
                event_kind: "hathor_power_combat_exit".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: Some(owner),
                to_actor: Some(owner),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    exact_amount: Some(lost),
                    ..Default::default()
                }),
            });
        }
        s.express_delivery_power = 0;
        s.out_of_combat = true;
    } else {
        ctx.state.actors[owner]
            .character
            .hathor_mut()?
            .out_of_combat = false;
        super::gaze::initialize_owner(ctx, owner, "ProcCallback")?;
    }
    Ok(crate::simulation::GameActorCombatTransitionReceipt {
        event_index: event,
        actor_index: owner,
        time_us: ctx.state.current_time_us,
        leaving,
        malice_granted: F::new(0, 1)?,
        leave_grant_due_us: None,
        resource_deposits: Default::default(),
    })
}
