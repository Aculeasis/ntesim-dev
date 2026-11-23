use super::*;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_pressed::request(targets);
    let model = crate::characters::chiz::cancel::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn jump(time: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    }
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

#[test]
fn chiz_legal_next_skill_inherits_remaining_real_pause_and_cancels_unreached_half_notify() {
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actions = vec![
            super::chiz_pressed::press(0),
            super::chiz_pressed::press(954_108),
            super::chiz_pressed::press(1_661_079),
        ];
        let report = run(&req);
        assert_eq!(report["hits_settled"], 7 * targets);
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["target_index"] == 0)
            .collect();
        let times: Vec<_> = hits
            .iter()
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        // New Half retains42.15ms at rate.1:37.935ms lost progress.
        // The next Full cancels Half's unreached.966861 notify.
        assert_eq!(
            times,
            [
                183_298, 779_558, 1_591_936, 1_913_252, 2_412_335, 2_606_316, 2_770_758
            ]
        );
        assert_eq!(hits[2]["command_index"], 1);
        assert!(hits[3..].iter().all(|hit| hit["command_index"] == 2));
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], 16);
        assert_eq!(actor["charge_remainder"], json!("53/2500"));
        assert_eq!(actor["cycle"]["current"], 22);
        assert_eq!(actor["cycle"]["carry"], json!("149/1000"));
        req.actions.push(super::chiz_pressed::press(3_401_346));
        let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(failed["rejected_action"]["reason"]["until_us"], 3_401_347);

        // A second paid train must rearm cancellation and stock state without
        // resetting the first train's Charge/Cycle or pending refill deadlines.
        req.actions.pop();
        req.relative_duration_us = 20_000_000;
        for offset in [0, 954_108, 1_661_079] {
            req.actions
                .push(super::chiz_pressed::press(9_000_000 + offset));
        }
        let continued = run(&req);
        assert_eq!(continued["hits_settled"], 14 * targets);
        let second: Vec<_> = continued["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["target_index"] == 0 && hit["command_index"].as_u64().unwrap() >= 3)
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            second,
            times
                .iter()
                .map(|time| time + 9_000_000)
                .collect::<Vec<_>>()
        );
        let actor = &continued["final_actor_states"][0];
        assert_eq!(actor["energy"], 32);
        assert_eq!(actor["charge_remainder"], json!("53/1250"));
        assert_eq!(actor["cycle"]["current"], 44);
        assert_eq!(actor["cycle"]["carry"], json!("149/500"));
    }
}

#[test]
fn chiz_jump_at_live_normal_gate_removes_future_packets_and_does_not_rearm_grain_per_pulse() {
    let mut req = request(1);
    req.actions = vec![
        super::chiz_clock::normal(0),
        super::chiz_clock::normal(590_609),
        jump(839_481),
        super::chiz_clock::normal(839_482),
    ];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 3);
    let actor = &report["final_actor_states"][0];
    assert_eq!(
        actor["fractional_resources"]["chiz.grain"]["current"],
        json!("1350/1")
    );
    assert_eq!(actor["energy"], 5);
    assert_eq!(actor["charge_remainder"], json!("427/2500"));
    let mut req = request(1);
    req.actions = vec![super::chiz_clock::normal(0), jump(393_216)];
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 393_217);
}

#[test]
fn chiz_jump_cancels_half_future_hp_and_financial_snapshots_remain_owned_by_actual_inputs() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    req.actions = vec![
        super::chiz_pressed::press(0),
        super::chiz_pressed::press(954_108),
        jump(1_651_977),
        super::chiz_pressed::press(1_651_978),
    ];
    let report = run(&req);
    let halves: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill2_2Damage")
        .collect();
    assert_eq!(halves.len(), 1);
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_base_override"].is_string())
            .count(),
        3
    );
    let next = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["command_index"] == 3)
        .unwrap();
    assert_eq!(
        next["damage_effect_id"],
        "GE_Player_Chiichan073_Skill1_1Damage"
    );
}
