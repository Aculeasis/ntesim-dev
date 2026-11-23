use super::*;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_pressed::request(targets);
    let model = crate::characters::chiz::pressed::held::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.relative_duration_us = 30_000_000;
    req
}

pub(super) fn hold(req: &mut GameScenarioRequest, time: u64, duration: u64) {
    let index = req.actions.len();
    req.actions.push(super::chiz_pressed::press(time));
    req.hold_inputs.insert(
        index,
        GameHoldInput {
            reserved_stocks: None,
            duration_us: duration,
            moving: false,
        },
    );
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
fn current(report: &Value, key: &str) -> Value {
    report["final_actor_states"][0]["fractional_resources"][key]["current"].clone()
}

#[test]
fn chiz_two_stock_hold_uses_sixteen_fixed_exact_loan_spends_and_finite_real_clocks() {
    for targets in [1, 3] {
        let mut req = request(targets);
        hold(&mut req, 0, 10_000_000);
        let report = run(&req);
        assert_eq!(report["hits_settled"], 36 * targets);
        assert_eq!(current(&report, "chiz.grain"), json!("0/1"));
        assert_eq!(current(&report, "chiz.loan_debt"), json!("15000/1"));
        let hits = report["hit_receipts"].as_array().unwrap();
        let coins: Vec<_> = hits
            .iter()
            .filter(|hit| hit["damage_base_override"].is_string())
            .collect();
        assert_eq!(coins.len(), 16 * targets);
        for coin in coins {
            //15000/16=937.5, flat base3*floor(937.5)=2811.
            assert_eq!(coin["damage_base_override"], json!("2811/1"));
            assert_eq!(coin["critical"], false);
        }
        let loops: Vec<_> = hits
            .iter()
            .filter(|hit| {
                hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill2_Damage"
                    && hit["target_index"] == 0
            })
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(
            loops,
            [
                2_513_136, 2_746_470, 2_979_803, 3_213_136, 3_446_470, 3_679_803, 3_913_137,
                4_146_470, 4_379_803, 4_613_137, 4_846_469, 5_079_803, 5_313_137, 5_546_469,
                5_779_803, 5_996_620
            ]
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 9);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("24/25")
        );
        assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 13);
        assert_eq!(
            report["final_actor_states"][0]["cycle"]["carry"],
            json!("4/5")
        );
    }
}

#[test]
fn chiz_two_continuing_held_periods_repay_existing_debt_and_never_reset_available_credit() {
    let mut req = request(1);
    for start in [0, 15_000_000] {
        super::chiz_clock::chain(&mut req, start);
        hold(&mut req, start + 5_000_000, 10_000_000);
    }
    let report = run(&req);
    assert_eq!(report["hits_settled"], 97);
    let coins: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill2_CoinDamage")
        .collect();
    assert_eq!(coins.len(), 32);
    for (index, coin) in coins.iter().enumerate() {
        assert_eq!(
            coin["damage_base_override"],
            json!(format!("{}/{}", if index < 16 { 3234 } else { 420 }, 1))
        );
    }
    let actor = &report["final_actor_states"][0];
    assert_eq!(
        actor["fractional_resources"]["chiz.loan_debt"]["granted"],
        json!("17250/1")
    );
    assert_eq!(
        actor["fractional_resources"]["chiz.loan_debt"]["spent"],
        json!("2250/1")
    );
    assert_eq!(current(&report, "chiz.loan_debt"), json!("15000/1"));
    assert_eq!(actor["energy"], 59);
    assert_eq!(actor["charge_remainder"], json!("1/25"));
    assert_eq!(actor["cycle"]["current"], 81);
    assert_eq!(actor["cycle"]["carry"], json!("199/250"));
    assert_eq!(actor["resources"]["chiz.skill_stocks"]["current"], 3);
}

