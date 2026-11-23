use super::{GameActorHpCostReceipt, GameEventPayload, GameEventReceipt};
use crate::runtime_api::{CallbackContext, record_hypothesis_id};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::CombatHpCostFixtureV1;

pub(super) fn validate(request: &super::GameScenarioRequest) -> Result<(), String> {
    if !request.hp_cost_events.is_empty()
        && request
            .model_assumptions
            .get(HYPOTHESIS_ID.as_str())
            .map(String::as_str)
            != Some(HYPOTHESIS_ID.description())
    {
        return Err(format!(
            "hp_cost_events require accepted model assumption {HYPOTHESIS_ID}"
        ));
    }
    let mut previous = 0;
    for (index, event) in request.hp_cost_events.iter().enumerate() {
        if event.timestamp_us < previous || event.timestamp_us > request.absolute_limit_us {
            return Err(format!(
                "HP cost event {index} timestamp violates monotonicity or horizon"
            ));
        }
        if event.source.trim().is_empty() || event.source.len() > 256 {
            return Err(format!(
                "HP cost event {index} requires a source of 1..256 bytes"
            ));
        }
        let actor = request
            .actors
            .get(event.actor_index)
            .ok_or_else(|| format!("HP cost event {index} actor index out of bounds"))?;
        for proc in &actor.gear_procs {
            if let super::GearProcConfig::JingmotingyuanCrit { duration_us, .. } = proc {
                event
                    .timestamp_us
                    .checked_add(*duration_us)
                    .ok_or("Silent Garden HP cost deadline overflow")?;
            }
        }
        previous = event.timestamp_us;
    }
    Ok(())
}

pub(super) fn settle(
    ctx: &mut CallbackContext<'_>,
    event_index: usize,
) -> Result<GameActorHpCostReceipt, String> {
    let event = &ctx.request.hp_cost_events[event_index];
    record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let config = &ctx.state.actors[event.actor_index];
    let max_hp = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        config.hp_max_base,
        &config.hp_max_static_bonus,
        event.actor_index,
        config.hp_max,
    )?;
    let actor = &mut ctx.state.actors[event.actor_index];
    let hp_before = actor.hp;
    // Consumption is nonlethal and bypasses damage absorption. Dead actors pay nothing.
    let actual_hp_loss = event.amount.min(hp_before.saturating_sub(1));
    actor.hp -= actual_hp_loss;
    actor.record_hp_loss(actual_hp_loss, max_hp);
    let receipt = GameActorHpCostReceipt {
        event_index,
        time_us: ctx.state.current_time_us,
        actor_index: event.actor_index,
        source: event.source.clone(),
        requested_hp: event.amount,
        hp_before,
        actual_hp_loss,
        hp_after: actor.hp,
    };
    ctx.trace.push(GameEventReceipt {
        time_us: receipt.time_us,
        phase: "HitDelivery".into(),
        event_kind: "actor_hp_cost".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(receipt.actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            event_index: Some(event_index),
            source: Some(receipt.source.clone()),
            amount: Some(event.amount),
            hp_before: Some(hp_before),
            applied: Some(actual_hp_loss),
            hp_after: Some(receipt.hp_after),
            ..Default::default()
        }),
    });
    crate::weapons::jingmotingyuan::on_hp_consumed(ctx, event.actor_index, actual_hp_loss)?;
    Ok(receipt)
}
