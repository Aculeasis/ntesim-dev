use super::*;

#[test]
fn baicang_multi_target_all_in_range_skill_and_burst() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Test Skill across 2 targets: 1 hit each = 2 hits
    req.actions = vec![make_action(
        0,
        0,
        "GA_Cang_Skill",
        "GE_Player_Cang_Skill_Damage",
        0,
        6_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(report["hit_receipts"][0]["target_index"], 0);
    assert_eq!(report["hit_receipts"][1]["target_index"], 1);

    // Test Burst across 2 targets with horizon 1_000_000 (expansion hit only):
    // 1 expansion hit on target 0 + 1 expansion hit on target 1 = 2 hits
    let mut req_burst = request();
    let mut target2 = req_burst.targets[0].clone();
    target2.target_index = 1;
    req_burst.targets.push(target2);
    req_burst.actors[0].initial_energy = 120;
    req_burst.actors[0].energy_capacity = 120;
    req_burst.relative_duration_us = 1_000_000;
    req_burst.actions = vec![make_action(
        0,
        0,
        "GA_Cang_UltraSkill",
        "GE_Player_Cang_UltraSkill2_Damage",
        120,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["hits_settled"], 2);

    // Strict mode requires ALL_IN_RANGE hypothesis
    req_burst.actors[0].talent_levels = None;
    req_burst.actors[0].purchased_skill_level = 8;
    req_burst.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req_burst.model_assumptions.insert(
        crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID.into(),
        crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req_burst.model_assumptions.insert(
        crate::characters::baicang::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::baicang::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report_strict = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_strict["hits_settled"], 2);
}
