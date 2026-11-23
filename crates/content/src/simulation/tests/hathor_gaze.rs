use super::*;
use crate::HypothesisId;
use serde_json::{Value, json};

fn request(targets: usize, horizon: u64) -> GameScenarioRequest {
    let mut req = super::hathor_clock::request(targets);
    req.relative_duration_us = horizon;
    req.actions.clear();
    for model in [HypothesisId::HathorTapPowerV1, HypothesisId::HathorGazeV1] {
        req.model_assumptions
            .insert(model.into(), model.description().into());
    }
    req
}
fn attack(time: u64, target: usize, actor: usize) -> GameDummyCombatEvent {
    GameDummyCombatEvent::AttackStart {
        timestamp_us: time,
        target_index: target,
        actor_index: actor,
    }
}
fn proximity(time: u64, target: usize, in_proximity: bool) -> GameDummyCombatEvent {
    GameDummyCombatEvent::Proximity {
        timestamp_us: time,
        target_index: target,
        in_proximity,
    }
}
fn run(req: &GameScenarioRequest) -> Value {
    super::edgar_full::run(req)
}
fn balance(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]["current"]
}
fn gaze_times(report: &Value) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_gaze")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn hathor_gaze_attack_start_consumes_once_without_incoming_damage_and_rearms_fresh() {
    let mut req = request(1, 20_000_000);
    req.actors[0].awakening = serde_json::from_value(json!([2, 5])).unwrap();
    req.dummy_combat_events = vec![
        attack(4_999_999, 0, 0),
        attack(9_900_000, 0, 0),
        attack(9_900_001, 0, 0),
        attack(14_899_999, 0, 0),
    ];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 0);
    assert!(r["incoming_damage_receipts"].is_null());
    // Consumed at9.9; the old10s expiry cannot pay A2, reapply or refresh.
    assert_eq!(
        gaze_times(&r),
        [5_000_000, 9_900_000, 14_900_000, 19_900_000]
    );
    assert_eq!(balance(&r), &json!("2/1"));
    assert_eq!(r["final_actor_states"][0]["energy"], 10);
    assert_eq!(
        r["final_actor_states"][0]["resources"]["hathor.a5_earned"]["current"],
        2
    );
}

#[test]
fn hathor_gaze_expiry_precedes_same_time_attack_and_only_a2_pays_natural_completion() {
    for a2 in [false, true] {
        let mut req = request(1, 20_000_000);
        if a2 {
            req.actors[0].awakening = serde_json::from_value(json!([2])).unwrap();
        }
        req.dummy_combat_events = vec![attack(10_000_000, 0, 0), attack(20_000_000, 0, 0)];
        let r = run(&req);
        assert_eq!(
            gaze_times(&r),
            [5_000_000, 10_000_000, 15_000_000, 20_000_000]
        );
        assert_eq!(
            balance(&r),
            &json!(format!("{}/{}", if a2 { 2 } else { 0 }, 1))
        );
    }
}

#[test]
fn hathor_gaze_proximity_changes_reset_accumulation_but_do_not_dispel_a_mark() {
    let mut req = request(1, 23_000_000);
    req.dummy_combat_events = vec![
        proximity(5_000_000, 0, false), // Beats the due accumulation event.
        proximity(6_000_000, 0, true),
        proximity(7_000_000, 0, true), // Repeated true does not postpone it.
        proximity(12_000_000, 0, false),
        attack(13_000_000, 0, 0), // Existing mark survives leaving proximity.
        proximity(17_000_000, 0, true),
    ];
    let r = run(&req);
    assert_eq!(gaze_times(&r), [11_000_000, 13_000_000, 22_000_000]);
    assert_eq!(balance(&r), &json!("1/1"));
}

#[test]
fn hathor_gaze_tracks_each_dummy_and_accepts_attack_on_a_living_off_field_teammate() {
    let mut req = request(3, 12_000_000);
    let mut peer = req.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1020".into();
    req.actors.push(peer);
    req.actions = vec![GameActionCommand::SwitchActor {
        timestamp_us: 0,
        target_actor_index: 1,
        cooldown_us: 0,
    }];
    req.dummy_combat_events = vec![attack(6_000_000, 2, 0), attack(6_000_000, 0, 1)];
    let r = run(&req);
    assert_eq!(balance(&r), &json!("2/1"));
    assert_eq!(r["final_actor_states"][0]["energy"], 10);
    assert_eq!(r["final_actor_states"][1]["energy"], 0);
    assert_eq!(r["final_buff_state"]["active_actor_index"], 1);
    assert_eq!(
        gaze_times(&r),
        [
            5_000_000, 5_000_000, 5_000_000, 6_000_000, 6_000_000, 10_000_000, 11_000_000,
            11_000_000
        ]
    );
}

