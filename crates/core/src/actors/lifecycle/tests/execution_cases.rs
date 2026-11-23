use super::*;

#[test]
pub(crate) fn field_ownership_initial_state_and_switching() {
    let mut team = create_test_team();
    assert_eq!(team.field.active_actor(), 0);
    assert_eq!(team.field_status(0).unwrap(), FieldStatus::OnField);
    assert_eq!(team.field_status(1).unwrap(), FieldStatus::OffField);

    let rec = team.switch_to(1, 0).unwrap();
    assert_eq!(rec.from_index, 0);
    assert_eq!(rec.to_index, 1);
    assert_eq!(team.field.active_actor(), 1);
    assert_eq!(team.field_status(1).unwrap(), FieldStatus::OnField);
    assert_eq!(team.field_status(0).unwrap(), FieldStatus::OffField);

    let err = team.switch_to(2, 500_000).unwrap_err();
    assert_eq!(
        err,
        LifecycleError::SwitchOnCooldown {
            until_us: 1_000_000,
            now_us: 500_000,
        }
    );

    let err = team.switch_to(1, 1_500_000).unwrap_err();
    assert_eq!(err, LifecycleError::CannotSwitchToSelf(1));

    let rec2 = team.switch_to(2, 2_000_000).unwrap();
    assert_eq!(rec2.from_index, 1);
    assert_eq!(rec2.to_index, 2);
    assert_eq!(team.field.active_actor(), 2);
}

#[test]
pub(crate) fn require_on_field_guards_actions() {
    let mut team = create_test_team();
    let timings = ActionPhaseTimings {
        startup_us: 200_000,
        active_us: 100_000,
        recovery_us: 300_000,
        recovery_cancels: vec![],
        startup_cancels: vec![],
    };

    let err = team
        .start_action(
            1,
            "skill".to_string(),
            timings.clone(),
            BTreeMap::new(),
            0,
            true,
        )
        .unwrap_err();
    assert_eq!(
        err,
        LifecycleError::RequiresOnField {
            actor_index: 1,
            active_actor: 0,
        }
    );

    team.start_action(
        1,
        "off_field_skill".to_string(),
        timings,
        BTreeMap::new(),
        0,
        false,
    )
    .unwrap();
    assert_eq!(
        team.actors[1].action_state.phase.kind(),
        ActionPhaseKind::Startup
    );
}

#[test]
pub(crate) fn action_phase_progression_and_advance_time() {
    let mut team = create_test_team();
    let timings = ActionPhaseTimings {
        startup_us: 200_000,
        active_us: 300_000,
        recovery_us: 400_000,
        recovery_cancels: vec![],
        startup_cancels: vec![],
    };

    team.start_action(
        0,
        "basic_combo".to_string(),
        timings,
        BTreeMap::new(),
        100_000,
        true,
    )
    .unwrap();

    assert_eq!(
        team.actors[0].action_state.phase.kind(),
        ActionPhaseKind::Startup
    );

    let t1 = team.advance_time(250_000).unwrap();
    assert!(t1.is_empty());
    assert_eq!(
        team.actors[0].action_state.phase.kind(),
        ActionPhaseKind::Startup
    );

    let t2 = team.advance_time(300_000).unwrap();
    assert_eq!(t2.len(), 1);
    assert_eq!(t2[0].from_kind, ActionPhaseKind::Startup);
    assert_eq!(t2[0].to_kind, ActionPhaseKind::Active);
    assert_eq!(
        team.actors[0].action_state.phase.kind(),
        ActionPhaseKind::Active
    );

    let t3 = team.advance_time(600_000).unwrap();
    assert_eq!(t3.len(), 1);
    assert_eq!(t3[0].from_kind, ActionPhaseKind::Active);
    assert_eq!(t3[0].to_kind, ActionPhaseKind::Recovery);
    assert_eq!(
        team.actors[0].action_state.phase.kind(),
        ActionPhaseKind::Recovery
    );

    let t4 = team.advance_time(1_000_000).unwrap();
    assert_eq!(t4.len(), 1);
    assert_eq!(t4[0].from_kind, ActionPhaseKind::Recovery);
    assert_eq!(t4[0].to_kind, ActionPhaseKind::Idle);
    assert!(team.actors[0].action_state.is_idle());
}

