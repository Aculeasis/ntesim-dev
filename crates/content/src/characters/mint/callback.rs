use crate::runtime_api::CallbackContext;
use crate::simulation::GameEventReceipt;

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    amount: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    let actor = &mut state.actors[actor_index];
    let before = crate::simulation::energy::EnergySnapshot::capture(actor);
    let charge_basis = crate::simulation::ExactFraction::new(amount.into(), 1)?;
    let _ = actor.grant_energy(amount)?;

    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "mint_a6_energy".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: Some(before.receipt(
            actor,
            crate::simulation::GameEnergySource::MintA6,
            actor_index,
            actor_index,
            Some(charge_basis),
        )?),
        energy_fanout: None,
        data: None,
    });
    Ok(())
}
