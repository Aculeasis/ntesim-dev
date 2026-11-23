//! Explicit interruption keeps native emission ownership and committed costs.
use super::cancel::{EmissionKind, Window};
use crate::runtime_api::CallbackContext;
use crate::simulation::{GameActorInterruptReceipt, GameScenarioRequest, events::GameEventData};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::BaicangForcedInterruptV1;

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(request) || !super::words::accepted(request) {
        return Err("Baicang interrupt_events require baicang.forced_interrupt_v1 and baicang.power_words_v1".into());
    }
    Ok(())
}

fn cancelled_deliveries(
    ctx: &CallbackContext<'_>,
    actor_index: usize,
    window: Window,
) -> Result<std::collections::BTreeSet<u64>, String> {
    let mut cancelled = std::collections::BTreeSet::new();
    for event in ctx.queue.iter() {
        if let GameEventData::HitDelivery {
            command_index,
            actor_index: source,
            ability_id,
            damage_effect_id,
            ..
        } = &event.data
            && *command_index == window.command_index
            && *source == actor_index
            && ability_id == window.ability_id
            && !damage_effect_id.starts_with("GE_Player_Cang_World")
            && damage_effect_id != "GE_Player_Cang_UltraSkill_Damage"
            && emission_at(
                ctx,
                actor_index,
                window,
                event.seq,
                damage_effect_id,
                event.time_us,
            )? > ctx.state.current_time_us
        {
            cancelled.insert(event.seq);
        }
    }
    Ok(cancelled)
}

fn emission_at(
    ctx: &CallbackContext<'_>,
    actor: usize,
    window: Window,
    seq: u64,
    effect: &str,
    impact: u64,
) -> Result<u64, String> {
    let crate::characters::RuntimeState::Baicang(state) = &ctx.state.actors[actor].character else {
        return Err("Baicang interruption state missing".into());
    };
    if let Some(time) = state.clock.emitted_at(seq) {
        return Ok(time);
    }
    window.emitted_at(effect, impact)
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    event_index: usize,
) -> Result<GameActorInterruptReceipt, String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let now = ctx.state.current_time_us;
    let actor = &ctx.state.actors[actor_index];
    let mut receipt = GameActorInterruptReceipt {
        event_index,
        actor_index,
        time_us: now,
        interrupted_command_index: None,
        cancelled_hit_deliveries: 0,
        recovery_before_us: actor.locked_until_us,
        recovery_after_us: actor.locked_until_us,
        skill_cooldown_before_us: None,
        skill_cooldown_after_us: None,
        field_cancelled: false,
    };
    let crate::characters::RuntimeState::Baicang(state) = &actor.character else {
        return Err("Baicang interruption state missing".into());
    };
    let Some(window) = state
        .cancel_window
        .filter(|_| actor.hp > 0 && now <= actor.locked_until_us)
    else {
        return Ok(receipt);
    };
    let cancelled = cancelled_deliveries(ctx, actor_index, window)?;
    let skill_deadline = state
        .clock
        .first_skill_emission()
        .or(window.first_skill_emission()?)
        .filter(|first| now < *first)
        .map(|_| {
            now.checked_add(3_000_000)
                .ok_or("Baicang cancel cooldown overflow".to_owned())
        })
        .transpose()?;
    let derived_end = now
        .checked_add(2_000_000)
        .ok_or("Baicang derived expiry overflow")?;
    let field_cancelled = window.emission == EmissionKind::Burst && now < state.domain_start_us;
    let field_generation = state
        .domain_generation
        .checked_add(u64::from(field_cancelled))
        .ok_or("Baicang field generation overflow")?;
    // Preflight all fallible projection/deadline work before changing the actor or queue.
    let mut events = ctx.queue.drain().collect::<Vec<_>>();
    events.retain(|event| !cancelled.contains(&event.seq));
    ctx.queue.replace(events);
    let actor = &mut ctx.state.actors[actor_index];
    if window.ability_id == "GA_Cang_Skill" {
        receipt.skill_cooldown_before_us = actor.ability_cooldowns.get("GA_Cang_Skill").copied();
        if let Some(deadline) = skill_deadline {
            actor
                .ability_cooldowns
                .insert("GA_Cang_Skill".into(), deadline);
        }
        receipt.skill_cooldown_after_us = actor.ability_cooldowns.get("GA_Cang_Skill").copied();
    }
    actor.locked_until_us = now;
    let state = actor.character.baicang_mut()?;
    state.cancel_window = None;
    state.clock.end_action();
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.riposte_ready = false;
    state.heavy_dodge_until_us = 0;
    state.derived_skill_stage_expires_at_us = derived_end;
    if field_cancelled {
        state.domain_generation = field_generation;
        state.domain_until_us = now;
        state.fatal_hit_armed = false;
    }
    receipt.interrupted_command_index = Some(window.command_index);
    receipt.cancelled_hit_deliveries = cancelled.len();
    receipt.recovery_after_us = now;
    receipt.field_cancelled = field_cancelled;
    Ok(receipt)
}
