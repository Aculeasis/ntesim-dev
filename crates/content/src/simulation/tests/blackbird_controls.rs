use super::blackbird_evolved_secondary::{burst, evolve_melee, hits, request as base_request};
use super::*;
use serde_json::{Value, json};

fn request() -> GameScenarioRequest {
    let mut req = base_request();
    for id in [
        crate::HypothesisId::BlackbirdBranchesV1,
        crate::HypothesisId::BlackbirdCancelGatesV1,
        crate::HypothesisId::BlackbirdExplicitParryV1,
        crate::HypothesisId::BlackbirdCombatTransitionsV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req
}

fn ordinary(time: u64) -> GameActionCommand {
    let mut action = evolve_melee(time);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        ..
    } = &mut action
    {
        *ability_id = "GA_BlackBird_Melee".into();
        *damage_effect_id = "GE_Player_BlackBird_Melee1_Damage".into();
    }
    action
}

fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
}

fn hold(req: &mut GameScenarioRequest, index: usize) {
    req.hold_inputs.insert(
        index,
        GameHoldInput {
            reserved_stocks: None,
            duration_us: 200_000,
            moving: false,
        },
    );
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

fn malice(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["blackbird.malice"]
}

#[test]
fn blackbird_jump_gate_adds_only_prior_pauses_and_cancels_unemitted_packets() {
    let mut req = request();
    // Normal1 recovery .755920+.029700, Normal2 recovery .733943+2*.029700.
    req.actions = vec![
        ordinary(0),
        ordinary(785_620),
        ordinary(1_578_963),
        jump(1_977_026),
    ];
    let rejected = run(&req);
    assert_eq!(rejected["execution_completed"], false);
    // Stage3 Jump .368364 plus the first contact's .029700; later contacts never occur.
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 1_977_027);
    req.actions[3] = jump(1_977_027);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(hits(&report, "GE_Player_BlackBird_Melee3_Damage").len(), 1);
    assert_eq!(report["hits_settled"], 4);
    assert_eq!(malice(&report)["conserved"], true);
}

#[test]
fn blackbird_roostcall_pays_hold_time_and_grants_once_across_aoe() {
    for targets in [1, 3] {
        let mut req = request();
        for index in 1..targets {
            let mut target = req.targets[0].clone();
            target.target_index = index;
            req.targets.push(target);
        }
        req.actions = vec![ordinary(0), ordinary(214_249)];
        hold(&mut req, 1);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let branch = hits(
            &report,
            crate::characters::blackbird::branches::ROOSTCALL_PACKET,
        );
        assert_eq!(branch.len(), targets);
        for hit in branch {
            assert_eq!(hit["time_us"], 614_725);
            assert_eq!(hit["non_crit_damage"], json!("1139/2"));
        }
        assert_eq!(malice(&report)["current"], json!("6245249/250000"));
        req.actions[1] = ordinary(214_248);
        assert_eq!(run(&req)["execution_completed"], false);
    }
}

#[test]
fn blackbird_evolved_branches_replace_the_montage_and_keep_captured_extras() {
    let mut req = request();
    req.actions = vec![
        burst(0),
        evolve_melee(6_000_000),
        evolve_melee(7_100_000),
        evolve_melee(7_233_625),
        evolve_melee(8_129_654),
    ];
    hold(&mut req, 3);
    hold(&mut req, 4);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    let best = hits(
        &report,
        crate::characters::blackbird::branches::BESTOWED_GRACE_PACKET,
    );
    assert_eq!(best.len(), 1);
    assert_eq!(best[0]["time_us"], 8_051_337);
    let tidings = hits(
        &report,
        crate::characters::blackbird::branches::TIDINGS_PACKET,
    );
    assert_eq!(tidings.len(), 5);
    assert_eq!(tidings[0]["time_us"], 8_473_499);
    let extras = hits(&report, "GE_Player_BlackBird_Evolve_Melee2_Extra_Damage");
    assert_eq!(extras.len(), 1);
    assert_eq!(extras[0]["time_us"], 7_260_687);
    assert_eq!(malice(&report)["current"], json!("14460769/1000000"));
    assert_eq!(malice(&report)["conserved"], true);
}

#[test]
fn blackbird_hold_rejects_wrong_stage_and_missing_model_without_mutation() {
    let mut req = request();
    req.actions = vec![ordinary(0)];
    hold(&mut req, 0);
    assert_eq!(run(&req)["execution_completed"], false);
    req.actions = vec![ordinary(0), ordinary(214_249)];
    req.hold_inputs.clear();
    hold(&mut req, 1);
    req.model_assumptions
        .remove(crate::HypothesisId::BlackbirdBranchesV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("branches_v1")
    );
}