#[test]
pub(crate) fn cancel_windows_and_recovery_cancel() {
    let mut team = create_test_team();
    let timings = ActionPhaseTimings {
        startup_us: 100_000,
        active_us: 200_000,
        recovery_us: 500_000,
        recovery_cancels: vec![
            CancelWindow {
                cancel_type: CancelType::Dash,
                earliest_offset_us: 100_000,
            },
            CancelWindow {
                cancel_type: CancelType::Switch,
                earliest_offset_us: 0,
            },
        ],
        startup_cancels: vec![],
    };

    team.start_action(
        0,
        "heavy_slash".to_string(),
        timings,
        BTreeMap::new(),
        0,
        true,
    )
    .unwrap();

    let err = team.cancel_action(0, CancelType::Dash, 50_000).unwrap_err();
    assert_eq!(
        err,
        LifecycleError::CancelNotAllowed {
            phase: ActionPhaseKind::Startup,
            cancel_type: CancelType::Dash,
        }
    );

    team.advance_time(300_000).unwrap();
    assert_eq!(
        team.actors[0].action_state.phase.kind(),
        ActionPhaseKind::Recovery
    );

    let err2 = team
        .cancel_action(0, CancelType::Dash, 350_000)
        .unwrap_err();
    assert_eq!(
        err2,
        LifecycleError::CancelTooEarly {
            earliest_us: 400_000,
            now_us: 350_000,
        }
    );

    let err3 = team
        .cancel_action(0, CancelType::Jump, 400_000)
        .unwrap_err();
    assert_eq!(
        err3,
        LifecycleError::CancelNotAllowed {
            phase: ActionPhaseKind::Recovery,
            cancel_type: CancelType::Jump,
        }
    );

    let rec = team.cancel_action(0, CancelType::Dash, 400_000).unwrap();
    assert_eq!(rec.cancel_type, CancelType::Dash);
    assert_eq!(rec.phase, ActionPhaseKind::Recovery);
    assert!(team.actors[0].action_state.is_idle());
}

#[test]
pub(crate) fn switch_cancels_active_actor_if_permitted() {
    let mut team = create_test_team();
    let timings = ActionPhaseTimings {
        startup_us: 100_000,
        active_us: 200_000,
        recovery_us: 500_000,
        recovery_cancels: vec![CancelWindow {
            cancel_type: CancelType::Switch,
            earliest_offset_us: 0,
        }],
        startup_cancels: vec![],
    };

    team.start_action(0, "skill".to_string(), timings, BTreeMap::new(), 0, true)
        .unwrap();

    let err = team.switch_to(1, 50_000).unwrap_err();
    assert_eq!(
        err,
        LifecycleError::ActiveActorLockedInPhase {
            phase: ActionPhaseKind::Startup,
            until_us: 100_000,
        }
    );

    let switch_rec = team.switch_to(1, 1_000_000).unwrap();
    assert_eq!(switch_rec.from_index, 0);
    assert_eq!(switch_rec.to_index, 1);
    assert!(team.actors[0].action_state.is_idle());
    assert_eq!(team.field.active_actor(), 1);
}

#[test]
pub(crate) fn actor_death_and_auto_field_transfer() {
    let mut team = create_test_team();
    let timings = ActionPhaseTimings {
        startup_us: 500_000,
        active_us: 100_000,
        recovery_us: 100_000,
        recovery_cancels: vec![],
        startup_cancels: vec![],
    };
    team.start_action(0, "charge".to_string(), timings, BTreeMap::new(), 0, true)
        .unwrap();

    let switch_rec = team.kill_actor(0, 100_000).unwrap().unwrap();
    assert_eq!(switch_rec.from_index, 0);
    assert_eq!(switch_rec.to_index, 1);
    assert_eq!(team.field.active_actor(), 1);
    assert!(!team.field.is_alive(0));
    assert!(team.actors[0].action_state.is_idle());

    let err = team.switch_to(0, 2_000_000).unwrap_err();
    assert_eq!(err, LifecycleError::ActorDead(0));

    let _ = team.kill_actor(1, 2_100_000).unwrap().unwrap();
    assert_eq!(team.field.active_actor(), 2);

    let _ = team.kill_actor(2, 2_200_000).unwrap().unwrap();
    assert_eq!(team.field.active_actor(), 3);

    let wipe = team.kill_actor(3, 2_300_000).unwrap();
    assert!(wipe.is_none());
    assert!(!team.field.is_alive(3));
}

