//! Actual accepted Power deltas and direct self energy, independent of Charge.
use crate::runtime_api::CallbackContext;
use crate::simulation::{
    ExactFraction as F, GameActorConfig, GameScenarioRequest, state::ActorRuntime,
};
use ntesim_core::kernel::state::ExactPool;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorTapPowerV1;
#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(crate) pool: ExactPool,
    pub(crate) earned: u8,
    pub(crate) a1_used: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Kill {
    pub(crate) owner: usize,
    target: usize,
    battle: u64,
}
#[derive(Clone, Copy)]
pub(super) enum GazeEnd {
    Natural,
    Attack,
    Contact,
}
impl GazeEnd {
    pub(super) fn natural(self) -> bool {
        matches!(self, Self::Natural)
    }
    pub(super) fn phase(self) -> &'static str {
        match self {
            Self::Natural => "BuffTickOrExpiry",
            Self::Attack => "ProcCallback",
            Self::Contact => "HitDelivery",
        }
    }
}
pub(super) enum Cause {
    Riposte,
    Tap,
    Gaze { end: GazeEnd, enhanced: bool },
    FirstHeld,
    OwnKill { target: usize },
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn initialize(
    req: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    if accepted(req) {
        let pool = initial_pool(config.initial_resources.power)?;
        let state = actor.character.hathor_mut()?;
        state.express_delivery_power =
            u32::try_from(pool.current().numerator).map_err(|_| "Hathor initial Power overflow")?;
        state.power = Some(Box::new(State {
            pool,
            earned: 0,
            a1_used: false,
        }));
    }
    Ok(())
}
pub(crate) fn initial_pool(initial: Option<crate::formula::Ratio>) -> Result<ExactPool, String> {
    let value = initial.map(F::from_ratio).transpose()?.unwrap_or_default();
    if value.denominator != 1 {
        return Err("Hathor Power must be integral.".into());
    }
    ExactPool::new(F::new(6, 1)?, value)
}
pub(super) fn grant(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    cause: Cause,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let (amount, cause_key, condition_key, target_index, phase): (
        u64,
        &str,
        Option<&str>,
        Option<usize>,
        &str,
    ) = match cause {
        Cause::Riposte => (1, "riposte", None, None, "ProcCallback"),
        Cause::FirstHeld => (6, "first_held", None, None, "ActionAdmission"),
        Cause::Gaze { end, enhanced } => (
            if enhanced { 3 } else { 1 },
            if enhanced { "gaze_ex" } else { "gaze" },
            Some(match (end.natural(), enhanced) {
                (true, _) => "natural_expiry",
                (false, true) => "consumption",
                (false, false) => "attack_consumption",
            }),
            None,
            end.phase(),
        ),
        Cause::Tap => (2, "tap", None, None, "ActionAdmission"),
        Cause::OwnKill { target } => (1, "own_kill", None, Some(target), "ProcCallback"),
    };
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let blocked = if ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .out_of_combat
    {
        Some("outside_combat")
    } else if super::burst::active(&ctx.state.actors[owner], ctx.state.current_time_us) {
        Some("during_emergency")
    } else {
        None
    };
    if let Some(blocked) = blocked {
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: phase.into(),
            event_kind: "hathor_power_rejected".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(owner),
            to_actor: Some(owner),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                cause: Some(cause_key.into()),
                condition: condition_key.map(str::to_owned),
                reason: Some(blocked.into()),
                target_index,
                amount: Some(amount),
                applied: Some(0),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    let actor = &mut ctx.state.actors[owner];
    let snapshot = crate::simulation::energy::EnergySnapshot::capture(actor);
    let state = actor.character.hathor_mut()?;
    let power = state
        .power
        .as_mut()
        .ok_or("Hathor accepted Power pool missing")?;
    let (applied, excess) = power.pool.grant(F::new(amount.into(), 1)?)?;
    if applied.denominator != 1 {
        return Err("Hathor Power must remain integral".into());
    }
    let delta = u64::try_from(applied.numerator).map_err(|_| "Hathor Power delta overflow")?;
    state.express_delivery_power = u32::try_from(power.pool.current().numerator)
        .map_err(|_| "Hathor Power balance overflow")?;
    if ctx.request.actors[owner].awakening.has_effect(5) {
        power.earned = (u64::from(power.earned) + delta).min(15) as u8;
    }
    let current = state.express_delivery_power;
    actor.restore_energy(
        delta
            .checked_mul(5)
            .ok_or("Hathor direct energy overflow")?,
    )?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: phase.into(),
        event_kind: "hathor_power_gain".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: Some(snapshot.receipt(
            actor,
            crate::simulation::GameEnergySource::HathorPower,
            owner,
            owner,
            None,
        )?),
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            cause: Some(cause_key.into()),
            condition: condition_key.map(str::to_owned),
            target_index,
            amount: Some(amount),
            applied: Some(delta),
            exact_overflow: Some(excess),
            pool: Some(u64::from(current)),
            ..Default::default()
        }),
    });
    crate::simulation::energy::notify_active_actor_energy_gain(ctx, owner, delta > 0)?;
    Ok(())
}
pub(crate) fn on_target_defeated(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
) -> Result<(), String> {
    if !accepted(ctx.request)
        || ctx.request.actors[owner].character_id != "character:1025"
        || ctx.state.actors[owner].hp == 0
    {
        return Ok(());
    }
    let config = &ctx.request.actors[owner];
    let tier = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?["breakthrough"]
        .as_u64()
        .unwrap_or(0);
    if tier < 4 {
        return Ok(());
    }
    let battle = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .battle_generation;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx.state.current_time_us,
            priority: ntesim_core::execution::CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::HathorKill(Kill {
                    owner,
                    target,
                    battle,
                }),
            ),
        });
    Ok(())
}
pub(crate) fn kill(ctx: &mut CallbackContext<'_>, event: Kill) -> Result<(), String> {
    let s = ctx.state.actors[event.owner].character.hathor_mut()?;
    if !s.out_of_combat && s.battle_generation != event.battle {
        return Ok(());
    }
    grant(
        ctx,
        event.owner,
        Cause::OwnKill {
            target: event.target,
        },
    )
}
pub(crate) fn defeat(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.request.actors[owner].character_id != "character:1025" {
        return Ok(());
    }
    let state = ctx.state.actors[owner].character.hathor_mut()?;
    if let Some(power) = &mut state.power {
        let removed = power.pool.spend_up_to(power.pool.current())?;
        state.express_delivery_power = 0;
        ctx.trace.push(crate::simulation::GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "SettlementAndDeath".into(),
            event_kind: "hathor_power_death".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(owner),
            to_actor: Some(owner),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                exact_amount: Some(removed),
                count: Some(usize::from(power.earned)),
                ..Default::default()
            }),
        });
    }
    Ok(())
}
