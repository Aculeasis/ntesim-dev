use super::*;

pub fn evaluate_case(test_case: &TabularTestCase) -> Result<TabularCaseResult, String> {
    if [
        &test_case.case_id,
        &test_case.description,
        &test_case.reference,
        &test_case.applicability,
        &test_case.tolerance_rationale,
    ]
    .iter()
    .any(|text| text.trim().is_empty())
        || test_case
            .discrepancy_note
            .as_ref()
            .is_some_and(|note| note.trim().is_empty())
    {
        return Err(
            "reference cases require nonempty identity, context and tolerance rationale".into(),
        );
    }
    if !test_case.expected_non_crit.is_finite()
        || test_case.expected_non_crit < 0.0
        || test_case
            .expected_crit
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        || !test_case.tolerance_pct.is_finite()
        || test_case.tolerance_pct < 0.0
    {
        return Err("reference damage and tolerance must be finite and nonnegative".into());
    }
    let mut stats = DynamicStats::new(test_case.base_atk, 10000.0, 500.0);
    stats.atk_percent = test_case.atk_percent;
    stats.atk_flat = test_case.atk_flat;
    stats.crit_rate = test_case.crit_rate;
    stats.crit_damage = test_case.crit_damage;
    stats.damage_bonus = test_case.damage_bonus;
    stats.defense_shred = test_case.defense_shred;
    stats.defense_ignore = test_case.defense_ignore;

    let hit = DirectHitSpec {
        base: None,
        ratio: Some(test_case.hit_ratio),
        stat: "atk".into(),
        flat: test_case.hit_flat,
        element: "Physical".into(),
        kind: DamageKind::Elemental,
        crit_rule: CritRule::Normal,
        action_type: None,
        damage_bonus: 0.0,
        crit_damage_bonus: 0.0,
        final_multiplier: 1.0,
        special_multiplier: 1.0,
    };

    let non_crit_hit = DirectHitSpec {
        crit_rule: CritRule::None,
        ..hit.clone()
    };
    let non_crit_req = DirectDamageEvaluationRequest {
        attacker_level: test_case.attacker_level,
        attacker_stats: &stats,
        target_defense: test_case.target_defense,
        target_resistance: test_case.target_resistance,
        target_vulnerability: 0.0,
        hit: &non_crit_hit,
        crit_mode: EvaluationCritMode::Expected,
        rounding: Rounding::Floor,
        negative_res_model: test_case.negative_res_model,
    };
    let non_crit_res = combat::evaluate_direct_damage(&non_crit_req)
        .map_err(|e| format!("non-crit evaluation failed: {e:?}"))?;
    let engine_non_crit = non_crit_res.raw_damage;

    let non_crit_diff = if test_case.expected_non_crit > 0.0 {
        ((engine_non_crit - test_case.expected_non_crit).abs() / test_case.expected_non_crit)
            * 100.0
    } else if engine_non_crit == 0.0 {
        0.0
    } else {
        return Err("zero non-crit reference differs from nonzero engine damage; percentage tolerance is undefined".into());
    };

    let (engine_crit, crit_diff) = if let Some(exp_crit) = test_case.expected_crit {
        let crit_hit = DirectHitSpec {
            crit_rule: CritRule::Fixed100,
            ..hit
        };
        let crit_req = DirectDamageEvaluationRequest {
            attacker_level: test_case.attacker_level,
            attacker_stats: &stats,
            target_defense: test_case.target_defense,
            target_resistance: test_case.target_resistance,
            target_vulnerability: 0.0,
            hit: &crit_hit,
            crit_mode: EvaluationCritMode::Expected,
            rounding: Rounding::Floor,
            negative_res_model: test_case.negative_res_model,
        };
        let crit_res = combat::evaluate_direct_damage(&crit_req)
            .map_err(|e| format!("crit evaluation failed: {e:?}"))?;
        let eng_c = crit_res.raw_damage;
        let c_diff = if exp_crit > 0.0 {
            ((eng_c - exp_crit).abs() / exp_crit) * 100.0
        } else if eng_c == 0.0 {
            0.0
        } else {
            return Err("zero crit reference differs from nonzero engine damage; percentage tolerance is undefined".into());
        };
        (Some(eng_c), Some(c_diff))
    } else {
        (None, None)
    };

    let max_diff = crit_diff
        .map(|cd| cd.max(non_crit_diff))
        .unwrap_or(non_crit_diff);
    let passed = max_diff <= test_case.tolerance_pct;

    let status = if max_diff == 0.0 {
        VerificationCaseStatus::ExactMatch
    } else if passed {
        VerificationCaseStatus::WithinTolerance
    } else if test_case.discrepancy_note.is_some() {
        VerificationCaseStatus::DiscrepancyDocumented
    } else {
        VerificationCaseStatus::Failed
    };

    Ok(TabularCaseResult {
        case: test_case.clone(),
        engine_non_crit,
        non_crit_diff_pct: non_crit_diff,
        engine_crit,
        crit_diff_pct: crit_diff,
        status,
        passed,
    })
}

pub fn run_tabular_verification(
    cases: Option<&[TabularTestCase]>,
) -> Result<TabularVerificationReport, String> {
    let default_cases = default_tabular_cases();
    let cases_to_run = cases.unwrap_or(&default_cases);

    if cases_to_run.is_empty() {
        return Err("tabular verification requires at least one reference case".into());
    }
    let mut case_results = Vec::with_capacity(cases_to_run.len());
    let mut exact_matches = 0;
    let mut within_tolerance_cases = 0;
    let mut documented_discrepancy_cases = 0;
    let mut failed_cases = 0;
    let mut identities = std::collections::BTreeSet::new();

    for tc in cases_to_run {
        if !identities.insert(&tc.case_id) {
            return Err(format!("duplicate reference case identity: {}", tc.case_id));
        }
        let res = evaluate_case(tc)?;
        match res.status {
            VerificationCaseStatus::ExactMatch => exact_matches += 1,
            VerificationCaseStatus::WithinTolerance => within_tolerance_cases += 1,
            VerificationCaseStatus::DiscrepancyDocumented => documented_discrepancy_cases += 1,
            VerificationCaseStatus::Failed => failed_cases += 1,
        }
        case_results.push(res);
    }

    let passed_cases = exact_matches + within_tolerance_cases;

    Ok(TabularVerificationReport {
        schema_version: 2,
        report_kind: "tabular-verification-v2".into(),
        content_id: "ntesim-current".into(),
        status: "INCOMPLETE".into(),
        quality: Quality::Incomplete,
        game_dps_computed: false,
        acceptance_ready: false,
        total_cases: cases_to_run.len(),
        passed_cases,
        exact_matches,
        within_tolerance_cases,
        documented_discrepancy_cases,
        failed_cases,
        case_results,
        documented_discrepancies: documented_discrepancies(),
        blockers: vec![
            "Direct in-game damage measurements are unavailable; formulas verified against community and independent oracles".into(),
            "Negative resistance model retains a known +1.30% conflict against Drive Calculator".into(),
            "This formula-only battery does not execute or calibrate rotation timelines; source/native acceptance is separate".into(),
        ],
        engine_version: ENGINE_VERSION.to_string(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.to_string(),
    })
}
