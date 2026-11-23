use super::*;
use serde_json::json;

fn load_test_fixtures() -> (Value, Value, String) {
    let coverage = ntesim_data::character::released_inventory();
    let conditions = json!({});
    let variants_csv = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mixed-variants.csv"
    ))
    .to_string();
    (coverage, conditions, variants_csv)
}

#[test]
fn acceptance_runner_executes_screening_and_refinement_with_deliverables() {
    let (coverage, conditions, variants_csv) = load_test_fixtures();

    let mut req = default_acceptance_request();
    req.max_compositions_to_evaluate = Some(8);
    req.top_candidates_per_scenario = 4;
    req.finalist_seeds = vec![101, 102];
    req.worker_config.max_workers = 2;
    req.worker_config.chunk_size = 4;

    let report = run_acceptance_pipeline(
        &coverage,
        &conditions,
        &variants_csv,
        &req,
        &ExecutionLimits::default(),
    )
    .unwrap();

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.report_kind, "acceptance-large-scale-runner-v1");
    assert_eq!(report.status, "INCOMPLETE");
    assert!(!report.game_dps_computed);
    assert!(!report.acceptance_ready);
    assert_eq!(report.compositions_enumerated, 8855);
    assert_eq!(report.compositions_processed, 8);
    assert!(report.compositions_screened > 0);
    assert!(report.compositions_screened <= report.compositions_processed);
    assert_eq!(
        report.checkpoint.screening_results.len() + report.checkpoint.screening_failures.len(),
        report.compositions_processed * req.scenarios.len()
    );
    assert!(
        report
            .checkpoint
            .screening_results
            .values()
            .all(|result| req.scenarios.contains(&result.scenario))
    );
    assert!(
        report
            .checkpoint
            .screening_results
            .values()
            .any(|result| { result.scenario == AcceptanceScenarioKind::FourTargetReal120 })
    );
    assert_eq!(
        report
            .checkpoint
            .screening_failures
            .values()
            .filter(|failure| failure
                .reason
                .contains("native action HP scaling is unsupported"))
            .count(),
        2
    );
    assert!(
        report
            .checkpoint
            .screening_failures
            .values()
            .all(|failure| {
                failure.reason != "Adler Skill model only supports one declared target"
            })
    );
    assert!(!report.checkpoint.is_complete);
    assert!(report.finalists_evaluated == report.finalists.len());
    assert!(report.deliverables.is_some());
    assert_eq!(
        report.deliverables.as_ref().unwrap().total_compositions,
        8855
    );
    assert_eq!(report.deliverables.as_ref().unwrap().total_variants, 7);
    assert_eq!(report.deliverables.as_ref().unwrap().total_archetypes, 5);
}

#[test]
fn worker_concurrency_invariance_produces_identical_results() {
    let (coverage, conditions, variants_csv) = load_test_fixtures();

    let mut req = default_acceptance_request();
    req.max_compositions_to_evaluate = Some(6);
    req.top_candidates_per_scenario = 2;
    req.finalist_seeds = vec![101];

    let mut req_1 = req.clone();
    req_1.worker_config.max_workers = 1;
    req_1.worker_config.chunk_size = 6;
    let rep_1 = run_acceptance_pipeline(
        &coverage,
        &conditions,
        &variants_csv,
        &req_1,
        &ExecutionLimits::default(),
    )
    .unwrap();

    let mut req_4 = req;
    req_4.worker_config.max_workers = 4;
    req_4.worker_config.chunk_size = 2;
    let rep_4 = run_acceptance_pipeline(
        &coverage,
        &conditions,
        &variants_csv,
        &req_4,
        &ExecutionLimits::default(),
    )
    .unwrap();

    assert_eq!(
        rep_1.checkpoint.cumulative_sha256,
        rep_4.checkpoint.cumulative_sha256
    );

    assert_eq!(
        rep_1.checkpoint.screening_results.len(),
        rep_4.checkpoint.screening_results.len()
    );
    assert_eq!(
        rep_1.checkpoint.screening_failures.len(),
        rep_4.checkpoint.screening_failures.len()
    );
    for (comp, res1) in &rep_1.checkpoint.screening_results {
        let res4 = &rep_4.checkpoint.screening_results[comp];
        assert_eq!(res1.nominal_damage_exact, res4.nominal_damage_exact);
        assert_eq!(res1.events_processed, res4.events_processed);
        assert_eq!(res1.result_sha256, res4.result_sha256);
    }
    for (comp, res1) in &rep_1.checkpoint.screening_failures {
        let res4 = &rep_4.checkpoint.screening_failures[comp];
        assert_eq!(res1.reason, res4.reason);
        assert_eq!(res1.failure_sha256, res4.failure_sha256);
    }
}

