use super::reactions::{request, switch};
use super::*;

#[test]
fn nova_damage_and_removal_have_separate_deadlines_and_keep_the_tag_between_them() {
    for (horizon, ends) in [(5_050_000, 0), (5_100_000, 1)] {
        let mut req = request(&[(0, "character:1004", 100, 0), (1, "character:1020", 0, 0)]);
        req.relative_duration_us = horizon;
        req.actions = vec![switch(0, 1)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        // The earlier observation deliberately stops before its pending end.
        assert_eq!(report["execution_completed"], true);
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["time_us"], 5_000_000);
        let end: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "reaction_nova_ended")
            .collect();
        assert_eq!(end.len(), ends);
        if ends == 1 {
            assert_eq!(end[0]["time_us"], 5_100_000);
        }
        let statuses = report["final_target_states"][0]["statuses"].as_array();
        assert_eq!(
            statuses.map_or(0, |statuses| statuses
                .iter()
                .filter(|s| s["id"] == "ntesim.reaction.nova")
                .count()),
            1 - ends
        );
    }
}

#[test]
fn nova_refresh_after_fired_damage_still_ends_once_without_duplicate_or_stale_cleanup() {
    let mut req = request(&[(0, "character:1004", 100, 0), (1, "character:1020", 100, 0)]);
    req.relative_duration_us = 11_000_000;
    req.actions = vec![switch(0, 1), switch(5_050_000, 0)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.nova")
        .collect();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0]["time_us"], 5_000_000);
    assert_eq!(hits[1]["time_us"], 10_050_000);
    let ends: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "reaction_nova_ended")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(ends, [5_050_000, 10_150_000]);
}

#[test]
fn nova_removal_listeners_follow_settled_damage_and_dead_targets_have_no_end_damage() {
    let mut req = request(&[(0, "character:1070", 100, 0), (1, "character:1004", 100, 0)]);
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 7_000_000;
    req.actions = vec![switch(0, 1)];
    let model = crate::HypothesisId::AureliaHarmonicsB2V1;
    req.model_assumptions
        .insert(model.as_str().into(), model.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let times: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Reaction_4_new_1070_Damage")
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(times, [5_400_000, 5_700_000, 6_000_000]);
    let origin = &report["autonomous_events"][0];
    assert_eq!(
        origin["trigger"],
        serde_json::json!({"kind": "action", "command_index": 0})
    );
    assert_eq!(origin["actor_index"], 0);
    assert_eq!(origin["ability_id"], "GA_Mitsuki070_Passive_1");
    req.targets[0].hp_current = 1;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 1);
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "reaction_nova_ended")
    );
}
