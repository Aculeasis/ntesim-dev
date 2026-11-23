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
const CANDIDATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-candidate.nte"
));
const REQUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-seeds.json"
));

#[test]
fn identical_variants_have_exact_zero_deltas_and_replay_identically() {
    let report = run(FIXTURE, SOURCE, FIXTURE, SOURCE, REQUEST).unwrap();
    assert!(report.success && report.complete);
    assert_eq!(report.eligible_pairs, 8);
    assert_eq!(report.estimated_max_events, 192);
    for pair in &report.pairs {
        assert!(
            pair.target_deltas.as_ref().unwrap()["first"]
                .values()
                .all(|value| value == "0")
        );
    }
    for stats in report.paired_statistics.as_ref().unwrap()["first"].values() {
        assert_eq!(stats.mean_numerator, "0");
        assert_eq!(stats.approximate_sample_variance, Some(0.0));
    }
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(&run(FIXTURE, SOURCE, FIXTURE, SOURCE, REQUEST).unwrap()).unwrap()
    );
}

#[test]
fn paired_differences_are_signed_and_use_matching_seed_rows() {
    let report = run(FIXTURE, SOURCE, FIXTURE, CANDIDATE, REQUEST).unwrap();
    assert!(report.complete);
    assert_eq!(report.direction, "candidate-minus-baseline");
    let mut sum = 0_i128;
    for (index, pair) in report.pairs.iter().enumerate() {
        let left = &report.baseline.runs[index];
        let right = &report.candidate.runs[index];
        assert_eq!(left.seed, pair.seed);
        assert_eq!(right.seed, pair.seed);
        let deltas = &pair.target_deltas.as_ref().unwrap()["first"];
        let damage = deltas["hp_lost"].parse::<i128>().unwrap();
        assert!(matches!(damage, -10 | -25));
        assert_eq!(deltas["final_hp"].parse::<i128>().unwrap(), -damage);
        assert_eq!(
            damage,
            right.target_metrics["first"]["hp_lost"]
                .parse::<i128>()
                .unwrap()
                - left.target_metrics["first"]["hp_lost"]
                    .parse::<i128>()
                    .unwrap()
        );
        sum += damage;
    }
    let stats = &report.paired_statistics.as_ref().unwrap()["first"]["hp_lost"];
    assert_eq!(stats.sum, sum.to_string());
    assert_eq!(stats.mean_denominator, 8);
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    for outcome in fixture["actors"]["hero"]["actions"]["skill"]["effects"][0]["outcomes"]
        .as_array_mut()
        .unwrap()
    {
        outcome["amount"] = json!(outcome["amount"].as_u64().unwrap() + 5);
    }
    let report = run(
        FIXTURE,
        SOURCE,
        &serde_json::to_vec(&fixture).unwrap(),
        SOURCE,
        REQUEST,
    )
    .unwrap();
    let stats = &report.paired_statistics.as_ref().unwrap()["first"]["hp_lost"];
    assert_eq!(stats.mean_numerator, "120");
    assert_eq!(stats.mean_denominator, 8);
    assert_eq!(stats.approximate_sample_variance, Some(0.0));
}

#[test]
fn mixed_failures_and_pending_horizons_preserve_both_reports_without_ranking() {
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    fixture["targets"]["first"]["hp"] = json!(30);
    fixture["targets"]["first"]["max_hp"] = json!(30);
    let bytes = serde_json::to_vec(&fixture).unwrap();
    let request =
        serde_json::to_vec(&json!({"schema_version":1,"seeds":(0..64).collect::<Vec<_>>()}))
            .unwrap();
    let report = run(&bytes, SOURCE, &bytes, CANDIDATE, &request).unwrap();
    assert!(!report.success && !report.complete);
    assert!(report.candidate.complete);
    assert!(report.eligible_pairs > 0 && report.eligible_pairs < 64);
    assert!(report.paired_statistics.is_none());
    assert!(report.pairs.iter().any(|pair| pair.target_deltas.is_none()));
    assert!(
        report
            .baseline
            .runs
            .iter()
            .any(|r| !r.diagnostics.is_empty())
    );
    fixture["duration_us"] = json!(1);
    let bytes = serde_json::to_vec(&fixture).unwrap();
    let report = run(&bytes, SOURCE, &bytes, CANDIDATE, REQUEST).unwrap();
    assert!(report.success);
    assert!(!report.complete);
    assert_eq!(report.eligible_pairs, 0);
    assert!(report.paired_statistics.is_none());
}

#[test]
fn incompatible_context_and_combined_work_budget_fail_before_execution() {
    for field in [
        "duration_us",
        "mode_clock",
        "clocks",
        "targets",
        "rng_streams",
    ] {
        let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
        match field {
            "duration_us" => fixture[field] = json!(31),
            "mode_clock" => {
                fixture[field] = json!(1);
                fixture["clocks"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"id":1,"rate_ppm":1000000}));
            }
            "clocks" => fixture[field][0]["rate_ppm"] = json!(500000),
            "targets" => fixture[field]["first"]["hp"] = json!(199),
            _ => fixture[field]["outcomes"] = json!(55),
        }
        assert!(
            run(
                FIXTURE,
                SOURCE,
                &serde_json::to_vec(&fixture).unwrap(),
                SOURCE,
                REQUEST
            )
            .unwrap_err()
            .contains(field)
        );
    }
    let request =
        serde_json::to_vec(&json!({"schema_version":1,"seeds":(0..1000).collect::<Vec<_>>()}))
            .unwrap();
    let source = "clock 0; repeat 126 { do attack; wait 10; }";
    assert!(
        run(FIXTURE, source, FIXTURE, source, &request)
            .unwrap_err()
            .contains("combined events")
    );
}

#[test]
fn signed_uncertainty_uses_difference_samples_and_preserves_large_small_changes() {
    let stats = signed_statistics(&[-2, -4, -4, -4, -5, -5, -7, -9]).unwrap();
    assert_eq!(stats.sum, "-40");
    assert_eq!(stats.minimum, "-9");
    assert_eq!(stats.maximum, "-2");
    assert_eq!(stats.approximate_mean, -5.0);
    assert!((stats.approximate_sample_variance.unwrap() - 32.0 / 7.0).abs() < 1e-12);
    let anchor = -i128::from(u64::MAX);
    assert_eq!(
        signed_statistics(&[anchor - 1, anchor, anchor + 1])
            .unwrap()
            .approximate_sample_variance,
        Some(1.0)
    );
    assert_eq!(
        signed_statistics(&[-7]).unwrap().approximate_standard_error,
        None
    );
    assert!(signed_statistics(&[]).is_err());
    assert!(signed_statistics(&[i128::MIN, i128::MAX]).is_err());
    assert!(signed_statistics(&[i128::MAX, 1]).is_err());
}

#[test]
fn identity_binds_variant_order_source_bytes_and_request() {
    let report = run(FIXTURE, SOURCE, FIXTURE, CANDIDATE, REQUEST).unwrap();
    let swapped = run(FIXTURE, CANDIDATE, FIXTURE, SOURCE, REQUEST).unwrap();
    assert_ne!(report.input_sha256, swapped.input_sha256);
    let altered = run(
        FIXTURE,
        &(SOURCE.to_owned() + "# comment\n"),
        FIXTURE,
        CANDIDATE,
        REQUEST,
    )
    .unwrap();
    assert_ne!(report.input_sha256, altered.input_sha256);
    assert_ne!(report.baseline.input_sha256, altered.baseline.input_sha256);
    assert_eq!(
        report.candidate.input_sha256,
        altered.candidate.input_sha256
    );
}
