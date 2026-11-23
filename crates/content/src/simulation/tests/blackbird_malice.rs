use super::*;
use serde_json::{Value, json};

fn request(count: u64, targets: usize) -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    let model = crate::HypothesisId::BlackbirdOrdinaryMaliceV1;
    req.model_assumptions
        .insert(model.as_str().into(), model.description().into());
    req.relative_duration_us = 30_000_000;
    req.actors[0].initial_energy = 0;
    for index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = index;
        req.targets.push(target);
    }
    let action = req.actions[0].clone();
    req.actions = (0..count * 5)
        .map(|index| {
            let mut action = action.clone();
            if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
                *timestamp_us = index * 1000;
            }
            action
        })
        .collect();
    req
}

fn pool(report: &Value) -> &Value {
    &report["final_actor_states"][0]["fractional_resources"]["blackbird.malice"]
}

#[test]
fn blackbird_malice_exact_chain_gains_are_once_per_packet_with_caps_and_continuation() {
    for targets in [1, 4] {
        let req = request(1, targets);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        // Independent native sum:20 entry +.857919+2*1.762896+4*1.089593+5*.933937+5*1.317647.
        assert_eq!(pool(&report)["current"], json!("40000003/1000000"));
        assert_eq!(pool(&report)["conserved"], true);
        let report = run_game_scenario(&request(2, targets), &ExecutionLimits::default()).unwrap();
        assert_eq!(pool(&report)["current"], json!("60/1"));
        assert_eq!(pool(&report)["overflow"], json!("3/500000"));
        assert_eq!(pool(&report)["granted"], json!("30000003/500000"));
    }
}

#[test]
fn blackbird_malice_positive_hp_gate_uses_first_living_connected_recipient() {
    let mut req = request(1, 2);
    req.targets[0].hp_current = 0;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pool(&report)["current"], json!("40000003/1000000"));
    req.targets[1].resistance = Ratio::one();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(pool(&report)["current"], json!("20/1"));
}

#[test]
fn blackbird_skill_malice_activation_survives_miss_without_periodic_combat_refill() {
    let mut req = request(0, 1);
    req.targets[0].resistance = Ratio::one();
    let template = super::blackbird_actions::request().actions.remove(0);
    req.actions = [0, 16_000_000]
        .map(|time| {
            let mut action = template.clone();
            if let GameActionCommand::InvokeAbility {
                timestamp_us,
                ability_id,
                damage_effect_id,
                cooldown_us,
                ..
            } = &mut action
            {
                *timestamp_us = time;
                *ability_id = "GA_BlackBird_Skill".into();
                *damage_effect_id = "GE_Player_BlackBird_Skill_Damage".into();
                *cooldown_us = 16_000_000;
            }
            action
        })
        .into();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(pool(&report)["granted"], json!("100/1"));
    assert_eq!(pool(&report)["current"], json!("60/1"));
    assert_eq!(pool(&report)["overflow"], json!("40/1"));
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 100);
}

#[test]
fn blackbird_legacy_packet_boundary_omits_unselected_fractional_pool() {
    let report = run_game_scenario(
        &super::blackbird_actions::request(),
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        report["final_actor_states"][0]
            .get("fractional_resources")
            .is_none()
    );
}
