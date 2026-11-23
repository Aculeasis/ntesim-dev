use super::*;

#[test]
fn failed_dispatch_rolls_back_every_proc_and_does_not_consume_icd_or_resources() {
    let mut engine = DispatchEngine::new(
        1,
        0,
        BTreeMap::from([("energy".into(), Pool::new(0, 100).unwrap())]),
    )
    .unwrap();
    for (id, payload) in [
        (
            "energy",
            ProcPayload::GrantPoolResource {
                pool: "energy".into(),
                amount: 7,
            },
        ),
        (
            "recursive",
            ProcPayload::TriggerBonusDamage {
                element: "Anima".into(),
                multiplier_percent: 100,
                target_id: "target".into(),
            },
        ),
    ] {
        engine
            .register_callback(CallbackRegistration {
                id: id.into(),
                owner_actor: 0,
                conditions: vec![ProcCondition::AnyEvent],
                icd_rule: IcdRule::None,
                icd_scope: IcdScope::Global,
                payload,
                max_triggers: None,
                triggers_so_far: 0,
            })
            .unwrap();
    }
    let before = serde_json::to_value(&engine).unwrap();
    assert!(matches!(
        engine.dispatch(
            CombatEvent::HitDealt {
                source_actor: 0,
                target_id: "target".into(),
                action_id: "action".into(),
                element: "Anima".into(),
                is_crit: false,
                damage: 100
            },
            5
        ),
        Err(DispatchError::RecursionDepthExceeded { .. })
    ));
    assert_eq!(serde_json::to_value(&engine).unwrap(), before);
}
