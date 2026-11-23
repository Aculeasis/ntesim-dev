use super::chiz_market::MARKET_PAUSE_US;
use super::*;
use serde_json::{Value, json};

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::chiz_market::request(targets);
    let model = crate::HypothesisId::ChizHeldFullV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    let mut ally = crate::dsl::compiler::compile_scenario(
        "options duration=20; hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         target hp=100000000 def=1080 resist=0; active hotori;",
        Default::default(),
    )
    .unwrap()
    .request
    .actors
    .remove(0);
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![super::chiz_market::burst(0)];
    super::chiz_held::hold(&mut req, 4_500_000 + MARKET_PAUSE_US, 10_000_000);
    req.hold_inputs.get_mut(&1).unwrap().reserved_stocks = Some(3);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: (5_779_881 + MARKET_PAUSE_US),
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req
}

#[test]
fn chiz_switch_during_loop_repays_only_remaining_market_principal_without_another_credit_line() {
    let mut req = request(1);
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut req.actions[2] {
        *timestamp_us = 8_000_000 + MARKET_PAUSE_US;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 47);
    let pools = &report["final_actor_states"][0]["fractional_resources"];
    // Five1293.75 pulses precede exit: remaining14231.25 repays debt;
    // the eleven later pulses draw exactly that freed capacity, not15000.
    assert_eq!(pools["chiz.market"]["spent"], json!("20700/1"));
    assert_eq!(pools["chiz.loan_debt"]["spent"], json!("56925/4"));
    assert_eq!(pools["chiz.loan_debt"]["granted"], json!("116925/4"));
    assert_eq!(pools["chiz.loan_debt"]["current"], json!("15000/1"));
    assert_eq!(pools["chiz.grain"]["current"], json!("0/1"));
    assert_eq!(pools["chiz.loan_debt"]["conserved"], true);
}

#[test]
fn chiz_complete_invested_hold_never_silently_omits_a_missing_domain_model() {
    for model in [
        crate::HypothesisId::ChizOrdinaryClockV1,
        crate::HypothesisId::ChizExactFinanceV1,
        crate::HypothesisId::ChizPressedSkillV1,
        crate::HypothesisId::ChizHeldSkillV1,
        crate::HypothesisId::ChizHeldFullV1,
        crate::HypothesisId::ChizMarketV1,
    ] {
        let mut req = request(1);
        req.model_assumptions.remove(model.as_str());
        assert!(
            run_game_scenario(&req, &ExecutionLimits::default()).is_err(),
            "{}",
            model.as_str()
        );
    }
}

#[test]
fn chiz_paid_held_afterimage_switch_gate_preserves_market_exit_loans_and_reserved_full() {
    for targets in [1, 3] {
        let mut req = request(targets);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 47 * targets);
        let actor = &report["final_actor_states"][0];
        let pools = &actor["fractional_resources"];
        for (pool, current, granted, spent) in [
            ("chiz.grain", 0, 5700, 5700),
            ("chiz.loan_debt", 15000, 30000, 15000),
        ] {
            for (key, amount) in [("current", current), ("granted", granted), ("spent", spent)] {
                assert_eq!(pools[pool][key], json!(format!("{}/{}", amount, 1)));
            }
            assert_eq!(pools[pool]["conserved"], true);
        }
        let hits = report["hit_receipts"].as_array().unwrap();
        let coins: Vec<&Value> = hits
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill2_CoinDamage")
            .collect();
        assert_eq!(coins.len(), 16 * targets);
        assert!(coins.iter().all(|hit| hit["time_us"].as_u64().unwrap()
            > (5_779_881 + MARKET_PAUSE_US)
            && hit["damage_base_override"] == json!("3879/1")));
        let full = hits
            .iter()
            .filter(|hit| {
                matches!(
                    hit["damage_effect_id"].as_str(),
                    Some(
                        "GE_Player_Chiichan073_Skill3_Damage"
                            | "GE_Player_Chiichan073_Skill4_Damage"
                    )
                )
            })
            .count();
        assert_eq!(full, 4 * targets);
        let exits: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "chiz_market_exit")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(exits, [(5_779_881 + MARKET_PAUSE_US)]);
        if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut req.actions[2] {
            *timestamp_us -= 1;
        }
        let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(rejected["execution_completed"], false);
        assert_eq!(
            rejected["rejected_action"]["reason"]["until_us"],
            (5_779_881 + MARKET_PAUSE_US)
        );
    }
}
