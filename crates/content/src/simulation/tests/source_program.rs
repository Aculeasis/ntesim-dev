use crate::dsl::compiler::CompileLimits;
use crate::execution::ExecutionLimits;
use crate::simulation::{PreparedSourceScenario, SourceOccurrenceKind, SourceScenarioReport};

const PREFIX: &str = "options duration=20 iteration=1 crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0; target hp=100000000 def=0 resist=0; active mint;";

fn run(rotation: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(
        &format!("{PREFIX}{rotation}"),
        CompileLimits::default(),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
}

#[test]
fn source_loops_use_native_recovery_and_keep_continuing_combat_state() {
    let result = run(
        "let n=0; repeat 2 {mint attack:5; n=n+1; wait(1);} if n==2 && 0.1+0.2==0.3 {mint skill;}",
    );
    assert!(result.report.success, "{:?}", result.diagnostic);
    assert_eq!(
        result
            .occurrences
            .iter()
            .filter(|o| o.kind == SourceOccurrenceKind::Action)
            .count(),
        11
    );
    assert_eq!(result.action_locations.len(), result.request.actions.len());
    let direct =
        crate::simulation::run_game_scenario_report(&result.request, &ExecutionLimits::default())
            .unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let replay = serde_json::to_value(direct).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert!(value == &replay[key], "replay field {key} differs");
    }
}

#[test]
fn source_live_time_branch_and_energy_gate_compose_in_one_trial() {
    let result = run(
        "repeat 2 {mint attack; wait_for_energy(1); if .time>0 && .actor.energy>=1 {wait(0.1+0.2);} else {wait(9);} }",
    );
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.occurrences.len(), 6);
    assert_eq!(
        result.occurrences[2].completed_us.unwrap() - result.occurrences[2].started_us,
        300_000
    );
    assert_eq!(
        result.occurrences[5].completed_us.unwrap() - result.occurrences[5].started_us,
        300_000
    );
}

#[test]
fn source_sparse_waits_do_not_create_polls_and_empty_live_loops_fail() {
    let result = run("wait_until(19); if .time==19 {wait(0);}");
    assert!(result.report.success && result.report.execution_completed);
    assert_eq!(result.report.total_events_processed, 0);
    use crate::simulation::GameTimeCause;
    let intervals = &result.report.input_time.intervals;
    assert_eq!(intervals.len(), 2);
    assert_eq!(intervals[0].cause, GameTimeCause::WaitUntil);
    assert_eq!(intervals[0].duration_us, 19_000_000);
    assert_eq!(intervals[1].cause, GameTimeCause::ScriptExhaustion);
    assert_eq!(intervals[1].duration_us, 1_000_000);
    result.report.input_time.validate(20_000_000).unwrap();
    crate::simulation::tests::assert_native_replay(&result);
    let result = run("while .time<1 {}");
    assert!(!result.report.success && !result.report.execution_completed);
    crate::simulation::tests::assert_native_replay(&result);
    let diagnostic = result.diagnostic.unwrap();
    assert_eq!(
        diagnostic.code,
        crate::dsl::validation::DiagnosticCode::WorkLimit
    );
    assert_eq!(result.failure_context.unwrap().occurrence_index, 0);
    assert!(diagnostic.message.contains("budget"));
    assert!(diagnostic.message.contains("limit 60000 operations"));
    assert!(
        diagnostic
            .message
            .contains("independent of simulated combat time")
    );
    assert!(diagnostic.loc.length > 0);
}

#[test]
fn source_failure_export_stops_continuing_effects_and_rejects_invalid_boundaries() {
    let source = "options duration=30 crit=random; # 😀\n\
        jiuyuan char lvl=80/80 awakening=0 talent=1,1,1 qte=1;\
        target hp=100000000 def=0 resist=0; active jiuyuan;\
        jiuyuan skill; let n=1/(.actor.energy-.actor.energy);";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(!result.report.success && !result.report.execution_completed);
    assert!(result.report.pending_events > 0);
    assert!(result.report.absolute_elapsed_us < 30_000_000);
    crate::simulation::tests::assert_native_replay(&result);
    let halt = result.request.source_halt.as_ref().unwrap();
    assert_eq!(Some(&halt.diagnostic), result.diagnostic.as_ref());
    assert_eq!(Some(&halt.context), result.failure_context.as_ref());
    for variant in 0..6 {
        let mut request = result.request.clone();
        let halt = request.source_halt.as_mut().unwrap();
        match variant {
            0 => request.source_exhausted = true,
            1 => halt.time_us = request.relative_duration_us + 1,
            2 => halt.kind = crate::simulation::GameSourceHaltKind::Pending,
            3 => halt.context.command_index = Some(0),
            4 => halt.diagnostic.loc.length = 0,
            _ => halt.time_us = 0,
        }
        assert!(
            crate::simulation::run_game_scenario_report(&request, &Default::default()).is_err(),
            "variant {variant}"
        );
    }
}

