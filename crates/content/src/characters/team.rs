use crate::runtime_api::{HitModifiers, RecipientContext};

pub(crate) fn observe_admission(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    actor_index: usize,
    command_index: usize,
    ability: &str,
    metadata: &crate::simulation::metadata::ActionMetadata,
    assumptions: &mut std::collections::BTreeMap<String, String>,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
) -> Result<(), String> {
    crate::characters::daffodil::resonance::observe_admission(
        request,
        state,
        actor_index,
        metadata,
        assumptions,
        trace,
    )?;
    crate::characters::edgar::observe_admission(
        request,
        state,
        actor_index,
        ability,
        metadata,
        assumptions,
        trace,
    )?;
    crate::characters::iroi::clock::a2::observe_team_admission(
        request,
        state,
        actor_index,
        command_index,
        metadata,
        queue,
        sequence,
    )
}

pub(crate) fn modifiers(
    ctx: &RecipientContext<'_>,
    value: HitModifiers,
) -> Result<HitModifiers, String> {
    if ctx.source_actor.hp == 0 {
        return Ok(value);
    }
    crate::registry::recipient_modifiers(ctx, value)
}

pub(crate) fn on_target_defeated(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    target_index: usize,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) -> Result<(), String> {
    crate::characters::daffodil::insight::clear_target(
        &mut state.buffs,
        target_index,
        state.actors.len(),
    );
    crate::characters::daffodil::discord::clear_target(state, target_index);
    on_target_recovered(state, target_index);
    crate::characters::hotori::timepiece::on_team_target_defeated(
        request,
        state,
        target_index,
        trace,
    )
}

pub(crate) fn on_target_broken(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    target_index: usize,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) -> Result<(), String> {
    crate::characters::daffodil::broken::start(request, state, target_index, trace)
}

pub(crate) fn on_broken_extended(
    state: &mut crate::simulation::state::GameSimulatorState,
    target_index: usize,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) -> Result<(), String> {
    crate::characters::daffodil::broken::extend(state, target_index, trace)
}

pub(crate) fn on_target_recovered(
    state: &mut crate::simulation::state::GameSimulatorState,
    target_index: usize,
) {
    crate::characters::daffodil::broken::clear(state, target_index);
}
