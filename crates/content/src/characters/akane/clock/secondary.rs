use super::*;

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
        .akane_mut()?
        .clock
        .take()
        .ok_or("Akane Evade clock missing")?;
    clock.connected.clear();
    clock.action = Some(Action {
        owner,
        command,
        start: now,
        input_start: now,
        recovery: if back {
            profile::EVADE_BACK_RECOVERY
        } else {
            profile::EVADE_FORWARD_RECOVERY
        },
        kind: Kind::Dodge(back),
        groups: BTreeMap::new(),
    });
    project(&clock, actor, ctx.queue, now)?;
    actor.character.akane_mut()?.clock = Some(clock);
    Ok(())
}
