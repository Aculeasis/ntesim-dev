use crate::runtime_api::CallbackContext;
use crate::simulation::{GameEventPayload, GameEventReceipt};

pub(crate) fn apply_a1_karma(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let actor = state
        .actors
        .get_mut(actor_index)
        .ok_or("Adler actor not found")?;
    if actor.hp == 0 {
        return Ok(());
    }
    let adler = actor.character.adler_mut()?;
    adler.karma_stacks = adler.karma_stacks.saturating_add(20).min(20);
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "adler_a1_karma_grant".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            stacks: Some(20),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn apply_a5_energy(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let actor = state
        .actors
        .get_mut(actor_index)
        .ok_or("Adler actor not found")?;
    if actor.hp == 0 {
        return Ok(());
    }
    let before = crate::simulation::energy::EnergySnapshot::capture(actor);
    actor.restore_energy(20)?;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "adler_a5_energy_refund".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: Some(before.receipt(
            actor,
            crate::simulation::GameEnergySource::AdlerA5,
            actor_index,
            actor_index,
            None,
        )?),
        energy_fanout: None,
        data: None,
    });
    Ok(())
}
