use super::*;
use crate::combat::DamageKind;

fn sample_request() -> ExecutionScenarioRequest {
    ExecutionScenarioRequest {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        team_size: 2,
        active_actor: 0,
        horizon_us: 5_000_000,
        limits: None,
        actors: vec![
            ActorSpec {
                actor_index: 0,
                character_id: "character:1019".into(),
                level: 80,
                current_hp: 17720,
                max_hp: 17720,
                defense: 727,
                initial_shields: vec![],
            },
            ActorSpec {
                actor_index: 1,
                character_id: "character:1003".into(),
                level: 80,
                current_hp: 16483,
                max_hp: 16483,
                defense: 909,
                initial_shields: vec![],
            },
        ],
        targets: vec![TargetSpec {
            target_id: "target:dummy".into(),
            level: 80,
            current_hp: 100_000,
            max_hp: 100_000,
            defense: 600,
            initial_shields: vec![],
        }],
        resource_pools: vec![ResourcePoolSpec {
            pool_id: "energy".into(),
            initial_amount: 50,
            capacity: 100,
        }],
        steps: vec![
            ExecutionStepSpec {
                time_us: 500_000,
                priority: CausalPriority::ResourceCost,
                action: StepAction::GrantResource {
                    pool_id: "energy".into(),
                    amount: 20,
                },
            },
            ExecutionStepSpec {
                time_us: 1_000_000,
                priority: CausalPriority::ActionAdmission,
                action: StepAction::InvokeAction {
                    actor_index: 0,
                    action_name: "Skill".into(),
                    cost_pool: Some("energy".into()),
                    cost_amount: 30,
                    recovery_us: 500_000,
                    cooldown_us: 500_000,
                },
            },
            ExecutionStepSpec {
                time_us: 1_200_000,
                priority: CausalPriority::HitDelivery,
                action: StepAction::DeliverDamage {
                    actor_index: Some(0),
                    target_id: "target:dummy".into(),
                    base_damage: 5000,
                    damage_kind: DamageKind::Elemental,
                },
            },
        ],
    }
}

#[test]
fn run_execution_scenario_passes_and_conserves_resources() {
    let req = sample_request();
    let report = run_execution_scenario(&req).unwrap();
    assert_eq!(report.status, "ok");
    assert!(report.success);
    assert!(report.execution_completed);
    assert!(report.invariants_passed);
    assert_eq!(report.events.len(), 3);
    let pool = &report.resource_pools["energy"];
    assert_eq!(pool.current, 40);
    assert!(pool.conserved);
    assert!(!report.state_sha256.is_empty());
    assert!(!report.trace_sha256.is_empty());
}

#[test]
fn causal_tie_breaking_executes_admission_before_cost_at_same_timestamp() {
    let mut req = sample_request();
    req.steps.push(ExecutionStepSpec {
        time_us: 2_000_000,
        priority: CausalPriority::Observation,
        action: StepAction::ObserveState {
            label: "obs".into(),
        },
    });
    req.steps.push(ExecutionStepSpec {
        time_us: 2_000_000,
        priority: CausalPriority::ActionAdmission,
        action: StepAction::InvokeAction {
            actor_index: 0,
            action_name: "Action2".into(),
            cost_pool: None,
            cost_amount: 0,
            recovery_us: 100_000,
            cooldown_us: 100_000,
        },
    });
    let report = run_execution_scenario(&req).unwrap();
    assert_eq!(report.status, "ok");
    assert!(matches!(report.events[3], AuditEvent::ActionInvoked { .. }));
    assert!(matches!(report.events[4], AuditEvent::StateObserved { .. }));
}

#[test]
fn partial_failure_rollback_preserves_invariants_and_exact_hash() {
    let mut req = sample_request();
    req.steps.push(ExecutionStepSpec {
        time_us: 2_000_000,
        priority: CausalPriority::ResourceCost,
        action: StepAction::SpendResource {
            pool_id: "energy".into(),
            amount: 999,
        },
    });
    let report1 = run_execution_scenario(&req).unwrap();
    assert_eq!(report1.status, "failed");
    assert!(!report1.success);
    assert!(!report1.execution_completed);
    assert!(report1.halt_diagnostic.is_some());
    assert_eq!(
        report1.halt_diagnostic.as_ref().unwrap().code,
        "RESOURCE_SHORTAGE"
    );
    assert!(report1.invariants_passed);
    assert_eq!(report1.resource_pools["energy"].current, 40);

    let report2 = run_execution_scenario(&req).unwrap();
    assert_eq!(report1.state_sha256, report2.state_sha256);
    assert_eq!(report1.trace_sha256, report2.trace_sha256);
}

#[test]
fn actor_locked_rejection_halts_with_proper_diagnostic() {
    let mut req = sample_request();
    req.steps.push(ExecutionStepSpec {
        time_us: 1_200_000,
        priority: CausalPriority::ActionAdmission,
        action: StepAction::InvokeAction {
            actor_index: 0,
            action_name: "IllegalEarlyAction".into(),
            cost_pool: None,
            cost_amount: 0,
            recovery_us: 100_000,
            cooldown_us: 100_000,
        },
    });
    let report = run_execution_scenario(&req).unwrap();
    assert_eq!(report.status, "failed");
    let diag = report.halt_diagnostic.unwrap();
    assert_eq!(diag.code, "ACTOR_LOCKED");
    assert!(report.invariants_passed);
}

