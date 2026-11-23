use super::*;

#[test]
fn switch_commands_and_events_share_field_state_and_callback_execution() {
    let callback = CallbackRegistration {
        id: "switch-watch".into(),
        owner_actor: 1,
        conditions: vec![ProcCondition::RequiresOnField { on_field: true }],
        icd_rule: IcdRule::None,
        icd_scope: IcdScope::Global,
        payload: ProcPayload::GrantPoolResource {
            pool: "energy".into(),
            amount: 3,
        },
        max_triggers: Some(1),
        triggers_so_far: 0,
    };
    let event = CombatEvent::ActorSwitched {
        from_index: 0,
        to_index: 1,
    };
    let mut results = Vec::new();
    for step in [
        DispatchStep::SwitchActor {
            target_index: 1,
            timestamp_us: 5,
        },
        DispatchStep::EmitEvent {
            event: event.clone(),
            timestamp_us: 5,
        },
    ] {
        let request = DispatchScenarioRequest {
            content_id: "ntesim-current".into(),
            team_size: 2,
            initial_active: 0,
            initial_resources: BTreeMap::from([(
                "energy".into(),
                PoolInput {
                    current: 0,
                    capacity: 10,
                },
            )]),
            steps: vec![
                DispatchStep::RegisterCallback {
                    callback: callback.clone(),
                },
                step,
            ],
        };
        let result = evaluate_dispatch_scenario(&request).unwrap();
        assert_eq!(result["receipt"]["resources"]["energy"]["current"], 3);
        assert_eq!(result["receipt"]["final_active"], 1);
        results.push(result["receipt"]["resources"].clone());
    }
    assert_eq!(results[0], results[1]);
    assert!(DispatchEngine::new(0, 0, BTreeMap::new()).is_err());
    let mut engine = DispatchEngine::new(2, 0, BTreeMap::new()).unwrap();
    let before = serde_json::to_value(&engine).unwrap();
    assert!(
        engine
            .dispatch(
                CombatEvent::ActorSwitched {
                    from_index: 1,
                    to_index: 0
                },
                5
            )
            .is_err()
    );
    assert_eq!(serde_json::to_value(&engine).unwrap(), before);
}
