use crate::runtime_api::CallbackContext;

pub(crate) fn schedule(
    ctx: &mut crate::runtime_api::CastContext<'_>,
    delay: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(delay)
                .ok_or("Zankou spread notify overflow")?,
            priority: ntesim_core::execution::CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::ZankouSpread {
                    source_actor: ctx.actor_index,
                    from_target: ctx.target_index,
                },
            ),
        });
    Ok(())
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    source_actor: usize,
    from_target: usize,
) -> Result<(), String> {
    let now_us = ctx.state.current_time_us;
    if ctx.state.actors[source_actor].hp == 0
        || ctx.state.active_actor_index != source_actor
        || ctx.state.targets[from_target].hp_current == 0
    {
        return Ok(());
    }
    // Capture every donor before applying any copy, including to the donor.
    let carriers = ctx
        .state
        .actors
        .iter()
        .flat_map(|a| super::vile_ash::carriers(a, from_target, now_us))
        .collect::<Vec<_>>();
    let blood = ctx
        .state
        .actors
        .iter()
        .filter_map(|a| crate::characters::lacrimosa::dot::spread_carrier(a, from_target, now_us))
        .collect::<Vec<_>>();
    let adler = ctx
        .state
        .actors
        .iter()
        .enumerate()
        .filter_map(|(owner, actor)| {
            crate::characters::adler::skill::spread_source(actor, from_target, now_us)
                .then_some(owner)
        })
        .collect::<Vec<_>>();
    let existing_scorch = ctx
        .state
        .reactions
        .scorch
        .get(&from_target)
        .filter(|s| s.expires_at_us > now_us)
        .cloned();
    let mut copied = 0;
    for carrier in carriers.into_iter().take(4) {
        super::vile_ash::apply(ctx, carrier)?;
        copied += 1;
    }
    for (carrier, stacks) in blood.into_iter().take(4 - copied) {
        crate::characters::lacrimosa::dot::spread(ctx, carrier, stacks)?;
        copied += 1;
    }
    if copied < 4
        && let Some(existing) = existing_scorch
    {
        for target_idx in 0..ctx.state.targets.len() {
            if ctx.state.targets[target_idx].hp_current == 0 {
                continue;
            }
            for _ in 0..existing.stacks {
                crate::simulation::reactions::spread_scorch_to_target(
                    ctx,
                    target_idx,
                    existing.snapshot.clone(),
                )?;
            }
            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: now_us,
                phase: "CallbackDispatch".into(),
                event_kind: "zankou_scorch_spread".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: Some(source_actor),
                to_actor: None,
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    target_index: Some(from_target),
                    to_target_index: Some(target_idx),
                    ..Default::default()
                }),
            });
        }
        copied += 1;
    }
    for owner in adler.into_iter().take(4 - copied) {
        crate::characters::adler::skill::spread_train(ctx, owner)?;
        copied += 1;
    }
    if copied > 0 {
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: now_us,
            phase: "CallbackDispatch".into(),
            event_kind: "zankou_dot_spread".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(source_actor),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                target_index: Some(from_target),
                count: Some(copied),
                ..Default::default()
            }),
        });
    }
    Ok(())
}
