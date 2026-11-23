//! Authored parry and combat transitions, independent of dodge success.
use crate::simulation::{
    ExactFraction, GameActorCombatTransitionReceipt, GameActorConfig, GameActorParryReceipt,
    GameScenarioRequest,
};
use crate::{HypothesisId, runtime_api::CallbackContext};

pub(crate) fn validate(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    model: HypothesisId,
) -> Result<(), String> {
    if config.character_id != "character:1042" {
        return Err("unsupported parry/combat-transition owner".into());
    }
    if request
        .model_assumptions
        .get(model.as_str())
        .map(String::as_str)
        != Some(model.description())
        || !super::malice::accepted(request)
    {
        return Err(format!(
            "Blackbird event requires {} and ordinary_malice_v1",
            model.as_str()
        ));
    }
    Ok(())
}

fn record(ctx: &mut CallbackContext<'_>, model: HypothesisId) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        model,
    )
}

pub(crate) fn parry(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<GameActorParryReceipt, String> {
    let actor = &ctx.state.actors[owner];
    if actor.hp == 0
        || ctx.state.active_actor_index != owner
        || actor.character.blackbird()?.out_of_combat
    {
        return Err("Blackbird explicit parry requires a living active actor in combat".into());
    }
    if super::form::active(actor) && !super::skill::evolve_accepted(ctx.request) {
        return Err("Blackbird transformed parry requires evolve_actions_v1".into());
    }
    let until = ctx
        .state
        .current_time_us
        .checked_add(30_000_000)
        .ok_or("parry permission overflow")?;
    record(ctx, HypothesisId::BlackbirdExplicitParryV1)?;
    let transformed = super::form::active(&ctx.state.actors[owner]);
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    let grant = if transformed {
        state.skill_permission_until_us = Some(until);
        ExactFraction::new(0, 1)?
    } else {
        state
            .malice
            .as_mut()
            .ok_or("parry Malice pool missing")?
            .grant(ExactFraction::new(20, 1)?)?
            .0
    };
    Ok(GameActorParryReceipt {
        event_index,
        actor_index: owner,
        time_us: ctx.state.current_time_us,
        malice_granted: grant,
        skill_permission_until_us: state.skill_permission_until_us,
    })
}

pub(crate) fn transition(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<GameActorCombatTransitionReceipt, String> {
    let leaving = ctx.request.combat_transition_events[event_index].leaving;
    let now = ctx.state.current_time_us;
    let due = leaving
        .then(|| {
            now.checked_add(4_000_000)
                .ok_or("combat leave deadline overflow")
        })
        .transpose()?;
    let actor = &ctx.state.actors[owner];
    if actor.hp == 0 || actor.character.blackbird()?.out_of_combat == leaving {
        return Err(
            "Blackbird combat transition requires a living actor and alternating state".into(),
        );
    }
    record(ctx, HypothesisId::BlackbirdCombatTransitionsV1)?;
    let transformed = super::form::active(&ctx.state.actors[owner]);
    let state = ctx.state.actors[owner].character.blackbird_mut()?;
    let generation = state
        .combat_generation
        .checked_add(1)
        .ok_or("combat generation overflow")?;
    state.combat_generation = generation;
    state.out_of_combat = leaving;
    let grant = if leaving || transformed {
        ExactFraction::new(0, 1)?
    } else {
        state
            .malice
            .as_mut()
            .ok_or("combat Malice pool missing")?
            .grant(ExactFraction::new(20, 1)?)?
            .0
    };
    if let Some(due) = due {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: due,
                priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::Blackbird(
                        super::events::Event::CombatLeaveGrant {
                            actor_index: owner,
                            generation,
                        },
                    ),
                ),
            });
    }
    Ok(GameActorCombatTransitionReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        leaving,
        malice_granted: grant,
        leave_grant_due_us: due,
        resource_deposits: Default::default(),
    })
}

pub(crate) fn leave_grant(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let actor = &mut ctx.state.actors[owner];
    if actor.hp == 0 || super::form::active(actor) {
        return Ok(());
    }
    let state = actor.character.blackbird_mut()?;
    if state.out_of_combat && state.combat_generation == generation {
        let (applied, overflow) = state
            .malice
            .as_mut()
            .ok_or("combat Malice pool missing")?
            .grant(ExactFraction::new(20, 1)?)?;
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "BuffTickOrExpiry".into(),
            event_kind: "blackbird_leave_malice".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: None,
            to_actor: Some(owner),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                generation: Some(generation),
                exact_amount: Some(applied),
                exact_overflow: Some(overflow),
                ..Default::default()
            }),
        });
    }
    Ok(())
}
