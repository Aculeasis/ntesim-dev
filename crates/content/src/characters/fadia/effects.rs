//! Actual combat HP loss drives threshold fills and independently budgeted marks.
use crate::runtime_api::{ActorHitContext, CallbackContext, DamageContext, HitModifiers};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::{
    ExactFraction as F, GameCausalTrigger, GameEventReceipt, GameScenarioRequest,
};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::FadiaFullKitV1;
const PACKET: &str = "GE_Player_Fadia_ZhouYin_Damage";
const ABILITY: &str = "GA_Fadia_Transfer";
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
#[derive(Clone, Debug)]
pub(crate) struct Mark {
    generation: u64,
    origin: usize,
    budget: F,
    spent: F,
    reserved: F,
}
#[derive(Clone, Debug)]
pub(crate) struct Transfer {
    generation: u64,
    target: usize,
    amount: F,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    CombatExit {
        owner: usize,
    },
    Expire {
        owner: usize,
        target: usize,
        generation: u64,
    },
    Heal {
        owner: usize,
        command: usize,
        enhanced: bool,
    },
}
fn minimum(a: F, b: F) -> Result<F, String> {
    Ok(if a.try_cmp(b).map_err(String::from)?.is_lt() {
        a
    } else {
        b
    })
}
fn remainder(mark: &Mark) -> Result<F, String> {
    mark.budget
        .sub(minimum(mark.spent.add(mark.reserved)?, mark.budget)?)
}
pub(crate) fn threshold(level: u32) -> u64 {
    // DT_GlobalCommonData.FadiaQLimitHp: constant interpolation and endpoints.
    const VALUES: [u64; 80] = [
        705, 761, 818, 874, 930, 987, 1043, 1100, 1156, 1212, 1269, 1325, 1382, 1438, 1494, 1551,
        1607, 1664, 1720, 2340, 2397, 2453, 2510, 2566, 2622, 2679, 2735, 2792, 2848, 3468, 3525,
        3581, 3638, 3694, 3750, 3807, 3863, 3920, 3976, 4596, 4653, 4709, 4766, 4822, 4878, 4935,
        4991, 5048, 5104, 5724, 5781, 5837, 5894, 5950, 6006, 6063, 6119, 6176, 6232, 6852, 6909,
        6965, 7022, 7078, 7134, 7191, 7247, 7304, 7360, 7980, 8037, 8093, 8150, 8206, 8262, 8319,
        8375, 8432, 8488, 8544,
    ];
    VALUES[level.clamp(1, 80) as usize - 1]
}
pub(crate) fn changed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    changes: &[crate::simulation::vitals::HpChange],
    trigger: GameCausalTrigger,
) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let Some(change) = changes.iter().find(|c| c.actor_index == owner) else {
        return Ok(());
    };
    if change.after == 0 {
        ctx.state.actors[owner].character.fadia_mut()?.marks.clear();
        return Ok(());
    }
    for loss in &change.combat_losses {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            MODEL,
        )?;
        let limit = threshold(ctx.request.actors[owner].character_level);
        if loss.max_hp - loss.hp_before < limit && loss.max_hp - loss.hp_after >= limit {
            let actor = &mut ctx.state.actors[owner];
            let before = crate::simulation::energy::EnergySnapshot::capture(actor);
            let fill = 120u64.saturating_sub(actor.energy.current());
            actor.restore_energy(fill)?;
            let energy = before.receipt(
                actor,
                crate::simulation::GameEnergySource::FadiaThreshold,
                owner,
                owner,
                None,
            )?;
            ctx.trace.push(GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "VitalSettlement".into(),
                event_kind: "fadia_threshold_fill".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: Some(owner),
                to_actor: Some(owner),
                energy_change: Some(energy),
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    threshold: Some(limit),
                    amount: Some(fill),
                    ..Default::default()
                }),
            });
        }
        let ratio = if ctx.request.actors[owner].awakening.has_effect(2) {
            6
        } else {
            3
        };
        let amount = F::new(u128::from(loss.hp_before - loss.hp_after) * ratio, 1)?;
        let targets: Vec<_> = ctx.state.actors[owner]
            .character
            .fadia_mut()?
            .marks
            .keys()
            .copied()
            .collect();
        for target in targets {
            transfer(ctx, owner, target, amount, trigger)?;
        }
    }
    Ok(())
}
pub(crate) fn mark(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    let old: Vec<_> = ctx.state.actors[owner]
        .character
        .fadia_mut()?
        .marks
        .keys()
        .copied()
        .collect();
    for target in old {
        remove(ctx, owner, target, true)?;
    }
    let mut targets: Vec<_> = ctx
        .state
        .targets
        .iter()
        .enumerate()
        .filter(|(_, t)| t.hp_current > 0)
        .map(|(i, t)| (i, t.hp_max))
        .collect();
    targets.sort_by_key(|(i, hp)| (std::cmp::Reverse(*hp), *i));
    targets.truncate(if ctx.request.actors[owner].awakening.has_effect(6) {
        10
    } else {
        1
    });
    let actor = &ctx.state.actors[owner];
    let max = crate::simulation::state::effective_hp_max(
        &ctx.state.buffs,
        actor.hp_max_base,
        &actor.hp_max_static_bonus,
        owner,
        actor.hp_max,
    )?;
    let budget =
        F::new(max.into(), 1)?.mul(if ctx.request.actors[owner].awakening.has_effect(2) {
            F::new(1, 4)?
        } else {
            F::new(2, 25)?
        })?;
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: "GA_Fadia_DestructiveExperience".into(),
            time_us: ctx.state.current_time_us,
            trigger: GameCausalTrigger::Action {
                command_index: command,
            },
            recipient_healing: None,
        },
    )?;
    let (_, Some(origin)) = ctx.state.origins.identify(ctx.request, key)? else {
        return Err("Fadia mark origin missing".into());
    };
    let until = ctx
        .state
        .current_time_us
        .checked_add(15000000)
        .ok_or("Fadia mark expiry overflow")?;
    for (target, _) in targets {
        let state = ctx.state.actors[owner].character.fadia_mut()?;
        state.mark_generation = state
            .mark_generation
            .checked_add(1)
            .ok_or("Fadia mark generation overflow")?;
        let generation = state.mark_generation;
        state.marks.insert(
            target,
            Mark {
                generation,
                origin,
                budget,
                spent: F::new(0, 1)?,
                reserved: F::new(0, 1)?,
            },
        );
        if until <= ctx.request.absolute_limit_us {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: until,
                priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: GameEventData::Content(crate::callbacks::ContentEvent::FadiaEffects(
                    Event::Expire {
                        owner,
                        target,
                        generation,
                    },
                )),
            });
        }
        ctx.trace.push(GameEventReceipt {
            time_us: ctx.state.current_time_us,
            phase: "ProcCallback".into(),
            event_kind: "fadia_mark_applied".into(),
            active_actor: ctx.state.active_actor_index,
            hit_index: None,
            from_actor: Some(owner),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                target_index: Some(target),
                generation: Some(generation),
                hp_max: Some(max),
                exact_capacity: Some(budget),
                expires_at_us: Some(until),
                ..Default::default()
            }),
        });
    }
    Ok(())
}
fn transfer(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    request: F,
    trigger: GameCausalTrigger,
) -> Result<(), String> {
    if ctx.state.targets[target].hp_current == 0 {
        ctx.state.actors[owner]
            .character
            .fadia_mut()?
            .marks
            .remove(&target);
        return Ok(());
    }
    let Some(mark) = ctx.state.actors[owner]
        .character
        .fadia_mut()?
        .marks
        .get(&target)
        .cloned()
    else {
        return Ok(());
    };
    let amount = minimum(
        minimum(request, remainder(&mark)?)?,
        F::new(ctx.state.targets[target].hp_current.into(), 1)?,
    )?;
    if amount.round_u64()? == 0 {
        return Ok(());
    }
    let config = &ctx.request.actors[owner];
    let mut metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        "GA_Fadia_Skill",
        PACKET,
        None,
        &ctx.request.model_assumptions,
    )?;
    metadata.action.ability_id = ABILITY.into();
    metadata.action.input_role = Some(crate::simulation::metadata::InputActionRole::Other);
    metadata.binding.kind = crate::simulation::metadata::ActionPacketBindingKind::OwnerChild;
    metadata.binding.initiating_ability_id = ABILITY.into();
    let group = crate::runtime_api::next_packet_group(owner, &mut ctx.state.actors[owner])?;
    metadata.binding.energy_group_id = Some(group.clone());
    let key = ctx.state.origins.register(
        ctx.request,
        crate::simulation::GameAutonomousEventReceipt {
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            ability_id: ABILITY.into(),
            time_us: ctx.state.current_time_us,
            trigger,
            recipient_healing: None,
        },
    )?;
    let state = ctx.state.actors[owner].character.fadia_mut()?;
    state
        .marks
        .get_mut(&target)
        .ok_or("Fadia mark disappeared")?
        .reserved = mark.reserved.add(amount)?;
    state.transfers.insert(
        group,
        Transfer {
            generation: mark.generation,
            target,
            amount,
        },
    );
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.state.current_time_us,
        priority: ntesim_core::execution::CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: key,
            actor_index: owner,
            target_index: target,
            ability_id: ABILITY.into(),
            damage_effect_id: PACKET.into(),
            atk_rate: crate::formula::Ratio::zero(),
            scaling_stat: crate::simulation::NativeScalingStat::Hp,
            metadata: Box::new(metadata),
        },
    });
    Ok(())
}
fn remove(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    target: usize,
    purge: bool,
) -> Result<(), String> {
    let Some(mark) = ctx.state.actors[owner]
        .character
        .fadia_mut()?
        .marks
        .get(&target)
        .cloned()
    else {
        return Ok(());
    };
    if purge && ctx.request.actors[owner].awakening.has_effect(2) && ctx.state.actors[owner].hp > 0
    {
        transfer(
            ctx,
            owner,
            target,
            remainder(&mark)?,
            GameCausalTrigger::OwnerDeadline {
                autonomous_event_index: mark.origin,
            },
        )?;
    }
    ctx.state.actors[owner]
        .character
        .fadia_mut()?
        .marks
        .remove(&target);
    Ok(())
}
pub(crate) fn clear_marks(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let targets: Vec<_> = ctx.state.actors[owner]
        .character
        .fadia_mut()?
        .marks
        .keys()
        .copied()
        .collect();
    for target in targets {
        remove(ctx, owner, target, true)?;
    }
    Ok(())
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::CombatExit { owner } => super::stats::leave(ctx, owner)?,
        Event::Heal {
            owner,
            command,
            enhanced,
        } => {
            if ctx.state.actors[owner].hp == 0 {
                return Ok(());
            }
            let actor = &ctx.state.actors[owner];
            let max = crate::simulation::state::effective_hp_max(
                &ctx.state.buffs,
                actor.hp_max_base,
                &actor.hp_max_static_bonus,
                owner,
                actor.hp_max,
            )?;
            let numerator = (if enhanced { 6 } else { 3 })
                * (if ctx.request.actors[owner].awakening.has_effect(4) {
                    2
                } else {
                    1
                });
            let base = F::new(u128::from(max) * numerator, 20)?;
            let heal = ctx.settle_healing(owner, owner, base)?;
            ctx.state.origins.register(
                ctx.request,
                crate::simulation::GameAutonomousEventReceipt {
                    actor_index: owner,
                    active_actor_index: ctx.state.active_actor_index,
                    ability_id: "GA_Fadia_BurstHealing".into(),
                    time_us: ctx.state.current_time_us,
                    trigger: GameCausalTrigger::Action {
                        command_index: command,
                    },
                    recipient_healing: Some(crate::simulation::GameAutonomousHealing {
                        recipient_actor_index: owner,
                        base,
                        gross: heal.gross,
                        applied: heal.applied,
                        excess: heal.excess,
                        hp_before: heal.before_hp,
                        hp_after: heal.after_hp,
                    }),
                },
            )?;
            ctx.trace.push(GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "VitalSettlement".into(),
                event_kind: "fadia_burst_heal".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: Some(owner),
                to_actor: Some(owner),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    enhanced: Some(enhanced),
                    gross: Some(heal.gross),
                    applied: Some(heal.applied),
                    overflow: Some(heal.excess),
                    hp_before: Some(heal.before_hp),
                    hp_after: Some(heal.after_hp),
                    ..Default::default()
                }),
            });
            crate::weapons::wrong_gate::on_healing(ctx, owner, owner, heal.gross)?;
        }
        Event::Expire {
            owner,
            target,
            generation,
        } => {
            if ctx.state.actors[owner]
                .character
                .fadia_mut()?
                .marks
                .get(&target)
                .is_some_and(|m| m.generation == generation)
            {
                remove(ctx, owner, target, true)?;
            }
        }
    }
    Ok(())
}
pub(crate) fn damage(
    ctx: &DamageContext<'_>,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if ctx
        .damage_effect_id
        .starts_with("GE_Player_Fadia_UltraSkill")
        && super::clock::flags(ctx.actor, ctx.command_index).0
    {
        modifiers = modifiers.add_damage(F::new(3, 10)?)?;
    }
    if ctx.damage_effect_id != PACKET {
        return Ok(modifiers);
    }
    let crate::characters::RuntimeState::Fadia(state) = &ctx.actor.character else {
        return Err("Fadia transfer owner missing".into());
    };
    let transfer = ctx
        .packet_group_id
        .and_then(|g| state.transfers.get(g))
        .filter(|t| t.target == ctx.target_index)
        .ok_or("Fadia transfer context missing")?;
    modifiers.resolved_damage = Some(minimum(
        transfer.amount,
        F::new(ctx.target.hp_current.into(), 1)?,
    )?);
    Ok(modifiers)
}
pub(crate) fn connected(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.damage_effect_id != PACKET {
        return Ok(());
    }
    let Some(transfer) = ctx
        .packet_group_id
        .and_then(|g| ctx.actor.character.fadia_mut().ok()?.transfers.remove(g))
    else {
        return Err("Fadia delivered transfer context missing".into());
    };
    let state = ctx.actor.character.fadia_mut()?;
    if let Some(mark) = state
        .marks
        .get_mut(&transfer.target)
        .filter(|m| m.generation == transfer.generation)
    {
        mark.reserved = mark.reserved.sub(transfer.amount)?;
        mark.spent = minimum(
            mark.spent.add(F::new(ctx.applied_hp_loss.into(), 1)?)?,
            mark.budget,
        )?;
        // Terminal fractions that cannot remove one integer HP never emit a ghost hit.
        if remainder(mark)?.round_u64()? == 0 {
            state.marks.remove(&transfer.target);
        }
    }
    Ok(())
}