#[test]
pub(crate) fn forms_and_recasts() {
    let mut actor = ActorLifecycle::new(0, "sakiri".to_string());

    assert!(actor.form_state.active_form(0).is_none());
    actor
        .form_state
        .enter_form("demon_mode".to_string(), Some(5_000_000), 1_000_000)
        .unwrap();
    assert_eq!(actor.form_state.active_form(2_000_000), Some("demon_mode"));
    assert_eq!(actor.form_state.active_form(5_999_999), Some("demon_mode"));
    assert!(actor.form_state.active_form(6_000_000).is_none());
    actor.form_state.advance_time(7_000_000);
    assert!(actor.form_state.current_form().is_none());

    actor
        .recast_state
        .open_recast(
            "skill_e".to_string(),
            "skill_e2".to_string(),
            3_000_000,
            2,
            1_000_000,
        )
        .unwrap();
    assert!(actor.recast_state.can_recast("skill_e2", 2_000_000).is_ok());

    let err = actor
        .recast_state
        .can_recast("skill_e3", 2_000_000)
        .unwrap_err();
    assert_eq!(
        err,
        LifecycleError::RecastActionMismatch {
            expected: "skill_e2".to_string(),
            actual: "skill_e3".to_string(),
        }
    );

    let rem1 = actor
        .recast_state
        .consume_recast("skill_e2", 2_000_000)
        .unwrap();
    assert_eq!(rem1, 1);

    let rem2 = actor
        .recast_state
        .consume_recast("skill_e2", 2_500_000)
        .unwrap();
    assert_eq!(rem2, 0);
    assert!(actor.recast_state.active_recast().is_none());

    let err2 = actor
        .recast_state
        .can_recast("skill_e2", 2_600_000)
        .unwrap_err();
    assert_eq!(err2, LifecycleError::NoActiveRecast);
}

#[test]
pub(crate) fn evaluate_scenario_receipt_matches() {
    let mut initial_resources = BTreeMap::new();
    initial_resources.insert(
        "energy".to_string(),
        PoolInput {
            current: 100,
            capacity: 100,
        },
    );
    initial_resources.insert(
        "stamina".to_string(),
        PoolInput {
            current: 200,
            capacity: 200,
        },
    );

    let mut costs = BTreeMap::new();
    costs.insert("energy".to_string(), 40);

    let timings = ActionPhaseTimings {
        startup_us: 100_000,
        active_us: 200_000,
        recovery_us: 500_000,
        recovery_cancels: vec![
            CancelWindow {
                cancel_type: CancelType::Dash,
                earliest_offset_us: 100_000,
            },
            CancelWindow {
                cancel_type: CancelType::Switch,
                earliest_offset_us: 0,
            },
        ],
        startup_cancels: vec![],
    };

    let request = LifecycleScenarioRequest {
        content_id: "ntesim-current".to_string(),
        team_size: 4,
        characters: vec![
            "sakiri".to_string(),
            "lacrimosa".to_string(),
            "skia".to_string(),
            "nanally".to_string(),
        ],
        initial_active: 0,
        switch_cooldown_us: 1_000_000,
        initial_resources,
        steps: vec![
            LifecycleStep::StartAction {
                actor_index: 0,
                action_id: "slash_combo".to_string(),
                timings,
                costs,
                timestamp_us: 0,
                require_on_field: true,
            },
            LifecycleStep::AdvanceTime {
                timestamp_us: 300_000,
            },
            LifecycleStep::CancelAction {
                actor_index: 0,
                cancel_type: CancelType::Dash,
                timestamp_us: 400_000,
            },
            LifecycleStep::SwitchActor {
                target_index: 1,
                timestamp_us: 1_000_000,
            },
            LifecycleStep::EnterForm {
                actor_index: 1,
                form_id: "shadow_form".to_string(),
                duration_us: Some(2_000_000),
                timestamp_us: 1_000_000,
            },
            LifecycleStep::OpenRecast {
                actor_index: 1,
                base_action: "teleport".to_string(),
                recast_action: "teleport_strike".to_string(),
                duration_us: 1_000_000,
                max_uses: 1,
                timestamp_us: 1_000_000,
            },
            LifecycleStep::ConsumeRecast {
                actor_index: 1,
                recast_action: "teleport_strike".to_string(),
                timestamp_us: 1_200_000,
            },
            LifecycleStep::AdvanceTime {
                timestamp_us: 3_500_000,
            },
            LifecycleStep::SwitchActor {
                target_index: 2,
                timestamp_us: 4_000_000,
            },
        ],
    };

    let res = evaluate_lifecycle_scenario(&request).unwrap();
    assert_eq!(res["status"], "ok");
    assert_eq!(res["receipt"]["final_active"], 2);
    assert_eq!(res["receipt"]["total_steps"], 9);
    assert_eq!(res["receipt"]["resources"]["energy"]["current"], 60);
    assert_eq!(res["receipt"]["resources"]["stamina"]["current"], 200);
    assert_eq!(res["quality"]["lifecycle_model_complete"], true);
}