#[test]
fn unresolved_energy_wait_remains_incomplete() {
    let result = run("wait_for_energy(100); mint burst;");
    assert!(result.report.success);
    assert!(!result.report.execution_completed);
    assert!(result.diagnostic.is_some());
    assert!(result.report.hit_receipts.is_empty());
    assert_eq!(result.occurrences.len(), 1);
    assert_eq!(result.occurrences[0].completed_us, None);
    crate::simulation::tests::assert_native_replay(&result);
    assert_eq!(
        result.diagnostic.unwrap().code,
        crate::dsl::validation::DiagnosticCode::EnergyGate
    );
    assert_eq!(result.failure_context.unwrap().occurrence_index, 0);
    result.report.input_time.validate(20_000_000).unwrap();
}

#[test]
fn source_short_circuits_and_live_arithmetic_failure_retains_source() {
    let result = run(
        "# 😀\r\nif 0 && .missing {mint burst;} if 1 || .missing {mint attack;} let n=1/(.actor.energy-.actor.energy); mint skill;",
    );
    assert!(!result.report.success);
    crate::simulation::tests::assert_native_replay(&result);
    assert_eq!(result.occurrences.len(), 1);
    let diagnostic = result.diagnostic.unwrap();
    assert_eq!(
        diagnostic.code,
        crate::dsl::validation::DiagnosticCode::Arithmetic
    );
    assert_eq!(result.failure_context.unwrap().occurrence_index, 1);
    let slash = result.source.find("1/(").unwrap() + 1;
    assert_eq!(
        diagnostic.loc.offset,
        result.source[..slash].encode_utf16().count()
    );
    assert_eq!(diagnostic.loc.length, 1);
    assert_eq!(diagnostic.loc.line, 2);
    let line_start = result.source[..slash].rfind('\n').unwrap() + 1;
    assert_eq!(
        diagnostic.loc.column,
        result.source[line_start..slash].encode_utf16().count() + 1
    );
    assert!(diagnostic.message.contains("zero"));
    assert!(diagnostic.loc.offset > PREFIX.len());
    assert_eq!(
        result.source,
        format!(
            "{PREFIX}# 😀\r\nif 0 && .missing {{mint burst;}} if 1 || .missing {{mint attack;}} let n=1/(.actor.energy-.actor.energy); mint skill;"
        )
    );
}

#[test]
fn source_seed_branches_read_settled_hits_without_consuming_randomness() {
    let prefix = PREFIX.replace("crit=fixed", "crit=random") + "mint add stats cr=0.45;";
    let probe = PreparedSourceScenario::new(
        &(prefix.clone() + "mint attack;"),
        Default::default(),
        &ExecutionLimits::default(),
    )
    .unwrap();
    let samples = (0..8)
        .map(|seed| {
            let result = probe.run(seed).unwrap();
            (
                seed,
                result.report.final_target_states[0].hp_current,
                result.report.rng,
            )
        })
        .collect::<Vec<_>>();
    let min = samples.iter().min_by_key(|(_, hp, _)| hp).unwrap();
    let max = samples.iter().max_by_key(|(_, hp, _)| hp).unwrap();
    assert!(min.1 < max.1);
    let threshold = min.1 + (max.1 - min.1) / 2 + 1;
    let source = format!(
        "{prefix}mint attack; if .target.hp<{threshold} {{wait(0.1);}} else {{wait(0.2);}}"
    );
    let program =
        PreparedSourceScenario::new(&source, Default::default(), &ExecutionLimits::default())
            .unwrap();
    for (sample, delay) in [(min, 100_000), (max, 200_000)] {
        let result = program.run(sample.0).unwrap();
        assert!(result.report.success);
        let branch = &result.occurrences[1];
        assert_eq!(branch.completed_us.unwrap() - branch.started_us, delay);
        assert_eq!(result.report.rng, sample.2);
    }
}

