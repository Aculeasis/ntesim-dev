use super::*;
use crate::characters::hathor::air::MODEL as AIR;
use crate::characters::hathor::branch::MODEL as BRANCH;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};
fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::hathor_secondary::request(targets);
    for model in [AIR, BRANCH] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req.actions.clear();
    req
}
fn hold(duration_us: u64) -> GameHoldInput {
    GameHoldInput {
        duration_us,
        moving: false,
        reserved_stocks: None,
    }
}
fn branch(can_be_launched: bool, landing_delay_us: u64, height: Ratio) -> GameBranchInput {
    GameBranchInput {
        can_be_launched,
        landing_delay_us,
        height_bonus: height,
    }
}
fn normal(time: u64) -> GameActionCommand {
    super::hathor_clock::normal(time)
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
fn effects(report: &Value) -> Vec<&str> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["damage_effect_id"].as_str().unwrap())
        .collect()
}
fn times(report: &Value) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect()
}
fn trace_times(report: &Value, event_kind: &str) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == event_kind)
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn hathor_branch_launchable_lands_once_after_delivered_stage_contact_and_lock_includes_pause() {
    let mut req = request(1);
    req.actions = vec![normal(0)];
    req.hold_inputs.insert(0, hold(1_500_000));
    req.branch_inputs
        .insert(0, branch(true, 2_000_000, Ratio::zero()));
    let report = run(&req);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(
        effects(&report),
        [
            "GE_Player_Hathor_Melee1_Damage",
            "GE_Player_Hathor_Branch_Damage"
        ]
    );
    assert_eq!(times(&report), [216_746, 2_000_000]);
    assert_eq!(
        report["hit_receipts"][1]["non_crit_damage"],
        json!("4113/2")
    );
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("4529/2")
    );
    let owner = &report["final_actor_states"][0];
    assert_eq!(owner["energy"], 13);
    assert_eq!(owner["charge_remainder"], json!("219/1000"));
    assert_eq!(owner["cycle"]["current"], 22);
    assert_eq!(owner["cycle"]["carry"], json!("7/250"));
    assert_eq!(report["combat_stamina"]["total_spent"], json!("0/1"));
    assert_eq!(trace_times(&report, "hathor_branch_gate"), [1_333_333]);
    assert_eq!(trace_times(&report, "hathor_branch_landing"), [2_000_000]);

    // The delivered BranchEnd curve(0,.3),(.016666,.03),(.1,.03),(.216666,1)
    // loses78583.1us before the737229us recovery; integer deadlines round up.
    req.actions.push(normal(2_815_812));
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["execution_completed"], false);
    assert_eq!(early["rejected_action"]["reason"]["until_us"], 2_815_813);
    *req.actions.last_mut().unwrap() = normal(2_815_813);
    let continued = run(&req);
    assert_eq!(continued["hits_settled"], 3);
    assert_eq!(
        continued["hit_receipts"][2]["damage_effect_id"],
        "GE_Player_Hathor_Melee1_Damage"
    );
}

#[test]
fn hathor_branch_skill_is_rejected_until_the_landing_lock_ends() {
    let mut req = request(1);
    req.actions = vec![normal(0)];
    req.hold_inputs.insert(0, hold(1_500_000));
    req.branch_inputs
        .insert(0, branch(true, 2_000_000, Ratio::zero()));
    req.actions.push(super::hathor_burst::skill(1_500_000));
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 2_000_000);
    // Only the stage contact delivered before admission; the landing was cancelled.
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Hathor_Melee1_Damage"
    );
}

#[test]
fn hathor_branch_gate_cancels_only_contacts_not_yet_delivered_by_stage_two() {
    let mut req = request(1);
    req.actions = vec![normal(0), normal(1_134_310)];
    req.hold_inputs.insert(1, hold(600_000));
    req.branch_inputs
        .insert(1, branch(true, 1_500_000, Ratio::zero()));
    let report = run(&req);
    assert_eq!(report["hits_settled"], 5);
    assert_eq!(
        effects(&report),
        [
            "GE_Player_Hathor_Melee1_Damage",
            "GE_Player_Hathor_Melee2_Damage",
            "GE_Player_Hathor_Melee2_1_Damage",
            "GE_Player_Hathor_Melee2_1_Damage",
            "GE_Player_Hathor_Branch_Damage"
        ]
    );
    assert_eq!(
        times(&report),
        [216_746, 1_334_274, 1_550_838, 1_667_773, 2_634_310]
    );
    assert_eq!(trace_times(&report, "hathor_branch_gate"), [1_700_975]);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("5271/2")
    );
    let owner = &report["final_actor_states"][0];
    assert_eq!(owner["energy"], 15);
    assert_eq!(owner["charge_remainder"], json!("97/250"));
    assert_eq!(owner["cycle"]["current"], 25);
    assert_eq!(owner["cycle"]["carry"], json!("127/200"));
}

