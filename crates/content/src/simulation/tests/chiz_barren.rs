use super::*;
use crate::characters::chiz::barren::MODEL;
use serde_json::{Value, json};
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_secondary::request(targets);
    req.model_assumptions
        .insert(MODEL.into(), MODEL.description().into());
    req
}
pub(super) fn input(req: &mut GameScenarioRequest, time: u64, held: bool) {
    req.hold_inputs.insert(
        req.actions.len(),
        GameHoldInput {
            reserved_stocks: None,
            duration_us: u64::from(held),
            moving: false,
        },
    );
    req.actions.push(super::chiz_clock::normal(time));
}
fn run(req: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    report
}
fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
}
#[test]
fn chiz_barren_has_two_hits_per_dummy_but_distinct_channels_and_one_grant_per_branch() {
    for targets in [1, 2, 3] {
        let mut req = request(targets);
        input(&mut req, 0, false);
        let report = run(&req);
        assert_eq!(report["hits_settled"], 2 * targets);
        let times: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| {
                (
                    hit["target_index"].as_u64().unwrap(),
                    hit["time_us"].as_u64().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            times
                .iter()
                .filter(|(target, _)| *target == 0)
                .copied()
                .collect::<Vec<_>>(),
            [(0, 1_249_325), (0, 1_964_553)]
        );
        for target in 1..targets as u64 {
            assert_eq!(
                times
                    .iter()
                    .filter(|(index, _)| *index == target)
                    .copied()
                    .collect::<Vec<_>>(),
                [(target, 1_250_539), (target, 1_964_553)]
            );
        }
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], if targets == 1 { 11 } else { 17 });
        assert_eq!(
            actor["cycle"]["current"],
            if targets == 1 { 16 } else { 24 }
        );
        assert_eq!(
            actor["fractional_resources"]["chiz.grain"]["current"],
            json!("450/1")
        );
        assert_eq!(actor["resources"]["chiz.skill_stocks"]["current"], 3);

        req.relative_duration_us = 8_000_000;
        input(&mut req, 3_000_000, false);
        let continued = run(&req);
        assert_eq!(continued["hits_settled"], 4 * targets);
        let actor = &continued["final_actor_states"][0];
        assert_eq!(actor["energy"], if targets == 1 { 23 } else { 34 });
        assert_eq!(
            actor["charge_remainder"],
            if targets == 1 {
                json!("1/25")
            } else {
                json!("14/25")
            }
        );
        assert_eq!(
            actor["cycle"]["current"],
            if targets == 1 { 32 } else { 48 }
        );
        assert_eq!(
            actor["fractional_resources"]["chiz.grain"]["current"],
            json!("900/1")
        );
    }
}
#[test]
fn chiz_barren_keeps_first_phase_exclusion_after_fatal_lock_hit_and_falls_back_before_capture() {
    for dead in [false, true] {
        let mut req = request(2);
        input(&mut req, 0, false);
        req.targets[0].hp_current = if dead { 0 } else { 1 };
        let report = run(&req);
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), if dead { 2 } else { 3 });
        assert_eq!(hits[0]["target_index"], if dead { 1 } else { 0 });
        assert_eq!(hits[1]["target_index"], 1);
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
            json!("450/1")
        );
    }
}
#[test]
fn chiz_barren_cue_cancel_keeps_tap_limiter_and_held_combo_ignores_numeric_hold_duration() {
    let mut req = request(1);
    input(&mut req, 0, false);
    req.actions.push(jump(500_015));
    assert_eq!(run(&req)["hits_settled"], 0);
    req.actions.push(super::chiz_clock::normal(500_015));
    input(&mut req, 1_090_624, true);
    let report = run(&req);
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(report["hit_receipts"][1]["time_us"], 1_839_934);
    assert_eq!(report["hit_receipts"][2]["time_us"], 2_555_162);
    // The held flag is not a made-up hold-duration fee.
    req.hold_inputs.get_mut(&3).unwrap().duration_us = 900_000;
    assert_eq!(
        run(&req)["total_damage_exact"],
        report["total_damage_exact"]
    );
    req.actions.push(jump(1_958_884));
    input(&mut req, 2_000_000, false);
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 2_500_000);
}
