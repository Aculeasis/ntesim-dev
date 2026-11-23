use super::*;

pub(crate) fn initialize(
    ctx: &mut CallbackContext<'_>,
) -> Result<BTreeMap<String, String>, String> {
    let mut assumptions = BTreeMap::new();
    if ctx
        .request
        .actors
        .iter()
        .any(|config| parameters(config).is_some())
    {
        record_hypothesis_id(ctx.request, &mut assumptions, HYPOTHESIS_ID)?;
        enter(ctx, ctx.state.active_actor_index)?;
    }
    Ok(assumptions)
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if parameters(ctx.config).is_some() {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

fn enter(ctx: &mut CallbackContext<'_>, actor_index: usize) -> Result<(), String> {
    let Some((_, interval_us)) = parameters(&ctx.request.actors[actor_index]) else {
        return Ok(());
    };
    let state = &mut ctx.state.actors[actor_index].equipment.motor_candy;
    state.generation = state
        .generation
        .checked_add(1)
        .ok_or("Motor Candy generation overflow")?;
    receipt(
        ctx,
        "motor_candy_field_entry",
        Some(actor_index),
        None,
        None,
    );
    schedule(ctx, actor_index, interval_us)
}

pub(crate) fn on_switch(
    ctx: &mut CallbackContext<'_>,
    previous_actor: usize,
) -> Result<(), String> {
    if parameters(&ctx.request.actors[previous_actor]).is_some() {
        let state = &mut ctx.state.actors[previous_actor].equipment.motor_candy;
        state.generation = state
            .generation
            .checked_add(1)
            .ok_or("Motor Candy generation overflow")?;
        ctx.state
            .buffs
            .remove_owned_buff(BUFF_ID, SOURCE, previous_actor);
        receipt(ctx, "motor_candy_reset", Some(previous_actor), None, None);
    }
    enter(ctx, ctx.state.active_actor_index)
}
