use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

const SOURCE: &str = "options duration=50 crit=random seed=42 start_fons=1234;
    chiz char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_grains=0 start_loans=0 start_energy=0;
    target hp=100000000 def=0 resist=0;
    target hp=100000000 def=0 resist=0; active chiz;
    let stocks=2;
    repeat 2 {
        if .chiz.resources.skill_stocks!=3 {let assertion_failure=1/0;}
        chiz skill[stocks=stocks,hold=10s];
        wait(12);
        if .chiz.resources.loans!=15000 || .chiz.resources.skill_stocks!=3 || .team.fons!=1234 {let assertion_failure=1/0;}
        stocks=3;
    }";

fn run(source: &str) -> SourceScenarioReport {
    let prepared =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    PreparedSourceScenario::from_export(
        &serde_json::from_slice(&serde_json::to_vec(&prepared.export()).unwrap()).unwrap(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
}

#[test]
fn source_chiz_barren_binds_tap_and_live_normal_combo_with_locked_secondary_exclusion() {
    let result = run(include_str!(
        "../../../../../examples/chiz-barren-inputs.nte"
    ));
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.report.hit_receipts.len(), 20);
    assert_eq!(result.request.hold_inputs.len(), 4);
    for (index, tap) in [(0, true), (3, false), (5, true), (8, false)] {
        let occurrence = &result.occurrences[index];
        let command = occurrence.command_indices[0];
        let hits: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.command_index == Some(command))
            .collect();
        assert_eq!(hits.len(), 4);
        let first = if tap { 1_249_325 } else { 749_310 };
        assert_eq!(hits[0].target_index, 1);
        assert_eq!(hits[0].time_us, occurrence.started_us + first);
        assert_eq!(hits[1].target_index, 0);
        assert_eq!(hits[1].time_us, occurrence.started_us + first + 1_214);
        assert_eq!(hits[2].time_us, occurrence.started_us + first + 715_228);
        assert_eq!(hits[3].time_us, hits[2].time_us);
    }
    super::assert_native_replay(&result);

    // The cue settles before same-time source observation. Jump then cancels
    // the branch's final AoE, without erasing its admitted2.5s tap limiter.
    let failed_source = include_str!("../../../../../examples/chiz-barren-inputs.nte")
        .split("repeat 2")
        .next()
        .unwrap()
        .to_string()
        + "chiz attack[hold=0,window=jump], jump, attack[hold=0];";
    let failed = run(&failed_source);
    assert!(!failed.report.success && !failed.report.execution_completed);
    assert_eq!(failed.occurrences[0].completed_us, Some(1_368_275));
    assert_eq!(failed.report.hit_receipts.len(), 2);
    let value = serde_json::to_value(&failed.report).unwrap();
    assert_eq!(value["rejected_action"]["reason"]["until_us"], 2_500_000);
    super::assert_native_replay(&failed);
}