#[test]
fn proc_depth_limit_is_strictly_enforced() {
    let mut req = sample_request();
    req.steps.push(ExecutionStepSpec {
        time_us: 3_000_000,
        priority: CausalPriority::ProcCallback,
        action: StepAction::TriggerProc {
            proc_id: "deep_proc".into(),
            actor_index: 0,
            depth: 5,
            grant_pool: None,
            grant_amount: 0,
        },
    });
    let report = run_execution_scenario(&req).unwrap();
    assert_eq!(report.status, "failed");
    assert_eq!(
        report.halt_diagnostic.as_ref().unwrap().code,
        "PROC_DEPTH_EXCEEDED"
    );
}

#[test]
fn all_execution_inputs_obey_identity_capacity_and_explicit_dependency_contracts() {
    let mut request = sample_request();
    request.schema_version = 99;
    assert!(run_execution_scenario(&request).is_err());
    request = sample_request();
    request.actors[1].actor_index = 0;
    assert!(run_execution_scenario(&request).is_err());
    request = sample_request();
    request.resource_pools[0].initial_amount = 101;
    assert!(
        run_execution_scenario(&request)
            .unwrap_err()
            .to_string()
            .contains("capacity")
    );
    for action in [
        StepAction::InvokeAction {
            actor_index: 0,
            action_name: "new-action".into(),
            cost_pool: None,
            cost_amount: 1,
            recovery_us: 0,
            cooldown_us: 0,
        },
        StepAction::DeliverDamage {
            actor_index: None,
            target_id: "target:dummy".into(),
            base_damage: 100,
            damage_kind: DamageKind::Elemental,
        },
        StepAction::TriggerProc {
            proc_id: "new-proc".into(),
            actor_index: 0,
            depth: 0,
            grant_pool: None,
            grant_amount: 1,
        },
    ] {
        request = sample_request();
        request.steps = vec![ExecutionStepSpec {
            time_us: 0,
            priority: CausalPriority::ActionAdmission,
            action,
        }];
        let report = run_execution_scenario(&request).unwrap();
        assert!(!report.success && !report.execution_completed);
        assert!(
            report
                .halt_diagnostic
                .unwrap()
                .message
                .contains("missing field:")
        );
        assert_eq!(report.resource_pools["energy"].current, 50);
        assert_eq!(report.targets[0].current_hp, 100000);
    }
}

#[test]
fn healing_overflow_and_duplicate_shields_cannot_saturate_or_bypass_shared_vitals() {
    let mut req = sample_request();
    req.steps = vec![0, 1]
        .into_iter()
        .map(|time_us| ExecutionStepSpec {
            time_us,
            priority: CausalPriority::HitDelivery,
            action: StepAction::DeliverHeal {
                actor_index: 0,
                amount_hp: u64::MAX,
            },
        })
        .collect();
    let report = run_execution_scenario(&req).unwrap();
    assert!(!report.execution_completed);
    assert_eq!(report.actors[0].total_overheal, u64::MAX);
    assert_eq!(report.actors[0].current_hp, req.actors[0].current_hp);
    assert!(
        report
            .halt_diagnostic
            .unwrap()
            .message
            .contains("overheal counter overflow")
    );
    req.steps = vec![0, 1]
        .into_iter()
        .map(|time_us| ExecutionStepSpec {
            time_us,
            priority: CausalPriority::HitDelivery,
            action: StepAction::ApplyShield {
                actor_index: 0,
                shield_id: "new-shield".into(),
                source: "new-item".into(),
                amount_hp: 7,
            },
        })
        .collect();
    let report = run_execution_scenario(&req).unwrap();
    assert!(!report.execution_completed);
    assert_eq!(report.actors[0].active_shields.len(), 1);
    assert!(
        report
            .halt_diagnostic
            .unwrap()
            .message
            .contains("DuplicateShield")
    );
}

#[test]
fn duplicate_initial_shields_are_rejected_before_a_noop_scenario_can_pass() {
    let mut req = sample_request();
    req.steps.clear();
    req.actors[0].initial_shields = vec![
        ShieldSpec {
            id: "same".into(),
            source: "first".into(),
            amount_hp: 10,
        },
        ShieldSpec {
            id: "same".into(),
            source: "second".into(),
            amount_hp: 20,
        },
    ];
    assert!(matches!(
        run_execution_scenario(&req),
        Err(ExecutionError::InvariantViolation {
            invariant_name,
            ..
        }) if invariant_name == "shield_identity"
    ));

    req.actors[0].initial_shields.clear();
    req.targets[0].initial_shields = vec![
        ShieldSpec {
            id: "same".into(),
            source: "first".into(),
            amount_hp: 10,
        },
        ShieldSpec {
            id: "same".into(),
            source: "second".into(),
            amount_hp: 20,
        },
    ];
    assert!(matches!(
        run_execution_scenario(&req),
        Err(ExecutionError::InvariantViolation {
            invariant_name,
            ..
        }) if invariant_name == "shield_identity"
    ));
}
