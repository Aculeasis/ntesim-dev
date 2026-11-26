//! Live heal selectors, key capture, field lifetime and HP-loss settlement.
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::state::GameSimulatorState;
use crate::simulation::{ExactFraction, GameEventReceipt, GameScenarioRequest};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub(crate) generation: u64,
    pub(crate) coordinate: usize,
    pub(crate) until: u64,
}

pub(super) fn unlocked(
    config: &crate::simulation::GameActorConfig,
    breakthrough: u64,
) -> Result<bool, String> {
    Ok(ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?["breakthrough"]
        .as_u64()
        .unwrap_or(0)
        >= breakthrough)
}
pub(crate) fn observe_support(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    actor: usize,
    metadata: &crate::simulation::metadata::ActionMetadata,
    assumptions: &mut BTreeMap<String, String>,
    trace: &mut Vec<GameEventReceipt>,
) -> Result<(), String> {
    if state.actors[actor].hp == 0
        || metadata.input_role != Some(crate::simulation::metadata::InputActionRole::Qte)
        || metadata.native_ability_index != Some(4)
    {
        return Ok(());
    }
    for owner in 0..request.actors.len() {
        let config = &request.actors[owner];
        if config.character_id == "character:1021"
            && state.actors[owner].hp > 0
            && !state.actors[owner].character.edgar_mut()?.out_of_combat
            && unlocked(config, 4)?
            && (owner == actor || config.awakening.has_effect(1))
        {
            crate::runtime_api::record_hypothesis_id(request, assumptions, super::clock::MODEL)?;
            grant_key(state, owner, trace, "support")?;
        }
    }
    Ok(())
}

fn grant_key(
    state: &mut GameSimulatorState,
    owner: usize,
    trace: &mut Vec<GameEventReceipt>,
    reason: &str,
) -> Result<(), String> {
    let keys = &mut state.actors[owner].character.edgar_mut()?.keys_of_truth;
    let before = *keys;
    *keys = before.saturating_add(1).min(3);
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "edgar_key_of_truth_granted".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            reason: Some(reason.into()),
            stacks_before: Some(before),
            stacks: Some(*keys),
            max_stacks: Some(3),
            ..Default::default()
        }),
    });
    Ok(())
}

fn max_hp(ctx: &CallbackContext<'_>, actor: usize) -> Result<u64, String> {
    let runtime = &ctx.state.actors[actor];
    crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        runtime.hp_max_base,
        &runtime.hp_max_static_bonus,
        actor,
        runtime.hp_max,
    )
}
fn heal(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    recipient: usize,
    coordinate: usize,
    skill: bool,
) -> Result<(), String> {
    let (flat, ratio) = if skill {
        super::parameters::skill_heal(coordinate)?
    } else {
        let params = super::parameters::healing(
            coordinate
                .try_into()
                .map_err(|_| "Edgar healing coordinate overflow")?,
        )?;
        (params.flat, params.max_hp_ratio)
    };
    let source_max_hp = max_hp(ctx, owner)?;
    let mut base = flat.add(ExactFraction::new(source_max_hp.into(), 1)?.mul(ratio)?)?;
    if skill
        && ctx.request.actors[owner].awakening.has_effect(3)
        && u128::from(ctx.state.actors[recipient].hp) * 10
            <= u128::from(max_hp(ctx, recipient)?) * 3
    {
        base = base.mul(ExactFraction::new(23, 20)?)?;
    }
    let settlement = ctx.settle_healing(owner, recipient, base)?;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: if skill {
            "edgar_skill_heal"
        } else {
            "edgar_field_heal"
        }
        .into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(recipient),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            hp_max: Some(source_max_hp),
            gross: Some(settlement.gross),
            applied: Some(settlement.applied),
            overflow: Some(settlement.excess),
            hp_before: Some(settlement.before_hp),
            hp_after: Some(settlement.after_hp),
            ..Default::default()
        }),
    });
    crate::weapons::wrong_gate::on_healing(ctx, owner, recipient, settlement.gross)
}
pub(crate) fn skill_heal(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    coordinate: usize,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    if unlocked(&ctx.request.actors[owner], 4)?
        && ctx.state.current_time_us >= ctx.state.actors[owner].character.edgar_mut()?.key_ready_us
    {
        let until = ctx
            .state
            .current_time_us
            .checked_add(6_000_000)
            .ok_or("Edgar key gate overflow")?;
        ctx.state.actors[owner].character.edgar_mut()?.key_ready_us = until;
        grant_key(ctx.state, owner, ctx.trace, "skill_heal_request")?;
    }
    let mut candidates = Vec::new();
    for (index, actor) in ctx.state.actors.iter().enumerate() {
        if actor.hp > 0 && actor.hp < max_hp(ctx, index)? {
            candidates.push((actor.hp, index));
        }
    }
    if let Some((_, recipient)) = candidates.into_iter().min() {
        heal(ctx, owner, recipient, coordinate, true)?;
    }
    Ok(())
}

