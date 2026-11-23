use super::*;

pub(crate) fn start(ctx: &mut CastContext<'_>, transformed: bool, hold: u64) -> Result<(), String> {
    let (offsets, recovery) = if transformed {
        (&[167_196, 883_187][..], 899_451)
    } else {
        (&[166_691, 883_658][..], 898_334)
    };
    let groups = packet_anchors(ctx, offsets)?;
    let mut clock = ctx
        .actor
        .character
        .chaos_mut()?
        .clock
        .take()
        .ok_or("Chaos Verdict clock missing")?;
    if let Some(previous) = clock.action.take() {
        ctx.queue.retain(|event| !belongs(event, &previous));
    }
    clock.tracker = HitlagTracker::default();
    clock.connected.clear();
    clock.action = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start: ctx
            .time_us
            .checked_add(hold)
            .ok_or("Chaos hold start overflow")?,
        recovery,
        input_start: ctx.time_us,
        kind: Kind::VerdictPlain(transformed),
        groups,
    });
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx
            .time_us
            .checked_add(hold)
            .and_then(|time| time.checked_add(recovery))
            .ok_or("Chaos Verdict read overflow")?,
        priority: ntesim_core::execution::CausalPriority::ModePause,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Chaos(
            super::super::events::Event::VerdictRead {
                owner: ctx.actor_index,
                command: ctx.command_index,
                progress: recovery,
            },
        )),
    });
    project(&clock, ctx.actor, ctx.queue, ctx.time_us)?;
    ctx.actor.character.chaos_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn launch(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    transformed: bool,
    groups: BTreeMap<String, u64>,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let mut clock = actor
        .character
        .chaos_mut()?
        .clock
        .take()
        .ok_or("Chaos full Verdict clock missing")?;
    clock.tracker = HitlagTracker::default();
    clock.connected.clear();
    clock.action = Some(Action {
        owner,
        command,
        start: now,
        input_start: now,
        recovery: 633_193,
        kind: Kind::VerdictFull(transformed),
        groups,
    });
    project(&clock, actor, ctx.queue, now)?;
    actor.character.chaos_mut()?.clock = Some(clock);
    Ok(())
}
