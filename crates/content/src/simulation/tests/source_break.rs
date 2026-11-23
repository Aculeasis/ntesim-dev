use crate::dsl::validation::DiagnosticCode;
use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn source(actions: &str) -> String {
    format!(
        "# 🧪\noptions duration=4 crit=random seed=42;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=100000000 def=1080 resist=0.2;
        target hp=100000000 def=1080 resist=0.2
            break_gauge=30 max_break_gauge=30 broken_duration=1;
        active mint; {actions}"
    )
}

fn run(source: &str) -> SourceScenarioReport {
    let prepared =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let prepared = PreparedSourceScenario::from_export(
        &serde_json::from_slice(&serde_json::to_vec(&prepared.export()).unwrap()).unwrap(),
        &Default::default(),
    )
    .unwrap();
    prepared.run(42).unwrap()
}

#[test]
fn source_break_continues_exact_gauge_recovery_and_all_living_contributors() {
    let result = run(include_str!(
        "../../../../../examples/manual-break-cycles.nte"
    ));
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
    let hits = &result.report.hit_receipts;
    assert_eq!(hits.len(), 6);
    for hit in hits {
        assert_eq!(hit.target_index, 1);
        // lv80 3603 * (capacity30/3) * DEF .5 * RES .8; no ATK/crit/energy.
        assert_eq!(hit.damage, super::ExactFraction::new(14412, 1).unwrap());
        assert!(hit.break_contribution.is_some());
        assert!(!hit.critical);
        assert!(hit.crit_sample.is_none());
    }
    assert_eq!(
        hits.iter().map(|hit| hit.time_us).collect::<Vec<_>>(),
        vec![0, 0, 1_000_000, 1_000_000, 2_000_000, 2_000_000]
    );
    assert_eq!(result.report.final_target_states[0].hp_current, 100_000_000);
    assert_eq!(
        result.report.final_target_states[1].hp_current,
        100_000_000 - 86472
    );
    let bar = result.report.final_target_states[1]
        .break_state
        .as_ref()
        .unwrap();
    assert_eq!(bar.generation, 3);
    assert_eq!(bar.gauge, super::ExactFraction::new(30, 1).unwrap());
    assert_eq!(bar.broken_until_us, None);
    for actor in &result.report.final_actor_states {
        assert_eq!(actor.spent_energy, 0);
        assert_eq!(actor.granted_energy, 0);
    }
    let manual: Vec<_> = result
        .request
        .actions
        .iter()
        .filter_map(|command| {
            if let super::GameActionCommand::TargetGauge { gauge, .. } = command {
                Some(*gauge)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(manual.len(), 6);
    assert_eq!((manual[0].numerator, manual[0].denominator), (59, 2));
    assert_eq!(result.occurrences[0].completed_us, Some(0));
    assert_eq!(result.occurrences[1].completed_us, Some(0));
}

#[test]
fn source_break_rejects_live_increases_repeated_break_and_defeated_targets_without_retry() {
    for (actions, message, failed) in [
        (
            "target t1 break_gauge=1/3; target t1 break_gauge=1/2;",
            "cannot increase",
            1,
        ),
        ("target t1 break; target t1 break;", "already Broken", 1),
    ] {
        let result = run(&source(actions));
        assert!(!result.report.success && !result.report.execution_completed);
        assert!(
            result
                .diagnostic
                .as_ref()
                .unwrap()
                .message
                .contains(message)
        );
        assert_eq!(result.occurrences[failed].completed_us, None);
        assert_eq!(result.occurrences[failed].started_us, 0);
        super::assert_native_replay(&result);
    }
    let result = run(
        &source("target t1 break; wait(1); target t1 break;").replace(
            "target hp=100000000 def=1080 resist=0.2\n",
            "target hp=1 def=1080 resist=0.2\n",
        ),
    );
    assert!(!result.report.success);
    assert!(
        result
            .diagnostic
            .as_ref()
            .unwrap()
            .message
            .contains("defeated")
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_break_validates_unvisited_targets_profiles_and_exact_scalars_with_utf16_ranges() {
    for (actions, token) in [
        ("if 0 {target t2 break;}", "t2"),
        ("if 0 {target t01 break;}", "t01"),
        ("if 0 {target t0 break;}", "t0"),
        ("if 0 {target t1 break_gauge=-1;}", "-"),
        ("if 0 {target t1 break_gauge=true;}", "true"),
    ] {
        let text = source(actions);
        let errors = PreparedSourceScenario::new(&text, Default::default(), &Default::default())
            .err()
            .unwrap();
        assert_eq!(errors[0].code, DiagnosticCode::Configuration);
        let loc = &errors[0].loc;
        assert_eq!(
            String::from_utf16(
                &text
                    .encode_utf16()
                    .skip(loc.offset)
                    .take(loc.length)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            token
        );
    }
}

#[test]
fn source_break_external_input_preserves_unfinished_montage_and_native_rng() {
    let text = source("blackbird attack[window=hold]; target t1 break_gauge=29.5; wait(1);")
        .replace(
            "active mint;",
            "blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1; active blackbird;",
        );
    let result = run(&text);
    assert!(result.report.success, "{:?}", result.diagnostic);
    assert_eq!(result.occurrences[1].started_us, 214_249);
    assert_eq!(result.occurrences[1].completed_us, Some(214_249));
    let baseline = run(&text.replace("target t1 break_gauge=29.5;", ""));
    let hits = serde_json::to_value(&result.report.hit_receipts).unwrap();
    assert!(hits == serde_json::to_value(&baseline.report.hit_receipts).unwrap());
    super::assert_native_replay(&result);
}
