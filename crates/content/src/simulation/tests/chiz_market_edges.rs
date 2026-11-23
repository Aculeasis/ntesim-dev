use super::chiz_market::MARKET_PAUSE_US;
use super::chiz_market::{burst, request};
use super::chiz_pressed::press;
use super::*;
use serde_json::{Value, json};
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
fn coins(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill1_CoinDamage")
        .collect()
}
#[test]
fn chiz_market_negative_return_captures_input_price_and_residual_debt_bounds_next_hold_credit() {
    let mut req = request(1);
    req.actions = vec![burst(0), press(4_000_000 + MARKET_PAUSE_US)];
    let report = run(&req);
    assert_eq!(coins(&report)[0]["damage_base_override"], json!("13800/1"));
    assert_eq!(current(&report, "chiz.loan_debt"), json!("1200/1"));
    super::chiz_held::hold(&mut req, 12_000_000, 10_000_000);
    let report = run(&req);
    assert_eq!(current(&report, "chiz.loan_debt"), json!("15000/1"));
    let loop_coins: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill2_CoinDamage")
        .collect();
    assert_eq!(loop_coins.len(), 16);
    for coin in loop_coins {
        assert_eq!(coin["damage_base_override"], json!("2586/1"));
    } //3*floor(13800/16)
}
#[test]
fn chiz_market_expiry_closes_inputs_but_completes_reserved_settlement_before_repaying() {
    let mut req = request(1);
    req.actions = vec![burst(0), press(10_300_000 + MARKET_PAUSE_US)];
    let report = run(&req);
    assert_eq!(current(&report, "chiz.grain"), json!("3450/1"));
    assert_eq!(coins(&report)[0]["damage_base_override"], json!("18450/1"));
    assert_eq!(
        support::hit_metadata(&report, coins(&report)[0])["packet"]["fixed_crit_rate"],
        json!("1/1")
    );
    let exits: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "chiz_market_exit")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(exits, [10_500_000 + MARKET_PAUSE_US]);
    req.relative_duration_us = 10_450_000 + MARKET_PAUSE_US - 4_250_937;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert!(report["halt_diagnostic"].is_null());
    assert_eq!(current(&report, "chiz.grain"), json!("0/1"));
    assert_eq!(current(&report, "chiz.market"), json!("15000/1"));
    assert_eq!(current(&report, "chiz.loan_debt"), json!("15000/1"));
}
#[test]
fn chiz_market_first_critical_token_survives_all_miss_and_is_shared_after_an_immune_first_recipient()
 {
    for all_immune in [false, true] {
        let mut req = request(2);
        req.actions = vec![
            burst(0),
            press(4_000_000 + MARKET_PAUSE_US),
            press(5_500_000 + MARKET_PAUSE_US),
        ];
        req.targets[0].resistance = Ratio::one();
        if all_immune {
            req.targets[1].resistance = Ratio::one();
        }
        let report = run(&req);
        let coins = coins(&report);
        assert_eq!(coins.len(), 4);
        for coin in &coins[..2] {
            assert_eq!(
                support::hit_metadata(&report, coin)["packet"]["fixed_crit_rate"],
                json!("1/1")
            );
        }
        for coin in &coins[2..] {
            assert_eq!(
                support::hit_metadata(&report, coin)["packet"]["fixed_crit_rate"],
                json!(format!("{}/{}", if all_immune { 1 } else { 0 }, 1))
            );
        }
    }
}
#[test]
fn chiz_market_empty_first_settlement_retains_token_and_market_acquisition_is_forced_surplus() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 3, 4]).unwrap();
    super::chiz_held::hold(&mut req, 0, 10_000_000);
    req.actions.push(burst(7_000_000));
    req.actions.push(press(11_500_000 + MARKET_PAUSE_US));
    req.actions
        .push(super::chiz_clock::normal(13_000_000 + MARKET_PAUSE_US));
    req.actions.push(press(14_000_000 + MARKET_PAUSE_US));
    let report = run(&req);
    let coins = coins(&report);
    assert_eq!(coins.len(), 1);
    assert_eq!(coins[0]["damage_base_override"], json!("572/1"));
    assert_eq!(
        support::hit_metadata(&report, coins[0])["packet"]["fixed_crit_rate"],
        json!("1/1")
    );
    assert_eq!(current(&report, "chiz.loan_debt"), json!("79638/5"));
}
#[test]
fn chiz_burst_jump_obeys_actual_recovery_and_does_not_remove_the_separate_switch_lock() {
    let mut req = request(1);
    let model = crate::characters::chiz::cancel::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actions = vec![
        burst(0),
        GameActionCommand::PlayerInput {
            timestamp_us: 1_000_000,
            actor_index: 0,
            input: PlayerInput::Jump,
            critical_success: false,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 3_754_213);
    req.actions[1] = GameActionCommand::PlayerInput {
        timestamp_us: 3_754_213,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    };
    run(&req);
}

#[test]
fn chiz_market_defeat_cancels_unreached_burst_entry_and_unsettled_financial_transactions() {
    for (death, hits) in [(0, 0), (10_400_000 + MARKET_PAUSE_US - 4_250_937, 6)] {
        let mut req = request(1);
        req.actions = vec![burst(0)];
        if death > 0 {
            req.actions.push(press(10_300_000 + MARKET_PAUSE_US));
        }
        let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.incoming_damage_events.push(GameActorDamageEvent {
            timestamp_us: death,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });
        let report = run(&req);
        assert_eq!(report["hits_settled"], hits);
        assert!(coins(&report).is_empty());
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "chiz_market_settlement")
                .count(),
            0
        );
        if death > 0 {
            assert_eq!(current(&report, "chiz.loan_debt"), json!("0/1"));
        }
    }
}

#[test]
fn chiz_market_switch_cancels_unused_chart_deadlines_and_keeps_the_burst_switch_lock() {
    let mut req = request(1);
    req.relative_duration_us = 4_300_000;
    let mut ally=crate::dsl::compiler::compile_scenario("options duration=5; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=1080 resist=0; active mint;",Default::default()).unwrap().request.actors.remove(0);
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        burst(0),
        GameActionCommand::SwitchActor {
            timestamp_us: 4_249_999,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 4_250_000);
    req.actions[1] = GameActionCommand::SwitchActor {
        timestamp_us: 4_250_000,
        target_actor_index: 1,
        cooldown_us: 0,
    };
    let report = run(&req);
    assert_eq!(current(&report, "chiz.loan_debt"), json!("0/1"));
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "chiz_market_price")
            .count(),
        1
    );
}