#[test]
fn hathor_branch_fallback_costs_use_the_author_authored_landing_and_height() {
    let mut req = request(1);
    req.actions = vec![normal(0)];
    req.hold_inputs.insert(0, hold(1_500_000));
    req.branch_inputs.insert(
        0,
        branch(
            false,
            4_000_000,
            Ratio {
                numerator: 1,
                denominator: 2,
            },
        ),
    );
    let report = run(&req);
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(
        effects(&report),
        [
            "GE_Player_Hathor_Melee1_Damage",
            "GE_Player_Hathor_AirAttack_Damage"
        ]
    );
    assert_eq!(times(&report), [216_746, 4_116_417]);
    assert_eq!(
        report["hit_receipts"][1]["non_crit_damage"],
        json!("3213/2")
    );
    let spends = report["combat_stamina"]["spends"].as_array().unwrap();
    assert_eq!(spends.len(), 4);
    for (spend, (time, after)) in spends.iter().zip([
        (2_470_000, 90),
        (2_970_000, 80),
        (3_470_000, 70),
        (3_970_000, 60),
    ]) {
        assert_eq!(spend["time_us"], time);
        assert_eq!(spend["after"], json!(format!("{}/{}", after, 1)));
    }
    assert_eq!(report["combat_stamina"]["total_spent"], json!("40/1"));
    assert_eq!(report["combat_stamina"]["current"], json!("60/1"));
    let owner = &report["final_actor_states"][0];
    assert_eq!(owner["energy"], 7);
    assert_eq!(owner["charge_remainder"], json!("519/1000"));
    assert_eq!(owner["cycle"]["current"], 12);
    assert_eq!(owner["cycle"]["carry"], json!("107/250"));
    assert_eq!(trace_times(&report, "hathor_branch_fallback"), [1_853_333]);
    assert_eq!(
        trace_times(&report, "hathor_branch_fallback_landing"),
        [4_000_000]
    );
}

#[test]
fn hathor_branch_fallback_authored_landing_wins_over_an_equal_time_strength_cost() {
    for (landing, spent, impact) in [(2_470_000, 0, 2_586_417), (2_470_001, 10, 2_586_418)] {
        let mut req = request(1);
        req.actions = vec![normal(0)];
        req.hold_inputs.insert(0, hold(1_500_000));
        req.branch_inputs
            .insert(0, branch(false, landing, Ratio::zero()));
        let report = run(&req);
        assert_eq!(report["hit_receipts"][1]["time_us"], impact);
        assert_eq!(
            report["combat_stamina"]["total_spent"],
            json!(format!("{}/{}", spent, 1))
        );
    }
}

#[test]
fn hathor_branch_fallback_low_strength_lands_immediately_without_a_ten_point_fee() {
    for initial in [5, 10] {
        let mut req = request(1);
        req.actions = vec![normal(0)];
        req.hold_inputs.insert(0, hold(1_500_000));
        req.branch_inputs
            .insert(0, branch(false, 4_000_000, Ratio::zero()));
        super::blackbird_secondary::stamina(&mut req, initial);
        let report = run(&req);
        assert_eq!(report["hits_settled"], 2);
        assert_eq!(report["hit_receipts"][1]["time_us"], 1_969_750);
        assert_eq!(report["combat_stamina"]["total_spent"], json!("0/1"));
        assert_eq!(
            report["combat_stamina"]["current"],
            json!(format!("{}/{}", initial, 1))
        );
    }
}