#[test]
fn chiz_early_release_spends_one_terminal_remainder_without_extra_primary_and_pays_exit_recovery() {
    let mut req = request(1);
    hold(&mut req, 0, 2_550_000);
    let report = run(&req);
    assert_eq!(report["hits_settled"], 7);
    let bases: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|hit| hit.get("damage_base_override"))
        .cloned()
        .collect();
    assert_eq!(bases, vec![json!("2811/1"), json!("42186/1")]);
    assert_eq!(current(&report, "chiz.loan_debt"), json!("15000/1"));
    req.actions.push(super::chiz_clock::normal(2_999_392));
    let failed = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 2_999_393);
    *req.actions.last_mut().unwrap() = super::chiz_clock::normal(2_999_393);
    assert_eq!(run(&req)["execution_completed"], true);
}

#[test]
fn chiz_zero_remaining_credit_still_emits_plain_loop_packets_and_empty_terminal_has_no_bonus() {
    let mut req = request(1);
    hold(&mut req, 0, 10_000_000);
    hold(&mut req, 10_000_000, 10_000_000);
    let report = run(&req);
    assert_eq!(report["hits_settled"], 56);
    assert_eq!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill2_CoinDamage")
            .count(),
        16
    );
    assert_eq!(current(&report, "chiz.loan_debt"), json!("15000/1"));
}

#[test]
fn chiz_two_stock_reservation_rejects_insufficient_stock_before_any_new_spend() {
    let mut req = request(1);
    req.actions = vec![
        super::chiz_pressed::press(0),
        super::chiz_pressed::press(1_279_881),
    ];
    hold(&mut req, 2_330_365, 10_000_000);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["command_index"], 2);
    assert!(
        report["rejected_action"]["reason"]["message"]
            .as_str()
            .unwrap()
            .contains("required2, available1")
    );
    assert_eq!(
        report["final_actor_states"][0]["resources"]["chiz.skill_stocks"]["current"],
        1
    );
    assert_eq!(current(&report, "chiz.loan_debt"), json!("0/1"));
}

#[test]
fn chiz_defeat_cancels_future_loop_costs_without_refunding_already_drawn_principal() {
    for (death, loan, hits) in [(2_513_135, 0, 4), (2_513_137, 1875, 6)] {
        let mut req = request(1);
        hold(&mut req, 0, 10_000_000);
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
        assert_eq!(
            current(&report, "chiz.loan_debt"),
            if loan == 0 {
                json!("0/1")
            } else {
                json!("1875/2")
            }
        );
        assert_eq!(
            report["final_actor_states"][0]["resources"]["chiz.skill_stocks"]["current"],
            3
        );
    }
}

#[test]
fn chiz_fractional_a1_credit_and_coin_only_a6_use_actual_loop_and_debt_consumers() {
    let mut req = request(1);
    req.actors[0].character_level = 3;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(20);
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].purchased_skill_level = 1;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1]).unwrap();
    hold(&mut req, 0, 10_000_000);
    let report = run(&req);
    assert_eq!(current(&report, "chiz.loan_debt"), json!("1155/2"));
    for hit in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_base_override"].is_string())
    {
        assert_eq!(hit["damage_base_override"], json!("108/1"));
    }
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    hold(&mut req, 0, 10_000_000);
    let report = run(&req);
    for hit in report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_base_override"].is_string())
    {
        assert_eq!(hit["non_crit_damage"], json!("64653/40"));
    }
}

#[test]
fn chiz_actual_normal_acquisition_adds_a4_r3_before_repayment_and_does_not_pay_interest_in_debt() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2, 4, 6]).unwrap();
    hold(&mut req, 0, 10_000_000);
    super::chiz_clock::chain(&mut req, 7_000_000);
    let report = run(&req);
    // Five450*(1+.2+.5)=3825 grants repay real drawn principal.
    assert_eq!(current(&report, "chiz.loan_debt"), json!("11175/1"));
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.loan_debt"]["spent"],
        json!("3825/1")
    );
    assert_eq!(current(&report, "chiz.grain"), json!("0/1"));
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "chiz_surplus_interest")
    );
}
