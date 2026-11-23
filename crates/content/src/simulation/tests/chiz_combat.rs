use super::chiz_market::MARKET_PAUSE_US;
use super::*;
use serde_json::{Value, json};
pub(super) fn request() -> GameScenarioRequest {
    let mut req = super::chiz_held::request(1);
    req.relative_duration_us = 18_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4, 5, 6]).unwrap();
    let model = crate::characters::chiz::combat::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    super::chiz_held::hold(&mut req, 0, 10_000_000);
    super::chiz_clock::chain(&mut req, 7_000_000);
    for (time, leaving) in [
        (12_000_000, true),
        (13_000_000, false),
        (14_000_000, true),
        (15_000_000, false),
    ] {
        req.combat_transition_events
            .push(GameActorCombatTransitionEvent {
                timestamp_us: time,
                actor_index: 0,
                leaving,
            });
    }
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
fn fraction(value: u64) -> Value {
    json!(format!("{value}/1"))
}
#[test]
fn chiz_a5_real_battle_entries_pay_carried_debt_once_before_capped_savings_without_efficiency() {
    let req = request();
    let report = run(&req);
    let rows = report["combat_transition_receipts"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    let first = &rows[1]["resource_deposits"]["chiz.grain"];
    for (key, value) in [
        ("input", 15000),
        ("repaid", 11175),
        ("credited", 3825),
        ("overflow", 0),
    ] {
        assert_eq!(first[key], fraction(value));
    }
    let second = &rows[3]["resource_deposits"]["chiz.grain"];
    assert_eq!(second["credited"], fraction(11175));
    assert_eq!(second["overflow"], fraction(3825));
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["energy"], 29);
    assert_eq!(actor["resources"]["chiz.skill_stocks"]["current"], 3);
    assert_eq!(
        actor["fractional_resources"]["chiz.grain"]["current"],
        json!("15000/1")
    );
    assert_eq!(
        actor["fractional_resources"]["chiz.grain"]["overflow"],
        json!("3825/1")
    );
    assert_eq!(
        actor["fractional_resources"]["chiz.loan_debt"]["current"],
        json!("0/1")
    );
    assert_eq!(
        actor["fractional_resources"]["chiz.loan_debt"]["conserved"],
        true
    );
}
#[test]
fn chiz_reentry_without_selected_a5_preserves_loans_and_authored_transitions_reject_duplicates() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([]).unwrap();
    let report = run(&req);
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chiz.loan_debt"]["current"],
        json!("12750/1")
    );
    assert!(
        report["combat_transition_receipts"][1]
            .get("resource_deposits")
            .is_none()
    );
    req.combat_transition_events[1].leaving = true;
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("alternate")
    );
}
#[test]
fn chiz_a5_deposit_during_investment_preserves_capture_and_uses_market_cap_without_repaying_early()
{
    let mut req = super::chiz_market::request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    let model = crate::characters::chiz::combat::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actions = vec![
        super::chiz_market::burst(0),
        super::chiz_pressed::press(4_000_000 + MARKET_PAUSE_US),
    ];
    for (time, leaving) in [
        (4_250_000 + MARKET_PAUSE_US, true),
        (4_300_000 + MARKET_PAUSE_US, false),
    ] {
        req.combat_transition_events
            .push(GameActorCombatTransitionEvent {
                timestamp_us: time,
                actor_index: 0,
                leaving,
            });
    }
    let report = run(&req);
    let deposit = &report["combat_transition_receipts"][1]["resource_deposits"]["chiz.grain"];
    assert_eq!(deposit["repaid"], fraction(0));
    assert_eq!(deposit["credited"], fraction(2400));
    assert_eq!(deposit["overflow"], fraction(12600));
    let coin = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["damage_effect_id"] == "GE_Player_Chiichan073_Skill1_CoinDamage")
        .unwrap();
    assert_eq!(coin["damage_base_override"], json!("27600/1"));
}