#[test]
fn source_admission_failure_keeps_the_repeated_occurrence_and_stops() {
    let result = run("repeat 2 {mint skill;} mint attack;");
    assert!(!result.report.success && !result.report.execution_completed);
    crate::simulation::tests::assert_native_replay(&result);
    assert_eq!(result.occurrences.len(), 2);
    assert_eq!(result.occurrences[0].loc, result.occurrences[1].loc);
    assert_eq!(result.occurrences[1].index, 1);
    assert_eq!(result.occurrences[1].completed_us, None);
    let diagnostic = result.diagnostic.unwrap();
    assert_eq!(diagnostic.loc, result.occurrences[1].loc);
    assert_eq!(
        diagnostic.code,
        crate::dsl::validation::DiagnosticCode::Admission
    );
    let context = result.failure_context.unwrap();
    assert_eq!(
        (context.occurrence_index, context.command_index),
        (1, Some(1))
    );
    assert!(matches!(
        result.report.rejected_action.as_ref().unwrap().reason,
        crate::simulation::GameAdmissionReason::Cooldown { .. }
    ));
    assert_eq!(result.report.rejected_action.unwrap().command_index, 1);
    let empty = run("mint burst;");
    crate::simulation::tests::assert_native_replay(&empty);
    assert_eq!(
        empty.diagnostic.unwrap().code,
        crate::dsl::validation::DiagnosticCode::Admission
    );
    assert!(matches!(
        empty.report.rejected_action.unwrap().reason,
        crate::simulation::GameAdmissionReason::Energy { available: 0, .. }
    ));
    assert_eq!(empty.report.absolute_elapsed_us, 0);
    assert!(empty.report.input_time.intervals.is_empty());
}

#[test]
fn source_native_plunge_deadlines_preserve_profiles_and_continuing_resources() {
    let source = "options duration=3 crit=fixed;
        chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=1000000 def=1080 resist=0; active chaos;
        chaos plunge[airtime=600ms,height_bonus=1/2]:2;";
    let result =
        PreparedSourceScenario::new(source, Default::default(), &ExecutionLimits::default())
            .unwrap()
            .run(42)
            .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.hits_settled, 2);
    assert_eq!(result.request.plunge_inputs.len(), 2);
    assert_eq!(result.report.final_actor_states[0].energy, 4);
    assert!(result.occurrences[1].started_us > 600_000);
}

#[test]
fn source_implicit_switches_use_native_support_and_retain_occurrence_ownership() {
    let source = "options duration=20 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 start_cycle=100;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 start_cycle=100;
        target hp=100000000 def=1080 resist=0; active mint;
        repeat 2 {mint attack; zero attack; wait(2);}";
    let result =
        PreparedSourceScenario::new(source, Default::default(), &ExecutionLimits::default())
            .unwrap()
            .run(42)
            .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(
        result
            .request
            .actions
            .iter()
            .filter(|command| matches!(
                command,
                crate::simulation::GameActionCommand::SwitchActor { .. }
            ))
            .count(),
        3
    );
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .any(|hit| hit.ability_id.ends_with("QTE"))
    );
    for occurrence in &result.occurrences {
        for index in &occurrence.command_indices {
            assert_eq!(result.action_locations[*index], occurrence.loc);
        }
    }
}

