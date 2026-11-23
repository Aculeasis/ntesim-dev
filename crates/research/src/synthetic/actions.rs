use super::*;

pub(crate) struct Invocation {
    pub(crate) index: usize,
    pub(crate) clock: ClockId,
    pub(crate) local_time_us: u64,
    pub(crate) cause: u64,
}

pub(crate) fn invoke(
    context: Invocation,
    actor: &str,
    action: &str,
    effects: &[Effect],
    actors: &mut BTreeMap<String, Gate>,
    resources: &mut BTreeMap<String, Pool>,
    queue: &mut Scheduler<Task>,
) -> std::result::Result<Value, String> {
    let previous_gate = actors[actor].clone();
    let previous_resources = resources.clone();
    let admission = actors
        .get_mut(actor)
        .unwrap()
        .start(action, context.clock, context.local_time_us, resources)
        .map_err(|e| format!("{e:?}"))?;
    let tasks = effects
        .iter()
        .enumerate()
        .map(|(effect_index, effect)| Task::Apply {
            index: context.index,
            effect: effect.clone(),
            origin: Some(ActionOrigin {
                actor: actor.into(),
                action: action.into(),
                effect_index,
            }),
        })
        .collect();
    if let Err(error) = queue.schedule_batch_after(context.clock, 0, Some(context.cause), tasks) {
        actors.insert(actor.into(), previous_gate);
        *resources = previous_resources;
        return Err(format!("scheduler: {error:?}"));
    }
    Ok(json!({"actor":actor,"admission":admission}))
}
