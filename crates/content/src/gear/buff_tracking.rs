pub(crate) fn active_stacks(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    actor_index: usize,
) -> u32 {
    crate::sets::fireflies::stack_count(buffs, actor_index)
}

pub(crate) fn stacks(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    actor_index: usize,
) -> Result<Vec<crate::simulation::GameFirefliesStack>, String> {
    crate::sets::fireflies::stacks(buffs, actor_index)
}

pub(crate) struct ExpireGearBuffsTracker {
    fireflies_before: [u32; 4],
    actors_count: usize,
}

pub(crate) fn pre_expire_gear_buffs(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    actors_count: usize,
) -> ExpireGearBuffsTracker {
    let mut fireflies_before = [0u32; 4];
    if buffs
        .instances()
        .iter()
        .any(|b| b.id == crate::sets::fireflies::BUFF_ID)
    {
        let count = actors_count.min(4);
        for (actor, slot) in fireflies_before.iter_mut().enumerate().take(count) {
            *slot = active_stacks(buffs, actor);
        }
    }
    ExpireGearBuffsTracker {
        fireflies_before,
        actors_count,
    }
}

pub(crate) fn post_expire_gear_buffs(
    tracker: ExpireGearBuffsTracker,
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    current_time_us: u64,
    active_actor_index: usize,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
) {
    let count = tracker.actors_count.min(4);
    for (actor, &before) in tracker.fireflies_before.iter().enumerate().take(count) {
        if before > 0 {
            let remaining = active_stacks(buffs, actor);
            if remaining < before {
                trace.push(crate::simulation::GameEventReceipt {
                    time_us: current_time_us,
                    phase: "BuffTickOrExpiry".into(),
                    event_kind: "fireflies_stack_expired".into(),
                    active_actor: active_actor_index,
                    hit_index: None,
                    from_actor: None,
                    to_actor: Some(actor),
                    energy_change: None,
                    energy_fanout: None,
                    data: Some(crate::simulation::GameEventPayload {
                        stacks: Some(remaining),
                        ..Default::default()
                    }),
                });
            }
        }
    }
}