#[test]
fn hathor_branch_hold_on_an_ordinary_switch_does_not_start_an_attack() {
    let mut req = request(1);
    let mut old = req.actors[0].clone();
    old.slot = 1;
    old.character_id = "character:1020".into();
    req.actors.push(old);
    req.active_actor_index = 1;
    req.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 0,
        cooldown_us: 0,
    }];
    req.hold_inputs.insert(0, hold(600_000));
    req.branch_inputs
        .insert(0, branch(true, 2_000_000, Ratio::zero()));
    let report = run(&req);
    assert_eq!(report["hits_settled"], 0);
    assert!(trace_times(&report, "hathor_branch_gate").is_empty());
    assert_eq!(report["final_actor_states"][0]["energy"], 0);

    req.hold_inputs.insert(0, hold(500_000));
    assert!(
        validate_game_request(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("strictly beyond")
    );
}

#[test]
fn hathor_branch_validation_requires_its_authored_hold_landing_and_composition() {
    let base = {
        let mut req = request(1);
        req.actions = vec![normal(0)];
        req.hold_inputs.insert(0, hold(1_500_000));
        req.branch_inputs
            .insert(0, branch(true, 2_000_000, Ratio::zero()));
        req
    };
    validate_game_request(&base, &ExecutionLimits::default()).unwrap();
    let assert_error = |req: GameScenarioRequest, text: &str| {
        let error = validate_game_request(&req, &ExecutionLimits::default())
            .unwrap_err()
            .to_string();
        assert!(error.contains(text), "{error}");
    };
    let mut req = base.clone();
    req.model_assumptions.remove(BRANCH.as_str());
    assert_error(req, "requires branch_v1");
    let mut req = base.clone();
    req.model_assumptions.remove(AIR.as_str());
    assert_error(req, "requires ordinary_clock_v1 and airattack_v1");
    // The accepted tap-power model also selects the ordinary clock, so Branch
    // keeps its clock composition when only the clock-row assumption is absent.
    let mut req = base.clone();
    req.model_assumptions.remove("hathor.ordinary_clock_v1");
    validate_game_request(&req, &ExecutionLimits::default()).unwrap();
    let mut req = base.clone();
    req.actions[0] = super::hathor_burst::skill(0);
    assert_error(
        req,
        "must identify a Hathor held Normal or its entry switch",
    );
    let mut req = base.clone();
    req.hold_inputs.clear();
    assert_error(req, "requires the authored hold");
    let mut req = base.clone();
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            moving: true,
            ..hold(1_500_000)
        },
    );
    assert_error(req, "canonical stationary unreserved");
    let mut req = base.clone();
    req.hold_inputs.insert(
        0,
        GameHoldInput {
            reserved_stocks: Some(1),
            ..hold(1_500_000)
        },
    );
    assert_error(req, "hold reserved_stocks has no supported owner binding");
    let mut req = base.clone();
    req.hold_inputs.insert(0, hold(500_000));
    assert_error(req, "strictly beyond");
    let mut req = base.clone();
    req.branch_inputs.insert(0, branch(true, 0, Ratio::zero()));
    assert_error(req, "positive landing time");
    let mut req = base.clone();
    req.branch_inputs.insert(
        0,
        branch(
            true,
            2_000_000,
            Ratio {
                numerator: 1,
                denominator: 0,
            },
        ),
    );
    assert_error(req, "positive landing time");
    let mut req = base.clone();
    req.branch_inputs.insert(
        0,
        branch(
            true,
            2_000_000,
            Ratio {
                numerator: 2,
                denominator: 1,
            },
        ),
    );
    assert_error(req, "positive landing time");
    let mut req = base.clone();
    req.branch_inputs
        .insert(0, branch(false, 520_000, Ratio::zero()));
    assert_error(req, "must follow the0.52s fallback");
    let mut req = base.clone();
    req.plunge_inputs.insert(
        0,
        GamePlungeInput {
            landing_delay_us: 2_000_000,
            height_bonus: Ratio::zero(),
        },
    );
    assert_error(req, "plunge");
}

#[test]
fn hathor_branch_normal_stage_gate_is_checked_against_the_reached_stage() {
    let mut req = request(1);
    req.actions = vec![normal(0)];
    req.hold_inputs.insert(0, hold(1_300_000));
    req.branch_inputs
        .insert(0, branch(true, 2_000_000, Ratio::zero()));
    validate_game_request(&req, &ExecutionLimits::default()).unwrap();
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("does not strictly exceed"), "{error}");
}

