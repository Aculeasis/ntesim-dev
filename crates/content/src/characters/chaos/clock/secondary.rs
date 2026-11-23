use super::*;

pub(crate) fn cast(ctx: &mut CastContext<'_>, kind: Kind) -> Result<(), String> {
    let (landing, contact, recovery) = match kind {
        Kind::Plunge => (
            ctx.request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Chaos airborne clock profile missing")?
                .landing_delay_us,
            47_594,
            159_289,
        ),
        Kind::Riposte => (0, 250_157, 366_207),
        _ => return Err("Chaos secondary clock kind mismatch".into()),
    };
    let groups = packet_anchors(ctx, &[contact])?;
    let mut clock = ctx
        .actor
        .character
        .chaos_mut()?
        .clock
        .take()
        .ok_or("Chaos secondary clock missing")?;
    if let Some(previous) = clock.action.take() {
        ctx.queue.retain(|event| !belongs(event, &previous));
    }
    clock.connected.clear();
    clock.action = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        input_start: ctx.time_us,
        start: ctx
            .time_us
            .checked_add(landing)
            .ok_or("Chaos landing clock overflow")?,
        recovery,
        kind,
        groups,
    });
    project(&clock, ctx.actor, ctx.queue, ctx.time_us)?;
    ctx.actor.character.chaos_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn dodge(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    super::exit(ctx, owner)?;
    let now = ctx.state.current_time_us;
    let back = ctx.request.dash_directions.get(&command)
        == Some(&crate::simulation::GameDashDirection::Back);
    let actor = &mut ctx.state.actors[owner];
    let mut clock = actor
        .character
        .chaos_mut()?
        .clock
        .take()
        .ok_or("Chaos dodge clock missing")?;
    clock.connected.clear();
    clock.action = Some(Action {
        owner,
        command,
        input_start: now,
        start: now,
        recovery: if back { 795_195 } else { 998_284 },
        kind: Kind::Dodge(back),
        groups: BTreeMap::new(),
    });
    project(&clock, actor, ctx.queue, now)?;
    actor.character.chaos_mut()?.clock = Some(clock);
    Ok(())
}
