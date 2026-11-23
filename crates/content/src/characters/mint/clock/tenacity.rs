//! Explicit .55s cue: actor .01 dilation and a separate named-timer pause.
use super::*;
const DURATION_US: u64 = 550_000;

pub(super) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    let mut previous = 0;
    for (index, event) in req.tenacity_events.iter().enumerate() {
        if event.timestamp_us < previous || event.timestamp_us > req.absolute_limit_us {
            return Err(format!(
                "tenacity event{index} requires ordered times within the horizon"
            ));
        }
        if !super::accepted(req)
            || req
                .actors
                .get(event.actor_index)
                .is_none_or(|a| a.character_id != "character:1019")
        {
            return Err(format!(
                "tenacity event{index} requires Mint and ordinary_clock_v1"
            ));
        }
        event
            .timestamp_us
            .checked_add(DURATION_US)
            .ok_or("Mint tenacity expiry overflow")?;
        previous = event.timestamp_us;
    }
    Ok(())
}
pub(crate) fn initialize(ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    for (source, event) in ctx.request.tenacity_events.iter().enumerate() {
        push(
            ctx.queue,
            ctx.sequence,
            event.actor_index,
            source,
            Marker::TenacityBegin,
            event.timestamp_us,
        )?;
    }
    Ok(())
}
pub(super) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[event.owner];
    let mut clock = actor.character.mint_mut()?.clock.take().unwrap_or_default();
    let active = match event.marker {
        Marker::TenacityBegin if actor.hp > 0 => {
            crate::runtime_api::record_hypothesis_id(
                ctx.request,
                &mut ctx.state.execution_model_assumptions,
                MODEL,
            )?;
            crate::runtime_api::record_hypothesis_id(
                ctx.request,
                &mut ctx.state.execution_model_assumptions,
                super::super::hold::HYPOTHESIS_ID,
            )?;
            let window = HitlagWindow {
                actor_index: event.owner,
                start_real_us: now,
                duration_real_us: DURATION_US,
                dilated_rate_ppm: 10000,
            };
            clock
                .lag
                .add_window(window.clone())
                .map_err(|e| format!("Mint tenacity actor dilation: {e:?}"))?;
            clock
                .timer_lag
                .add_window(HitlagWindow {
                    dilated_rate_ppm: 0,
                    ..window.clone()
                })
                .map_err(|e| format!("Mint named timer pause: {e:?}"))?;
            clock.tenacity_sources.insert(event.command, window);
            push(
                ctx.queue,
                ctx.sequence,
                event.owner,
                event.command,
                Marker::TenacityEnd,
                now.checked_add(DURATION_US)
                    .ok_or("Mint tenacity expiry overflow")?,
            )?;
            project(&clock, actor, ctx.queue, now, false)?;
            true
        }
        Marker::TenacityEnd => {
            clock.tenacity_sources.remove(&event.command);
            false
        }
        Marker::TenacityBegin => {
            actor.character.mint_mut()?.clock = Some(clock);
            return Ok(());
        }
        _ => return Err("Mint tenacity received a montage marker".into()),
    };
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "mint_tenacity_cue".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(event.owner),
        to_actor: Some(event.owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(event.command),
            active: Some(active),
            count: Some(clock.tenacity_sources.len()),
            ..Default::default()
        }),
    });
    actor.character.mint_mut()?.clock = Some(clock);
    Ok(())
}
