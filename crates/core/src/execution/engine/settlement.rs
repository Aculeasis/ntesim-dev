use super::*;

pub(crate) fn execute_step_action(
    state: &mut SimState,
    time_us: u64,
    action: &StepAction,
    limits: &ExecutionLimits,
    sequence_counter: &mut u64,
    queue: &mut BinaryHeap<QueuedEvent>,
) -> Result<AuditEvent, ExecutionError> {
    match action {
        StepAction::InvokeAction {
            actor_index,
            action_name,
            cost_pool,
            cost_amount,
            recovery_us,
            cooldown_us,
        } => {
            if *actor_index >= state.actors.len() {
                return Err(ExecutionError::InvalidActorIndex(*actor_index));
            }
            if !state.actors[*actor_index].is_alive {
                return Err(ExecutionError::ActorDead(*actor_index));
            }
            if *actor_index != state.active_actor {
                return Err(ExecutionError::ActorNotActive {
                    actor_index: *actor_index,
                    active_actor: state.active_actor,
                });
            }
            if cost_pool.is_none() && *cost_amount > 0 {
                return Err(ExecutionError::MissingField(
                    "cost_pool for positive action cost".into(),
                ));
            }
            let timing = crate::action::TimingState {
                last_start_us: 0, // The event scheduler owns monotonic execution time.
                locked_until_us: state.actors[*actor_index].locked_until_us,
                ready_at_us: state.actors[*actor_index].cooldown_until_us,
            }
            .prepare(action_name, time_us, *recovery_us, *cooldown_us)
            .map_err(|error| match error {
                crate::action::ActionError::RecoveryLock { until } => ExecutionError::ActorLocked {
                    actor_index: *actor_index,
                    locked_until_us: until,
                    requested_us: time_us,
                },
                crate::action::ActionError::Cooldown { until, .. } => {
                    ExecutionError::ActorOnCooldown {
                        actor_index: *actor_index,
                        cooldown_until_us: until,
                        requested_us: time_us,
                    }
                }
                _ => ExecutionError::InvariantViolation {
                    invariant_name: "action_deadline".into(),
                    details: format!("{error:?}"),
                },
            })?;
            if let Some(pool_id) = cost_pool {
                let pool = state
                    .pools
                    .get_mut(pool_id)
                    .ok_or_else(|| ExecutionError::PoolNotFound(pool_id.clone()))?;
                pool.spend(*cost_amount)?;
            }

            state.actors[*actor_index].locked_until_us = timing.locked_until_us;
            state.actors[*actor_index].cooldown_until_us = timing.ready_at_us;

            Ok(AuditEvent::ActionInvoked {
                time_us,
                actor_index: *actor_index,
                action_name: action_name.clone(),
                cost_spent: *cost_amount,
                locked_until_us: state.actors[*actor_index].locked_until_us,
                cooldown_until_us: state.actors[*actor_index].cooldown_until_us,
            })
        }

        StepAction::GrantResource { pool_id, amount } => {
            let pool = state
                .pools
                .get_mut(pool_id)
                .ok_or_else(|| ExecutionError::PoolNotFound(pool_id.clone()))?;
            let (granted, overflow) = pool.grant(*amount)?;
            Ok(AuditEvent::ResourceGranted {
                time_us,
                pool_id: pool_id.clone(),
                amount_requested: *amount,
                amount_granted: granted,
                amount_overflow: overflow,
                new_balance: pool.current,
            })
        }

        StepAction::SpendResource { pool_id, amount } => {
            let pool = state
                .pools
                .get_mut(pool_id)
                .ok_or_else(|| ExecutionError::PoolNotFound(pool_id.clone()))?;
            pool.spend(*amount)?;
            Ok(AuditEvent::ResourceSpent {
                time_us,
                pool_id: pool_id.clone(),
                amount: *amount,
                new_balance: pool.current,
            })
        }

        StepAction::DeliverDamage {
            actor_index,
            target_id,
            base_damage,
            damage_kind,
        } => {
            let attacker_level = match actor_index {
                Some(idx) => {
                    if *idx >= state.actors.len() {
                        return Err(ExecutionError::InvalidActorIndex(*idx));
                    }
                    state.actors[*idx].level
                }
                None if *damage_kind == crate::combat::DamageKind::Elemental => {
                    return Err(ExecutionError::MissingField(
                        "actor_index for elemental damage mitigation".into(),
                    ));
                }
                None => 0, // unused by defense-bypassing damage kinds
            };
            let target = state
                .targets
                .iter_mut()
                .find(|t| &t.target_id == target_id)
                .ok_or_else(|| ExecutionError::TargetNotFound(target_id.clone()))?;

            if !target.is_alive {
                return Err(ExecutionError::TargetDead(target_id.clone()));
            }

            let effective = compute_damage_mitigation(
                *base_damage,
                *damage_kind,
                target.defense,
                attacker_level,
            )?;
            let (absorbed, hp_lost, overkill, died) = settle_target_damage(target, effective)?;

            Ok(AuditEvent::DamageResolved {
                time_us,
                target_id: target_id.clone(),
                raw_damage: *base_damage,
                effective_damage: effective,
                absorbed_by_shields: absorbed,
                hp_lost,
                overkill,
                target_died: died,
            })
        }

        StepAction::DeliverHeal {
            actor_index,
            amount_hp,
        } => {
            if *actor_index >= state.actors.len() {
                return Err(ExecutionError::InvalidActorIndex(*actor_index));
            }
            let a = &mut state.actors[*actor_index];
            if !a.is_alive {
                return Err(ExecutionError::ActorDead(*actor_index));
            }
            let error = |error| ExecutionError::InvariantViolation {
                invariant_name: "vital_settlement".into(),
                details: format!("{error:?}"),
            };
            let mut vitals = crate::state::VitalState::new(
                a.current_hp,
                a.max_hp,
                crate::state::ShieldOrder::OldestFirst,
            )
            .map_err(error)?;
            let grant = vitals.heal(*amount_hp).map_err(error)?;
            let effective = grant.applied;
            let overheal = grant.excess;
            let healed = a.total_healed.checked_add(effective).ok_or_else(|| {
                ExecutionError::InvariantViolation {
                    invariant_name: "healing_counter".into(),
                    details: "healing counter overflow".into(),
                }
            })?;
            let excess = a.total_overheal.checked_add(overheal).ok_or_else(|| {
                ExecutionError::InvariantViolation {
                    invariant_name: "healing_counter".into(),
                    details: "overheal counter overflow".into(),
                }
            })?;
            a.current_hp = vitals.hp();
            a.total_healed = healed;
            a.total_overheal = excess;

            Ok(AuditEvent::HealResolved {
                time_us,
                actor_index: *actor_index,
                requested_heal: *amount_hp,
                effective_heal: effective,
                overheal,
                new_hp: a.current_hp,
            })
        }

        StepAction::ApplyShield {
            actor_index,
            shield_id,
            source,
            amount_hp,
        } => {
            if *actor_index >= state.actors.len() {
                return Err(ExecutionError::InvalidActorIndex(*actor_index));
            }
            let a = &mut state.actors[*actor_index];
            if !a.is_alive {
                return Err(ExecutionError::ActorDead(*actor_index));
            }
            if a.active_shields.len() >= limits.max_active_shields {
                return Err(ExecutionError::LimitExceeded {
                    limit_name: "max_active_shields".into(),
                    current: a.active_shields.len() as u64,
                    max: limits.max_active_shields as u64,
                });
            }
            let mut vitals = crate::state::VitalState::new(
                a.current_hp,
                a.max_hp,
                crate::state::ShieldOrder::OldestFirst,
            )
            .map_err(|error| ExecutionError::InvariantViolation {
                invariant_name: "vital_settlement".into(),
                details: format!("{error:?}"),
            })?;
            for shield in &a.active_shields {
                vitals
                    .add_shield(
                        shield.id.clone(),
                        shield.source.clone(),
                        shield.remaining_hp,
                    )
                    .map_err(|error| ExecutionError::InvariantViolation {
                        invariant_name: "shield_identity".into(),
                        details: format!("{error:?}"),
                    })?;
            }
            vitals
                .add_shield(shield_id.clone(), source.clone(), *amount_hp)
                .map_err(|error| ExecutionError::InvariantViolation {
                    invariant_name: "shield_identity".into(),
                    details: format!("{error:?}"),
                })?;
            a.active_shields.push(ShieldState {
                id: shield_id.clone(),
                source: source.clone(),
                remaining_hp: *amount_hp,
                max_hp: *amount_hp,
            });

            Ok(AuditEvent::ShieldApplied {
                time_us,
                actor_index: *actor_index,
                shield_id: shield_id.clone(),
                source: source.clone(),
                amount_hp: *amount_hp,
            })
        }

        StepAction::SwitchActor { to_actor_index } => {
            if *to_actor_index >= state.actors.len() {
                return Err(ExecutionError::InvalidActorIndex(*to_actor_index));
            }
            if !state.actors[*to_actor_index].is_alive {
                return Err(ExecutionError::ActorDead(*to_actor_index));
            }
            let old_active = state.active_actor;
            state.active_actor = *to_actor_index;

            Ok(AuditEvent::ActorSwitched {
                time_us,
                from_actor: old_active,
                to_actor: *to_actor_index,
            })
        }

        StepAction::TriggerProc {
            proc_id,
            actor_index,
            depth,
            grant_pool,
            grant_amount,
        } => {
            if *actor_index >= state.actors.len() {
                return Err(ExecutionError::InvalidActorIndex(*actor_index));
            }
            if grant_pool.is_none() && *grant_amount > 0 {
                return Err(ExecutionError::MissingField(
                    "grant_pool for positive proc grant".into(),
                ));
            }
            if *depth > limits.max_proc_depth {
                return Err(ExecutionError::ProcDepthExceeded {
                    depth: *depth,
                    max_depth: limits.max_proc_depth,
                });
            }
            state.max_proc_depth_reached = state.max_proc_depth_reached.max(*depth);

            if let Some(pool_id) = grant_pool {
                *sequence_counter = sequence_counter.checked_add(1).ok_or_else(|| {
                    ExecutionError::InvariantViolation {
                        invariant_name: "sequence_counter".into(),
                        details: "sequence overflow".into(),
                    }
                })?;
                queue.push(QueuedEvent {
                    time_us,
                    priority: CausalPriority::ResourceCost,
                    sequence_id: *sequence_counter,
                    action: StepAction::GrantResource {
                        pool_id: pool_id.clone(),
                        amount: *grant_amount,
                    },
                    source_step: 0,
                });
            }

            Ok(AuditEvent::ProcTriggered {
                time_us,
                proc_id: proc_id.clone(),
                actor_index: *actor_index,
                depth: *depth,
            })
        }

        StepAction::ObserveState { label } => {
            let total_hp = state.actors.iter().try_fold(0u64, |total, a| {
                total
                    .checked_add(a.current_hp)
                    .ok_or_else(|| ExecutionError::InvariantViolation {
                        invariant_name: "observation".into(),
                        details: "total HP overflow".into(),
                    })
            })?;
            Ok(AuditEvent::StateObserved {
                time_us,
                label: label.clone(),
                active_actor: state.active_actor,
                total_hp,
            })
        }
    }
}
