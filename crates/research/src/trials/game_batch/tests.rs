use super::*;

#[test]
fn game_batch_completion_uses_stop_outcome_instead_of_pending_effect_counts() {
    for (options, hp, complete) in [
        ("duration=5", 100_000_000, true),
        ("duration=5 stop=victory", 1, true),
        ("duration=5 stop=victory", 100_000_000, false),
    ] {
        let source = format!(
            "options {options} iteration=1 crit=random;
            hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
            target hp={hp} def=0 resist=0; active hotori; hotori burst;"
        );
        let trial = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &Default::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        let request = serde_json::json!({"schema_version":1,"seeds":[42],
            "timing_policy":"scale_recovery_and_hit_delay_v1","timing_hypothesis":"Unscaled native source stop outcome",
            "timing_multipliers":["1/1"],
            "max_events_per_run":1000,"max_total_events":1000,"max_run_receipt_bytes":1000000});
        let batch = run(
            &serde_json::to_vec(&trial.request).unwrap(),
            &serde_json::to_vec(&request).unwrap(),
        )
        .unwrap();
        assert_eq!(batch.complete, complete);
        assert_eq!(batch.timing_statistics.is_some(), complete);
        if hp == 1 || !complete {
            assert!(batch.runs[0].pending_events.unwrap() > 0);
        } else {
            // Five relative seconds include the full stop and drain its finite
            // train. Completion remains valid with no pending work, too.
            assert_eq!(batch.runs[0].pending_events, Some(0));
        }
    }
}
fn inputs() -> (Vec<u8>, Vec<u8>) {
    let scenario = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/research-crit.json"
    ))
    .to_vec();
    let request = serde_json::json!({"schema_version":1,"seeds":[42,43,0],"timing_policy":"scale_recovery_and_hit_delay_v1","timing_hypothesis":"Test caller timing sensitivity","timing_multipliers":["1/1","11/10","6/5"],"max_events_per_run":100,"max_total_events":900,"max_run_receipt_bytes":100000});
    (scenario, serde_json::to_vec(&request).unwrap())
}
#[test]
fn seed_timing_batch_replays_and_retains_incomplete_quality() {
    let (scenario, request) = inputs();
    let report = run(&scenario, &request).unwrap();
    assert!(report.complete);
    assert_eq!(report.requested_runs, 9);
    assert_eq!(report.eligible_runs, 9);
    assert_eq!(report.timing_statistics.as_ref().unwrap().len(), 3);
    assert!(!report.acceptance_ready && !report.game_dps_computed);
    let encoded = serde_json::to_vec(&report).unwrap();
    assert_eq!(
        encoded,
        serde_json::to_vec(&run(&scenario, &request).unwrap()).unwrap()
    );
    for rows in report.runs.chunks(3) {
        assert_eq!(
            rows.iter().map(|row| row.seed).collect::<Vec<_>>(),
            vec![42, 43, 0]
        );
    }
    assert!(report.events_processed as u64 <= report.estimated_max_events);
}