#[test]
fn source_chiz_separate_third_input_buffers_held_full_without_terminal_grain_spend() {
    let result = run(include_str!(
        "../../../../../examples/chiz-buffered-held-full.nte"
    ));
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.occurrences[0].completed_us, Some(954_108));
    assert_eq!(result.occurrences[1].started_us, 954_108);
    assert_eq!(result.report.hit_receipts.len(), 40);
    for (index, offset) in [(1, 0), (4, 15_000_000)] {
        let command = result.occurrences[index].command_indices[0];
        let full: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| {
                hit.command_index == Some(command)
                    && matches!(
                        hit.damage_effect_id.as_str(),
                        "GE_Player_Chiichan073_Skill3_Damage"
                            | "GE_Player_Chiichan073_Skill4_Damage"
                    )
            })
            .collect();
        assert_eq!(full.len(), 8);
        assert_eq!(full[0].time_us, offset + 2_833_348);
    }
    assert!(
        !result
            .report
            .trace
            .iter()
            .any(|event| event.event_kind == "chiz_held_terminal_spend")
    );
    let value = serde_json::to_value(&result.report).unwrap();
    // Each one-pulse loop borrows1/16 of current headroom:15000/16, then
    // (15000-15000/16)/16. The continuing debt is58125/32, not two fresh pools.
    assert_eq!(
        value["final_actor_states"][0]["fractional_resources"]["chiz.loan_debt"]["current"],
        serde_json::json!("58125/32")
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_chiz_pressed_continuation_cancels_half_tail_and_keeps_actual_hitlag_between_inputs() {
    let result = run("options duration=22 crit=random seed=42;
        chiz char lvl=80/80 awakening=0 talent=8,8,8 qte=8
            start_grains=0 start_loans=0 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        target hp=100000000 def=1080 resist=0; active chiz;
        let end=9;
        repeat 2 {
            chiz skill[window=skill], skill[window=skill], skill;
            wait_until(end); end=end+9;
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.occurrences[0].completed_us, Some(954_108));
    assert_eq!(result.occurrences[1].completed_us, Some(1_661_079));
    assert_eq!(result.report.hit_receipts.len(), 28);
    for period in 0..2 {
        let hits: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.target_index == 0 && hit.time_us / 9_000_000 == period)
            .map(|hit| hit.time_us - period * 9_000_000)
            .collect();
        assert_eq!(
            hits,
            [
                183_298, 779_558, 1_591_936, 1_913_252, 2_412_335, 2_606_316, 2_770_758
            ]
        );
    }
    super::assert_native_replay(&result);
}

#[test]
fn source_chiz_reserves_native_stocks_and_continues_refills_and_debt_between_cycles() {
    let result = run(SOURCE);
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(
        result
            .request
            .hold_inputs
            .iter()
            .map(|(index, input)| (*index, input.duration_us, input.reserved_stocks))
            .collect::<Vec<_>>(),
        vec![(0, 10_000_000, Some(2)), (2, 10_000_000, Some(3))]
    );
    let full = |command| {
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| {
                hit.command_index == Some(command)
                    && matches!(
                        hit.damage_effect_id.as_str(),
                        "GE_Player_Chiichan073_Skill3_Damage"
                            | "GE_Player_Chiichan073_Skill4_Damage"
                    )
            })
            .count()
    };
    assert_eq!(full(0), 0);
    assert_eq!(full(2), 8);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.command_index == Some(0))
            .count(),
        72
    );
    super::assert_native_replay(&result);
    let scheduled=crate::dsl::compiler::compile_scenario(&SOURCE.replace("if .chiz.resources.skill_stocks!=3 {let assertion_failure=1/0;}","")
        .replace("if .chiz.resources.loans!=15000 || .chiz.resources.skill_stocks!=3 || .team.fons!=1234 {let assertion_failure=1/0;}",""), Default::default()).unwrap().request;
    assert_eq!(
        scheduled
            .hold_inputs
            .values()
            .map(|input| input.reserved_stocks)
            .collect::<Vec<_>>(),
        vec![Some(2), Some(3)]
    );
}

#[test]
fn source_stock_types_missing_duration_and_native_capacity_errors_remain_distinct() {
    let prefix = "options duration=15 crit=random;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_grains=0;
        target hp=100000000 def=0 resist=0; active chiz;";
    for (params, message) in [
        ("stocks=3", "stocks require an explicit hold duration"),
        ("hold=10,stocks=2.5", "stocks:"),
    ] {
        let result = crate::dsl::source::compile_response(&crate::dsl::compiler::SourceRequest {
            source: format!("{prefix} chiz skill[{params}];"),
            limits: Default::default(),
        });
        assert!(result.compiled.is_none());
        assert!(
            result.diagnostics[0].message.contains(message),
            "{:?}",
            result.diagnostics
        );
    }
    for stocks in [0, 1, 4] {
        let result = run(&format!("{prefix} chiz skill[hold=10,stocks={stocks}];"));
        assert!(!result.report.execution_completed);
        assert!(result.request.actions.is_empty());
        assert!(result.request.hold_inputs.is_empty());
        assert!(
            result
                .diagnostic
                .unwrap()
                .message
                .contains("reserves exactly2 or3")
        );
    }
}

#[test]
fn source_chiz_skill_declares_the_native_default_stock_model_before_initial_observation() {
    let result=run("options duration=12 crit=random;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=100000000 def=0 resist=0; active chiz;
        if .chiz.resources.skill_stocks!=3 || .chiz.resources.grains || .chiz.resources.loans {let assertion_failure=1/0;}
        chiz skill[hold=0];
        if .chiz.resources.skill_stocks!=2 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(&result);
}
