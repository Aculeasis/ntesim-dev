//! Finite World-time investment, reserved transactions and repay-first exit.
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{ExactFraction, GameEventReceipt, GameScenarioRequest};
use ntesim_core::kernel::{rng::Rng, state::WidePool};
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
mod ledger;
mod prices;
mod transactions;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizMarketV1;
const TRACE: crate::HypothesisId = crate::HypothesisId::ChizRecordedMarketTraceV1;
const BURST: &str = "GA_Chiichan073_UltraSkill";

#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(crate) wallet: WidePool,
    pub(crate) session: Option<Session>,
    rng: Rng,
    pub(super) generation: u64,
    pub(super) first_crit: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct Session {
    pub(crate) ends_at_us: u64,
    prices: [i16; 14],
    index: usize,
    pending: BTreeMap<usize, i16>,
    pub(crate) closing: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Enter {
        owner: usize,
        command: usize,
    },
    Price {
        owner: usize,
        generation: u64,
        index: usize,
    },
    Exit {
        owner: usize,
        generation: u64,
    },
}
pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    accepts(request, MODEL)
}
fn accepts(request: &GameScenarioRequest, id: crate::HypothesisId) -> bool {
    request
        .model_assumptions
        .get(id.as_str())
        .map(String::as_str)
        == Some(id.description())
}
pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if accepts(request, TRACE) && !accepted(request) {
        return Err("Chiz recorded market trace requires market_v1".into());
    }
    if accepted(request)
        && (!super::finance::accepted(request) || !super::pressed::accepted(request))
    {
        return Err("Chiz market requires exact finance and pressed Skill models".into());
    }
    Ok(())
}
fn enqueue(ctx: &mut CallbackContext<'_>, time: u64, event: Event) -> Result<(), String> {
    super::pressed::enqueue(
        ctx.queue,
        ctx.sequence,
        time,
        super::events::Event::Market(event),
    )
}
fn receipt(
    ctx: &mut CallbackContext<'_>,
    kind: &str,
    owner: usize,
    data: crate::simulation::GameEventPayload,
) {
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ModePause".into(),
        event_kind: kind.into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(data),
    });
}
pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Err("UNSUPPORTED_CHIZ_GRAIN_MARKET: accept chiz.market_v1 to bind the selected financial model".into());
    }
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(BURST)
    {
        return Err("Chiz market requires canonical Burst input".into());
    }
    ctx.record_hypothesis(MODEL)?;
    if accepts(ctx.request, TRACE) {
        ctx.record_hypothesis(TRACE)?;
    }
    let primary = ctx.scope_primary_aoe_group("GE_Player_Chiichan073_UltraSkill1_Damage")?;
    let mut groups = BTreeMap::from([(primary, (2_346_949, 10_000))]);
    for (effect, offset) in [
        ("GE_Player_Chiichan073_UltraSkill1_Damage", 2_456_949),
        ("GE_Player_Chiichan073_UltraSkill1_Damage", 2_556_949),
        ("GE_Player_Chiichan073_UltraSkill1_Damage", 2_656_949),
        ("GE_Player_Chiichan073_UltraSkill2_Damage", 2_836_949),
        ("GE_Player_Chiichan073_UltraSkill3_Damage", 3_326_949),
    ] {
        groups.insert(
            ctx.schedule_native_aoe_group(effect, ctx.time_us)?,
            (2_346_949, offset - 2_346_949),
        );
    }
    ctx.actor.switch_action_locked_until_us = ctx
        .time_us
        .checked_add(4_250_000)
        .ok_or("Chiz Burst switch lock overflow")?;
    ctx.actor.ability_cooldowns.insert(
        BURST.into(),
        ctx.time_us
            .checked_add(15_000_000)
            .ok_or("Chiz Burst cooldown overflow")?,
    );
    super::pressed::enqueue(
        ctx.queue,
        ctx.sequence,
        ctx.time_us
            .checked_add(3_420_264)
            .ok_or("Chiz market entry overflow")?,
        super::events::Event::Market(Event::Enter {
            owner: ctx.actor_index,
            command: ctx.command_index,
        }),
    )?;
    super::clock::start(ctx, super::clock::Kind::Burst, 0, groups)
}
pub(crate) fn apply(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::Enter { owner, command } => enter(ctx, owner, command),
        Event::Exit { owner, generation } => {
            let current = ctx.state.actors[owner]
                .character
                .chiz_mut()?
                .finance
                .as_ref()
                .and_then(|finance| finance.market.as_ref())
                .map(|market| market.generation);
            if current == Some(generation) {
                exit(ctx, owner, false)?;
            }
            Ok(())
        }
        Event::Price {
            owner,
            generation,
            index,
        } => {
            let market = ctx.state.actors[owner]
                .character
                .chiz_mut()?
                .finance
                .as_mut()
                .and_then(|finance| finance.market.as_mut());
            if let Some(market) = market
                && market.generation == generation
                && let Some(session) = &mut market.session
                && !session.closing
            {
                session.index = index;
                let price = session.prices[index];
                receipt(
                    ctx,
                    "chiz_market_price",
                    owner,
                    crate::simulation::GameEventPayload {
                        price_index: Some(index),
                        price_pct: Some(i32::from(price)),
                        ..Default::default()
                    },
                );
            }
            Ok(())
        }
    }
}
fn enter(ctx: &mut CallbackContext<'_>, owner: usize, command: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 || ctx.state.active_actor_index != owner {
        return Ok(());
    }
    exit(ctx, owner, false)?;
    let now = ctx.state.current_time_us;
    let ends_at_us = now
        .checked_add(super::parameters::PARAMETERS.market_duration_us)
        .ok_or("Chiz expiry overflow")?;
    let finance = ctx.state.actors[owner]
        .character
        .chiz_mut()?
        .finance
        .as_mut()
        .ok_or("Chiz market finance missing")?;
    finance.invest(
        ctx.state.seed,
        owner,
        ctx.request.actors[owner].awakening.has_effect(3),
        accepts(ctx.request, TRACE),
        ends_at_us,
    )?;
    let generation = finance.market.as_ref().unwrap().generation;
    let first_price = finance
        .market
        .as_ref()
        .unwrap()
        .session
        .as_ref()
        .unwrap()
        .prices[0];
    // Entry starts a fresh forced-Surplus phase, even if ordinary debt existed.
    if let Some(clock) = &mut finance.interest {
        clock.source_alive = false;
        clock.next_due_us = None;
    }
    enqueue(ctx, ends_at_us, Event::Exit { owner, generation })?;
    for index in 1..14 {
        enqueue(
            ctx,
            now.checked_add(index as u64 * 500_000)
                .ok_or("Chiz price deadline overflow")?,
            Event::Price {
                owner,
                generation,
                index,
            },
        )?;
    }
    receipt(
        ctx,
        "chiz_market_entry",
        owner,
        crate::simulation::GameEventPayload {
            command_index: Some(command),
            generation: Some(generation),
            count: Some(28),
            active: Some(accepts(ctx.request, TRACE)),
            ..Default::default()
        },
    );
    receipt(
        ctx,
        "chiz_market_price",
        owner,
        crate::simulation::GameEventPayload {
            price_index: Some(0),
            price_pct: Some(i32::from(first_price)),
            ..Default::default()
        },
    );
    super::finance::refresh(ctx, owner)
}
pub(crate) use transactions::{capture, exit, settle};
