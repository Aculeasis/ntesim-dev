use super::*;

#[test]
fn evaluate_dispatch_scenario_receipt_matches() {
    let mut initial_resources = BTreeMap::new();
    initial_resources.insert(
        "energy".to_string(),
        PoolInput {
            current: 0,
            capacity: 100,
        },
    );
    initial_resources.insert(
        "stamina".to_string(),
        PoolInput {
            current: 100,
            capacity: 200,
        },
    );

    let mut mods = BTreeMap::new();
    mods.insert(
        "atk_percent".to_string(),
        Ratio {
            numerator: 15,
            denominator: 100,
        },
    );

    let buff = BuffDefinition {
        stacking_scope: BuffStackingScope::PerSource,
        semantic_tags: Default::default(),
        source_lifetime: Default::default(),
        id: "attack_surge".to_string(),
        source: "relic".to_string(),
        recipient: BuffRecipient::ActiveActor,
        duration_us: Some(5_000_000),
        refresh_policy: RefreshPolicy::Reset,
        max_stacks: 1,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: mods,
    };

    let cb = CallbackRegistration {
        id: "crit_energize".to_string(),
        owner_actor: 0,
        conditions: vec![ProcCondition::RequiresCrit],
        icd_rule: IcdRule::None,
        icd_scope: IcdScope::Global,
        payload: ProcPayload::GrantPoolResource {
            pool: "energy".to_string(),
            amount: 25,
        },
        max_triggers: None,
        triggers_so_far: 0,
    };

    let request = DispatchScenarioRequest {
        content_id: "ntesim-current".to_string(),
        team_size: 4,
        initial_active: 0,
        initial_resources,
        steps: vec![
            DispatchStep::RegisterCallback { callback: cb },
            DispatchStep::ApplyBuff {
                buff,
                owner_actor: 0,
                timestamp_us: 0,
            },
            DispatchStep::EmitEvent {
                event: CombatEvent::HitDealt {
                    source_actor: 0,
                    target_id: "enemy".to_string(),
                    action_id: "slash".to_string(),
                    element: "Physical".to_string(),
                    is_crit: true,
                    damage: 500,
                },
                timestamp_us: 0,
            },
            DispatchStep::EmitEvent {
                event: CombatEvent::HitDealt {
                    source_actor: 0,
                    target_id: "enemy".to_string(),
                    action_id: "slash".to_string(),
                    element: "Physical".to_string(),
                    is_crit: false,
                    damage: 300,
                },
                timestamp_us: 100_000,
            },
            DispatchStep::SwitchActor {
                target_index: 1,
                timestamp_us: 1_000_000,
            },
            DispatchStep::AdvanceTime {
                timestamp_us: 6_000_000,
            },
        ],
    };

    let res = evaluate_dispatch_scenario(&request).unwrap();
    assert_eq!(res["status"], "ok");
    assert_eq!(res["receipt"]["final_active"], 1);
    assert_eq!(res["receipt"]["total_steps"], 6);
    assert_eq!(res["receipt"]["resources"]["energy"]["current"], 25);
    assert_eq!(res["receipt"]["active_buff_count"], 0);
    assert_eq!(res["quality"]["dispatch_model_complete"], true);
}
