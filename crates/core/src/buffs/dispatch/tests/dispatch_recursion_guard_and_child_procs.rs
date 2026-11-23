use super::*;

#[test]
fn dispatch_recursion_guard_and_child_procs() {
    let mut pools = BTreeMap::new();
    pools.insert("energy".to_string(), Pool::new(0, 100).unwrap());

    let mut engine = DispatchEngine::new(4, 0, pools).unwrap();

    let cb_crit = CallbackRegistration {
        id: "crit_shard".to_string(),
        owner_actor: 0,
        conditions: vec![ProcCondition::RequiresCrit],
        icd_rule: IcdRule::None,
        icd_scope: IcdScope::Global,
        payload: ProcPayload::TriggerBonusDamage {
            element: "Incantation".to_string(),
            multiplier_percent: 50,
            target_id: "boss".to_string(),
        },
        max_triggers: None,
        triggers_so_far: 0,
    };

    let cb_energy = CallbackRegistration {
        id: "shard_energy".to_string(),
        owner_actor: 0,
        conditions: vec![ProcCondition::ActionMatches {
            action_id: "crit_shard:proc".to_string(),
        }],
        icd_rule: IcdRule::None,
        icd_scope: IcdScope::Global,
        payload: ProcPayload::GrantPoolResource {
            pool: "energy".to_string(),
            amount: 10,
        },
        max_triggers: None,
        triggers_so_far: 0,
    };

    engine.register_callback(cb_crit).unwrap();
    engine.register_callback(cb_energy).unwrap();

    let hit = CombatEvent::HitDealt {
        source_actor: 0,
        target_id: "boss".to_string(),
        action_id: "slash".to_string(),
        element: "Physical".to_string(),
        is_crit: true,
        damage: 1000,
    };

    let receipt = engine.dispatch(hit, 0).unwrap();
    assert_eq!(receipt.procs_triggered.len(), 2);
    assert_eq!(engine.resources.get("energy").unwrap().current(), 10);
}