#[test]
fn zero_duration_phases_are_skipped_in_transition_receipts() {
    let mut team = create_test_team();
    team.start_action(
        0,
        "startup_into_recovery".into(),
        ActionPhaseTimings {
            startup_us: 5,
            active_us: 0,
            recovery_us: 7,
            recovery_cancels: vec![],
            startup_cancels: vec![],
        },
        BTreeMap::new(),
        0,
        true,
    )
    .unwrap();
    assert_eq!(
        team.actors[0].action_state.phase.kind(),
        ActionPhaseKind::Startup
    );
    assert!(team.advance_time(4).unwrap().is_empty());
    let transition = team.advance_time(5).unwrap();
    assert_eq!(transition.len(), 1);
    assert_eq!(transition[0].from_kind, ActionPhaseKind::Startup);
    assert_eq!(transition[0].to_kind, ActionPhaseKind::Recovery);
    assert_eq!(transition[0].timestamp_us, 5);
    assert_eq!(
        team.actors[0].action_state.phase.kind(),
        ActionPhaseKind::Recovery
    );
    let completed = team.advance_time(12).unwrap();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].from_kind, ActionPhaseKind::Recovery);
    assert_eq!(completed[0].to_kind, ActionPhaseKind::Idle);

    let request = LifecycleScenarioRequest {
        content_id: "ntesim-current".into(),
        team_size: 1,
        characters: vec!["test_actor".into()],
        initial_active: 0,
        switch_cooldown_us: 0,
        initial_resources: BTreeMap::new(),
        steps: vec![
            LifecycleStep::StartAction {
                actor_index: 0,
                action_id: "recovery_only".into(),
                timings: ActionPhaseTimings {
                    startup_us: 0,
                    active_us: 0,
                    recovery_us: 7,
                    recovery_cancels: vec![],
                    startup_cancels: vec![],
                },
                costs: BTreeMap::new(),
                timestamp_us: 0,
                require_on_field: true,
            },
            LifecycleStep::AdvanceTime { timestamp_us: 7 },
        ],
    };
    let report = evaluate_lifecycle_scenario(&request).unwrap();
    assert_eq!(
        report["receipt"]["step_receipts"][0]["details"]["phase"],
        "recovery"
    );
    assert_eq!(
        report["receipt"]["step_receipts"][1]["details"]["transitions"][0]["to_kind"],
        "idle"
    );
}

