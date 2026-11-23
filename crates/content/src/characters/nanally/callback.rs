use super::parameters::PARAMETERS;
use crate::runtime_api::CallbackContext;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventReceipt};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) fn apply_a1_energy(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let actor = state
        .actors
        .get_mut(actor_index)
        .ok_or("Nanally actor not found")?;

    if actor.hp == 0 {
        return Ok(());
    }
    let nanally = actor.character.nanally_mut()?;
    if state.current_time_us < nanally.a1_ready_us {
        return Ok(());
    }
    nanally.a1_ready_us = state
        .current_time_us
        .checked_add(PARAMETERS.a1_interval_us)
        .ok_or("Nanally A1 deadline overflow")?;

    let before = crate::simulation::energy::EnergySnapshot::capture(actor);
    let charge_basis = PARAMETERS.a1_charge_basis;
    let _ = actor.grant_fractional_energy(charge_basis)?;

    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "nanally_a1_energy_grant".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: Some(before.receipt(
            actor,
            crate::simulation::GameEnergySource::NanallyA1,
            actor_index,
            actor_index,
            Some(charge_basis),
        )?),
        energy_fanout: None,
        data: None,
    });
    Ok(())
}

pub(crate) fn apply_a5_buff(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    duration_us: u64,
) -> Result<(), String> {
    if ctx.state.active_actor_index != actor_index
        || ctx.state.actors[actor_index].hp == 0
        || ctx.state.actors[actor_index]
            .character
            .nanally_mut()?
            .out_of_combat
    {
        return Ok(());
    }
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let actor = state
        .actors
        .get_mut(actor_index)
        .ok_or("Nanally actor not found")?;
    let until = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("Nanally A5 expiry overflow")?;
    let nanally_state = actor.character.nanally_mut()?;
    if state.current_time_us >= nanally_state.a5_until_us {
        nanally_state.a5_stacks = 0;
    }
    nanally_state.a5_stacks = (nanally_state.a5_stacks + 1).min(10);
    nanally_state.a5_until_us = until;
    let stacks = nanally_state.a5_stacks;

    // per 1010AwakenEffect.json / DT_NanallyEffectFigure.json Nanally_Lv5_AtkUp = 0.02
    let atk_percent = PARAMETERS
        .a5_atk_per_stack
        .mul(ExactFraction::new(stacks as u128, 1)?)?;

    state.apply_buff(
        actor_index,
        TimedBuffDeclaration::reset(
            "Buff_Nanally010_Level5",
            "character:1010",
            BuffRecipient::SelfActor,
            duration_us,
            BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([("atk_percent".into(), atk_percent)]),
        ),
    )?;

    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "nanally_a5_atk_buff".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            atk_percent: Some(atk_percent),
            stacks: Some(stacks),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn schedule_awakenings(
    config: &crate::simulation::GameActorConfig,
    owner: usize,
    now: u64,
    sequence: &mut u64,
    queue: &mut crate::simulation::timer_queue::EventQueue,
) -> Result<(), String> {
    use crate::callbacks::ContentEvent;
    for event in [
        config
            .awakening
            .has_effect(1)
            .then_some(ContentEvent::NanallyA1EnergyGrant { actor_index: owner }),
        config
            .awakening
            .has_effect(5)
            .then_some(ContentEvent::NanallyA5AtkBuff {
                actor_index: owner,
                duration_us: PARAMETERS.a5_duration_us,
            }),
    ]
    .into_iter()
    .flatten()
    {
        crate::runtime_api::increment_sequence(sequence)?;
        queue.push(crate::simulation::events::ScheduledGameEvent {
            time_us: now,
            priority: ntesim_core::execution::CausalPriority::ProcCallback,
            seq: *sequence,
            data: crate::simulation::events::GameEventData::Content(event),
        });
    }
    Ok(())
}
