use super::*;

#[test]
fn daffodil_multi_target_all_in_range_skill_and_burst() {
    let mut req = request();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    // Test Skill across 2 targets: 8 hits each = 16 hits
    req.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
        16_000_000,
    )];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 16);
    let receipts = report["hit_receipts"].as_array().unwrap();
    for receipt in &receipts[..8] {
        assert_eq!(receipt["target_index"], 0);
    }
    for receipt in &receipts[8..16] {
        assert_eq!(receipt["target_index"], 1);
    }

    // Test Burst across 2 targets: 14 hits each = 28 hits
    let mut req_burst = request();
    let mut target2 = req_burst.targets[0].clone();
    target2.target_index = 1;
    req_burst.targets.push(target2);
    req_burst.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_UltraSkill",
        "GE_Player_Daffodill_UltraSkill1_Damage",
        120,
        20_000_000,
    )];
    let report_burst = run_game_scenario(&req_burst, &ExecutionLimits::default()).unwrap();
    assert_eq!(report_burst["execution_completed"], true);
    assert_eq!(report_burst["hits_settled"], 28);
    let receipts_burst = report_burst["hit_receipts"].as_array().unwrap();
    for receipt in &receipts_burst[..14] {
        assert_eq!(receipt["target_index"], 0);
    }
    for receipt in &receipts_burst[14..28] {
        assert_eq!(receipt["target_index"], 1);
    }
}

#[test]
fn daffodil_multi_target_strict_acceptance() {
    let mut req = request();
    let resonance = crate::characters::daffodil::resonance::HYPOTHESIS;
    req.model_assumptions
        .insert(resonance.into(), resonance.description().into());
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        crate::simulation::energy::DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID
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
        crate::characters::daffodil::skill::PACKETS_HYPOTHESIS_ID.into(),
        crate::characters::daffodil::skill::PACKETS_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
        16_000_000,
    )];

    req.model_assumptions.insert(
        crate::characters::daffodil::skill::ALL_IN_RANGE_HYPOTHESIS_ID.into(),
        crate::characters::daffodil::skill::ALL_IN_RANGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 16);
}
