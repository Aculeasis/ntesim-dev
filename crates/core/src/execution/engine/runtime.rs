use super::*;

pub fn run_execution_scenario(
    request: &ExecutionScenarioRequest,
) -> Result<ExecutionScenarioReport, ExecutionError> {
    if request.schema_version != 1 {
        return Err(ExecutionError::ExtraField(format!(
            "unsupported schema_version {}",
            request.schema_version
        )));
    }
    if request.team_size == 0 {
        return Err(ExecutionError::EmptyTeam);
    }
    if request.team_size > 4 {
        return Err(ExecutionError::TeamSizeExceeded(request.team_size));
    }
    if request.actors.len() != request.team_size {
        return Err(ExecutionError::InvalidActorIndex(request.actors.len()));
    }
    if request.active_actor >= request.team_size {
        return Err(ExecutionError::InvalidActiveActor(request.active_actor));
    }
    if request.horizon_us == 0 {
        return Err(ExecutionError::ZeroHorizon);
    }
    if request.content_id != "ntesim-current" {
        return Err(ExecutionError::InvalidPackage(request.content_id.clone()));
    }

    let limits = request.limits.unwrap_or_default();
    if request.horizon_us > limits.max_horizon_us {
        return Err(ExecutionError::LimitExceeded {
            limit_name: "max_horizon_us".into(),
            current: request.horizon_us,
            max: limits.max_horizon_us,
        });
    }

    let mut actors = Vec::with_capacity(request.actors.len());
    for (index, a) in request.actors.iter().enumerate() {
        if a.actor_index != index {
            return Err(ExecutionError::InvalidActorIndex(a.actor_index));
        }
        if a.level == 0 {
            return Err(ExecutionError::MissingField(format!(
                "actors/{index}/positive level"
            )));
        }
        if a.max_hp == 0 {
            return Err(ExecutionError::ZeroHealthCapacity(a.actor_index));
        }
        if a.current_hp > a.max_hp {
            return Err(ExecutionError::CurrentHealthExceedsMax {
                actor_index: a.actor_index,
                current: a.current_hp,
                max: a.max_hp,
            });
        }
        let mut shield_ids = std::collections::BTreeSet::new();
        for shield in &a.initial_shields {
            if !shield_ids.insert(&shield.id) {
                return Err(ExecutionError::InvariantViolation {
                    invariant_name: "shield_identity".into(),
                    details: format!("duplicate initial shield {}", shield.id),
                });
            }
        }
        let shields: Vec<ShieldState> = a
            .initial_shields
            .iter()
            .map(|s| ShieldState {
                id: s.id.clone(),
                source: s.source.clone(),
                remaining_hp: s.amount_hp,
                max_hp: s.amount_hp,
            })
            .collect();
        actors.push(ActorState {
            actor_index: a.actor_index,
            character_id: a.character_id.clone(),
            level: a.level,
            current_hp: a.current_hp,
            max_hp: a.max_hp,
            defense: a.defense,
            is_alive: a.current_hp > 0,
            locked_until_us: 0,
            cooldown_until_us: 0,
            total_damage_taken: 0,
            total_absorbed: 0,
            total_hp_lost: 0,
            total_healed: 0,
            total_overheal: 0,
            active_shields: shields,
        });
    }

    let mut targets = Vec::with_capacity(request.targets.len());
    let mut target_ids = std::collections::BTreeSet::new();
    for t in &request.targets {
        if !target_ids.insert(&t.target_id) {
            return Err(ExecutionError::ExtraField(format!(
                "duplicate target {}",
                t.target_id
            )));
        }
        if t.max_hp == 0 {
            return Err(ExecutionError::TargetNotFound(t.target_id.clone()));
        }
        let mut shield_ids = std::collections::BTreeSet::new();
        for shield in &t.initial_shields {
            if !shield_ids.insert(&shield.id) {
                return Err(ExecutionError::InvariantViolation {
                    invariant_name: "shield_identity".into(),
                    details: format!("duplicate initial shield {}", shield.id),
                });
            }
        }
        let shields: Vec<ShieldState> = t
            .initial_shields
            .iter()
            .map(|s| ShieldState {
                id: s.id.clone(),
                source: s.source.clone(),
                remaining_hp: s.amount_hp,
                max_hp: s.amount_hp,
            })
            .collect();
        targets.push(TargetState {
            target_id: t.target_id.clone(),
            level: t.level,
            current_hp: t.current_hp,
            max_hp: t.max_hp,
            defense: t.defense,
            is_alive: t.current_hp > 0,
            total_damage_taken: 0,
            total_absorbed: 0,
            total_hp_lost: 0,
            active_shields: shields,
        });
    }

    let mut pools = BTreeMap::new();
    for p in &request.resource_pools {
        if pools.contains_key(&p.pool_id) {
            return Err(ExecutionError::DuplicatePoolId(p.pool_id.clone()));
        }
        if p.initial_amount > p.capacity {
            return Err(ExecutionError::InvariantViolation {
                invariant_name: "resource_capacity".into(),
                details: format!("initial amount for {} exceeds capacity", p.pool_id),
            });
        }
        if p.capacity > limits.max_pool_capacity {
            return Err(ExecutionError::LimitExceeded {
                limit_name: "max_pool_capacity".into(),
                current: p.capacity,
                max: limits.max_pool_capacity,
            });
        }
        pools.insert(
            p.pool_id.clone(),
            ResourcePoolState {
                pool_id: p.pool_id.clone(),
                current: p.initial_amount,
                capacity: p.capacity,
                initial: p.initial_amount,
                total_granted: 0,
                total_spent: 0,
                total_overflow: 0,
            },
        );
    }

    let mut prev_time = 0u64;
    for s in &request.steps {
        if s.time_us < prev_time {
            return Err(ExecutionError::TimeReversal {
                previous_us: prev_time,
                current_us: s.time_us,
            });
        }
        prev_time = s.time_us;
    }

    let mut queue = BinaryHeap::new();
    let mut sequence_counter = 0u64;
    for (idx, step) in request.steps.iter().enumerate() {
        if step.time_us <= request.horizon_us {
            sequence_counter += 1;
            queue.push(QueuedEvent {
                time_us: step.time_us,
                priority: step.priority,
                sequence_id: sequence_counter,
                action: step.action.clone(),
                source_step: idx,
            });
        }
    }

    let mut state = SimState {
        active_actor: request.active_actor,
        actors,
        targets,
        pools,
        events: Vec::new(),
        last_event_time_us: 0,
        max_proc_depth_reached: 0,
    };

    let mut total_events_processed = 0u64;
    let mut total_invariants_checked = 0usize;
    let mut halt_diagnostic: Option<HaltDiagnostic> = None;

    let init_checks = verify_state_invariants(&state, &limits)?;
    total_invariants_checked += init_checks;

    while let Some(queued) = queue.pop() {
        if total_events_processed >= limits.max_events {
            halt_diagnostic = Some(HaltDiagnostic {
                failed_step_index: queued.source_step,
                time_us: queued.time_us,
                code: "LIMIT_EXCEEDED".into(),
                message: format!("total events exceeded limit {}", limits.max_events),
            });
            break;
        }

        let pre_step_state = state.clone();

        match execute_step_action(
            &mut state,
            queued.time_us,
            &queued.action,
            &limits,
            &mut sequence_counter,
            &mut queue,
        ) {
            Ok(event) => {
                total_events_processed += 1;
                state.last_event_time_us = queued.time_us;
                state.events.push(event);

                match verify_state_invariants(&state, &limits) {
                    Ok(checks) => {
                        total_invariants_checked += checks;
                    }
                    Err(e) => {
                        state = pre_step_state;
                        halt_diagnostic = Some(HaltDiagnostic {
                            failed_step_index: queued.source_step,
                            time_us: queued.time_us,
                            code: "INVARIANT_VIOLATION".into(),
                            message: e.to_string(),
                        });
                        break;
                    }
                }
            }
            Err(e) => {
                state = pre_step_state;
                halt_diagnostic = Some(HaltDiagnostic {
                    failed_step_index: queued.source_step,
                    time_us: queued.time_us,
                    code: match &e {
                        ExecutionError::ResourceShortage { .. } => "RESOURCE_SHORTAGE".into(),
                        ExecutionError::ActorLocked { .. } => "ACTOR_LOCKED".into(),
                        ExecutionError::ActorOnCooldown { .. } => "ACTOR_ON_COOLDOWN".into(),
                        ExecutionError::ActorNotActive { .. } => "ACTOR_NOT_ACTIVE".into(),
                        ExecutionError::ActorDead(..) => "ACTOR_DEAD".into(),
                        ExecutionError::TargetDead(..) => "TARGET_DEAD".into(),
                        ExecutionError::ProcDepthExceeded { .. } => "PROC_DEPTH_EXCEEDED".into(),
                        ExecutionError::LimitExceeded { .. } => "LIMIT_EXCEEDED".into(),
                        _ => "EXECUTION_ERROR".into(),
                    },
                    message: e.to_string(),
                });
                break;
            }
        }
    }

    let final_checks = verify_state_invariants(&state, &limits)?;
    total_invariants_checked += final_checks;

    let mut pool_reports = BTreeMap::new();
    for (id, p) in &state.pools {
        pool_reports.insert(
            id.clone(),
            ResourcePoolReport {
                pool_id: id.clone(),
                current: p.current,
                capacity: p.capacity,
                initial: p.initial,
                total_granted: p.total_granted,
                total_spent: p.total_spent,
                total_overflow: p.total_overflow,
                conserved: p.is_conserved(),
            },
        );
    }

    let state_json = serde_json::json!({
        "active_actor": state.active_actor,
        "actors": state.actors,
        "targets": state.targets,
        "resource_pools": pool_reports,
    });
    let state_bytes =
        serde_json::to_vec(&state_json).map_err(|error| ExecutionError::InvariantViolation {
            invariant_name: "state_serialization".into(),
            details: error.to_string(),
        })?;
    let state_sha256 = sha256(&state_bytes);

    let trace_bytes =
        serde_json::to_vec(&state.events).map_err(|error| ExecutionError::InvariantViolation {
            invariant_name: "trace_serialization".into(),
            details: error.to_string(),
        })?;
    let trace_sha256 = sha256(&trace_bytes);

    let is_ok = halt_diagnostic.is_none();

    Ok(ExecutionScenarioReport {
        status: if is_ok { "ok".into() } else { "failed".into() },
        content_id: request.content_id.clone(),
        success: is_ok,
        execution_completed: is_ok,
        halt_diagnostic,
        horizon_us: request.horizon_us,
        final_time_us: state.last_event_time_us,
        final_active_actor: state.active_actor,
        party_wiped: state.actors.iter().all(|a| !a.is_alive),
        total_events_processed,
        max_proc_depth_reached: state.max_proc_depth_reached,
        invariants_checked: total_invariants_checked,
        invariants_passed: true,
        state_sha256,
        trace_sha256,
        actors: state.actors,
        targets: state.targets,
        resource_pools: pool_reports,
        events: state.events,
        quality: "incomplete".into(),
        combat_adapter_complete: false,
        game_dps_computed: false,
    })
}

pub fn evaluate_execution_scenario(
    request: &ExecutionScenarioRequest,
) -> Result<serde_json::Value, String> {
    let report = run_execution_scenario(request).map_err(|e| e.to_string())?;
    serde_json::to_value(report).map_err(|e| e.to_string())
}
