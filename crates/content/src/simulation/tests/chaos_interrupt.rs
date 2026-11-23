use super::chaos_actions::make_action;
use super::*;
use crate::simulation::GameActorInterruptEvent;
use serde_json::json;

fn request(time: u64, targets: usize) -> GameScenarioRequest {
    let mut req = super::chaos_clock::request(targets);
    let model = crate::HypothesisId::ChaosForcedInterruptV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: time,
        actor_index: 0,
        armor_damage: None,
    }];
    req.relative_duration_us = 20_000_000;
    req
}

fn skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Skill",
        "GE_Player_Chaos_Skill1_Damage",
        0,
        0,
    )
}

#[test]
fn chaos_interrupt_skill_selects_three_or_fifteen_seconds_at_exact_boundary() {
    for (time, deadline) in [(199_999, 3_199_999), (200_000, 15_200_000)] {
        for next in [deadline - 1, deadline] {
            let mut req = request(time, 3);
            req.actions = vec![skill(0), skill(next)];
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(report["execution_completed"], next == deadline);
            let receipt = &report["interrupt_receipts"][0];
            assert_eq!(receipt["cancelled_hit_deliveries"], 6);
            assert_eq!(receipt["skill_cooldown_before_us"], 15_243_640);
            assert_eq!(receipt["skill_cooldown_after_us"], deadline);
            assert_eq!(receipt["recovery_after_us"], time);
            assert_eq!(report["hits_settled"], if next == deadline { 6 } else { 0 });
        }
    }
}

#[test]
fn chaos_interrupt_after_contact_retains_damage_crime_and_warrant_without_second_bite() {
    let mut req = request(298_280, 3);
    req.actions = vec![skill(0)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 3);
    assert_eq!(
        report["interrupt_receipts"][0]["cancelled_hit_deliveries"],
        3
    );
    assert_eq!(
        report["interrupt_receipts"][0]["recovery_before_us"],
        1_005_494
    );
    assert_eq!(
        report["interrupt_receipts"][0]["skill_cooldown_after_us"],
        15_298_280
    );
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["current"],
        json!("50/1")
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "chaos_skill_warrant")
            .count(),
        1
    );
}

#[test]
fn chaos_interrupt_burst_cancels_unreached_entry_but_preserves_live_form_and_committed_cost() {
    for (time, hits, ticks) in [(2_718_887, 0, 0), (2_718_888, 0, 60), (3_180_295, 6, 60)] {
        let mut req = request(time, 3);
        let model = crate::HypothesisId::ChaosPrimaryFormV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.actors[0].initial_energy = 100;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Chaos071_UltraSkill",
            "GE_Player_Chaos_UltraSkill1_Damage",
            100,
            0,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], hits);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["event_kind"] == "chaos_auto_crime")
                .count(),
            ticks
        );
    }
}

#[test]
fn chaos_interrupt_requires_explicit_model_and_is_idle_after_recovery() {
    let mut req = request(1_054_994, 1);
    req.actions = vec![skill(0)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(report["interrupt_receipts"][0]["interrupted_command_index"].is_null());
    req.model_assumptions
        .remove(crate::HypothesisId::ChaosForcedInterruptV1.as_str());
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("forced_interrupt_v1")
    );
}
