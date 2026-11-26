use super::*;

fn fixture_coverage() -> Value {
    ntesim_data::character::released_inventory()
}

#[test]
fn default_pilot_runs_and_measures_profiling_and_workload_projection() {
    let coverage = fixture_coverage();

    let mut req = default_pilot_request();
    req.teams = vec![
        "mint+sakiri+zero+iroi".into(),
        "blackbird+daffodil+haniel+sakiri".into(),
    ];
    req.scenarios = vec![
        AcceptanceScenarioKind::SingleTargetReal120,
        AcceptanceScenarioKind::FourTargetReal120,
    ];
    req.timing_multipliers = vec![
        Ratio {
            numerator: 1,
            denominator: 1,
        },
        Ratio {
            numerator: 11,
            denominator: 10,
        },
    ];
    req.seeds = vec![42];

    let report = run_pilot(&coverage, &req, &ExecutionLimits::default()).unwrap();

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.report_kind, "native-pilot-profiling-v1");
    assert_eq!(report.status, "INCOMPLETE");
    assert!(!report.game_dps_computed);
    assert!(!report.acceptance_ready);

    // fireflies.all_in_range_v1 (2026-10-14): the 4-target dps_4_real120 units no
    // longer fail on the removed proximity guard; all 8 units complete.
    assert_eq!(report.profiling.units_completed, 8);
    assert_eq!(report.profiling.units_failed, 0);
    assert!(report.checkpoint.is_complete);
    assert_eq!(report.checkpoint.completed_units.len(), 8);
    assert_eq!(report.checkpoint.failed_units.len(), 0);
    assert!(report.profiling.total_events_processed > 0);
    assert!(report.profiling.throughput_runs_per_second > 0.0);
    assert!(report.profiling.peak_memory_estimate_bytes > 0);

    assert_eq!(report.projection.total_four_character_combinations, 12650);
    assert_eq!(report.projection.projected_screening_runs, 632_500);
    assert_eq!(report.projection.projected_finalist_runs, 750_000);
    assert_eq!(report.projection.total_projected_runs, 1_382_500);
    assert!(report.projection.projected_total_cpu_hours >= 0.0);
    assert_eq!(report.projection.uncertainty_range_pct, None);
}

#[test]
fn deterministic_stop_and_resume_checkpointing_produces_identical_results() {
    let coverage = fixture_coverage();

    let mut req = default_pilot_request();
    req.teams = vec!["mint+sakiri+zero+iroi".into()];
    req.scenarios = vec![
        AcceptanceScenarioKind::SingleTargetReal120,
        AcceptanceScenarioKind::FourTargetReal120,
    ];
    req.timing_multipliers = vec![Ratio {
        numerator: 1,
        denominator: 1,
    }];
    req.seeds = vec![101, 102];

    let report_single = run_pilot(&coverage, &req, &ExecutionLimits::default()).unwrap();
    assert!(report_single.checkpoint.is_complete);
    // fireflies.all_in_range_v1 (2026-10-14): all 4 units complete now; the two
    // FourTargetReal120 units no longer fail on the removed proximity guard.
    assert_eq!(report_single.checkpoint.completed_units.len(), 4);
    assert_eq!(report_single.checkpoint.failed_units.len(), 0);

    let mut req_part1 = req.clone();
    req_part1.budget.max_units_per_checkpoint = Some(3);
    let report_part1 = run_pilot(&coverage, &req_part1, &ExecutionLimits::default()).unwrap();
    assert!(!report_part1.checkpoint.is_complete);
    assert_eq!(report_part1.checkpoint.completed_units.len(), 3);
    assert_eq!(report_part1.checkpoint.failed_units.len(), 0);
    assert_eq!(report_part1.checkpoint.next_unit_index, 3);

    let mut req_part2 = req.clone();
    req_part2.resume_checkpoint = Some(report_part1.checkpoint.clone());
    let report_part2 = run_pilot(&coverage, &req_part2, &ExecutionLimits::default()).unwrap();
    assert!(report_part2.checkpoint.is_complete);
    assert_eq!(report_part2.checkpoint.completed_units.len(), 4);
    assert_eq!(report_part2.checkpoint.failed_units.len(), 0);
    assert_eq!(report_part2.checkpoint.resume_count, 1);

    assert_eq!(
        report_single.checkpoint.cumulative_sha256,
        report_part2.checkpoint.cumulative_sha256
    );
    for idx in 0..4 {
        let u1 = &report_single.checkpoint.completed_units[idx];
        let u2 = &report_part2.checkpoint.completed_units[idx];
        assert_eq!(u1.damage_exact, u2.damage_exact);
        assert_eq!(u1.events_processed, u2.events_processed);
        assert_eq!(u1.unit_sha256, u2.unit_sha256);
    }
    assert_eq!(
        report_single.checkpoint.failed_units,
        report_part2.checkpoint.failed_units
    );
}

