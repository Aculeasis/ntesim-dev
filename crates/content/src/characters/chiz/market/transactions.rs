use super::*;

pub(crate) fn capture(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let finance = ctx
        .actor
        .character
        .chiz_mut()?
        .finance
        .as_mut()
        .ok_or("Chiz capture finance missing")?;
    if let Some(market) = &mut finance.market
        && let Some(session) = &mut market.session
    {
        if session.closing {
            return Err("Chiz market is closing; no new financial input admitted".into());
        }
        session
            .pending
            .insert(ctx.command_index, session.prices[session.index]);
    }
    Ok(())
}

pub(crate) fn settle(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<Option<(u128, bool, u64)>, String> {
    let finance = ctx.state.actors[owner]
        .character
        .chiz_mut()?
        .finance
        .as_mut()
        .ok_or("Chiz settlement finance missing")?;
    let Some(market) = &mut finance.market else {
        return Ok(None);
    };
    let Some(session) = &mut market.session else {
        return Ok(None);
    };
    let Some(price) = session.pending.remove(&command) else {
        return Ok(None);
    };
    let amount = finance.revalue(price)?;
    let market = finance.market.as_mut().unwrap();
    let force = amount > 0 && market.first_crit;
    let generation = market.generation;
    let session = market.session.as_ref().unwrap();
    let close = session.closing && session.pending.is_empty();
    receipt(
        ctx,
        "chiz_market_settlement",
        owner,
        crate::simulation::GameEventPayload {
            command_index: Some(command),
            price_pct: Some(i32::from(price)),
            exact_amount: Some(ExactFraction::new(amount, 1)?),
            first_crit: Some(force),
            ..Default::default()
        },
    );
    if close {
        exit(ctx, owner, false)?;
    }
    Ok(Some((amount, force, generation)))
}

pub(crate) fn exit(ctx: &mut CallbackContext<'_>, owner: usize, death: bool) -> Result<(), String> {
    let Some(finance) = &mut ctx.state.actors[owner].character.chiz_mut()?.finance else {
        return Ok(());
    };
    let Some(market) = &mut finance.market else {
        return Ok(());
    };
    let Some(session) = &mut market.session else {
        return Ok(());
    };
    session.closing = true;
    let generation = market.generation;
    ctx.queue.retain(|event|!matches!(&event.data,crate::simulation::events::GameEventData::Content(crate::callbacks::ContentEvent::Chiz(super::super::events::Event::Market(Event::Price{owner:source,generation:epoch,..}|Event::Exit{owner:source,generation:epoch}))) if *source==owner && *epoch==generation));
    if death {
        session.pending.clear();
    }
    if !session.pending.is_empty() {
        return super::super::finance::refresh(ctx, owner);
    }
    finance.withdraw()?;
    if let Some(clock) = &mut finance.interest {
        clock.source_alive = false;
        clock.next_due_us = None;
    }
    receipt(
        ctx,
        "chiz_market_exit",
        owner,
        crate::simulation::GameEventPayload {
            death: Some(death),
            ..Default::default()
        },
    );
    super::super::finance::refresh(ctx, owner)
}
