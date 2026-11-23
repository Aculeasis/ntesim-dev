use super::haniel_actions::{make_action, request};
use super::*;

#[test]
fn haniel_multi_target_all_in_range_skill_and_burst() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Test Skill across 2 targets: 2 hits each = 4 hits total
    req.actions = vec![make_action(
        0,
        0,
        "GA_Haniel_Skill",
        "GE_Player_Haniel_Skill1_Damage",
        0,
        20_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 4);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts[0]["target_index"], 0);
    assert_eq!(receipts[1]["target_index"], 0);
    assert_eq!(receipts[2]["target_index"], 1);
    assert_eq!(receipts[3]["target_index"], 1);

    // Test Burst across 2 targets: 1 hit each = 2 hits total
    let mut req_burst = request();
    let mut target2 = req_burst.targets[0].clone();
    target2.target_index = 1;
    req_burst.targets.push(target2);
    req_burst.actions = vec![make_action(
        0,
        0,
        "GA_Haniel_UltraSkill",
        "GE_Player_Haniel_Ultraskill1_Damage",
        100,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    assert_eq!(report_burst["hits_settled"], 2);
    let receipts_burst = report_burst["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts_burst[0]["target_index"], 0);
    assert_eq!(receipts_burst[1]["target_index"], 1);
}

#[test]
fn haniel_multi_target_strict_acceptance() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.actors[0].static_stats.base_atk = 1000;
    req.model_assumptions.insert(
        crate::simulation::energy::HANIEL_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::HANIEL_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::haniel::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::haniel::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::haniel::skill::TEAM_ATK_CLOCK_HYPOTHESIS_ID.into(),
        crate::characters::haniel::skill::TEAM_ATK_CLOCK_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Haniel_Skill",
        "GE_Player_Haniel_Skill1_Damage",
        0,
        20_000_000,
    )];

    req.model_assumptions.insert(
        crate::characters::haniel::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::haniel::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::haniel::hootie::HYPOTHESIS.into(),
        crate::characters::haniel::hootie::HYPOTHESIS
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 4);
}
