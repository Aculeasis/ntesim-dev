use super::*;

pub(super) fn source(rotation: &str) -> String {
    format!(
        "options duration=10 crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0; target hp=100000000 def=0 resist=0; active mint; {rotation}"
    )
}

pub(super) fn request() -> Request {
    Request {
        schema_version: 1,
        seeds: vec![42, 43],
        max_events_per_run: 1000,
        max_total_events: 10000,
        max_receipt_bytes: 8 * 1024 * 1024,
    }
}

#[test]
fn relative_rates_and_zero_timer_kills_keep_raw_completion_separate_from_comparison() {
    let source = "options duration=120 crit=fixed; hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120; target hp=1000000000 def=0 resist=0; active hotori; hotori burst;";
    let normal = batch(source, &request()).unwrap();
    assert!(normal.complete);
    assert_eq!(
        normal.mean_dps.unwrap(),
        normal
            .mean_damage
            .unwrap()
            .div(ExactFraction::new(120, 1).unwrap())
            .unwrap()
    );
    for run in &normal.runs {
        let execution = run.execution.as_ref().unwrap();
        assert_eq!(execution.report.absolute_elapsed_us, 134_183_333);
        assert_eq!(execution.report.relative_elapsed_us, 120_000_000);
    }
    let early = source.replace("hp=1000000000", "hp=1");
    let undefined = batch(&early, &request()).unwrap();
    assert!(!undefined.complete && undefined.mean_dps.is_none());
    for run in &undefined.runs {
        let execution = run.execution.as_ref().unwrap();
        assert!(execution.report.success && execution.report.execution_completed);
        assert_eq!(execution.report.relative_elapsed_us, 0);
        assert_eq!(execution.report.absolute_elapsed_us, 2_316_080);
        assert!(execution.report.total_damage_dealt > 0.);
        assert!(dps(execution).unwrap_err().contains("zero relative"));
    }
    let comparison = compare(&early, &early, &request()).unwrap();
    assert!(comparison.mean_dps_delta.is_none());
}

#[test]
fn batch_keeps_exact_native_seed_trials_and_never_averages_a_failed_prefix() {
    let source = source("mint attack;");
    let request = request();
    let report = batch(&source, &request).unwrap();
    assert!(report.complete);
    assert_eq!(report.eligible_runs, 2);
    assert_eq!(
        report.mean_dps.unwrap(),
        report
            .mean_damage
            .unwrap()
            .div(ExactFraction::new(10, 1).unwrap())
            .unwrap()
    );
    for row in &report.runs {
        let expected = request.prepare(&source).unwrap().run(row.seed).unwrap();
        assert_eq!(
            encode(row.execution.as_ref().unwrap()).unwrap(),
            encode(&expected).unwrap()
        );
    }
    let failed = batch(&source.replace("mint attack;", "mint burst;"), &request).unwrap();
    assert!(!failed.complete && failed.mean_damage.is_none());
    assert_eq!(failed.runs.len(), 2);
    assert!(
        failed.runs[0]
            .execution
            .as_ref()
            .unwrap()
            .diagnostic
            .is_some()
    );
    let mut limited = request.clone();
    limited.max_events_per_run = 1;
    let report = batch(&source, &limited).unwrap();
    assert!(!report.complete && report.mean_dps.is_none());
    assert!(
        report.runs[0]
            .execution
            .as_ref()
            .unwrap()
            .report
            .limit_reached
            .is_some()
    );
    limited.max_receipt_bytes = 1;
    assert!(batch(&source, &limited).unwrap().runs.is_empty());
    limited = request.clone();
    limited.max_total_events = limited.max_events_per_run;
    let report = batch(&source, &limited).unwrap();
    assert_eq!(report.runs.len(), 1);
    assert!(!report.complete && report.mean_damage.is_none());
    for source in [
        source.replace("duration=10", "duration=1 stop=victory"),
        source.replace("mint attack;", "while 1==1 { let x=1; }"),
    ] {
        let failed = batch(&source, &request).unwrap();
        assert!(!failed.complete && failed.eligible_runs == 0 && failed.mean_dps.is_none());
    }
    assert!(
        batch(
            &source.replace("crit=fixed", "crit=fixed mode=strict"),
            &request
        )
        .is_err()
    );
}

#[test]
fn comparisons_pair_exact_signed_deltas_and_reject_context_changes() {
    let left = source("mint attack;");
    let right = source("mint attack:2;");
    let request = request();
    let comparison = compare(&left, &right, &request).unwrap();
    assert!(comparison.complete);
    let positive = comparison.mean_damage_delta.unwrap();
    assert!(!positive.numerator.starts_with('-'));
    let reverse = compare(&right, &left, &request).unwrap();
    let negative = reverse.mean_damage_delta.unwrap();
    assert_eq!(negative.numerator, format!("-{}", positive.numerator));
    assert_eq!(negative.denominator, positive.denominator);
    assert_eq!(
        SignedFraction::difference(
            ExactFraction::new(u128::MAX, 1).unwrap(),
            ExactFraction::new(0, 1).unwrap()
        )
        .unwrap()
        .numerator,
        u128::MAX.to_string()
    );
    assert!(
        compare(&left, &right.replace("def=0", "def=1"), &request)
            .unwrap_err()
            .contains("contexts differ")
    );
    let failed = compare(&left, &source("mint burst;"), &request).unwrap();
    assert!(!failed.complete && failed.mean_damage_delta.is_none());
    assert!(failed.pairs.iter().all(|pair| pair.damage_delta.is_none()));
    let mut duplicate = request;
    duplicate.seeds = vec![42, 42];
    assert!(batch(&left, &duplicate).is_err());
}

#[test]
fn comparisons_bind_authored_incoming_laws_even_when_the_initial_samples_match() {
    let script = source("mint attack;").replace(
        "active mint;",
        "hurt every interval=20s,30s amount=1,5; active mint;",
    );
    let changed = script.replace("30s", "40s");
    let request = request();
    for source in [&script, &changed] {
        assert!(
            request
                .prepare(source)
                .unwrap()
                .export()
                .request
                .incoming_damage_events
                .is_empty()
        );
    }
    assert!(
        compare(&script, &changed, &request)
            .unwrap_err()
            .contains("contexts differ")
    );
    let other_seed = script.replace("duration=10", "duration=10 seed=999");
    assert!(compare(&script, &other_seed, &request).unwrap().complete);
}
