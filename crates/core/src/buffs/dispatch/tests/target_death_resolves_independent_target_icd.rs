use super::*;

#[test]
fn target_death_resolves_independent_target_icd() {
    let mut engine = DispatchEngine::new(
        1,
        0,
        BTreeMap::from([("energy".into(), Pool::new(0, 100).unwrap())]),
    )
    .unwrap();
    engine
        .register_callback(CallbackRegistration {
            id: "arbitrary-death-listener".into(),
            owner_actor: 0,
            conditions: vec![ProcCondition::AnyEvent],
            icd_rule: IcdRule::TimerOnly { window_us: 2500 },
            icd_scope: IcdScope::PerTarget("placeholder".into()),
            payload: ProcPayload::GrantPoolResource {
                pool: "energy".into(),
                amount: 1,
            },
            max_triggers: None,
            triggers_so_far: 0,
        })
        .unwrap();
    for (target, count) in [("a", 1), ("b", 1), ("a", 0)] {
        let result = engine
            .dispatch(
                CombatEvent::TargetDied {
                    target_id: target.into(),
                },
                0,
            )
            .unwrap();
        assert_eq!(result.procs_triggered.len(), count);
    }
    assert_eq!(engine.resources["energy"].current(), 2);
}
