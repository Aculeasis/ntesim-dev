//! Low-HP Fadia coordination is an autonomous owner event, never a Normal input.
use crate::runtime_api::CallbackContext;
use crate::simulation::{
    GameCausalTrigger, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
};

mod packet;
pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::BaicangFadiaAssistV1;
pub(crate) const ABILITY: &str = "GA_Cang_WorldPas_Fadia";
pub(crate) const PACKET: &str = "GE_Player_Cang_Fadia_Damage";

pub(crate) fn observes(
    request: &GameScenarioRequest,
    config: &crate::simulation::GameActorConfig,
) -> bool {
    request
        .actors
        .iter()
        .any(|actor| actor.character_id == "character:1039")
        && super::etd::has_passive(config)
}

fn recipient(ctx: &CallbackContext<'_>, owner: usize) -> Result<Option<usize>, String> {
    let active = ctx.state.active_actor_index;
    let actor = &ctx.state.actors[active];
    if ctx.request.actors[active].character_id != "character:1039"
        || actor.hp == 0
        || ctx.state.actors[owner].hp == 0
        || !super::etd::has_passive(&ctx.request.actors[owner])
        || ctx
            .state
            .targets
            .first()
            .is_none_or(|target| target.hp_current == 0)
    {
        return Ok(None);
    }
    let max = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        actor.hp_max_base,
        &actor.hp_max_static_bonus,
        active,
        actor.hp_max,
    )?;
    Ok((u128::from(actor.hp) * 5 <= u128::from(max)).then_some(active))
}

pub(crate) fn request_check(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    trigger: GameCausalTrigger,
) -> Result<(), String> {
    if recipient(ctx, owner)?.is_none()
        || ctx.state.actors[owner]
            .ability_cooldowns
            .get(ABILITY)
            .is_some_and(|&deadline| deadline > ctx.state.current_time_us)
    {
        return Ok(());
    }
    schedule(ctx, owner, ctx.state.current_time_us, trigger)
}

pub(crate) fn changed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    changes: &[crate::simulation::vitals::HpChange],
    trigger: GameCausalTrigger,
) -> Result<(), String> {
    if changes.iter().any(|change| {
        change.actor_index == ctx.state.active_actor_index && change.before >= change.after
    }) {
        request_check(ctx, owner, trigger)?;
    }
    Ok(())
}

pub(crate) fn check(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    trigger: GameCausalTrigger,
) -> Result<(), String> {
    let Some(active) = recipient(ctx, owner)? else {
        return Ok(());
    };
    let now = ctx.state.current_time_us;
    if ctx.state.actors[owner]
        .ability_cooldowns
        .get(ABILITY)
        .is_some_and(|&deadline| deadline > now)
    {
        return Ok(());
    }
    if ctx
        .request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        != Some(HYPOTHESIS_ID.description())
        || !super::words::accepted(ctx.request)
    {
        return Err("UNSUPPORTED_BAICANG_FADIA_ASSIST: live low-HP Fadia requires baicang.fadia_assist_v1 and baicang.power_words_v1".into());
    }
    let deadline = now
        .checked_add(30_000_000)
        .ok_or("Baicang Fadia cooldown overflow")?;
    let prepared = packet::prepare(ctx, owner)?;
    ctx.state.actors[owner]
        .packet_group_ordinal
        .checked_add(1)
        .ok_or("native packet group ordinal overflow")?;
    let work = ctx
        .state
        .targets
        .iter()
        .filter(|target| target.hp_current > 0)
        .count()
        + usize::from(deadline <= ctx.request.absolute_limit_us);
    ctx.sequence
        .checked_add(work as u64)
        .ok_or("Baicang Fadia event sequence overflow")?;
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: active,
            ability_id: ABILITY.into(),
            time_us: now,
            trigger,
            recipient_healing: None,
        },
    )?;
    let (_, Some(index)) = ctx.state.origins.identify(ctx.request, key)? else {
        return Err("Baicang Fadia autonomous identity missing".into());
    };
    ctx.state.actors[owner]
        .ability_cooldowns
        .insert(ABILITY.into(), deadline);
    ctx.state.actors[owner]
        .character
        .baicang_mut()?
        .power_words_mode = true;
    packet::enqueue(ctx, owner, key, prepared)?;
    if deadline <= ctx.request.absolute_limit_us {
        schedule(
            ctx,
            owner,
            deadline,
            GameCausalTrigger::OwnerDeadline {
                autonomous_event_index: index,
            },
        )?;
    }
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ProcCallback".into(),
        event_kind: "baicang_fadia_assist".into(),
        active_actor: active,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(active),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            event_index: Some(index),
            cooldown_until_us: Some(deadline),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn connected(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    key: usize,
    _group: Option<&str>,
    effect: &str,
) -> Result<(), String> {
    if effect != PACKET {
        return Ok(());
    }
    let (_, Some(index)) = ctx.state.origins.identify(ctx.request, key)? else {
        return Err("Baicang Fadia hit has no autonomous event".into());
    };
    let event = ctx.state.origins.get(index)?;
    if event.actor_index != owner || event.ability_id != ABILITY {
        return Err("Baicang Fadia source does not match its hit".into());
    }
    if event.recipient_healing.is_some() || ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let recipient = event.active_actor_index;
    let base = ctx
        .live_atk(owner)?
        .mul(crate::simulation::ExactFraction::new(27, 10)?)?;
    let heal = ctx.settle_healing(owner, recipient, base)?;
    ctx.state.origins.healing(
        index,
        crate::simulation::GameAutonomousHealing {
            recipient_actor_index: recipient,
            base,
            gross: heal.gross,
            applied: heal.applied,
            excess: heal.excess,
            hp_before: heal.before_hp,
            hp_after: heal.after_hp,
        },
    )?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "VitalSettlement".into(),
        event_kind: "baicang_fadia_heal".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(recipient),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            event_index: Some(index),
            gross: Some(heal.gross),
            applied: Some(heal.applied),
            overflow: Some(heal.excess),
            hp_before: Some(heal.before_hp),
            hp_after: Some(heal.after_hp),
            ..Default::default()
        }),
    });
    Ok(())
}

fn schedule(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    time_us: u64,
    trigger: GameCausalTrigger,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: ntesim_core::execution::CausalPriority::ProcCallback,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::BaicangFadiaCheck {
            actor_index,
            trigger,
        }),
    });
    Ok(())
}