#[test]
fn blackbird_explicit_parry_caps_malice_and_in_form_grants_only_skill_permission() {
    let mut req = request();
    req.parry_events = [0, 1, 2]
        .map(|timestamp_us| GameActorParryEvent::At {
            timestamp_us,
            actor_index: 0,
        })
        .to_vec();
    let report = run(&req);
    assert_eq!(malice(&report)["current"], json!("60/1"));
    assert_eq!(report["parry_receipts"][2]["malice_granted"], json!("0/1"));
    assert_eq!(report["hits_settled"], 0);
    req.actions = vec![burst(0)];
    req.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 6_000_000,
        actor_index: 0,
    }];
    let report = run(&req);
    assert_eq!(malice(&report)["current"], json!("20/1"));
    assert_eq!(
        report["parry_receipts"][0]["skill_permission_until_us"],
        36_000_000
    );
    assert_eq!(report["parry_receipts"][0]["malice_granted"], json!("0/1"));
}

#[test]
fn blackbird_combat_leave_is_one_callback_and_reentry_cancels_old_generations() {
    let mut req = request();
    req.combat_transition_events = [
        (0, true),
        (3_000_000, false),
        (6_000_000, true),
        (12_000_000, false),
    ]
    .map(|(timestamp_us, leaving)| GameActorCombatTransitionEvent {
        timestamp_us,
        actor_index: 0,
        leaving,
    })
    .to_vec();
    let report = run(&req);
    // Initial20 + entry20 + the second leave's20; last entry overflows20.
    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "blackbird_leave_malice")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(grants, [10_000_000]);
    assert_eq!(malice(&report)["current"], json!("60/1"));
    assert_eq!(malice(&report)["granted"], json!("80/1"));
    assert_eq!(
        report["combat_transition_receipts"][1]["malice_granted"],
        json!("20/1")
    );
    assert_eq!(
        report["combat_transition_receipts"][3]["malice_granted"],
        json!("0/1")
    );
    assert_eq!(malice(&report)["conserved"], true);
    req.actions = vec![burst(0)];
    req.combat_transition_events = vec![
        GameActorCombatTransitionEvent {
            timestamp_us: 6_000_000,
            actor_index: 0,
            leaving: true,
        },
        GameActorCombatTransitionEvent {
            timestamp_us: 12_000_000,
            actor_index: 0,
            leaving: false,
        },
    ];
    assert_eq!(malice(&run(&req))["current"], json!("20/1"));
}

#[test]
fn blackbird_event_validation_rejects_invalid_owner_order_horizon_and_budget() {
    let mut req = request();
    req.combat_transition_events = vec![GameActorCombatTransitionEvent {
        timestamp_us: 0,
        actor_index: 0,
        leaving: false,
    }];
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("alternate")
    );
    req.combat_transition_events.clear();
    req.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 0,
        actor_index: 4,
    }];
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("bounds")
    );
    req.parry_events[0] = GameActorParryEvent::At {
        timestamp_us: req.absolute_limit_us + 1,
        actor_index: 0,
    };
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("horizon")
    );
    req.parry_events = vec![
        GameActorParryEvent::At {
            timestamp_us: 0,
            actor_index: 0
        };
        2
    ];
    let limits = ExecutionLimits {
        max_events: 1,
        ..ExecutionLimits::default()
    };
    assert!(
        run_game_scenario(&req, &limits)
            .unwrap_err()
            .contains("event count")
    );
}

#[test]
fn blackbird_spawned_melee5_survives_jump_even_when_all_contacts_miss() {
    let mut req = request();
    req.targets[0].resistance = Ratio::one();
    req.actions = vec![
        burst(0),
        evolve_melee(6_000_000),
        evolve_melee(7_100_000),
        evolve_melee(8_100_000),
        evolve_melee(10_100_000),
        evolve_melee(11_300_000),
        jump(11_824_134),
    ];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let packets = hits(&report, "GE_Player_BlackBird_Evolve_Melee5_Damage");
    assert_eq!(packets.len(), 6);
    assert_eq!(packets[5]["time_us"], 12_237_118);
    for hit in packets {
        assert_eq!(hit["applied_hp_loss"], 0);
    }
    assert_eq!(malice(&report)["spent"], json!("0/1"));
}

#[test]
fn blackbird_skill_to_tidings_consumes_permission_and_uses_shared_healing_once() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    req.actors[0].static_stats.hp = 10_000;
    req.actors[0].initial_hp_max = 10_000;
    req.actors[0].initial_hp = 1_000;
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    let mut skill = evolve_melee(6_000_000);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        ..
    } = &mut skill
    {
        *ability_id = "GA_BlackBird_Evolve_Skill".into();
        *damage_effect_id = "GE_Player_BlackBird_Evolve_Skill_Damage".into();
    }
    req.actions = vec![burst(0), skill, evolve_melee(6_835_975)];
    hold(&mut req, 2);
    req.parry_events = vec![GameActorParryEvent::At {
        timestamp_us: 5_900_000,
        actor_index: 0,
    }];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 7_000);
    let heals = report["autonomous_events"].as_array().unwrap();
    assert_eq!(heals.len(), 1);
    assert_eq!(
        heals[0]["trigger"],
        json!({"kind":"action","command_index":1})
    );
    assert_eq!(heals[0]["recipient_healing"]["gross"], 6_000);
    assert_eq!(
        hits(
            &report,
            crate::characters::blackbird::branches::TIDINGS_PACKET
        )
        .len(),
        5
    );
}
