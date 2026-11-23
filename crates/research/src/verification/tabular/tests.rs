use super::*;

#[test]
fn default_tabular_battery_passes_all_cases() {
    let report = run_tabular_verification(None).unwrap();

    assert_eq!(report.schema_version, 2);
    assert_eq!(report.report_kind, "tabular-verification-v2");
    assert_eq!(report.status, "INCOMPLETE");
    assert!(!report.game_dps_computed);
    assert!(!report.acceptance_ready);
    assert_eq!(report.total_cases, 6);
    assert_eq!(report.failed_cases, 0);
    assert_eq!(report.passed_cases, 6);
    assert!(report.exact_matches >= 2);
    assert_eq!(report.documented_discrepancies.len(), 3);

    for cr in &report.case_results {
        assert!(cr.passed, "Case {} did not pass", cr.case.case_id);
        assert!(cr.non_crit_diff_pct <= cr.case.tolerance_pct);
        if let Some(cd) = cr.crit_diff_pct {
            assert!(cd <= cr.case.tolerance_pct);
        }
        let saved: TabularCaseResult =
            serde_json::from_slice(&serde_json::to_vec(cr).unwrap()).unwrap();
        let replay = evaluate_case(&saved.case).unwrap();
        assert_eq!(
            serde_json::to_value(replay).unwrap(),
            serde_json::to_value(saved).unwrap()
        );
    }
}

#[test]
fn negative_resistance_discrepancy_is_accurately_isolated() {
    let cases = default_tabular_cases();
    let standard_case = cases
        .iter()
        .find(|c| c.case_id == "drive_calc_negative_res_standard")
        .unwrap();
    let community_case = cases
        .iter()
        .find(|c| c.case_id == "drive_calc_negative_res_conflict")
        .unwrap();

    let standard_res = evaluate_case(standard_case).unwrap();
    let community_res = evaluate_case(community_case).unwrap();

    assert!(standard_res.passed);
    assert!(community_res.passed);

    // Community model is ~1.30% higher at R = -0.20 (1.2987%)
    let delta_pct = ((community_res.engine_non_crit - standard_res.engine_non_crit)
        / standard_res.engine_non_crit)
        * 100.0;
    assert!((delta_pct - 1.2987).abs() < 1e-3);
}

#[test]
fn zero_reference_and_invalid_tolerance_never_hide_nonzero_damage() {
    let mut case = default_tabular_cases().remove(0);
    case.expected_non_crit = 0.0;
    assert!(
        evaluate_case(&case)
            .unwrap_err()
            .contains("zero non-crit reference")
    );
    case.expected_non_crit = -1.0;
    assert!(evaluate_case(&case).is_err());
    assert!(run_tabular_verification(Some(&[])).is_err());
}

#[test]
fn calibration_tolerance_status_and_aggregate_agree_even_below_old_exact_threshold() {
    let mut case = default_tabular_cases().remove(1);
    case.expected_non_crit = 1920.000001;
    case.tolerance_pct = 0.0;
    for documented in [false, true] {
        case.discrepancy_note = documented.then(|| "Deliberately incompatible reference".into());
        let report = run_tabular_verification(Some(&[case.clone()])).unwrap();
        let result = &report.case_results[0];
        assert!(!result.passed);
        assert!(result.non_crit_diff_pct > 0.0 && result.non_crit_diff_pct < 1e-4);
        assert_eq!(report.passed_cases, 0);
        assert_eq!(report.failed_cases, usize::from(!documented));
        assert_eq!(report.documented_discrepancy_cases, usize::from(documented));
        assert_eq!(
            result.status,
            if documented {
                VerificationCaseStatus::DiscrepancyDocumented
            } else {
                VerificationCaseStatus::Failed
            }
        );
    }
    case.tolerance_pct = 1e-6;
    let result = evaluate_case(&case).unwrap();
    assert!(result.passed);
    assert_eq!(result.status, VerificationCaseStatus::WithinTolerance);
    case.expected_non_crit = 1920.0;
    case.expected_crit = Some(3840.000001);
    case.tolerance_pct = 0.0;
    assert!(!evaluate_case(&case).unwrap().passed);
}

#[test]
fn calibration_context_and_unique_identities_are_required() {
    let case = default_tabular_cases().remove(0);
    let mut value = serde_json::to_value(&case).unwrap();
    for field in [
        "case_id",
        "description",
        "reference",
        "applicability",
        "tolerance_rationale",
    ] {
        value[field] = " ".into();
        let bad: TabularTestCase = serde_json::from_value(value.clone()).unwrap();
        assert!(evaluate_case(&bad).is_err(), "{field}");
        value[field] = serde_json::to_value(&case).unwrap()[field].clone();
    }
    assert!(
        run_tabular_verification(Some(&[case.clone(), case]))
            .unwrap_err()
            .contains("duplicate")
    );
}
