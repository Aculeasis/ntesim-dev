use super::*;

pub(super) fn schedule(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    interval_us: u64,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    // A continuing passive's next tick outside the requested horizon is not
    // unfinished player input. Only schedule the ticks belonging to this run.
    if interval_us > ctx.request.absolute_limit_us - now {
        return Ok(());
    }
    increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: now + interval_us,
        priority: CausalPriority::BuffTickOrExpiry,
        seq: *ctx.sequence,
        data: GameEventData::Content(ContentEvent::MotorCandyTick {
            actor_index,
            generation: ctx.state.actors[actor_index]
                .equipment
                .motor_candy
                .generation,
        }),
    });
    Ok(())
}

pub(crate) fn tick(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    generation: u64,
) -> Result<(), String> {
    if ctx.state.active_actor_index != actor_index
        || ctx.state.actors[actor_index]
            .equipment
            .motor_candy
            .generation
            != generation
    {
        return Ok(());
    }
    if ctx.state.actors[actor_index].hp == 0 {
        ctx.state
            .buffs
            .remove_owned_buff(BUFF_ID, SOURCE, actor_index);
        return Ok(());
    }
    let (atk_per_stack, interval_us) = parameters(&ctx.request.actors[actor_index])
        .ok_or("Motor Candy tick lacks equipped parameters")?;
    ctx.state
        .buffs
        .apply_buff(
            BuffDefinition {
                stacking_scope: BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: BuffSourceLifetime::WhileAlive,
                id: BUFF_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::SelfActor,
                duration_us: None,
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: MAX_STACKS,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: BTreeMap::from([("atk_percent".into(), atk_per_stack)]),
            },
            actor_index,
            ctx.state.mode_clock.relative_us(),
        )
        .map_err(|error| format!("Motor Candy buff failed: {error:?}"))?;
    ctx.state.max_proc_depth_reached = ctx.state.max_proc_depth_reached.max(1);
    let stacks = ctx
        .state
        .buffs
        .instances()
        .iter()
        .find(|buff| buff.id == BUFF_ID && buff.owner_actor == actor_index)
        .ok_or("Motor Candy stack receipt missing")?
        .stacks;
    receipt(
        ctx,
        "motor_candy_atk_stack",
        Some(actor_index),
        Some(actor_index),
        Some(GameEventPayload {
            stacks: Some(stacks),
            max_stacks: Some(MAX_STACKS),
            ..Default::default()
        }),
    );
    if stacks < MAX_STACKS {
        schedule(ctx, actor_index, interval_us)?;
    }
    Ok(())
}
