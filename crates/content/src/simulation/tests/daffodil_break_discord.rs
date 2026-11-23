use super::*;
use crate::characters::daffodil::discord;
use crate::simulation::{ExactFraction, GameTargetBreakProfileConfig, state::GameSimulatorState};
use serde_json::json;

fn profile(current_gauge: u64) -> GameTargetBreakProfileConfig {
    GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 100,
            denominator: 1,
        },
        current_gauge: Ratio {
            numerator: current_gauge,
            denominator: 1,
        },
        broken_duration_us: 1_000_000,
        discord_fraction: Some(Ratio {
            numerator: 1,
            denominator: 5,
        }),
    }
}

#[test]
fn discord_gauge_loss_scales_only_by_target_acceptance() {
    let mut req = super::daffodil_actions::request();
    req.targets[0].break_profile = Some(profile(70));
    // Discord is target-dependent: the owner's ordinary Break Efficiency is
    // not part of the selected loss equation.
    req.actors[0].static_stats.break_efficiency_multiplier = Ratio {
        numerator: 3,
        denominator: 1,
    };

    for (incoming, expected_gauge) in [((0, 1), 70), ((1, 2), 60), ((1, 1), 50)] {
        let mut state = GameSimulatorState::new(&req);
        state.targets[0]
            .break_state
            .as_mut()
            .unwrap()
            .incoming_efficiency_multiplier = ExactFraction::new(incoming.0, incoming.1).unwrap();
        let mut trace = Vec::new();
        let broke = crate::simulation::break_state::apply_discord_gauge_loss(
            &req,
            &mut state,
            &mut Default::default(),
            &mut 0,
            &mut trace,
            0,
            0,
            ExactFraction::new(1, 5).unwrap(),
        )
        .unwrap();
        assert!(!broke);
        assert_eq!(
            state.targets[0].break_state.as_ref().unwrap().gauge,
            ExactFraction::new(expected_gauge, 1).unwrap()
        );
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].event_kind, "target_discord_gauge_reduced");
    }
}

#[test]
fn daffodil_discord_b2_has_separate_strict_acceptance() {
    let mut req = super::daffodil_actions::request();
    req.targets[0].break_profile = Some(profile(100));
    let id = crate::HypothesisId::DaffodilDiscordCapReductionV1;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    let mut accepted = GameSimulatorState::new(&req);
    assert!(
        !discord::on_discord(
            &req,
            &mut accepted,
            &mut Default::default(),
            &mut 0,
            &mut Vec::new(),
            0,
            0,
        )
        .unwrap()
    );
    assert_eq!(
        accepted
            .execution_model_assumptions
            .get(id.as_str())
            .map(String::as_str),
        Some(id.description())
    );
    assert!(
        !accepted
            .execution_model_assumptions
            .contains_key(crate::HypothesisId::CombatDiscordBreakGaugeV1.as_str())
    );
}

