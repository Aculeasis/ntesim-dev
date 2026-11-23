use super::*;

fn dash(time: u64, critical: bool) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Dash,
        critical_success: critical,
    }
}

#[test]
fn ordinary_dodge_requires_stamina_and_fails_before_creating_a_derived_skill() {
    let mut req = phase2_request();
    req.actions = vec![dash(0, false)];
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("Baicang dodge requires an explicit combat stamina profile"));
    dodge(&mut req);
    req.actions = vec![dash(0, false)];
    req.combat_stamina.as_mut().unwrap().initial = Ratio {
        numerator: 14,
        denominator: 1,
    };
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["hits_settled"], 0);
    assert!(
        report["combat_stamina"]["spends"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 10000);
}

#[test]
fn dodge_immunity_is_half_open_and_does_not_arm_fatal_debt() {
    let mut req = phase2_request();
    req.actions.clear();
    dodge(&mut req);
    req.actions = vec![dash(0, false)];
    let id = crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    req.incoming_damage_events = vec![
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 349999,
            actor_index: 0,
            delivered_damage: 500,
        },
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 350000,
            actor_index: 0,
            delivered_damage: 500,
        },
    ];
    let report = run(&req);
    assert_eq!(
        report["incoming_damage_receipts"][0]["prevented_hp_loss"],
        500
    );
    assert_eq!(report["incoming_damage_receipts"][0]["hp_after"], 10000);
    assert_eq!(report["incoming_damage_receipts"][1]["actual_hp_loss"], 500);
    assert_eq!(report["final_actor_states"][0]["hp"], 9500);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["event_kind"] != "fatal_debt_installment")
    );
}

#[test]
fn dodge_derived_choice_expires_two_seconds_after_recovery() {
    for (time, packet, hp) in [
        (2499999, "GE_Player_Cang_SkillA_Damage", 9700),
        (2500000, "GE_Player_Cang_Skill_Damage", 10000),
    ] {
        let mut req = phase2_request();
        req.actions.clear();
        dodge(&mut req);
        req.actions = vec![
            dash(0, false),
            make_action(
                time,
                0,
                "GA_Cang_Skill",
                "GE_Player_Cang_Skill_Damage",
                0,
                14_000_000,
            ),
        ];
        let report = run(&req);
        assert_eq!(report["hit_receipts"][0]["damage_effect_id"], packet);
        assert_eq!(report["final_actor_states"][0]["hp"], hp);
    }
}

#[test]
fn ordinary_dodge_consumes_the_old_riposte_and_cooldown_rejections_spend_nothing() {
    let mut req = phase2_request();
    req.actions.clear();
    dodge(&mut req);
    req.actions.extend([dash(1000000, false), normal(1500000)]);
    let report = run(&req);
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Cang_Melee1_Damage"
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 10000);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("30/1"));
    req.actions = vec![dash(0, false), dash(800000, false)];
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert_eq!(
        failed["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
    assert_eq!(failed["combat_stamina"]["current"], json!("85/1"));
}
