use super::blackbird_secondary::dash;
use super::daffodil_clock::{normal, run};
use super::*;
use serde_json::json;

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::daffodil_secondary::request(targets);
    let model = crate::characters::daffodil::held::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn hold(req: &mut GameScenarioRequest, command: usize) {
    req.hold_inputs.insert(
        command,
        GameHoldInput {
            duration_us: 1,
            moving: false,
            reserved_stocks: None,
        },
    );
}
#[test]
fn daffodil_neutral_held_normal_uses_all_nine_rows_and_one_actor_with_independent_channels() {
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actions = vec![normal(0), normal(1_552_123)];
        hold(&mut req, 0);
        let report = run(&req);
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["command_index"] == 0)
            .collect();
        assert_eq!(hits.len(), 9 * targets);
        let own: Vec<_> = hits.iter().filter(|h| h["target_index"] == 0).collect();
        assert_eq!(
            own.iter()
                .map(|h| h["time_us"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [
                104_054, 400_088, 400_780, 784_106, 833_130, 892_724, 935_062, 1_007_713, 1_208_015
            ]
        );
        let damage: u64 = own
            .iter()
            .map(|h| {
                let r: ExactFraction =
                    serde_json::from_value(h["non_crit_damage"].clone()).unwrap();
                (r.numerator * 1000 / r.denominator) as u64
            })
            .sum();
        assert_eq!(damage, 1_078_000);
        assert_eq!(report["final_actor_states"][0]["energy"], 8);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("839/1000")
        );
        assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 14);
        assert_eq!(
            report["final_actor_states"][0]["cycle"]["carry"],
            json!("837/1000")
        );
        req.actions[1] = normal(1_552_122);
        let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(rejected["execution_completed"], false);
        assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 1_552_123);
    }
}
#[test]
fn daffodil_combo_held_normal_starts_at_jump_gate_and_cancels_unreached_normal_packets() {
    let mut req = request(1);
    req.actions = vec![normal(0), normal(1_003_012), normal(1_164_546)];
    hold(&mut req, 2);
    let report = run(&req);
    assert_eq!(report["hits_settled"], 11);
    assert!(
        !report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Daffodill_Melee2_1_Damage")
    );
    req.actions[2] = normal(1_164_545);
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["execution_completed"], false);
    assert_eq!(rejected["hits_settled"], 2);
}
#[test]
fn daffodil_displaced_edge_preserves_remaining_normal_progress_without_replaying_contacts() {
    let mut req = request(2);
    req.actions = vec![
        normal(0),
        normal(1_003_012),
        dash(1_164_546, false),
        normal(2_732_900),
    ];
    let report = run(&req);
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["target_index"] == 0)
        .collect();
    assert_eq!(hits.len(), 9);
    assert_eq!(
        hits[2]["damage_effect_id"],
        "GE_Player_Daffodill_Melee2_1_Damage"
    );
    assert_eq!(hits[2]["time_us"], 2_363_006);
    assert_eq!(
        hits[3]["damage_effect_id"],
        "GE_Player_Daffodill_Melee3_Damage"
    );
    assert_eq!(hits[3]["time_us"], 2_919_026);
    assert_eq!(
        report["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
    req.actions[2] = dash(1_164_545, false);
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["execution_completed"], false);
    assert_eq!(
        rejected["combat_stamina"]["spends"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn daffodil_displaced_edge_ignores_another_actors_same_time_back_dash() {
    let mut req = request(1);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    req.actors.push(ally);
    let mut other_dash = dash(1_164_546, false);
    if let GameActionCommand::PlayerInput { actor_index, .. } = &mut other_dash {
        *actor_index = 1;
    }
    req.actions = vec![
        normal(0),
        normal(1_003_012),
        dash(1_164_546, false),
        other_dash,
    ];
    req.dash_directions.insert(3, GameDashDirection::Back);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["command_index"], 3);
    assert_eq!(
        report["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
}
#[test]
fn daffodil_cancelling_displaced_edge_drops_future_normal_and_remaining_dodge_time() {
    let mut req = request(1);
    req.actions = vec![
        normal(0),
        normal(1_003_012),
        dash(1_164_546, false),
        GameActionCommand::PlayerInput {
            timestamp_us: 1_247_090,
            actor_index: 0,
            input: PlayerInput::Jump,
            critical_success: false,
        },
        normal(1_247_090),
    ];
    let report = run(&req);
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(
        report["hit_receipts"][2]["damage_effect_id"],
        "GE_Player_Daffodill_Melee1_Damage"
    );
    assert_eq!(report["hit_receipts"][2]["time_us"], 1_432_398);
}

#[test]
fn daffodil_displaced_immunity_ends_at_actual_cancel_while_world_damage_keeps_real_time() {
    let mut req = request(1);
    let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actions = vec![
        normal(0),
        normal(1_003_012),
        dash(1_164_546, false),
        GameActionCommand::PlayerInput {
            timestamp_us: 1_247_090,
            actor_index: 0,
            input: PlayerInput::Jump,
            critical_success: false,
        },
    ];
    req.incoming_damage_events = vec![
        GameActorDamageEvent {
            timestamp_us: 1_200_000,
            actor_index: 0,
            delivered_damage: 23,
        },
        GameActorDamageEvent {
            timestamp_us: 1_247_091,
            actor_index: 0,
            delivered_damage: 17,
        },
    ];
    let report = run(&req);
    assert_eq!(
        report["final_actor_states"][0]["hp"],
        req.actors[0].initial_hp - 17
    );
    assert_eq!(report["hits_settled"], 2);
}