#[test]
fn deterministic_stop_and_resume_checkpointing_matches_single_pass() {
    let (coverage, conditions, variants_csv) = load_test_fixtures();

    let mut req = default_acceptance_request();
    req.max_compositions_to_evaluate = Some(6);
    req.top_candidates_per_scenario = 2;
    req.finalist_seeds = vec![101];

    let report_full = run_acceptance_pipeline(
        &coverage,
        &conditions,
        &variants_csv,
        &req,
        &ExecutionLimits::default(),
    )
    .unwrap();

    let mut req_part1 = req.clone();
    req_part1.max_compositions_to_evaluate = Some(3);
    let report_part1 = run_acceptance_pipeline(
        &coverage,
        &conditions,
        &variants_csv,
        &req_part1,
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(report_part1.compositions_processed, 3);
    assert!(!report_part1.checkpoint.is_complete);

    let mut req_part2 = req;
    req_part2.resume_checkpoint = Some(report_part1.checkpoint);
    let report_part2 = run_acceptance_pipeline(
        &coverage,
        &conditions,
        &variants_csv,
        &req_part2,
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(report_part2.compositions_processed, 6);
    assert_eq!(report_part2.checkpoint.resume_count, 1);

    assert_eq!(
        report_full.checkpoint.cumulative_sha256,
        report_part2.checkpoint.cumulative_sha256
    );
    for (comp, res1) in &report_full.checkpoint.screening_results {
        let res2 = &report_part2.checkpoint.screening_results[comp];
        assert_eq!(res1.nominal_damage_exact, res2.nominal_damage_exact);
        assert_eq!(res1.events_processed, res2.events_processed);
        assert_eq!(res1.result_sha256, res2.result_sha256);
    }
    assert_eq!(
        report_full.checkpoint.screening_failures.len(),
        report_part2.checkpoint.screening_failures.len()
    );
    for (comp, res1) in &report_full.checkpoint.screening_failures {
        let res2 = &report_part2.checkpoint.screening_failures[comp];
        assert_eq!(res1.failure_sha256, res2.failure_sha256);
    }
}

#[test]
fn tampered_checkpoint_fails_closed() {
    let (coverage, conditions, variants_csv) = load_test_fixtures();

    let mut req = default_acceptance_request();
    req.max_compositions_to_evaluate = Some(4);

    let tampered_checkpoint = AcceptanceCheckpoint {
        checkpoint_id: "fake_id".into(),
        schema_version: 1,
        request_sha256: "fake_request".into(),
        current_phase: AcceptancePhase::Screening,
        completed_composition_indices: vec![1],
        processed_composition_indices: vec![1],
        screening_results: BTreeMap::new(),
        screening_failures: BTreeMap::new(),
        finalist_results: BTreeMap::new(),
        cumulative_sha256: "corrupted_hash".into(),
        is_complete: false,
        resume_count: 0,
    };

    req.resume_checkpoint = Some(tampered_checkpoint);
    assert!(
        run_acceptance_pipeline(
            &coverage,
            &conditions,
            &variants_csv,
            &req,
            &ExecutionLimits::default(),
        )
        .unwrap_err()
        .contains("corrupted checkpoint")
    );
}

#[test]
fn screening_executes_every_declared_seed_and_scenario_and_does_not_invent_a_baseline() {
    let (coverage, _, variants) = load_test_fixtures();
    let compositions = composition::enumerate_compositions(&coverage, &variants).unwrap();
    let scoring_team = compositions
        .compositions
        .iter()
        .find(|record| {
            record.composition.split('+').any(|slug| slug == "zero")
                && !record.composition.split('+').any(|slug| slug == "mint")
                && !record.composition.split('+').any(|slug| slug == "adler")
        })
        .unwrap();
    let fireflies_team = compositions
        .compositions
        .iter()
        .find(|record| record.composition == "iroi+mint+sakiri+zero")
        .unwrap();
    let mut request = default_acceptance_request();
    request.screening_seeds = vec![7, 9];
    // fireflies.all_in_range_v1 (2026-10-14): the 4-target scenario now screens
    // successfully under the declared all-in-range rule instead of failing on
    // the removed proximity guard. The nominal damage must still equal the mean
    // of the per-seed runs (no invented baseline).
    let fireflies_summary = screen_single_composition(
        fireflies_team,
        AcceptanceScenarioKind::FourTargetReal120,
        &request.screening_seeds,
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(fireflies_summary.seeds, vec![7, 9]);
    assert!(fireflies_summary.events_processed > 0);
    let mut fireflies_sum = ExactFraction::new(0, 1).unwrap();
    for seed in &request.screening_seeds {
        let scenario = build_pilot_scenario(
            &fireflies_summary.composition,
            fireflies_summary.scenario,
            Ratio {
                numerator: 1,
                denominator: 1,
            },
            *seed,
            None,
        )
        .unwrap();
        let run: crate::simulation::GameScenarioReport = serde_json::from_value(
            crate::simulation::run_game_scenario(&scenario, &ExecutionLimits::default()).unwrap(),
        )
        .unwrap();
        fireflies_sum = fireflies_sum.add(run.total_damage_exact).unwrap();
    }
    assert_eq!(
        fireflies_summary.nominal_damage_exact,
        fireflies_sum
            .div(ExactFraction::new(2, 1).unwrap())
            .unwrap()
    );
    for kind in &request.scenarios {
        let summary = screen_single_composition(
            scoring_team,
            *kind,
            &request.screening_seeds,
            &ExecutionLimits::default(),
        )
        .unwrap();
        let mut sum = ExactFraction::new(0, 1).unwrap();
        let mut events = 0;
        for seed in &request.screening_seeds {
            let scenario = build_pilot_scenario(
                &summary.composition,
                summary.scenario,
                Ratio {
                    numerator: 1,
                    denominator: 1,
                },
                *seed,
                None,
            )
            .unwrap();
            let run: crate::simulation::GameScenarioReport = serde_json::from_value(
                crate::simulation::run_game_scenario(&scenario, &ExecutionLimits::default())
                    .unwrap(),
            )
            .unwrap();
            sum = sum.add(run.total_damage_exact).unwrap();
            events += run.total_events_processed;
        }
        assert_eq!(
            summary.nominal_damage_exact,
            sum.div(ExactFraction::new(2, 1).unwrap()).unwrap()
        );
        assert_eq!(summary.events_processed, events);
        assert_eq!(summary.seeds, vec![7, 9]);
    }
}

#[test]
fn changed_semantics_and_tampered_receipts_cannot_resume_or_rank_failed_executions() {
    let (coverage, conditions, variants) = load_test_fixtures();
    let mut request = default_acceptance_request();
    request.max_compositions_to_evaluate = Some(1);
    request.top_candidates_per_scenario = 0;
    request.finalist_seeds = vec![11];
    let report = run_acceptance_pipeline(
        &coverage,
        &conditions,
        &variants,
        &request,
        &ExecutionLimits::default(),
    )
    .unwrap();
    request.resume_checkpoint = Some(report.checkpoint.clone());
    request.screening_seeds = vec![99];
    assert!(
        run_acceptance_pipeline(
            &coverage,
            &conditions,
            &variants,
            &request,
            &ExecutionLimits::default()
        )
        .unwrap_err()
        .contains("identity")
    );
    request.screening_seeds = vec![42];
    request
        .resume_checkpoint
        .as_mut()
        .unwrap()
        .screening_results
        .values_mut()
        .next()
        .expect("the accepted single-composition screening result is retained")
        .result_sha256
        .push_str(" tampered");
    assert!(
        run_acceptance_pipeline(
            &coverage,
            &conditions,
            &variants,
            &request,
            &ExecutionLimits::default()
        )
        .unwrap_err()
        .contains("corrupted")
    );
    request.resume_checkpoint = None;
    let limits = ExecutionLimits {
        max_events: 9,
        ..ExecutionLimits::default()
    };
    let failed =
        run_acceptance_pipeline(&coverage, &conditions, &variants, &request, &limits).unwrap();
    assert!(!failed.checkpoint.is_complete);
    assert!(failed.finalists.is_empty());
    assert!(!failed.checkpoint.screening_failures.is_empty());
}