#[test]
fn corrupted_checkpoint_fails_closed_before_execution() {
    let coverage = fixture_coverage();

    let req = default_pilot_request();
    let mut bogus_checkpoint = PilotCheckpoint {
        checkpoint_id: "fake_id".into(),
        schema_version: 1,
        request_sha256: "fake_req".into(),
        cumulative_sha256: "bad_tampered_hash".into(),
        completed_units: vec![],
        failed_units: vec![],
        next_unit_index: 0,
        total_planned_units: 36,
        is_complete: false,
        resume_count: 0,
    };

    bogus_checkpoint.completed_units.push(PilotUnitResult {
        unit_index: 0,
        team: "mint+sakiri+zero+iroi".into(),
        scenario: "dps_1_real120".into(),
        timing_multiplier: Ratio {
            numerator: 1,
            denominator: 1,
        },
        seed: 42,
        damage_exact: ExactFraction {
            numerator: 100,
            denominator: 1,
        },
        damage_f64: 100.0,
        dps: 0.833,
        events_processed: 10,
        elapsed_us: 100,
        estimated_peak_memory_bytes: 1024,
        storage_bytes: 128,
        unit_sha256: "tampered_unit_hash".into(),
    });

    let mut test_req = req;
    test_req.resume_checkpoint = Some(bogus_checkpoint);

    let err = run_pilot(&coverage, &test_req, &ExecutionLimits::default()).unwrap_err();
    assert!(err.contains("corrupted checkpoint"));
}

#[test]
fn fireflies_multi_target_completes_under_all_in_range_contract() {
    let coverage = fixture_coverage();
    let mut request = default_pilot_request();
    request.teams = vec![
        "iroi+sakiri+zero+haniel".into(),
        "mint+sakiri+zero+iroi".into(),
    ];
    request.scenarios = vec![AcceptanceScenarioKind::FourTargetReal120];
    request.timing_multipliers = vec![Ratio {
        numerator: 1,
        denominator: 1,
    }];
    request.seeds = vec![42];

    // fireflies.all_in_range_v1 (2026-10-14): the 4-target unit that used to fail
    // with "Fireflies team-hit geometry requires exactly one target; received 4"
    // now completes under the declared all-in-range scenario rule.
    let report = run_pilot(&coverage, &request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report.checkpoint.completed_units.len(), 2);
    assert_eq!(report.checkpoint.failed_units.len(), 0);
    assert_eq!(report.checkpoint.next_unit_index, 2);
    assert!(report.checkpoint.is_complete);
    assert_eq!(report.profiling.units_completed, 2);
    assert_eq!(report.profiling.units_failed, 0);
    let completed_teams: std::collections::BTreeSet<_> = report
        .checkpoint
        .completed_units
        .iter()
        .map(|unit| unit.team.as_str())
        .collect();
    assert_eq!(
        completed_teams,
        ["iroi+sakiri+zero+haniel", "mint+sakiri+zero+iroi"]
            .into_iter()
            .collect()
    );

    request.resume_checkpoint = Some(report.checkpoint.clone());
    let resumed = run_pilot(&coverage, &request, &ExecutionLimits::default()).unwrap();
    assert_eq!(resumed.checkpoint.completed_units.len(), 2);
    assert_eq!(
        resumed.checkpoint.failed_units,
        report.checkpoint.failed_units
    );
    assert_eq!(
        resumed.checkpoint.cumulative_sha256,
        report.checkpoint.cumulative_sha256
    );
}