#[test]
fn seed_timing_aggregates_and_failed_receipts_keep_the_same_model_assumptions() {
    let (scenario, request) = inputs();
    let mut scenario: Value = serde_json::from_slice(&scenario).unwrap();
    scenario["model_assumptions"] =
        serde_json::json!({"test.coefficient_v1":"Explicit partial coefficient model"});
    let report = run(&serde_json::to_vec(&scenario).unwrap(), &request).unwrap();
    assert!(report.complete);
    assert_eq!(
        serde_json::to_value(report.model_assumptions).unwrap(),
        scenario["model_assumptions"]
    );
    scenario["actors"][0]["initial_energy"] = serde_json::json!(0);
    let report = run(&serde_json::to_vec(&scenario).unwrap(), &request).unwrap();
    assert!(!report.complete && report.timing_statistics.is_none());
    assert_eq!(
        serde_json::to_value(report.model_assumptions).unwrap(),
        scenario["model_assumptions"]
    );
    for row in report.runs {
        assert_eq!(
            row.partial_report.unwrap()["model_assumptions"],
            scenario["model_assumptions"]
        );
    }
}
#[test]
fn time_truncation_aggregates_while_failures_and_receipt_limits_withhold_statistics() {
    let (scenario, request) = inputs();
    let original: Value = serde_json::from_slice(&scenario).unwrap();
    for pending in [false, true] {
        let mut value = original.clone();
        if pending {
            value["actions"][2]["hit_delay_us"] = serde_json::json!(30_000_000);
        } else {
            value["actors"][0]["initial_energy"] = serde_json::json!(0);
        }
        let report = run(&serde_json::to_vec(&value).unwrap(), &request).unwrap();
        assert_eq!(report.complete, pending);
        assert_eq!(report.timing_statistics.is_some(), pending);
        assert!(
            report
                .runs
                .iter()
                .all(|row| row.partial_report.is_some() != pending)
        );
        if pending {
            assert!(
                report
                    .runs
                    .iter()
                    .all(|row| row.pending_events.unwrap() > 0)
            );
        }
    }
    let mut config: Value = serde_json::from_slice(&request).unwrap();
    config["max_run_receipt_bytes"] = serde_json::json!(1);
    let report = run(&scenario, &serde_json::to_vec(&config).unwrap()).unwrap();
    assert!(report.runs.is_empty() && report.timing_statistics.is_none());
    assert!(report.halt_diagnostic.unwrap().contains("receipt budget"));
}
#[test]
fn invalid_domains_and_work_budgets_fail_before_running() {
    let (scenario, request) = inputs();
    let original: Value = serde_json::from_slice(&request).unwrap();
    for (key, value) in [
        ("seeds", serde_json::json!([1, 1])),
        ("max_total_events", serde_json::json!(899)),
        ("timing_hypothesis", serde_json::json!("")),
        ("timing_multipliers", serde_json::json!(["1/0"])),
        ("timing_multipliers", serde_json::json!(["1/2"])),
        ("timing_multipliers", serde_json::json!(["1/1", "2/2"])),
    ] {
        let mut config = original.clone();
        config[key] = value;
        assert!(
            run(&scenario, &serde_json::to_vec(&config).unwrap()).is_err(),
            "{key}"
        );
    }
}
#[test]
fn timing_rounding_changes_only_recovery_and_hit_delays() {
    let (scenario, _) = inputs();
    let mut req: GameScenarioRequest = serde_json::from_slice(&scenario).unwrap();
    if let GameActionCommand::InvokeAbility {
        recovery_us,
        hit_delay_us,
        ..
    } = &mut req.actions[0]
    {
        *recovery_us = 1;
        *hit_delay_us = 1;
    }
    let changed = transform_timing(
        &req,
        Ratio {
            numerator: 11,
            denominator: 10,
        },
    )
    .unwrap();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        recovery_us,
        hit_delay_us,
        cooldown_us,
        ..
    } = changed.actions[0]
    {
        assert_eq!(
            (timestamp_us, recovery_us, hit_delay_us, cooldown_us),
            (0, 2, 2, 0)
        );
    } else {
        panic!("expected ability");
    }
    assert_eq!(changed.relative_duration_us, req.relative_duration_us);
    assert_eq!(changed.seed, req.seed);
}
#[test]
fn engine_errors_preserve_rows_without_invented_damage_or_event_counts() {
    let (scenario, request) = inputs();
    let mut changed: Value = serde_json::from_slice(&scenario).unwrap();
    changed["actors"][0]
        .as_object_mut()
        .unwrap()
        .remove("purchased_qte_level");
    changed["actors"][0]["character_level"] = serde_json::json!(0);
    let report = run(&serde_json::to_vec(&changed).unwrap(), &request).unwrap();
    assert_eq!(report.runs.len(), 9);
    assert!(!report.complete && !report.event_counts_complete);
    assert!(report.timing_statistics.is_none());
    for row in report.runs {
        assert!(row.total_damage_exact.is_none() && row.events_processed.is_none());
        assert!(row.halt_diagnostic.unwrap().contains("attacker level"));
    }
}

#[test]
fn event_caps_stop_without_processing_an_extra_event() {
    let (scenario, request) = inputs();
    let mut changed: Value = serde_json::from_slice(&request).unwrap();
    changed["max_events_per_run"] = serde_json::json!(3);
    let report = run(&scenario, &serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(!report.complete && report.timing_statistics.is_none());
    for row in report.runs {
        assert_eq!(row.events_processed, Some(3));
        assert!(row.pending_events.unwrap() > 0);
        assert!(row.partial_report.is_some());
    }
}

#[test]
fn preparation_errors_still_produce_each_seed_row() {
    let (scenario, request) = inputs();
    let mut scenario: Value = serde_json::from_slice(&scenario).unwrap();
    scenario["actors"][0]["talent_levels"] = serde_json::json!([0, 1, 1]);
    scenario["actors"][0]["character_max_level"] = serde_json::json!(80);
    let scenario_bytes = serde_json::to_vec(&scenario).unwrap();
    let report = run(&scenario_bytes, &request).unwrap();
    assert_eq!(report.runs.len(), 9);
    assert!(!report.complete && !report.event_counts_complete);
    assert!(report.timing_statistics.is_none());
    for row in report.runs {
        let mut seeded: GameScenarioRequest = serde_json::from_value(scenario.clone()).unwrap();
        seeded.seed = row.seed;
        let error = ntesim_content::simulation::run_game_scenario_report(
            &seeded,
            &ExecutionLimits::default(),
        )
        .unwrap_err();
        assert_eq!(row.halt_diagnostic, Some(error));
        assert!(row.events_processed.is_none() && row.total_damage_exact.is_none());
    }
}

#[test]
fn exact_statistics_match_hand_calculated_fractions() {
    let stats = statistics(&[
        ExactFraction::new(1, 2).unwrap(),
        ExactFraction::new(3, 2).unwrap(),
        ExactFraction::new(5, 2).unwrap(),
    ])
    .unwrap();
    assert_eq!(stats.mean, ExactFraction::new(3, 2).unwrap());
    assert_eq!(stats.approximate_sample_variance, Some(1.0));
    assert!((stats.approximate_standard_error.unwrap() - (1.0f64 / 3.0).sqrt()).abs() < 1e-15);
    let single = statistics(&[ExactFraction::new(1, 3).unwrap()]).unwrap();
    assert!(single.approximate_sample_variance.is_none());
}