pub(crate) fn spawn(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    coordinate: usize,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let keys = if unlocked(&ctx.request.actors[owner], 4)? {
        ctx.state.actors[owner].character.edgar_mut()?.keys_of_truth
    } else {
        0
    };
    let duration = 10_000_000 + u64::from(keys) * 1_000_000;
    let until = now
        .checked_add(duration)
        .ok_or("Edgar field deadline overflow")?;
    let state = ctx.state.actors[owner].character.edgar_mut()?;
    state.healing_generation = state
        .healing_generation
        .checked_add(1)
        .ok_or("Edgar field generation overflow")?;
    let generation = state.healing_generation;
    state.keys_of_truth = 0;
    state.finnegan_wake_until_us = until;
    state.field = Some(Field {
        generation,
        coordinate,
        until,
    });
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push_on(
        ntesim_core::kernel::timing::ClockDomain::ObjectLifetime,
        crate::simulation::events::ScheduledGameEvent {
            time_us: until,
            priority: ntesim_core::execution::CausalPriority::SettlementAndDeath,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::Edgar(super::clock::Event::FieldExpiry {
                    owner,
                    generation,
                }),
            ),
        },
    );
    if ctx.request.actors[owner].awakening.has_effect(6) {
        use ntesim_core::buffs::dispatch::{BuffRecipient, BuffSourceLifetime, BuffStackingScope};
        let declaration = crate::simulation::state::TimedBuffDeclaration::reset(
            "edgar.a6_wake_team_def",
            "character:1021",
            BuffRecipient::TeamAll,
            duration,
            BuffStackingScope::PerSource,
            BTreeMap::from([("def_percent".into(), ExactFraction::new(3, 20)?)]),
        );
        crate::simulation::state::apply_timed_buff_with_lifecycle(
            &mut ctx.state.buffs,
            declaration,
            owner,
            ctx.state.mode_clock.relative_us(),
            BuffSourceLifetime::WhileAlive,
        )?;
    }
    ctx.trace.push(GameEventReceipt {
        time_us: now,
        phase: "ProcCallback".into(),
        event_kind: "edgar_field_spawned".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            consumed: Some(keys),
            generation: Some(generation),
            expires_at_us: Some(until),
            ..Default::default()
        }),
    });
    super::clock::enqueue(
        ctx,
        now.checked_add(1_000_000)
            .ok_or("Edgar field tick overflow")?,
        super::clock::Event::FieldTick { owner, generation },
    )
}
pub(crate) fn tick(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let Some(field) = ctx.state.actors[owner]
        .character
        .edgar_mut()?
        .field
        .clone()
        .filter(|f| f.generation == generation)
    else {
        return Ok(());
    };
    let now = ctx.state.current_time_us;
    if ctx.state.actors[owner].hp == 0 || now > field.until {
        ctx.state.actors[owner].character.edgar_mut()?.field = None;
        return Ok(());
    }
    let recipient = ctx.state.active_actor_index;
    if ctx.state.actors[recipient].hp > 0 {
        heal(ctx, owner, recipient, field.coordinate, false)?;
    }
    if now == field.until {
        ctx.state.actors[owner].character.edgar_mut()?.field = None;
    } else {
        super::clock::enqueue(
            ctx,
            now.checked_add(1_000_000)
                .ok_or("Edgar field tick overflow")?,
            super::clock::Event::FieldTick { owner, generation },
        )?;
    }
    Ok(())
}

pub(crate) fn expire(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.edgar_mut()?;
    if state
        .field
        .as_ref()
        .is_some_and(|field| field.generation == generation)
    {
        state.field = None;
        ctx.queue.retain(|event| !matches!(&event.data, crate::simulation::events::GameEventData::Content(
            crate::callbacks::ContentEvent::Edgar(super::clock::Event::FieldTick { owner: source, generation: live }))
            if *source == owner && *live == generation));
        ctx.state
            .buffs
            .remove_owned_buff("edgar.a6_wake_team_def", "character:1021", owner);
    }
    Ok(())
}