#[test]
fn candidate_screening_generates_pruned_receipts_below_threshold() {
    let coverage = fixture_coverage();

    let mut req = default_pilot_request();
    req.teams = vec![
        "mint+sakiri+zero+iroi".into(),
        "aurelia+baicang+daffodil+sakiri".into(),
    ];
    req.scenarios = vec![AcceptanceScenarioKind::SingleTargetReal120];
    req.timing_multipliers = vec![Ratio {
        numerator: 1,
        denominator: 1,
    }];
    req.seeds = vec![42];
    req.budget.screening_prune_threshold_pct = Some(0.1);

    let report = run_pilot(&coverage, &req, &ExecutionLimits::default()).unwrap();
    assert!(report.checkpoint.is_complete);
    assert!(!report.pruned_receipts.is_empty());
    let pr = &report.pruned_receipts[0];
    assert!(pr.deficit_pct > 0.1);
    assert!(pr.reason.contains("screening threshold"));
}

#[test]
fn invalid_pilot_requests_fail_closed() {
    let coverage = fixture_coverage();

    let mut req_empty_teams = default_pilot_request();
    req_empty_teams.teams.clear();
    assert!(
        run_pilot(&coverage, &req_empty_teams, &ExecutionLimits::default())
            .unwrap_err()
            .contains("at least 1 team")
    );

    let mut req_unreleased = default_pilot_request();
    req_unreleased.teams = vec!["unknown_char+sakiri+zero+iroi".into()];
    assert!(run_pilot(&coverage, &req_unreleased, &ExecutionLimits::default()).is_err());

    let mut req_bad_schema = default_pilot_request();
    req_bad_schema.schema_version = 99;
    assert!(
        run_pilot(&coverage, &req_bad_schema, &ExecutionLimits::default())
            .unwrap_err()
            .contains("schema version")
    );
}

#[test]
fn pilot_checkpoint_binds_data_limits_and_receipts_and_partial_work_never_prunes() {
    let coverage = fixture_coverage();
    let mut request = default_pilot_request();
    request.scenarios = vec![AcceptanceScenarioKind::SingleTargetReal120];
    request.timing_multipliers = vec![Ratio {
        numerator: 1,
        denominator: 1,
    }];
    request.seeds = vec![42];
    request.budget.max_units_per_checkpoint = Some(1);
    let report = run_pilot(&coverage, &request, &ExecutionLimits::default()).unwrap();
    assert!(!report.checkpoint.is_complete);
    assert!(report.pruned_receipts.is_empty());
    request.resume_checkpoint = Some(report.checkpoint.clone());
    request.seeds = vec![43];
    assert!(run_pilot(&coverage, &request, &ExecutionLimits::default()).is_err());
    request.seeds = vec![42];
    request.resume_checkpoint.as_mut().unwrap().completed_units[0]
        .damage_exact
        .numerator += 1;
    assert!(
        run_pilot(&coverage, &request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("corrupted")
    );
    request.resume_checkpoint = None;
    let limits = ExecutionLimits {
        max_events: 9,
        ..ExecutionLimits::default()
    };
    assert!(
        run_pilot(&coverage, &request, &limits)
            .unwrap_err()
            .contains("incomplete")
    );
}

#[test]
fn shared_preview_constructor_obeys_input_order_complete_overrides_and_actual_effect_identity() {
    let scenario = build_pilot_scenario(
        "mint+sakiri+zero+iroi",
        AcceptanceScenarioKind::SingleTargetReal120,
        Ratio {
            numerator: 1,
            denominator: 1,
        },
        42,
        None,
    )
    .unwrap();
    assert_eq!(scenario.active_actor_index, 0);
    assert_eq!(
        scenario.actors[0].character_id,
        get_character_profile("mint").unwrap().character_id
    );
    assert!(
        build_pilot_scenario(
            "mint+sakiri+zero+iroi",
            AcceptanceScenarioKind::SingleTargetMode120,
            Ratio {
                numerator: 1,
                denominator: 1
            },
            42,
            None
        )
        .unwrap_err()
        .contains("mode-clock")
    );
    assert!(
        build_pilot_scenario(
            "mint+sakiri+zero+iroi",
            AcceptanceScenarioKind::SingleTargetReal120,
            Ratio {
                numerator: 1,
                denominator: 1
            },
            42,
            Some(&BTreeMap::new())
        )
        .is_err()
    );
    let crit = scenario.stochastic_crit.unwrap();
    for action in scenario.actions {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            damage_effect_id,
            ..
        } = action
        {
            let actor = &scenario.actors[actor_index];
            assert!(crit.eligible_effects[&actor.character_id].contains(&damage_effect_id));
            if actor.character_id == "character:1075" {
                assert_eq!(damage_effect_id, "GE_Player_Oneiroi_Skill_Gun_Damage");
                assert!(
                    !crit.eligible_effects[&actor.character_id]
                        .contains("GE_Player_Oneiroi_Skill_Syringe_Damage")
                );
            }
        }
    }
}

