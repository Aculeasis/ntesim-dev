//! Current A3/A5 owner effects; explicit fixtures never create enemy behavior.
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::{ExactFraction as F, GameEventPayload, GameEventReceipt};
use ntesim_core::buffs::dispatch::*;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ZankouCombatPassivesV1;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Heal { owner: usize },
}
fn push(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    seq: &mut u64,
    owner: usize,
    at: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(seq)?;
    queue.push(ScheduledGameEvent {
        time_us: at,
        priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
        seq: *seq,
        data: GameEventData::Content(crate::callbacks::ContentEvent::ZankouCombat(Event::Heal {
            owner,
        })),
    });
    Ok(())
}
fn delusion(actor: &crate::simulation::state::ActorRuntime, now: u64) -> bool {
    matches!(&actor.character,crate::characters::RuntimeState::Zankou(s) if actor.hp>0 && (s.a1 || s.delusion_until_us>now))
}
pub(crate) fn arm_heal(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx.config.awakening.has_effect(3)
        || !delusion(ctx.actor, ctx.time_us)
        || ctx.actor.character.zankou_mut()?.healing_next_us.is_some()
    {
        return Ok(());
    }
    ctx.record_hypothesis(MODEL)?;
    let at = ctx
        .time_us
        .checked_add(1_000_000)
        .ok_or("Zankou heal overflow")?;
    if at > ctx.request.absolute_limit_us {
        return Ok(());
    }
    ctx.actor.character.zankou_mut()?.healing_next_us = Some(at);
    push(ctx.queue, ctx.sequence, ctx.actor_index, at)
}
pub(crate) fn initialize(ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    for (owner, config) in ctx.request.actors.iter().enumerate() {
        if config.character_id != "character:1036" || ctx.state.actors[owner].hp == 0 {
            continue;
        }
        if config.awakening.has_effect(5) {
            crate::runtime_api::record_hypothesis_id(
                ctx.request,
                &mut ctx.state.execution_model_assumptions,
                MODEL,
            )?;
            ctx.state
                .buffs
                .apply_buff(
                    BuffDefinition {
                        id: "zankou.a5_break".into(),
                        source: "character:1036".into(),
                        recipient: BuffRecipient::TeamAll,
                        source_lifetime: BuffSourceLifetime::WhileAlive,
                        stacking_scope: BuffStackingScope::PerSource,
                        duration_us: None,
                        refresh_policy: RefreshPolicy::Reset,
                        max_stacks: 1,
                        decay_policy: DecayPolicy::AllAtOnce,
                        semantic_tags: Default::default(),
                        stat_modifiers: [(
                            "break_damage_bonus".into(),
                            crate::formula::Ratio {
                                numerator: 3,
                                denominator: 1,
                            },
                        )]
                        .into(),
                    },
                    owner,
                    0,
                )
                .map_err(|e| format!("Zankou A5 modifier: {e:?}"))?;
        }
        if config.awakening.has_effect(3)
            && delusion(&ctx.state.actors[owner], 0)
            && ctx.request.absolute_limit_us >= 1_000_000
        {
            crate::runtime_api::record_hypothesis_id(
                ctx.request,
                &mut ctx.state.execution_model_assumptions,
                MODEL,
            )?;
            ctx.state.actors[owner]
                .character
                .zankou_mut()?
                .healing_next_us = Some(1_000_000);
            push(ctx.queue, ctx.sequence, owner, 1_000_000)?;
        }
    }
    Ok(())
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    let Event::Heal { owner } = event;
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    if actor.character.zankou_mut()?.healing_next_us != Some(now) {
        return Ok(());
    }
    actor.character.zankou_mut()?.healing_next_us = None;
    if !delusion(actor, now) {
        return Ok(());
    }
    let max = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        actor.hp_max_base,
        &actor.hp_max_static_bonus,
        owner,
        actor.hp_max,
    )?;
    let heal = ctx.settle_healing(owner, owner, F::new(max.into(), 25)?)?;
    ctx.trace.push(GameEventReceipt {
        time_us: now,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "zankou_a3_heal".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            gross: Some(heal.gross),
            applied: Some(heal.applied),
            ..Default::default()
        }),
    });
    let at = now.checked_add(1_000_000).ok_or("Zankou heal overflow")?;
    if at > ctx.request.absolute_limit_us {
        return Ok(());
    }
    ctx.state.actors[owner]
        .character
        .zankou_mut()?
        .healing_next_us = Some(at);
    push(ctx.queue, ctx.sequence, owner, at)
}
pub(crate) fn armor_pressure(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    pressure: crate::formula::Ratio,
) -> Result<bool, String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    if actor.hp == 0 {
        return Ok(false);
    }
    let state = actor.character.zankou_mut()?;
    let coefficient = if ctx.request.actors[owner].awakening.has_effect(3) && state.is_in_hunt(now)
    {
        F::new(1, 2)?
    } else {
        F::new(1, 1)?
    };
    let effective = F::from_ratio(pressure)?.mul(coefficient)?;
    let total = state
        .armor_pressure_halves
        .unwrap_or(F::new(0, 1)?)
        .add(effective)?;
    state.armor_pressure_halves = Some(total);
    let interrupted = total
        .try_cmp(F::new(1, 1)?)
        .map_err(|e| e.to_string())?
        .is_ge();
    ctx.trace.push(GameEventReceipt {
        time_us: now,
        phase: "SettlementAndDeath".into(),
        event_kind: "zankou_armor_pressure".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            exact_delta: Some(effective),
            exact_after: Some(total),
            active: Some(interrupted),
            ..Default::default()
        }),
    });
    Ok(interrupted)
}
pub(crate) fn impulse(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    let actor = &ctx.state.actors[owner];
    if actor.hp == 0
        || owner != ctx.state.active_actor_index
        || !ctx.request.actors[owner].awakening.has_effect(5)
    {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let story = ctx
        .request
        .model_assumptions
        .contains_key(crate::HypothesisId::ZankouStoryModeV1.as_str());
    if story {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            crate::HypothesisId::ZankouStoryModeV1,
        )?;
    }
    for target in 0..ctx.state.targets.len() {
        if ctx.state.targets[target].hp_current == 0
            || (story && ctx.request.targets[target].is_boss)
            || !ctx.state.actors[owner]
                .character
                .zankou_mut()?
                .a5_targets
                .insert(target)
        {
            continue;
        }
        crate::simulation::break_state::schedule_unbalance(
            ctx.state,
            ctx.queue,
            ctx.sequence,
            command,
            owner,
            target,
            Some(F::new(999_999, 1)?.into()),
            true,
            false,
        )?;
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "BuffTickOrExpiry".into(),
            event_kind: "zankou_a5_impulse".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(owner),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                command_index: Some(command),
                target_index: Some(target),
                ..Default::default()
            }),
        });
    }
    Ok(())
}
