use super::*;
use serde_json::{Value, json};

fn request() -> GameScenarioRequest {
    let mut req = super::chiz_clock::request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    let model = crate::characters::chiz::finance::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn run(req: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    report
}
fn grain(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]
}
fn times(report: &Value) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "chiz_surplus_interest")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn chiz_surplus_interest_credits_ten_percent_once_per_real_second_with_exact_carry() {
    for (horizon, n, d, events) in [
        (999_999, 450, 1, vec![]),
        (1_000_000, 495, 1, vec![1_000_000]),
        (2_000_000, 1089, 2, vec![1_000_000, 2_000_000]),
    ] {
        let mut req = request();
        req.relative_duration_us = horizon;
        req.actions = vec![super::chiz_clock::normal(0)];
        let report = run(&req);
        assert_eq!(grain(&report)["current"], json!(format!("{}/{}", n, d)));
        assert_eq!(grain(&report)["conserved"], true);
        assert_eq!(times(&report), events);
    }
}

#[test]
fn chiz_interest_two_continuing_chains_preserve_phase_and_unrounded_wallet() {
    let mut req = request();
    req.relative_duration_us = 10_000_000;
    super::chiz_clock::chain(&mut req, 0);
    super::chiz_clock::chain(&mut req, 5_000_000);
    let report = run(&req);
    assert_eq!(report["hits_settled"], 24);
    assert_eq!(grain(&report)["current"], json!("424436816727/50000000"));
    assert_eq!(grain(&report)["granted"], grain(&report)["current"]);
    assert_eq!(
        times(&report),
        (1..=10)
            .map(|second| second * 1_000_000)
            .collect::<Vec<_>>()
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 39);
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 54);
}

#[test]
fn chiz_interest_off_field_a5_cap_overflow_ignores_acquisition_multiplier_and_stops_on_death() {
    for dead in [false, true] {
        let mut req = request();
        req.relative_duration_us = 2_000_000;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2, 4, 5]).unwrap();
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.character_id = "character:1019".into();
        ally.awakening = ntesim_data::AwakeningSelection::prefix(0);
        req.actors.push(ally);
        req.active_actor_index = 1;
        if dead {
            let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
            req.model_assumptions
                .insert(model.into(), model.description().into());
            req.incoming_damage_events
                .push(crate::simulation::GameActorDamageEvent {
                    timestamp_us: 500_000,
                    actor_index: 0,
                    delivered_damage: req.actors[0].initial_hp,
                });
        }
        let report = run(&req);
        assert_eq!(grain(&report)["initial"], json!("0/1"));
        assert_eq!(grain(&report)["current"], json!("15000/1"));
        assert_eq!(
            grain(&report)["granted"],
            json!(format!("{}/{}", if dead { 15000 } else { 18000 }, 1))
        );
        assert_eq!(
            grain(&report)["overflow"],
            json!(format!("{}/{}", if dead { 0 } else { 3000 }, 1))
        );
        assert_eq!(
            times(&report),
            if dead {
                vec![]
            } else {
                vec![1_000_000, 2_000_000]
            }
        );
    }
}

#[test]
fn chiz_empty_wallet_has_no_polling_and_unselected_a2_has_no_interest() {
    let mut req = request();
    req.relative_duration_us = 1_000_000;
    let short = run(&req);
    req.relative_duration_us = 500_000_000;
    let long = run(&req);
    assert_eq!(
        short["total_events_processed"],
        long["total_events_processed"]
    );
    assert!(times(&long).is_empty());
    req.relative_duration_us = 3_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
    req.actions = vec![super::chiz_clock::normal(0)];
    let report = run(&req);
    assert_eq!(grain(&report)["current"], json!("450/1"));
    assert!(times(&report).is_empty());
}

#[test]
fn chiz_long_interest_wait_keeps_wide_exact_components_and_lossless_report_roundtrip() {
    let mut req = request();
    req.relative_duration_us = 150_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 2]).unwrap();
    req.actions = vec![super::chiz_clock::normal(0)];
    let report = run(&req);
    assert_eq!(times(&report).len(), 150);
    assert_eq!(grain(&report)["current"], json!("16500/1"));
    let (granted_num, granted_den) = grain(&report)["granted"]
        .as_str()
        .unwrap()
        .split_once('/')
        .unwrap();
    assert!(granted_num.parse::<u128>().is_err() || granted_den.parse::<u128>().is_err());
    let typed: crate::simulation::GameActorStateSummary =
        serde_json::from_value(report["final_actor_states"][0].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(typed).unwrap(),
        report["final_actor_states"][0]
    );
}