#[test]
fn adler_pilot_projects_def_and_executes_the_skill_train() {
    let scenario = build_pilot_scenario(
        "adler+zero+mint+sakiri",
        AcceptanceScenarioKind::SingleTargetReal120,
        Ratio::one(),
        42,
        None,
    )
    .unwrap();
    let GameActionCommand::InvokeAbility {
        damage_effect_id,
        scaling_stat,
        atk_rate,
        ..
    } = &scenario.actions[0]
    else {
        panic!("pilot must open with Adler Skill");
    };
    assert_eq!(damage_effect_id, "GE_Player_Adler_Skill2_Damage");
    assert_eq!(
        *scaling_stat,
        ntesim_content::simulation::NativeScalingStat::Def
    );
    assert_eq!(
        *atk_rate,
        Ratio {
            numerator: 162,
            denominator: 125
        }
    );
    let report = simulation::run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    let hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["actor_index"] == 0)
        .collect::<Vec<_>>();
    assert_eq!(hits.len(), 13);
    assert!(hits[..3].iter().all(|hit| hit["scaling_stat"] == "def"));
    assert_eq!(
        hits[0]["non_crit_damage"],
        serde_json::json!("1932336/3125")
    );
    assert!(
        scenario.stochastic_crit.as_ref().unwrap().eligible_effects["character:1033"]
            .contains("GE_Player_Adler_Skill_Damage")
    );
}

#[test]
fn preview_entry_offset_keeps_hotori_first_cast_timepiece_legal() {
    assert_eq!(preview_entry_offset_us("mint+sakiri+zero+iroi"), 0);
    assert_eq!(
        preview_entry_offset_us("hotori+iroi+jiuyuan+zero"),
        10_000_000
    );
    let scenario = build_pilot_scenario(
        "hotori+iroi+jiuyuan+zero",
        AcceptanceScenarioKind::SingleTargetReal120,
        Ratio::one(),
        42,
        None,
    )
    .unwrap();
    let first_cast_us = scenario
        .actions
        .iter()
        .find_map(|action| match action {
            GameActionCommand::InvokeAbility {
                actor_index: 0,
                timestamp_us,
                ..
            } => Some(*timestamp_us),
            _ => None,
        })
        .expect("Hotori must open the preview");
    assert_eq!(first_cast_us, 10_250_000);
    let GameActionCommand::Wait {
        timestamp_us,
        duration_us,
    } = scenario.actions.last().unwrap()
    else {
        panic!("preview must end with a wait");
    };
    assert_eq!(*timestamp_us, 23_470_000);
    assert_eq!(*duration_us, 120_000_000 - 23_470_000);
    let report = simulation::run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
}
