use super::*;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_held::request(targets);
    let model = crate::characters::chiz::pressed::held::finish::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.relative_duration_us = 12_000_000;
    req
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
fn full_hits(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            matches!(
                hit["damage_effect_id"].as_str(),
                Some("GE_Player_Chiichan073_Skill3_Damage" | "GE_Player_Chiichan073_Skill4_Damage")
            )
        })
        .collect()
}

#[test]
fn chiz_reserved_full_continues_into_buffered_full_without_resetting_loans_or_stock() {
    for targets in [1, 3] {
        let mut req = request(targets);
        super::chiz_held::hold(&mut req, 0, 10_000_000);
        req.hold_inputs.get_mut(&0).unwrap().reserved_stocks = Some(3);
        let report = run(&req);
        assert_eq!(report["hits_settled"], 40 * targets);
        let full = full_hits(&report);
        assert_eq!(full.len(), 4 * targets);
        let times: Vec<_> = full
            .iter()
            .filter(|hit| hit["target_index"] == 0)
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(times, [6_298_642, 6_797_725, 6_991_706, 7_156_007]);
        assert!(full.iter().all(|hit| hit["command_index"] == 0));
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], 19);
        assert_eq!(actor["charge_remainder"], json!("503/2500"));
        assert_eq!(actor["cycle"]["current"], 26);
        assert_eq!(actor["cycle"]["carry"], json!("599/1000"));
        assert_eq!(
            actor["fractional_resources"]["chiz.loan_debt"]["current"],
            json!("15000/1")
        );
        req.actions.push(super::chiz_clock::normal(7_786_329));
        let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(failed["rejected_action"]["reason"]["until_us"], 7_786_330);

        // Continue from a reserved Full into a later, actually buffered third
        // input. Reuse owner fixtures while retaining the independent loan
        // balances and both original command identities from native acceptance.
        let mut req = request(targets);
        req.relative_duration_us = 30_000_000;
        for (period, time) in [0, 15_000_000].into_iter().enumerate() {
            super::chiz_clock::chain(&mut req, time);
            let command = req.actions.len();
            super::chiz_held::hold(&mut req, time + 5_000_000, 10_000_000);
            if period == 0 {
                req.hold_inputs.get_mut(&command).unwrap().reserved_stocks = Some(3);
            } else {
                req.actions
                    .push(super::chiz_pressed::press(time + 5_954_108));
            }
        }
        let continued = run(&req);
        assert_eq!(continued["hits_settled"], 75 * targets);
        let actor = &continued["final_actor_states"][0];
        let debt = &actor["fractional_resources"]["chiz.loan_debt"];
        for (field, expected) in [
            ("current", json!("103125/8")),
            ("granted", json!("121125/8")),
            ("spent", json!("2250/1")),
        ] {
            assert_eq!(debt[field], expected, "{field}");
        }
        assert_eq!(debt["conserved"], true);
        assert_eq!(actor["energy"], 77);
        assert_eq!(actor["charge_remainder"], json!("653/1250"));
        assert_eq!(actor["cycle"]["current"], 100);
        assert_eq!(actor["cycle"]["carry"], json!("197/500"));
        assert_eq!(actor["cycle"]["overflow"], 7);
        let full = full_hits(&continued);
        assert_eq!(full.len(), 8 * targets);
        for (index, hit) in full.iter().enumerate() {
            assert_eq!(
                hit["command_index"],
                if index < 4 * targets { 5 } else { 12 }
            );
        }
        let refills: Vec<_> = continued["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "chiz_skill_stock_refill")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            refills,
            [
                8_000_000, 11_000_000, 14_000_000, 23_000_000, 26_000_000, 29_000_000
            ]
        );
        assert_eq!(actor["resources"]["chiz.skill_stocks"]["current"], 3);
    }
}

#[test]
fn chiz_real_third_press_buffers_to_loop_gate_then_preserves_remaining_credit_and_input_origin() {
    for (input, start) in [
        (954_108, 2_581_175),
        (2_000_000, 2_581_175),
        (2_400_000, 2_581_175),
        (2_600_000, 2_600_000),
    ] {
        let mut req = request(1);
        super::chiz_held::hold(&mut req, 0, 10_000_000);
        req.actions.push(super::chiz_pressed::press(input));
        let report = run(&req);
        assert_eq!(report["hits_settled"], 10);
        let full = full_hits(&report);
        assert_eq!(full.len(), 4);
        assert_eq!(full[0]["time_us"], start + 252_173);
        assert!(full.iter().all(|hit| hit["command_index"] == 1));
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chiz.loan_debt"]["current"],
            json!("1875/2")
        );
        assert!(
            !report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == "chiz_held_terminal_spend")
        );
    }
    let mut req = request(1);
    super::chiz_held::hold(&mut req, 0, 10_000_000);
    req.actions.push(super::chiz_pressed::press(2_400_000));
    req.actions.push(super::chiz_pressed::press(2_450_000));
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert!(
        failed["rejected_action"]["reason"]["message"]
            .as_str()
            .unwrap()
            .contains("already reserved")
    );
}

