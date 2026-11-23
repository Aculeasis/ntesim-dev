use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn source(actions: &str) -> String {
    format!(
        "options duration=60 crit=random seed=42;
        chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        target hp=100000000 def=1080 resist=0; active chaos; {actions}"
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

fn completed(result: &SourceScenarioReport) {
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(result);
}

#[test]
fn source_chaos_combo_window_keeps_caused_pause_and_cancels_only_future_normal_requests() {
    let result = run(&source(
        "chaos attack:2; chaos attack[window=hold]; chaos attack[hold=200ms];",
    ));
    completed(&result);
    assert_eq!(result.occurrences[2].completed_us, Some(2_113_639));
    assert_eq!(result.occurrences[3].started_us, 2_113_639);
    assert_eq!(result.report.hit_receipts.len(), 12);
    let hits: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.target_index == 0)
        .map(|hit| hit.time_us)
        .collect();
    assert_eq!(
        hits,
        [214_573, 756_182, 918_306, 2_035_883, 2_280_330, 2_997_297]
    );
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["current"],
        serde_json::json!("98/1")
    );

    let observed = run(&source("chaos attack:2; chaos attack[window=hold];"));
    let baseline = run(&source("chaos attack:3; if 0 {chaos attack[hold=200ms];}"));
    completed(&observed);
    super::assert_same_combat(&observed.report, &baseline.report);
}

#[test]
fn source_chaos_verdict_window_reaches_post_recovery_continuations_in_two_funded_cycles() {
    let result = run(include_str!(
        "../../../../../examples/chaos-verdict-continuations.nte"
    ));
    completed(&result);
    // Lead1s + stage1 .898334 + strict full-stage1.060001 + .370506 caused pause.
    assert_eq!(result.occurrences[0].completed_us, Some(3_328_841));
    assert_eq!(result.occurrences[1].started_us, 3_328_841);
    assert_eq!(result.report.hit_receipts.len(), 168);
    let value = serde_json::to_value(&result.report).unwrap();
    let crime = &value["final_actor_states"][0]["fractional_resources"]["chaos.crime"];
    // Six chains grant6*382; each full stage spends1000 once across both dummies.
    for (field, amount) in [
        ("granted", 2292),
        ("spent", 2000),
        ("overflow", 292),
        ("current", 1000),
    ] {
        assert_eq!(
            crime[field],
            serde_json::json!(format!("{}/{}", amount, 1)),
            "{field}"
        );
    }
    assert_eq!(result.request.hold_inputs.len(), 4);
}

#[test]
fn source_chaos_continuation_keeps_the_old_form_gate_but_uses_the_live_new_row() {
    let text = source(
        "chaos burst; wait_until(14); chaos attack[hold=1,window=hold]; chaos attack[hold=100ms];",
    )
    .replace("start_energy=0", "start_energy=100 start_crime=1000");
    let result = run(&text);
    completed(&result);
    // Transformed stage1 .899451, full predicate1.060001 and two .185253 pauses.
    // The parent captures form at14s; form naturally expires at14.718888s.
    assert_eq!(result.occurrences[2].completed_us, Some(17_329_958));
    let command = result.occurrences[3].command_indices[0];
    let hits: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.command_index == Some(command) && hit.target_index == 0)
        .collect();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].damage_effect_id, "GE_Player_Chaos_Branch1_Damage");
    assert_eq!(hits[0].time_us, 17_496_649);
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["spent"],
        serde_json::json!("1000/1")
    );
}

#[test]
fn source_chaos_hold_window_does_not_wait_for_a_longer_authored_hold_or_allow_idle_short_holds() {
    let result = run(&source(
        "chaos attack[window=hold]; chaos attack[hold=400ms];",
    ));
    assert!(!result.report.execution_completed);
    assert_eq!(result.occurrences[0].completed_us, Some(319_801));
    let rejected = result.report.rejected_action.as_ref().unwrap();
    assert_eq!(rejected.time_us, 319_801);
    assert!(matches!(
        rejected.reason,
        crate::simulation::GameAdmissionReason::Recovery { until_us: 400_000 }
    ));
    super::assert_native_replay(&result);
    let idle = run(&source("chaos attack[hold=200ms];"));
    assert!(!idle.report.execution_completed);
    assert_eq!(idle.occurrences[0].completed_us, None);
    super::assert_native_replay(&idle);
}
