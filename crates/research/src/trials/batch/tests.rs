use super::*;
use serde_json::{Value, json};
const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-random.json"
));
const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-random.nte"
));
const REQUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-seeds.json"
));

#[test]
fn statistics_match_independent_variance_and_large_integer_oracles() {
    let result = statistics(&[2, 4, 4, 4, 5, 5, 7, 9]).unwrap();
    assert_eq!(result.count, 8);
    assert_eq!(result.sum, "40");
    assert_eq!(result.minimum, "2");
    assert_eq!(result.maximum, "9");
    assert_eq!(result.approximate_mean, 5.0);
    assert!((result.approximate_sample_variance.unwrap() - 32.0 / 7.0).abs() < 1e-12);
    assert!((result.approximate_standard_error.unwrap() - (4.0_f64 / 7.0).sqrt()).abs() < 1e-12);
    let anchor = u128::from(u64::MAX);
    let large = statistics(&[anchor - 1, anchor, anchor + 1]).unwrap();
    assert_eq!(large.sum, (anchor * 3).to_string());
    assert_eq!(large.approximate_sample_variance, Some(1.0));
    assert_eq!(statistics(&[7]).unwrap().approximate_standard_error, None);
    assert_eq!(
        statistics(&[7, 7]).unwrap().approximate_sample_variance,
        Some(0.0)
    );
    assert!(statistics(&[]).is_err());
    assert!(statistics(&[u128::MAX, 1]).is_err());
}

#[test]
fn every_seed_replays_full_report_and_aggregate_matches_independent_sum() {
    let report = run(FIXTURE, SOURCE, REQUEST).unwrap();
    assert!(report.success && report.complete);
    assert_eq!(report.eligible_runs, 8);
    assert_eq!(report.estimated_max_events, 96);
    assert_eq!(
        report.runs.iter().map(|r| r.seed).collect::<Vec<_>>(),
        (42..50).collect::<Vec<_>>()
    );
    let mut sum = 0_u128;
    for entry in &report.runs {
        let single = rotation::run_seeded(FIXTURE, SOURCE, entry.seed).unwrap();
        assert_eq!(
            entry.report_sha256,
            sha256(&serde_json::to_vec(&serde_json::to_value(&single).unwrap()).unwrap())
        );
        assert_eq!(entry.input_sha256, single.simulation.input_sha256);
        let damage: u128 = single
            .simulation
            .trace
            .iter()
            .filter_map(|e| e.changes.get("random_damage"))
            .map(|e| u128::from(e["amount"].as_u64().unwrap()))
            .sum();
        assert_eq!(entry.target_metrics["first"]["hp_lost"], damage.to_string());
        assert_eq!(
            entry.target_metrics["first"]["offered_damage"],
            damage.to_string()
        );
        assert_eq!(single.simulation.targets["first"].hp(), 200 - damage as u64);
        sum += damage;
    }
    let stats = &report.target_statistics.as_ref().unwrap()["first"]["hp_lost"];
    assert_eq!(stats.sum, sum.to_string());
    assert_eq!(stats.mean_numerator, sum.to_string());
    assert_eq!(stats.mean_denominator, 8);
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(&run(FIXTURE, SOURCE, REQUEST).unwrap()).unwrap()
    );
}

#[test]
fn mixed_seed_failures_are_retained_and_withhold_all_statistics() {
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    fixture["targets"]["first"]["hp"] = json!(30);
    fixture["targets"]["first"]["max_hp"] = json!(30);
    let request =
        serde_json::to_vec(&json!({"schema_version":1,"seeds":(0..64).collect::<Vec<_>>()}))
            .unwrap();
    let report = run(&serde_json::to_vec(&fixture).unwrap(), SOURCE, &request).unwrap();
    assert!(!report.success && !report.complete);
    assert!(report.eligible_runs > 0 && report.eligible_runs < 64);
    assert_eq!(report.runs.len(), 64);
    assert!(report.target_statistics.is_none());
    let failed = report.runs.iter().find(|r| !r.success).unwrap();
    assert!(!failed.diagnostics.is_empty());
    assert_eq!(failed.target_metrics["first"]["hp_lost"], "30");
    assert!(failed.diagnostics[0].message.contains("DeadRecipient"));
}