#[test]
fn hathor_branch_switch_interrupt_and_death_cancel_unissued_landings() {
    for (switch, interrupt, damage, hits) in [
        (Some(1_500_000), None, None, 1),
        (None, Some(1_000_000), None, 1),
        (None, None, Some(1_100_000), 1),
    ] {
        let mut req = request(1);
        req.actions = vec![normal(0)];
        req.hold_inputs.insert(0, hold(1_500_000));
        req.branch_inputs
            .insert(0, branch(true, 2_000_000, Ratio::zero()));
        if let Some(time) = switch {
            let mut peer = req.actors[0].clone();
            peer.slot = 1;
            peer.character_id = "character:1020".into();
            req.actors.push(peer);
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: time,
                target_actor_index: 1,
                cooldown_us: 0,
            });
        }
        if let Some(time) = interrupt {
            req.interrupt_events.push(GameActorInterruptEvent {
                timestamp_us: time,
                actor_index: 0,
                armor_damage: None,
            });
        }
        if let Some(time) = damage {
            let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
            req.model_assumptions
                .insert(model.into(), model.description().into());
            req.incoming_damage_events.push(GameActorDamageEvent {
                timestamp_us: time,
                actor_index: 0,
                delivered_damage: 100_000,
            });
            req.incoming_dummy_sources
                .insert(req.incoming_damage_events.len() - 1, 0);
        }
        let report = run(&req);
        assert_eq!(report["hits_settled"], hits);
        assert_eq!(trace_times(&report, "hathor_branch_landing").len(), 0);
        assert_eq!(report["final_actor_states"][0]["energy"], 1);
    }
}

#[test]
fn hathor_branch_fallback_interrupt_keeps_only_actual_strength_spends() {
    let mut req = request(1);
    req.actions = vec![normal(0)];
    req.hold_inputs.insert(0, hold(1_500_000));
    req.branch_inputs
        .insert(0, branch(false, 4_000_000, Ratio::zero()));
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 3_000_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("20/1"));
    assert_eq!(report["combat_stamina"]["current"], json!("80/1"));
    assert_eq!(report["final_actor_states"][0]["energy"], 1);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("219/1000")
    );
}

#[test]
fn hathor_branch_multi_target_packets_deduplicate_resources_once() {
    let mut req = request(3);
    req.actions = vec![normal(0)];
    req.hold_inputs.insert(0, hold(1_500_000));
    req.branch_inputs
        .insert(0, branch(true, 2_000_000, Ratio::zero()));
    let report = run(&req);
    assert_eq!(report["hits_settled"], 6);
    for target in 0..3 {
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["target_index"] == target)
            .collect();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0]["time_us"], 216_746);
        assert_eq!(hits[1]["time_us"], 2_000_000);
        assert_eq!(hits[1]["non_crit_damage"], json!("4113/2"));
    }
    let owner = &report["final_actor_states"][0];
    assert_eq!(owner["energy"], 13);
    assert_eq!(owner["cycle"]["current"], 22);
    assert_eq!(owner["cycle"]["carry"], json!("7/250"));
}

#[test]
fn hathor_branch_two_continuing_cycles_keep_stage_and_resources_without_reset() {
    let mut req = request(1);
    req.actions = vec![normal(0), normal(2_815_813)];
    for index in 0..2 {
        req.hold_inputs.insert(index, hold(1_500_000));
        req.branch_inputs
            .insert(index, branch(true, 2_000_000, Ratio::zero()));
    }
    let report = run(&req);
    assert_eq!(report["hits_settled"], 4);
    assert_eq!(
        effects(&report),
        [
            "GE_Player_Hathor_Melee1_Damage",
            "GE_Player_Hathor_Branch_Damage",
            "GE_Player_Hathor_Melee1_Damage",
            "GE_Player_Hathor_Branch_Damage"
        ]
    );
    assert_eq!(times(&report), [216_746, 2_000_000, 3_032_559, 4_815_813]);
    let owner = &report["final_actor_states"][0];
    assert_eq!(owner["energy"], 26);
    assert_eq!(owner["charge_remainder"], json!("219/500"));
    assert_eq!(owner["cycle"]["current"], 44);
    assert_eq!(owner["cycle"]["carry"], json!("7/125"));
    assert_eq!(report["combat_stamina"]["total_spent"], json!("0/1"));
}
