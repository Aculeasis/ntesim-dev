use super::*;
use serde_json::{Value, json};
const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-cycles.json"
));
const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-cycles.nte"
));
const CHECKS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-cycle-checks.json"
));
fn checks() -> Value {
    serde_json::from_slice(CHECKS).unwrap()
}

#[test]
fn successful_execution_with_resource_debt_fails_declared_finite_invariants() {
    let report = run(FIXTURE, SOURCE, CHECKS).unwrap();
    assert!(report.simulation.simulation.success);
    assert!(report.complete);
    assert!(!report.passed);
    assert_eq!(report.boundaries.len(), 4);
    assert_eq!(report.intervals.len(), 3);
    assert_eq!(report.estimated_max_events, 20);
    assert!(report.boundaries[0].resource_deltas.is_none());
    for boundary in &report.boundaries[1..] {
        assert_eq!(boundary.resource_deltas.as_ref().unwrap()["energy"], "-1")
    }
    assert_eq!(report.diagnostics.len(), 3);
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| d.code == "RESOURCE_RELATION")
    );
    assert!(
        report
            .intervals
            .iter()
            .all(|i| i.actual_millionths == "10000000" && i.error_millionths == "0")
    );
}

#[test]
fn explicit_replenishment_satisfies_only_the_requested_boundary_relations() {
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    fixture["actors"]["hero"]["actions"]["skill"]["effects"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"gain","pool":"energy","amount":1}));
    let bytes = serde_json::to_vec(&fixture).unwrap();
    let report = run(&bytes, SOURCE, CHECKS).unwrap();
    assert!(report.complete && report.passed);
    assert!(report.diagnostics.is_empty());
    assert!(report.boundaries.iter().all(|b| b.resources["energy"] == 3));
    assert_eq!(report.simulation.simulation.targets["first"].hp(), 140);
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(&run(&bytes, SOURCE, CHECKS).unwrap()).unwrap()
    );
}

#[test]
fn phase_tolerance_is_explicit_and_fractional_error_is_never_discarded() {
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    fixture["clocks"][0]["rate_ppm"] = json!(500_001);
    fixture["duration_us"] = json!(61);
    let bytes = serde_json::to_vec(&fixture).unwrap();
    let mut request = checks();
    request["resources"]["energy"]["relation"] = json!("unconstrained");
    let report = run(&bytes, SOURCE, &serde_json::to_vec(&request).unwrap()).unwrap();
    assert!(report.complete);
    assert!(!report.passed);
    assert!(
        report
            .intervals
            .iter()
            .all(|i| i.actual_millionths == "10000020" && i.error_millionths == "20")
    );
    assert_eq!(report.boundaries[3].total_phase_millionths, "30000060");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| d.code == "CYCLE_INTERVAL")
    );
    request["max_phase_error_millionths"] = json!(20);
    assert!(
        run(&bytes, SOURCE, &serde_json::to_vec(&request).unwrap())
            .unwrap()
            .passed
    );
    request["max_phase_error_millionths"] = json!(19);
    assert!(
        !run(&bytes, SOURCE, &serde_json::to_vec(&request).unwrap())
            .unwrap()
            .passed
    );
}

#[test]
fn resource_floors_equal_relations_and_exact_boundary_counts_are_independent_checks() {
    let mut request = checks();
    request["resources"]["energy"] = json!({"minimum":1,"relation":"unconstrained"});
    let report = run(FIXTURE, SOURCE, &serde_json::to_vec(&request).unwrap()).unwrap();
    assert!(report.complete && !report.passed);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].code, "RESOURCE_FLOOR");
    assert!(report.diagnostics[0].path.contains("observations/3"));
    request["resources"]["energy"] = json!({"minimum":0,"relation":"equal"});
    let report = run(FIXTURE, SOURCE, &serde_json::to_vec(&request).unwrap()).unwrap();
    assert_eq!(report.diagnostics.len(), 3);
    request["resources"] = json!({});
    request["expected_observations"] = json!(5);
    let report = run(FIXTURE, SOURCE, &serde_json::to_vec(&request).unwrap()).unwrap();
    assert!(!report.complete && !report.passed);
    assert_eq!(report.diagnostics[0].code, "OBSERVATION_COUNT");
    request["label"] = json!("missing_label");
    let report = run(FIXTURE, SOURCE, &serde_json::to_vec(&request).unwrap()).unwrap();
    assert_eq!(report.observed_observations, 0);
    assert!(!report.passed);
}

#[test]
fn failed_execution_and_pending_boundaries_never_pass_even_with_available_snapshots() {
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    fixture["duration_us"] = json!(41);
    let mut request = checks();
    request["resources"] = json!({});
    let report = run(
        &serde_json::to_vec(&fixture).unwrap(),
        &SOURCE.replace("repeat 3", "repeat 4"),
        &serde_json::to_vec(&request).unwrap(),
    )
    .unwrap();
    assert_eq!(report.observed_observations, 4);
    assert!(!report.complete && !report.passed);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.code == "SIMULATION_INCOMPLETE")
    );
    assert_eq!(
        report.simulation.simulation.diagnostics[0].code,
        "ACTION_ADMISSION"
    );
    fixture["duration_us"] = json!(1);
    let report = run(&serde_json::to_vec(&fixture).unwrap(), SOURCE, CHECKS).unwrap();
    assert!(report.simulation.simulation.success);
    assert_eq!(report.observed_observations, 1);
    assert!(!report.complete && !report.passed);
}

#[test]
fn request_validation_and_three_input_identity_remain_auditable() {
    for invalid in [
        "schema",
        "label",
        "clock",
        "interval",
        "count",
        "resource",
        "relation",
        "unknown_field",
    ] {
        let mut request = checks();
        match invalid {
            "schema" => request["schema_version"] = json!(2),
            "label" => request["label"] = json!(" "),
            "clock" => request["clock"] = json!(9),
            "interval" => request["interval_us"] = json!(0),
            "count" => request["expected_observations"] = json!(1),
            "resource" => request["resources"]["missing"] = json!({"minimum":0,"relation":"equal"}),
            "relation" => request["resources"]["energy"]["relation"] = json!("probably_stable"),
            _ => request["typo"] = json!(true),
        }
        assert!(
            run(FIXTURE, SOURCE, &serde_json::to_vec(&request).unwrap()).is_err(),
            "{invalid}"
        );
    }
    let first = run(FIXTURE, SOURCE, CHECKS).unwrap();
    let mut request = checks();
    request["max_phase_error_millionths"] = json!(1);
    let second = run(FIXTURE, SOURCE, &serde_json::to_vec(&request).unwrap()).unwrap();
    assert_ne!(first.input_sha256, second.input_sha256);
    assert_ne!(first.checks_sha256, second.checks_sha256);
    assert_eq!(
        first.simulation.simulation.input_sha256,
        second.simulation.simulation.input_sha256
    );
    assert!(
        run(FIXTURE, SOURCE, &vec![b' '; 1_048_577])
            .unwrap_err()
            .contains("1 MiB")
    );
}