#[test]
fn successful_horizon_with_pending_commands_is_not_a_complete_sample() {
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    fixture["duration_us"] = json!(1);
    let report = run(&serde_json::to_vec(&fixture).unwrap(), SOURCE, REQUEST).unwrap();
    assert!(report.success);
    assert!(!report.complete);
    assert_eq!(report.eligible_runs, 0);
    assert!(report.target_statistics.is_none());
    for entry in &report.runs {
        assert_eq!(entry.pending_events, 3);
        assert_eq!(entry.target_metrics["first"]["offered_damage"], "0");
    }
}

#[test]
fn gross_hp_loss_shield_absorption_and_overkill_are_distinct_metrics() {
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    fixture["actors"] = json!({});
    fixture["targets"]["first"]["hp"] = json!(10);
    fixture["targets"]["first"]["max_hp"] = json!(20);
    fixture["actions"] = json!({
        "shield":[{"kind":"shield","target":"first","id":"s","source":"a","amount":5}],
        "attack":[{"kind":"damage","target":"first","amount":8,"hit_delay_us":1}],
        "heal":[{"kind":"heal","target":"first","amount":7}]
    });
    let report = run(
        &serde_json::to_vec(&fixture).unwrap(),
        "clock 0; do shield; do attack; wait 2; do heal; do attack;",
        REQUEST,
    )
    .unwrap();
    assert!(report.complete);
    let metric = &report.runs[0].target_metrics["first"];
    assert_eq!(metric["final_hp"], "6");
    assert_eq!(metric["hp_lost"], "11");
    assert_eq!(metric["shield_absorbed"], "5");
    assert_eq!(metric["offered_damage"], "16");
    assert_eq!(metric["overkill"], "0");
    fixture["actions"]["attack"][0]["amount"] = json!(25);
    let report = run(
        &serde_json::to_vec(&fixture).unwrap(),
        "clock 0; do attack;",
        REQUEST,
    )
    .unwrap();
    assert!(report.complete);
    assert_eq!(report.runs[0].target_metrics["first"]["hp_lost"], "10");
    assert_eq!(report.runs[0].target_metrics["first"]["overkill"], "15");
}

#[test]
fn invalid_seed_requests_and_preflight_work_limits_fail() {
    for request in [
        json!({"schema_version":2,"seeds":[1]}),
        json!({"schema_version":1,"seeds":[]}),
        json!({"schema_version":1,"seeds":[1,1]}),
        json!({"schema_version":1,"seeds":[-1]}),
        json!({"schema_version":1,"seeds":[1],"typo":true}),
        json!({"schema_version":1,"seeds":(0..1001).collect::<Vec<_>>()}),
    ] {
        assert!(run(FIXTURE, SOURCE, &serde_json::to_vec(&request).unwrap()).is_err());
    }
    let request =
        serde_json::to_vec(&json!({"schema_version":1,"seeds":(0..1000).collect::<Vec<_>>()}))
            .unwrap();
    assert!(
        run(
            FIXTURE,
            "clock 0; repeat 251 { do attack; wait 10; }",
            &request
        )
        .unwrap_err()
        .contains("estimated events")
    );
    assert!(
        run(FIXTURE, SOURCE, &vec![b' '; 1_048_577])
            .unwrap_err()
            .contains("1 MiB")
    );
}

#[test]
fn batch_identity_binds_exact_bytes_order_and_seed_override() {
    let first = run(FIXTURE, SOURCE, REQUEST).unwrap();
    let changed = run(FIXTURE, &(SOURCE.to_owned() + "# changed\n"), REQUEST).unwrap();
    assert_ne!(first.input_sha256, changed.input_sha256);
    assert_ne!(first.rotation_sha256, changed.rotation_sha256);
    let reversed = br#"{"schema_version":1,"seeds":[49,48,47,46,45,44,43,42]}"#;
    assert_ne!(
        first.input_sha256,
        run(FIXTURE, SOURCE, reversed).unwrap().input_sha256
    );
    let original = rotation::run(FIXTURE, SOURCE).unwrap();
    let overridden = rotation::run_seeded(FIXTURE, SOURCE, 42).unwrap();
    assert_ne!(
        original.simulation.input_sha256,
        overridden.simulation.input_sha256
    );
    assert!(original.rotation.seed_override.is_none());
    assert_eq!(overridden.rotation.seed_override, Some(42));
    assert_eq!(original.simulation.targets, overridden.simulation.targets);
}