pub(crate) fn on_vitals(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    changes: &[crate::simulation::vitals::HpChange],
) -> Result<(), String> {
    if !super::clock::accepted(ctx.request) {
        return Ok(());
    }
    if ctx.state.actors[owner].hp == 0 {
        let state = ctx.state.actors[owner].character.edgar_mut()?;
        state.keys_of_truth = 0;
        if let Some(pool) = &mut state.pending_charge {
            pool.spend_up_to(ExactFraction::new(20, 1)?)?;
        }
        state.field = None;
        return super::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
        );
    }
    if !ctx.request.actors[owner].awakening.has_resonance_6() {
        return Ok(());
    }
    let pending = ctx.state.actors[owner]
        .character
        .edgar_mut()?
        .pending_charge
        .as_mut()
        .ok_or("Edgar R6 ledger not initialized")?;
    for change in changes {
        for loss in &change.combat_losses {
            pending.grant(ExactFraction::new(
                u128::from(loss.hp_before - loss.hp_after) * 100,
                u128::from(loss.max_hp) * 3,
            )?)?;
        }
    }
    Ok(())
}
pub(super) fn burst_refund(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !ctx.config.awakening.has_resonance_6() {
        return Ok(());
    }
    let pending = ctx
        .actor
        .character
        .edgar_mut()?
        .pending_charge
        .as_mut()
        .ok_or("Edgar R6 ledger not initialized")?
        .spend_up_to(ExactFraction::new(20, 1)?)?;
    let raw: u64 = (pending.numerator / pending.denominator)
        .try_into()
        .map_err(|_| "Edgar pending charge overflow")?;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue
        .push(crate::simulation::events::ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::Edgar(super::clock::Event::Refund {
                    owner: ctx.actor_index,
                    raw,
                }),
            ),
        });
    Ok(())
}

pub(crate) fn charge(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<Option<u64>, String> {
    if !super::clock::accepted(request) {
        return Ok(None);
    }
    let recipient = state.active_actor_index;
    let now = state.current_time_us;
    let mut source = None;
    for owner in 0..request.actors.len() {
        if request.actors[owner].character_id != "character:1021" {
            continue;
        }
        let guarded = state.actors[owner]
            .character
            .edgar_mut()?
            .charge_guards
            .get(&recipient)
            .is_some_and(|until| *until > now);
        if guarded {
            return Ok(Some(0));
        }
        if state.actors[owner].hp > 0
            && !state.actors[owner].character.edgar_mut()?.out_of_combat
            && unlocked(&request.actors[owner], 2)?
        {
            source = Some(owner);
        }
    }
    let Some(owner) = source else {
        return Ok(None);
    };
    let until = now
        .checked_add(30_000_000)
        .ok_or("Edgar Charge guard overflow")?;
    crate::runtime_api::record_hypothesis_id(request, assumptions, super::clock::MODEL)?;
    state.actors[owner]
        .character
        .edgar_mut()?
        .charge_guards
        .insert(recipient, until);
    Ok(Some(120))
}

pub(crate) fn transition(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<crate::simulation::GameActorCombatTransitionReceipt, String> {
    let leaving = ctx.request.combat_transition_events[event].leaving;
    if ctx.state.actors[owner].hp == 0
        || ctx.state.actors[owner].character.edgar_mut()?.out_of_combat == leaving
    {
        return Err("Edgar combat transition requires a living actor and alternating state".into());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        super::clock::MODEL,
    )?;
    ctx.state.actors[owner].character.edgar_mut()?.out_of_combat = leaving;
    if leaving {
        super::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
        )?;
        let state = ctx.state.actors[owner].character.edgar_mut()?;
        state.field = None;
        state.charge_guards.clear();
        state.riposte_ready = false;
        state.normal_stage = 0;
        ctx.state
            .buffs
            .remove_owned_buff("edgar.a6_wake_team_def", "character:1021", owner);
        ctx.state.actors[owner].locked_until_us = ctx.state.current_time_us;
    }
    Ok(crate::simulation::GameActorCombatTransitionReceipt {
        event_index: event,
        actor_index: owner,
        time_us: ctx.state.current_time_us,
        leaving,
        malice_granted: ExactFraction::new(0, 1)?,
        leave_grant_due_us: None,
        resource_deposits: Default::default(),
    })
}
