use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::CallbackContext;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt, GearProcConfig};
use ntesim_core::execution::CausalPriority;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsBeHappyV1;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) ready_at_us: u64,
}

fn reserve_cooldown(state: &mut State, time_us: u64, cooldown_us: u64) -> Result<bool, String> {
    if time_us < state.ready_at_us {
        return Ok(false);
    }
    state.ready_at_us = time_us
        .checked_add(cooldown_us)
        .ok_or("Be Happy cooldown deadline overflow")?;
    Ok(true)
}

pub(crate) fn on_cast(ctx: &mut crate::runtime_api::CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::BeHappy { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn on_target_defeated(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
) -> Result<(), String> {
    let config = &ctx.request.actors[actor_index];
    let actor = &mut ctx.state.actors[actor_index];
    let time_us = ctx.state.current_time_us;
    for proc in &config.gear_procs {
        let GearProcConfig::BeHappy {
            heal_ratio,
            cooldown_us,
        } = proc
        else {
            continue;
        };
        if !reserve_cooldown(&mut actor.equipment.be_happy, time_us, *cooldown_us)? {
            continue;
        }
        let hp_max = crate::simulation::state::effective_hp_max(
            &ctx.state.buffs,
            actor.hp_max_base,
            &actor.hp_max_static_bonus,
            actor_index,
            actor.hp_max,
        )?;
        let base_heal =
            ExactFraction::new(hp_max.into(), 1)?.mul(ExactFraction::from_ratio(*heal_ratio)?)?;
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us,
                priority: CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    crate::callbacks::ContentEvent::BeHappyHeal {
                        actor_index,
                        target_index,
                        base_heal,
                    },
                ),
            });
    }
    Ok(())
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    base_heal: ExactFraction,
) -> Result<(), String> {
    ctx.state.max_proc_depth_reached = ctx.state.max_proc_depth_reached.max(1);
    let before = ctx
        .state
        .actors
        .get(actor_index)
        .ok_or("Be Happy owner actor missing")?
        .hp;
    if before == 0 {
        let gross = base_heal.round_u64()?;
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "be_happy_heal".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: Some(actor_index),
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                target_index: Some(target_index),
                gross: Some(gross),
                applied: Some(0),
                overflow: Some(gross),
                reason: Some("dead_wearer".into()),
                ..Default::default()
            }),
        });
        return Ok(());
    }
    let settlement = ctx.settle_healing(actor_index, actor_index, base_heal)?;
    let gross_heal = settlement.gross;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "be_happy_heal".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            gross: Some(gross_heal),
            applied: Some(settlement.applied),
            overflow: Some(settlement.excess),
            hp_before: Some(before),
            hp_after: Some(settlement.after_hp),
            ..Default::default()
        }),
    });
    Ok(())
}
