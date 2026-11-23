use super::hathor_burst::{burst, skill};
use super::*;
use serde_json::{Value, json};
fn request() -> GameScenarioRequest {
    let mut r = super::hathor_burst::request(1);
    let m = crate::HypothesisId::HathorCombatLifecycleV1;
    r.model_assumptions.insert(m.into(), m.description().into());
    r
}
fn transitions(r: &mut GameScenarioRequest, exit: u64, enter: u64) {
    r.combat_transition_events = vec![
        GameActorCombatTransitionEvent {
            timestamp_us: exit,
            actor_index: 0,
            leaving: true,
        },
        GameActorCombatTransitionEvent {
            timestamp_us: enter,
            actor_index: 0,
            leaving: false,
        },
    ];
}
fn run(r: &GameScenarioRequest) -> Value {
    super::edgar_full::run(r)
}
fn power(r: &Value) -> &Value {
    &r["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"]
}
#[test]
fn hathor_combat_resets_battle_markers_but_retains_conserved_resources_and_world_cooldowns() {
    let mut r = request();
    r.relative_duration_us = 35_000_000;
    r.actors[0].initial_energy = 0;
    r.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![1, 5]).unwrap();
    r.actions = vec![skill(0), skill(20_000_000)];
    for i in [0, 1] {
        r.hold_inputs.insert(
            i,
            GameHoldInput {
                duration_us: 5_000_000,
                moving: false,
                reserved_stocks: None,
            },
        );
    }
    transitions(&mut r, 10_000_000, 15_000_000);
    let report = run(&r);
    assert_eq!(report["hits_settled"], 54);
    assert_eq!(power(&report)["granted"], json!("12/1"));
    assert_eq!(power(&report)["spent"], json!("6/1"));
    assert_eq!(power(&report)["current"], json!("6/1"));
    assert_eq!(power(&report)["overflow"], json!("0/1"));
    // Both births grant6 but A5 resets at exit: second hold stays+.06, never+.12.
    let first = &report["hit_receipts"][0];
    let second = &report["hit_receipts"][27];
    assert_eq!(first["non_crit_damage"], second["non_crit_damage"]);
    assert_eq!(first["non_crit_damage"], json!("50509/100"));
    assert_eq!(report["final_actor_states"][0]["energy"], 100);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 27);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("169/250")
    );
    let mut early = r.clone();
    early.actions[1] = skill(16_236_651);
    assert_eq!(
        run_game_scenario(&early, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]
            ["until_us"],
        16_236_652
    );
}
#[test]
fn hathor_combat_reentry_rearms_gaze_without_accepting_stale_old_generation_or_lost_proximity() {
    let mut r = request();
    r.relative_duration_us = 23_000_000;
    r.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![2]).unwrap();
    transitions(&mut r, 2_000_000, 3_000_000);
    r.combat_transition_events.extend([
        GameActorCombatTransitionEvent {
            timestamp_us: 10_000_000,
            actor_index: 0,
            leaving: true,
        },
        GameActorCombatTransitionEvent {
            timestamp_us: 12_000_000,
            actor_index: 0,
            leaving: false,
        },
    ]);
    r.dummy_combat_events = vec![
        GameDummyCombatEvent::AttackStart {
            timestamp_us: 6_000_000,
            target_index: 0,
            actor_index: 0,
        },
        GameDummyCombatEvent::AttackStart {
            timestamp_us: 9_000_000,
            target_index: 0,
            actor_index: 0,
        },
        GameDummyCombatEvent::Proximity {
            timestamp_us: 11_000_000,
            target_index: 0,
            in_proximity: false,
        },
        GameDummyCombatEvent::Proximity {
            timestamp_us: 15_000_000,
            target_index: 0,
            in_proximity: true,
        },
        GameDummyCombatEvent::AttackStart {
            timestamp_us: 21_000_000,
            target_index: 0,
            actor_index: 0,
        },
    ];
    let report = run(&r);
    let applied: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_gaze" && e["data"]["action"] == "applied")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(applied, [8_000_000, 20_000_000]);
    assert_eq!(power(&report)["granted"], json!("2/1"));
    assert_eq!(power(&report)["spent"], json!("1/1"));
    assert_eq!(power(&report)["current"], json!("1/1"));
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["time_us"] == 21_000_000
                && e["event_kind"] == "hathor_gaze_deadline"
                && e["data"]["expires_at_us"] == 26_000_000)
    );
}
#[test]
fn hathor_combat_exit_cancels_unissued_skill_and_outside_admission_preserves_started_cooldown() {
    let mut r = request();
    r.actions = vec![skill(0), skill(3_100_000)];
    transitions(&mut r, 100_000, 1_000_000);
    let report = run(&r);
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(power(&report)["granted"], json!("2/1"));
    r.actions[1] = skill(3_099_999);
    assert_eq!(
        run_game_scenario(&r, &ExecutionLimits::default()).unwrap()["rejected_action"]["reason"]["until_us"],
        3_100_000
    );
    r.actions[1] = skill(500_000);
    assert!(
        run_game_scenario(&r, &ExecutionLimits::default())
            .unwrap_err()
            .contains("outside combat")
    );
}
#[test]
fn hathor_combat_exit_clears_emergency_and_does_not_refill_spent_burst_energy() {
    let mut r = request();
    r.actions = vec![
        skill(0),
        burst(2_000_000),
        super::hathor_clock::normal(8_000_001),
    ];
    transitions(&mut r, 7_000_000, 8_000_000);
    let report = run(&r);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(power(&report)["current"], json!("0/1"));
    assert_eq!(
        report["hit_receipts"].as_array().unwrap().last().unwrap()["non_crit_damage"],
        json!("208/1")
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 1);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("359/500")
    );
}
#[test]
fn hathor_combat_exit_cancels_pending_cyclone_atk_and_its_unissued_contacts() {
    let mut r = request();
    r.relative_duration_us = 75_000_000;
    let m = crate::HypothesisId::HathorCycloneV1;
    r.model_assumptions.insert(m.into(), m.description().into());
    r.actions = vec![
        skill(0),
        skill(20_000_000),
        skill(40_000_000),
        burst(50_000_000),
        skill(59_499_990),
        super::hathor_clock::normal(60_000_001),
    ];
    transitions(&mut r, 59_800_000, 60_000_000);
    let report = run(&r);
    assert_eq!(report["hits_settled"], 13);
    assert_eq!(power(&report)["spent"], json!("6/1"));
    assert_eq!(power(&report)["current"], json!("0/1"));
    assert_eq!(
        report["hit_receipts"].as_array().unwrap().last().unwrap()["non_crit_damage"],
        json!("208/1")
    );
    let end: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "hathor_emergency_end")
        .map(|e| e["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(end, [59_800_000]);
}
#[test]
fn hathor_combat_exit_precedes_later_equal_time_own_kill_reward_and_rejects_its_power() {
    let mut r = request();
    r.relative_duration_us = 1_000_000;
    r.actors[0].initial_energy = 0;
    r.actions = vec![super::hathor_clock::normal(0)];
    r.targets[0].hp_current = 1;
    r.targets[0].hp_max = 1;
    r.combat_transition_events = vec![GameActorCombatTransitionEvent {
        timestamp_us: 216_746,
        actor_index: 0,
        leaving: true,
    }];
    let report = run(&r);
    assert_eq!(report["hits_settled"], 1);
    assert_eq!(power(&report)["granted"], json!("0/1"));
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "hathor_power_rejected"
                && e["data"]["reason"] == "outside_combat")
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 1);
}
#[test]
fn hathor_combat_restart_cannot_admit_an_old_battle_kill_callback() {
    let mut r = request();
    r.relative_duration_us = 1_000_000;
    r.actors[0].initial_energy = 0;
    r.actions = vec![super::hathor_clock::normal(0)];
    r.targets[0].hp_current = 1;
    r.targets[0].hp_max = 1;
    transitions(&mut r, 216_746, 216_746);
    let report = run(&r);
    assert_eq!(power(&report)["granted"], json!("0/1"));
    assert_eq!(report["final_actor_states"][0]["energy"], 1);
}