#[test]
fn chiz_reserved_full_keeps_same_time_terminal_coin_capture_and_release_cancels_without_refund() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2, 5]).unwrap();
    super::chiz_held::hold(&mut req, 0, 10_000_000);
    req.hold_inputs.get_mut(&0).unwrap().reserved_stocks = Some(3);
    let report = run(&req);
    let terminal = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| {
            hit["time_us"] == 6_046_469
                && hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill2_CoinDamage"
        })
        .unwrap();
    assert_eq!(terminal["damage_base_override"], json!("3654/1"));
    assert_eq!(full_hits(&report).len(), 4);
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill1_CoinDamage")
            .count(),
        1
    );
    let mut req = request(1);
    super::chiz_held::hold(&mut req, 0, 2_550_000);
    req.hold_inputs.get_mut(&0).unwrap().reserved_stocks = Some(3);
    req.relative_duration_us = 2_999_393;
    let report = run(&req);
    assert!(full_hits(&report).is_empty());
    assert_eq!(report["hits_settled"], 7);
    assert_eq!(
        report["final_actor_states"][0]["resources"]["chiz.skill_stocks"]["current"],
        0
    );
}

#[test]
fn chiz_third_input_obeys_actual_pause_gate_and_immune_packets_do_not_refund_financial_cost() {
    let mut req = request(1);
    super::chiz_held::hold(&mut req, 0, 10_000_000);
    req.actions.push(super::chiz_pressed::press(954_107));
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 954_108);
    assert_eq!(
        failed["final_actor_states"][0]["resources"]["chiz.skill_stocks"]["current"],
        1
    );
    req.targets[0].resistance = Ratio::one();
    *req.actions.last_mut().unwrap() = super::chiz_pressed::press(632_043);
    let report = run(&req);
    assert_eq!(full_hits(&report)[0]["time_us"], 2_473_348);
    let actor = &report["final_actor_states"][0];
    assert_eq!(
        actor["fractional_resources"]["chiz.loan_debt"]["current"],
        json!("1875/2")
    );
    assert_eq!(actor["energy"], 0);
    assert!(actor["cycle"].is_null());
}

#[test]
fn chiz_reserved_stocks_validate_owner_choice_and_funded_initial_three_stock_admission() {
    let mut req = request(1);
    req.actions.push(super::chiz_pressed::press(0));
    super::chiz_held::hold(&mut req, 1_279_881, 10_000_000);
    req.hold_inputs.get_mut(&1).unwrap().reserved_stocks = Some(3);
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert!(
        failed["rejected_action"]["reason"]["message"]
            .as_str()
            .unwrap()
            .contains("required3, available2")
    );
    let mut other=crate::dsl::compiler::compile_scenario("options duration=1; blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=1000000 def=1080 resist=0; active blackbird; blackbird attack;",Default::default()).unwrap().request;
    other.hold_inputs.insert(
        0,
        GameHoldInput {
            duration_us: 0,
            moving: false,
            reserved_stocks: Some(2),
        },
    );
    assert!(
        run_game_scenario(&other, &ExecutionLimits::default())
            .unwrap_err()
            .contains("reserved_stocks has no supported owner binding")
    );
}

#[test]
fn chiz_defeat_cancels_reserved_or_buffered_full_damage_without_refunding_stocks_or_principal() {
    for (reserved, death, hits, debt) in [(false, 1_000_000, 2, 0), (true, 6_100_000, 36, 15000)] {
        let mut req = request(1);
        super::chiz_held::hold(&mut req, 0, 10_000_000);
        if reserved {
            req.hold_inputs.get_mut(&0).unwrap().reserved_stocks = Some(3);
        } else {
            req.actions.push(super::chiz_pressed::press(954_108));
        }
        let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.incoming_damage_events
            .push(crate::simulation::GameActorDamageEvent {
                timestamp_us: death,
                actor_index: 0,
                delivered_damage: req.actors[0].initial_hp,
            });
        let report = run(&req);
        assert_eq!(report["hits_settled"], hits);
        assert!(full_hits(&report).is_empty());
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chiz.loan_debt"]["current"],
            json!(format!("{}/{}", debt, 1))
        );
        assert_eq!(
            report["final_actor_states"][0]["resources"]["chiz.skill_stocks"]["current"],
            3
        );
    }
}

#[test]
fn chiz_release_before_loop_pays_committed_segments_without_unrequested_credit_spend() {
    for duration in [1, 1_000_000, 2_313_136] {
        for stocks in [2, 3] {
            let mut req = request(1);
            super::chiz_held::hold(&mut req, 0, duration);
            req.hold_inputs.get_mut(&0).unwrap().reserved_stocks = Some(stocks);
            req.relative_duration_us = 2_329_791;
            let report = run(&req);
            assert_eq!(report["hits_settled"], 4);
            assert!(full_hits(&report).is_empty());
            assert_eq!(
                report["final_actor_states"][0]["fractional_resources"]["chiz.loan_debt"]["current"],
                json!("0/1")
            );
            assert_eq!(
                report["final_actor_states"][0]["resources"]["chiz.skill_stocks"]["current"],
                3 - stocks
            );
            req.actions.push(super::chiz_clock::normal(2_329_790));
            let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(failed["rejected_action"]["reason"]["until_us"], 2_329_791);
        }
    }
    let mut req = request(1);
    super::chiz_held::hold(&mut req, 0, 1_000_000);
    req.actions.push(super::chiz_pressed::press(954_108));
    let report = run(&req);
    assert_eq!(report["hits_settled"], 8);
    assert_eq!(full_hits(&report)[0]["time_us"], 2_833_348);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.loan_debt"]["current"],
        json!("0/1")
    );
}