#[test]
fn source_action_and_native_event_limits_retain_the_completed_prefix() {
    let source = format!("{PREFIX}while 1 {{mint attack;}}");
    let program = PreparedSourceScenario::new(
        &source,
        CompileLimits {
            max_steps: 1000,
            max_actions: 2,
        },
        &ExecutionLimits::default(),
    )
    .unwrap();
    let result = program.run(42).unwrap();
    assert!(!result.report.success && !result.report.execution_completed);
    crate::simulation::tests::assert_native_replay(&result);
    assert_eq!(result.occurrences.len(), 2);
    assert!(
        result
            .occurrences
            .iter()
            .all(|occurrence| occurrence.completed_us.is_some())
    );
    let diagnostic = result.diagnostic.unwrap();
    assert_eq!(
        diagnostic.code,
        crate::dsl::validation::DiagnosticCode::ActionLimit
    );
    assert!(diagnostic.message.contains("limit 2 inputs"));
    assert_eq!(result.failure_context.unwrap().occurrence_index, 2);
    let source = format!("{PREFIX}mint attack;");
    let program = PreparedSourceScenario::new(
        &source,
        Default::default(),
        &ExecutionLimits {
            max_events: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let result = program.run(42).unwrap();
    assert!(!result.report.success && !result.report.execution_completed);
    assert_eq!(result.report.total_events_processed, 1);
    crate::simulation::tests::assert_native_replay(&result);
    assert_eq!(result.occurrences[0].completed_us, None);
    let diagnostic = result.diagnostic.unwrap();
    assert_eq!(
        diagnostic.code,
        crate::dsl::validation::DiagnosticCode::EventLimit
    );
    assert!(diagnostic.message.contains("limit 1 events processed"));
    assert_eq!(result.failure_context.unwrap().occurrence_index, 0);
    assert_eq!(
        result.report.limit_reached,
        Some(crate::simulation::GameExecutionLimit::EventCount { limit: 1 })
    );
    let mut corrupted = result.request.clone();
    corrupted.source_halt.as_mut().unwrap().kind =
        crate::simulation::GameSourceHaltKind::EventLimit { limit: 2 };
    assert!(crate::simulation::run_game_scenario_report(&corrupted, &Default::default()).is_err());

    // An event budget can also stop autonomous work after lexical exhaustion.
    let source = "options duration=30; jiuyuan char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active jiuyuan; jiuyuan skill;";
    let prefix = PreparedSourceScenario::new(
        &format!("{source} let n=1/(.actor.energy-.actor.energy);"),
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    let limit = prefix.report.total_events_processed as u64 + 1;
    // Choose a budget after the cast completes but before all continuing ticks.
    let result = PreparedSourceScenario::new(
        source,
        Default::default(),
        &ExecutionLimits {
            max_events: limit,
            ..Default::default()
        },
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(result.request.source_exhausted);
    assert!(!result.report.success);
    crate::simulation::tests::assert_native_replay(&result);
}

#[test]
fn source_gap_length_does_not_increase_scheduler_work() {
    let counts = [1, 599].map(|gap| {
        let source = format!("options duration=600; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=1000 def=0 resist=0; active mint; wait_until({gap}); if .time=={gap} {{wait(0);}}");
        let result = PreparedSourceScenario::new(&source, Default::default(), &ExecutionLimits::default()).unwrap().run(42).unwrap();
        assert!(result.report.success && result.report.execution_completed);
        result.report.total_events_processed
    });
    assert_eq!(counts, [0, 0]);
}

#[test]
fn source_command_budget_retains_an_admitted_implicit_switch_prefix() {
    let source =
        format!("{PREFIX}zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1; zero attack;");
    let program = PreparedSourceScenario::new(
        &source,
        CompileLimits {
            max_actions: 1,
            ..Default::default()
        },
        &Default::default(),
    )
    .unwrap();
    let result = program.run(42).unwrap();
    crate::simulation::tests::assert_native_replay(&result);
    assert!(!result.report.success && !result.report.execution_completed);
    let diagnostic = result.diagnostic.unwrap();
    assert_eq!(
        diagnostic.code,
        crate::dsl::validation::DiagnosticCode::ActionLimit
    );
    assert!(diagnostic.message.contains("limit 1 native commands"));
    let context = result.failure_context.unwrap();
    assert_eq!((context.occurrence_index, context.command_index), (0, None));
    assert_eq!(result.occurrences[0].command_indices, [0]);
    assert_eq!(result.request.actions.len(), 1);
    assert!(matches!(
        result.request.actions[0],
        crate::simulation::GameActionCommand::SwitchActor { .. }
    ));
    assert_eq!(result.report.final_buff_state.active_actor_index, 1);
    assert!(result.report.rejected_action.is_none());
}

#[test]
fn source_idle_warnings_use_contiguous_time_and_coalesce_source_occurrences() {
    use crate::simulation::GameTimeCause;
    for (rotation, expected) in [
        ("wait(4.999999);", None),
        ("repeat 2 {wait(2.5);}", Some(5_000_000)),
        ("repeat 4 {wait(2.5); mint attack;}", None),
    ] {
        let result = run(rotation);
        assert!(result.report.success && result.report.execution_completed);
        let warnings = &result.report.input_time.warnings;
        let wait = warnings
            .iter()
            .find(|warning| warning.cause == GameTimeCause::AuthoredWait);
        assert_eq!(wait.map(|warning| warning.total_duration_us), expected);
        if let Some(wait) = wait {
            assert_eq!(
                (wait.episodes, wait.longest_duration_us, wait.first_start_us),
                (1, 5_000_000, 0)
            );
        }
        assert!(warnings.len() <= 2);
        crate::simulation::tests::assert_native_replay(&result);
    }
}
