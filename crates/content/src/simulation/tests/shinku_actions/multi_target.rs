use super::*;

#[test]
fn shinku_multi_target_skill_and_burst_delivery() {
    let mut req = request();
    req.actors[0].initial_energy = 60;
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);

    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            0,
            3_000_000,
        ),
        make_action(
            1_000_000,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            0,
            20_000_000,
        ),
    ];
    req.relative_duration_us = 3_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // Skill: 3 hits * 2 targets = 6 hits
    // Burst: 9 hits * 2 targets = 18 hits
    // Total = 24 hits settled
    assert_eq!(report["hits_settled"], 24);

    let hits = report["hit_receipts"].as_array().unwrap();
    let skill_hits = hits
        .iter()
        .filter(|h| h["ability_id"] == "GA_Shinku_Skill")
        .collect::<Vec<_>>();
    assert_eq!(skill_hits.len(), 6);
    assert_eq!(
        skill_hits.iter().filter(|h| h["target_index"] == 0).count(),
        3
    );
    assert_eq!(
        skill_hits.iter().filter(|h| h["target_index"] == 1).count(),
        3
    );

    let burst_hits = hits
        .iter()
        .filter(|h| h["ability_id"] == "GA_Shinku_UltraSkill")
        .collect::<Vec<_>>();
    assert_eq!(burst_hits.len(), 18);
    assert_eq!(
        burst_hits.iter().filter(|h| h["target_index"] == 0).count(),
        9
    );
    assert_eq!(
        burst_hits.iter().filter(|h| h["target_index"] == 1).count(),
        9
    );

    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::shinku::skill::ALL_IN_RANGE_HYPOTHESIS_ID.as_str()],
        crate::characters::shinku::skill::ALL_IN_RANGE_HYPOTHESIS_ID.description()
    );
}