#[test]
fn recast_windows_reject_unusable_definitions_atomically() {
    let mut state = RecastState::default();
    state
        .open_recast("base".into(), "followup".into(), 10, 2, 5)
        .unwrap();
    let before = state.clone();
    assert_eq!(
        state.open_recast("base".into(), "zero_time".into(), 0, 2, 6),
        Err(LifecycleError::InvalidRecastDuration)
    );
    assert_eq!(state, before);
    assert_eq!(
        state.open_recast("base".into(), "zero_uses".into(), 10, 0, 6),
        Err(LifecycleError::InvalidRecastUses)
    );
    assert_eq!(state, before);
    assert!(state.can_recast("followup", 5).is_ok());
    assert_eq!(
        state.can_recast("followup", 4),
        Err(LifecycleError::TimeReversal {
            last_us: 5,
            now_us: 4,
        })
    );
    state.advance_time(15);
    assert!(state.active_recast().is_none());
}

#[test]
fn lifecycle_state_deserialization_rejects_inconsistent_snapshots() {
    let invalid_action = serde_json::json!({
        "phase": {"phase": "recovery", "action_id": "cast", "started_at_us": 10, "ends_at_us": 20},
        "timings": null,
        "pending_costs": {},
    });
    assert!(serde_json::from_value::<ActionStateMachine>(invalid_action).is_err());

    let invalid_form = serde_json::json!({
        "current_form": {"form_id": "", "entered_at_us": 10, "expires_at_us": 10},
    });
    assert!(serde_json::from_value::<FormState>(invalid_form).is_err());

    let invalid_recast = serde_json::json!({
        "active_recast": {"base_action": "cast", "recast_action": "followup", "opened_at_us": 10, "expires_at_us": 10, "uses_remaining": 1},
    });
    assert!(serde_json::from_value::<RecastState>(invalid_recast).is_err());

    let valid_roundtrip = serde_json::to_value(ActionStateMachine::default()).unwrap();
    assert_eq!(
        serde_json::from_value::<ActionStateMachine>(valid_roundtrip).unwrap(),
        ActionStateMachine::default()
    );
}

#[test]
fn dead_actors_cannot_enter_forms_or_open_or_consume_recasts() {
    let mut team = create_test_team();
    team.enter_form(0, "form".into(), Some(100), 0).unwrap();
    team.open_recast(0, "base".into(), "followup".into(), 100, 1, 0)
        .unwrap();
    team.kill_actor(0, 1).unwrap();
    assert!(team.actors[0].form_state.current_form().is_none());
    assert!(team.actors[0].recast_state.active_recast().is_none());
    assert_eq!(
        team.enter_form(0, "later".into(), None, 2),
        Err(LifecycleError::ActorDead(0))
    );
    assert_eq!(
        team.open_recast(0, "base".into(), "later".into(), 100, 1, 2),
        Err(LifecycleError::ActorDead(0))
    );
    assert_eq!(
        team.consume_recast(0, "followup", 2),
        Err(LifecycleError::ActorDead(0))
    );
    assert_eq!(team.last_time_us, 1);
}

#[test]
fn standalone_action_clock_rejects_reversal_across_phases_and_restore() {
    let timings = ActionPhaseTimings {
        startup_us: 5,
        active_us: 10,
        recovery_us: 3,
        startup_cancels: Vec::new(),
        recovery_cancels: Vec::new(),
    };
    let mut state = ActionStateMachine::default();
    state
        .start("new-carrier".into(), timings.clone(), BTreeMap::new(), 2)
        .unwrap();
    state.advance_time(0, 8).unwrap();
    let before = state.clone();
    assert_eq!(
        state.advance_time(0, 7),
        Err(LifecycleError::TimeReversal {
            last_us: 8,
            now_us: 7
        })
    );
    assert_eq!(state, before);
    let mut restored: ActionStateMachine =
        serde_json::from_value(serde_json::to_value(&state).unwrap()).unwrap();
    assert_eq!(
        restored.advance_time(0, 7),
        Err(LifecycleError::TimeReversal {
            last_us: 8,
            now_us: 7
        })
    );
    restored.advance_time(0, 20).unwrap();
    assert!(restored.is_idle());
    assert_eq!(
        restored.start("second".into(), timings, BTreeMap::new(), 19),
        Err(LifecycleError::TimeReversal {
            last_us: 20,
            now_us: 19
        })
    );
    let mut forged = serde_json::to_value(&state).unwrap();
    forged["last_time_us"] = serde_json::json!(100);
    assert!(serde_json::from_value::<ActionStateMachine>(forged).is_err());
}