#[test]
fn discord_pairs_reduce_gauge_once_per_generation_and_restore_cap_at_latest_expiry() {
    let mut req = super::reactions::request(&[
        (0, "character:1003", 100, 0), // Incantation applies Scorch.
        (1, "character:1054", 100, 0), // Daffodil supplies Chaos and B2.
        (2, "character:1020", 100, 0), // Psyche creates Nova generations.
        (3, "character:1004", 100, 0), // Chaos refreshes Nova.
    ]);
    req.relative_duration_us = 31_000_000;
    req.targets[0].hp_max = 1_000_000_000;
    req.targets[0].hp_current = 1_000_000_000;
    req.targets[0].break_profile = Some(profile(75));
    req.actions = vec![
        super::reactions::switch(0, 1),
        super::reactions::switch(1, 2),
        super::reactions::switch(2, 3),
    ];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let passive_hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_AnHunZhouTwo_Damage")
        .collect::<Vec<_>>();
    assert_eq!(passive_hits.len(), 2);
    for (hit, expected_command) in passive_hits.iter().zip([1, 2]) {
        assert!(hit["command_index"].is_null());
        assert!(hit["parent_hit_index"].is_null());
        assert_eq!(hit["actor_index"], 3);
        let origin =
            &report["autonomous_events"][hit["autonomous_event_index"].as_u64().unwrap() as usize];
        assert_eq!(origin["ability_id"], "GA_lacrimosa_Passive_1");
        assert_eq!(
            origin["trigger"],
            json!({"kind":"action","command_index":expected_command})
        );
    }
    let trace = report["trace"].as_array().unwrap();
    for kind in [
        "reaction_discord_triggered",
        "daffodil_discord_cap_applied",
        "target_discord_gauge_reduced",
    ] {
        assert_eq!(
            trace
                .iter()
                .filter(|event| event["event_kind"] == kind)
                .count(),
            2,
            "{kind}"
        );
    }

    // Starting at cap100/gauge75, each B2 stack reduces the cap by10 while
    // preserving depletion, and each Discord removes20 from original cap:
    // 75 -> 65 -> 45 -> 35 -> 15. Restoring cap100 preserves depletion65,
    // giving gauge35 after the newest 30s expiry. Lacrimosa's paid QTE
    // adds 2.5 ordinary gauge loss during that same lifetime.
    assert_eq!(
        report["final_target_states"][0]["break_state"]["capacity"],
        json!("100/1")
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["gauge"],
        json!("65/2")
    );
    assert!(
        report["final_target_states"][0]["statuses"]
            .as_array()
            .is_none_or(|statuses| statuses
                .iter()
                .all(|status| status["id"] != discord::STATUS_ID))
    );
    for id in [
        crate::HypothesisId::CombatDiscordBreakGaugeV1,
        crate::HypothesisId::DaffodilDiscordCapReductionV1,
    ] {
        assert_eq!(
            report["resolved_model_assumptions"][id.as_str()],
            id.description()
        );
    }
}

#[test]
fn third_discord_at_cap_two_does_not_repeat_a_live_break_transition() {
    let mut req = super::reactions::request(&[
        (0, "character:1003", 100, 0),
        (1, "character:1054", 100, 0),
        (2, "character:1020", 100, 0),
        (3, "character:1004", 100, 0),
    ]);
    req.relative_duration_us = 31_000_000;
    req.targets[0].hp_max = 1_000_000_000;
    req.targets[0].hp_current = 1_000_000_000;
    req.targets[0].break_profile = Some(profile(20));
    req.targets[0]
        .break_profile
        .as_mut()
        .unwrap()
        .broken_duration_us = 1_000_000;
    req.actions = vec![
        super::reactions::switch(0, 1),
        super::reactions::switch(1, 2),
        super::reactions::switch(2, 3),
        // A fresh qualified Scorch application overlaps the latest Nova.
        super::reactions::switch(600_002, 0),
    ];

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let trace = report["trace"].as_array().unwrap();
    assert_eq!(
        trace
            .iter()
            .filter(|event| event["event_kind"] == "reaction_scorch_applied")
            .count(),
        2
    );
    let scorch_ticks = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.scorch")
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(scorch_ticks.first(), Some(&990_000));
    assert_eq!(
        trace
            .iter()
            .filter(|event| event["event_kind"] == "reaction_discord_triggered")
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event["event_kind"] == "daffodil_discord_cap_applied")
            .count(),
        3
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event["event_kind"] == "target_broken")
            .count(),
        1
    );
    assert_eq!(
        trace
            .iter()
            .filter(|event| event["event_kind"] == "target_discord_gauge_ignored_broken")
            .count(),
        2
    );
    // The first overlap takes gauge20 -> cap-shifted10 -> direct-loss0.
    // Later overlaps and the qualified-entry Support occur while Broken,
    // refresh the capped effect without another transition, and the shared
    // team Break is scheduled only once.
    assert_eq!(
        trace
            .iter()
            .filter(|event| event["event_kind"] == "break_damage")
            .count(),
        4
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["capacity"],
        json!("100/1")
    );
    assert_eq!(
        report["final_target_states"][0]["break_state"]["gauge"],
        json!("100/1")
    );
}
