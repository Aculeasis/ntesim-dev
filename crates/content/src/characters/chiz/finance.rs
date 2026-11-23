//! Exact savings, admitted held-Skill principal and living Surplus interest.
//! The admitted market owns investment, captured prices and repay-first exit.
use crate::runtime_api::{ActorHitContext, CallbackContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameActorConfig, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::state::{WidePool, wide_fraction};
use num_traits::Zero;
mod loans;
#[cfg(test)]
mod tests;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizExactFinanceV1;

#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(crate) wallet: WidePool,
    pub(crate) debt: Option<WidePool>,
    pub(crate) market: Option<Box<super::market::State>>,
    pub(super) interest: Option<Interest>,
}

#[derive(Clone, Debug)]
pub(super) struct Interest {
    pub(super) since_us: u64,
    pub(super) next_due_us: Option<u64>,
    pub(super) generation: u64,
    pub(super) source_alive: bool,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn observes(request: &GameScenarioRequest, config: &GameActorConfig) -> bool {
    accepted(request)
        && (config.awakening.has_effect(2)
            || super::market::accepted(request)
            || super::secondary::accepted(request))
}

pub(crate) fn initialize(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    if !accepted(request) {
        return Ok(());
    }
    let living = actor.hp > 0;
    let state = actor.character.chiz_mut()?;
    let grain = state
        .grain
        .take()
        .ok_or("Chiz exact finance requires initialized Grain")?;
    let wallet = WidePool::from_exact(&grain)?;
    let debt = if super::pressed::held::accepted(request) || super::market::accepted(request) {
        let cap = grain.summary()?.capacity;
        Some(WidePool::from_exact(
            &ntesim_core::kernel::state::ExactPool::new(
                cap,
                crate::simulation::ExactFraction::from_ratio(
                    config
                        .initial_resources
                        .loans
                        .unwrap_or(crate::formula::Ratio::zero()),
                )?,
            )?,
        )?)
    } else {
        None
    };
    state.finance = Some(Box::new(State {
        wallet,
        debt,
        market: None,
        interest: config.awakening.has_effect(2).then_some(Interest {
            since_us: 0,
            next_due_us: None,
            generation: 0,
            source_alive: living,
        }),
    }));
    Ok(())
}

fn arm(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
    owner: usize,
    now: u64,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
) -> Result<(), String> {
    let living = actor.hp > 0;
    let Some(finance) = &mut actor.character.chiz_mut()?.finance else {
        return Ok(());
    };
    let positive = !finance.balance().is_zero();
    let surplus = finance.in_surplus();
    let Some(clock) = &mut finance.interest else {
        return Ok(());
    };
    if !living || !surplus {
        if clock.source_alive {
            clock.generation = clock
                .generation
                .checked_add(1)
                .ok_or("Chiz interest generation overflow")?;
            clock.source_alive = false;
        }
        if clock.next_due_us.take().is_some() {
            queue.retain(|event| !matches!(&event.data,GameEventData::Content(crate::callbacks::ContentEvent::Chiz(super::events::Event::Interest {owner:source,..})) if *source==owner));
        }
        return Ok(());
    }
    if !clock.source_alive {
        clock.source_alive = true;
        clock.since_us = now;
        clock.generation = clock
            .generation
            .checked_add(1)
            .ok_or("Chiz interest generation overflow")?;
    }
    if !positive {
        return Ok(());
    }
    if clock.next_due_us.is_some() {
        return Ok(());
    }
    let interval = super::parameters::PARAMETERS.interest_interval_us;
    let elapsed = now
        .checked_sub(clock.since_us)
        .ok_or("Chiz interest time reversal")?;
    let next = (elapsed / interval)
        .checked_add(1)
        .and_then(|n| n.checked_mul(interval))
        .and_then(|offset| clock.since_us.checked_add(offset))
        .ok_or("Chiz interest deadline overflow")?;
    clock.next_due_us = Some(next);
    // Retain the continuing phase without inventing an event beyond this
    // observation horizon or polling while the wallet has no positive balance.
    if next <= request.absolute_limit_us {
        crate::runtime_api::increment_sequence(sequence)?;
        queue.push(ScheduledGameEvent {
            time_us: next,
            priority: crate::execution::CausalPriority::ModePause,
            seq: *sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
                super::events::Event::Interest {
                    owner,
                    generation: clock.generation,
                },
            )),
        });
    }
    Ok(())
}

pub(crate) fn refresh(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    arm(
        ctx.request,
        &mut ctx.state.actors[owner],
        owner,
        ctx.state.current_time_us,
        ctx.queue,
        ctx.sequence,
    )
}

pub(crate) fn after_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    arm(
        ctx.request,
        ctx.actor,
        ctx.actor_index,
        ctx.time_us,
        ctx.queue,
        ctx.sequence,
    )
}

pub(crate) fn interest(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return refresh(ctx, owner);
    }
    let now = ctx.state.current_time_us;
    let Some(finance) = &mut ctx.state.actors[owner].character.chiz_mut()?.finance else {
        return Ok(());
    };
    let debt = finance.in_debt();
    let Some(clock) = &mut finance.interest else {
        return Ok(());
    };
    if !clock.source_alive || clock.generation != generation || clock.next_due_us != Some(now) {
        return Ok(());
    }
    clock.next_due_us = None;
    if debt {
        return refresh(ctx, owner);
    }
    let basis = finance.balance().clone();
    if basis.is_zero() {
        return Ok(());
    }
    let amount = &basis * wide_fraction(super::parameters::PARAMETERS.surplus_interest)?;
    let (_, overflow) = finance.balance_mut().grant(amount.clone())?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "chiz_surplus_interest".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            wide_basis: Some(ntesim_core::kernel::state::WideFraction::from(&basis)),
            wide_amount: Some(ntesim_core::kernel::state::WideFraction::from(&amount)),
            wide_overflow: Some(ntesim_core::kernel::state::WideFraction::from(&overflow)),
            ..Default::default()
        }),
    });
    refresh(ctx, owner)
}

impl State {
    pub(crate) fn pause_world(&mut self, _from: u64, delta: u64) -> Result<(), String> {
        if let Some(interest) = &mut self.interest {
            crate::simulation::timer_state::anchor(&mut interest.since_us, delta)?;
            if let Some(due) = &mut interest.next_due_us {
                crate::simulation::timer_state::anchor(due, delta)?;
            }
        }
        if let Some(market) = &mut self.market
            && let Some(session) = &mut market.session
        {
            crate::simulation::timer_state::anchor(&mut session.ends_at_us, delta)?;
        }
        Ok(())
    }
}