#[test]
fn hathor_gaze_source_death_clears_marks_without_a2_award_and_dead_recipient_cannot_consume() {
    let mut req = request(1, 15_000_000);
    req.actors[0].awakening = serde_json::from_value(json!([2])).unwrap();
    req.model_assumptions.insert(
        HypothesisId::CombatIncomingDamageFixtureV1.into(),
        HypothesisId::CombatIncomingDamageFixtureV1
            .description()
            .into(),
    );
    req.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 6_000_000,
        actor_index: 0,
        delivered_damage: 100_000,
    }];
    req.dummy_combat_events = vec![attack(6_000_000, 0, 0)];
    let r = run(&req);
    assert_eq!(balance(&r), &json!("0/1"));
    assert_eq!(gaze_times(&r), [5_000_000]);
    // Kill only the other recipient; surviving Hathor retains her original mark.
    let mut peer = req.actors[0].clone();
    peer.slot = 1;
    peer.character_id = "character:1020".into();
    req.actors.push(peer);
    req.incoming_damage_events[0].actor_index = 1;
    req.dummy_combat_events = vec![attack(6_000_000, 0, 1)];
    req.relative_duration_us = 10_000_000;
    let r = run(&req);
    assert_eq!(gaze_times(&r), [5_000_000, 10_000_000]);
    assert_eq!(balance(&r), &json!("1/1"));
}

#[test]
fn hathor_gaze_target_death_does_not_manufacture_natural_completion() {
    let mut req = request(1, 15_000_000);
    req.actors[0].awakening = serde_json::from_value(json!([2])).unwrap();
    req.targets[0].hp_current = 1;
    req.targets[0].hp_max = 1;
    req.actions = vec![super::hathor_clock::normal(6_000_000)];
    req.dummy_combat_events = vec![attack(7_000_000, 0, 0)];
    let r = run(&req);
    assert_eq!(gaze_times(&r), [5_000_000]);
    // Only the actual B4 kill grants1; the mark's later expiry and attack pay0.
    assert_eq!(balance(&r), &json!("1/1"));
}

#[test]
fn hathor_gaze_caps_natural_awards_and_schedules_only_real_boundaries() {
    let mut req = request(2, 40_000_000);
    req.actors[0].awakening = serde_json::from_value(json!([2, 5])).unwrap();
    let r = run(&req);
    let pool = &r["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"];
    assert_eq!(pool["current"], json!("6/1"));
    assert_eq!(pool["granted"], json!("8/1"));
    assert_eq!(pool["overflow"], json!("2/1"));
    assert_eq!(r["final_actor_states"][0]["energy"], 30);
    assert_eq!(gaze_times(&r).len(), 16);
    assert_eq!(r["pending_events"], 2); // Future natural boundaries remain queued.
}

#[test]
fn hathor_gaze_dummy_declarations_reject_invalid_and_unsupported_inputs() {
    let req = request(1, 20_000_000);
    for events in [
        vec![attack(req.absolute_limit_us + 1, 0, 0)],
        vec![attack(1, 1, 0)],
        vec![attack(1, 0, 1)],
        vec![attack(2, 0, 0), attack(1, 0, 0)],
    ] {
        let mut bad = req.clone();
        bad.dummy_combat_events = events;
        assert!(validate_game_request(&bad, &ExecutionLimits::default()).is_err());
    }
    let mut bounded = req.clone();
    bounded.dummy_combat_events = vec![attack(1, 0, 0), attack(2, 0, 0)];
    let limits = ExecutionLimits {
        max_events: 1,
        ..Default::default()
    };
    assert!(
        validate_game_request(&bounded, &limits)
            .unwrap_err()
            .contains("event work limit")
    );
    let mut bad = req.clone();
    bad.model_assumptions
        .remove(HypothesisId::HathorTapPowerV1.as_str());
    assert!(
        validate_game_request(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("tap_power_v1")
    );
    let mut bad = req;
    bad.model_assumptions
        .remove(HypothesisId::HathorGazeV1.as_str());
    bad.dummy_combat_events = vec![attack(1, 0, 0)];
    assert!(
        validate_game_request(&bad, &ExecutionLimits::default())
            .unwrap_err()
            .contains("content observer")
    );
}
