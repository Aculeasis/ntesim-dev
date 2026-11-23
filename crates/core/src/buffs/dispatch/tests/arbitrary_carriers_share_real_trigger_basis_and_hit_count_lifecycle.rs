use super::*;

#[test]
fn arbitrary_carriers_share_real_trigger_basis_and_hit_count_lifecycle() {
    let mut engine = DispatchEngine::new(
        2,
        0,
        BTreeMap::from([("energy".into(), Pool::new(0, 100).unwrap())]),
    )
    .unwrap();
    engine
        .register_callback(CallbackRegistration {
            id: "new-owner-energy".into(),
            owner_actor: 1,
            conditions: vec![ProcCondition::ElementMatches {
                element: "Anima".into(),
            }],
            icd_rule: IcdRule::HitCountOnly { hit_interval: 3 },
            icd_scope: IcdScope::PerTarget("dynamic".into()),
            payload: ProcPayload::GrantPoolResource {
                pool: "energy".into(),
                amount: 17,
            },
            max_triggers: None,
            triggers_so_far: 0,
        })
        .unwrap();
    let hit = |target: &str, damage| CombatEvent::HitDealt {
        source_actor: 1,
        target_id: target.into(),
        action_id: "new-action".into(),
        element: "Anima".into(),
        is_crit: false,
        damage,
    };
    for index in 0..2 {
        assert!(
            engine
                .dispatch(hit("a", 240), index)
                .unwrap()
                .procs_triggered
                .is_empty()
        );
    }
    assert_eq!(
        engine
            .dispatch(hit("a", 240), 2)
            .unwrap()
            .procs_triggered
            .len(),
        1
    );
    assert!(
        engine
            .dispatch(hit("b", 240), 3)
            .unwrap()
            .procs_triggered
            .is_empty()
    );
    assert_eq!(engine.resources["energy"].current(), 17);
    engine
        .register_callback(CallbackRegistration {
            id: "new-followup".into(),
            owner_actor: 1,
            conditions: vec![ProcCondition::ActionMatches {
                action_id: "new-action".into(),
            }],
            icd_rule: IcdRule::None,
            icd_scope: IcdScope::Global,
            payload: ProcPayload::TriggerBonusDamage {
                element: "Anima".into(),
                multiplier_percent: 25,
                target_id: "a".into(),
            },
            max_triggers: Some(1),
            triggers_so_far: 0,
        })
        .unwrap();
    let receipt = engine.dispatch(hit("a", 240), 4).unwrap();
    let proc = receipt
        .procs_triggered
        .iter()
        .find(|proc| proc.callback_id == "new-followup")
        .unwrap();
    assert_eq!(proc.details["damage"], 60); // independent integer oracle: 240 / 4
}
