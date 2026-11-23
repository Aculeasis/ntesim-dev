use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_timepiece_start_and_off_field_queries_preserve_exact_native_units() {
    let result = run("options duration=11 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 start_timepiece=66+0.1;
        target hp=100000000 def=0 resist=0; active mint;
        if .hotori.resources.timepiece!=66.1 || !.hotori.skill.ready {let assertion_failure=1/0;}
        wait_until(0.000001);
        if .hotori.resources.timepiece!=66.100006 {let assertion_failure=1/0;}
        wait_until(1);
        if .hotori.resources.timepiece!=72.1 {let assertion_failure=1/0;}
        wait_until(10);
        if .hotori.resources.timepiece!=120 || !.hotori.burst.ready
            || .hotori.energy!=0 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.total_events_processed, 0);
    let pool = serde_json::to_value(&result.report.final_actor_states[1]).unwrap();
    assert_eq!(
        pool["fractional_resources"]["hotori.timepiece"]["initial"],
        serde_json::json!("661/10")
    );
    assert_eq!(
        pool["fractional_resources"]["hotori.timepiece"]["conserved"],
        true
    );
}

#[test]
fn source_timepiece_rewind_modifier_and_time_stop_queries_follow_equal_time_transitions() {
    let skill = run("options duration=16 crit=fixed;
        hotori char lvl=80/80 awakening=3 talent=1,1,1 qte=1 start_energy=0 start_timepiece=66;
        target hp=100000000 def=0 resist=0; active hotori;
        if .hotori.rewind.active || .hotori.mods.momentary_gaze!=0 {let assertion_failure=1/0;}
        hotori skill;
        if !.hotori.rewind.active || .hotori.resources.timepiece!=6
            || .hotori.time_stop.active {let assertion_failure=1/0;}
        wait_until(5.167475);
        if .hotori.rewind.active || .hotori.mods.momentary_gaze!=10
            || .hotori.resources.timepiece!=6 {let assertion_failure=1/0;}
        wait_until(6.167475);
        if .hotori.mods.momentary_gaze!=9 || .hotori.resources.timepiece!=18 {let assertion_failure=1/0;}
        wait_until(15.167475);
        if .hotori.mods.momentary_gaze!=0 || .hotori.resources.timepiece!=120 {let assertion_failure=1/0;}");
    assert!(
        skill.report.success && skill.report.execution_completed,
        "{:?}",
        skill.diagnostic
    );
    let burst = run("options duration=16 crit=fixed;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 start_timepiece=120;
        target hp=100000000 def=0 resist=0; active hotori;
        if .hotori.time_stop.active || !.hotori.burst.ready {let assertion_failure=1/0;}
        hotori burst;
        if !.hotori.time_stop.active || .hotori.resources.timepiece!=0 {let assertion_failure=1/0;}
        wait_until(14.183333);
        if .hotori.time_stop.active || .hotori.resources.timepiece!=0 {let assertion_failure=1/0;}
        wait(1);
        if .hotori.resources.timepiece!=6 {let assertion_failure=1/0;}");
    assert!(
        burst.report.success && burst.report.execution_completed,
        "{:?}",
        burst.diagnostic
    );
    for result in [skill, burst] {
        let replay =
            crate::simulation::run_game_scenario_report(&result.request, &Default::default())
                .unwrap();
        let actual = serde_json::to_value(&result.report).unwrap();
        let replay = serde_json::to_value(replay).unwrap();
        for (key, value) in actual.as_object().unwrap() {
            assert!(value == &replay[key], "Hotori resource replay field {key}");
        }
    }
}

#[test]
fn source_cycle_query_reads_shared_native_switch_payment() {
    let result = run("options duration=13 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint;
        if .mint.resources.cycle!=100 {let assertion_failure=1/0;}
        zero swap;
        if .mint.resources.cycle!=0 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic,
    );
    assert_eq!(
        result.report.final_actor_states[0]
            .cycle
            .as_ref()
            .unwrap()
            .spent,
        100
    );
}

#[test]
fn source_timepiece_two_paid_skills_keep_balance_rewind_and_a3_cooldown() {
    let result = run("options duration=30 crit=fixed;
        hotori char lvl=80/80 awakening=3 talent=1,1,1 qte=1 start_energy=0 start_timepiece=66;
        target hp=100000000 def=0 resist=0; active hotori;
        hotori skill;
        if .hotori.resources.timepiece!=6 {let assertion_failure=1/0;}
        wait_until(20);
        if .hotori.resources.timepiece!=120 || !.hotori.skill.ready {let assertion_failure=1/0;}
        hotori skill;
        if .hotori.resources.timepiece!=60 || !.hotori.rewind.active {let assertion_failure=1/0;}
        wait_until(25.167475);
        if .hotori.rewind.active || .hotori.mods.momentary_gaze!=0
            || .hotori.resources.timepiece!=60 {let assertion_failure=1/0;}
        wait(1);
        if .hotori.resources.timepiece!=66 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let pool = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
    let pool = &pool["fractional_resources"]["hotori.timepiece"];
    assert_eq!(pool["initial"], serde_json::json!("66/1"));
    assert_eq!(pool["spent"], serde_json::json!("120/1"));
    assert_eq!(pool["conserved"], true);
}

#[test]
fn source_timepiece_initialization_validates_direct_native_inputs_and_saved_artifacts() {
    let source = "options duration=2 crit=fixed;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=0.1+0.2;
        target hp=100000000 def=0 resist=0; active hotori;
        if .hotori.resources.timepiece!=0.3 {let assertion_failure=1/0;}";
    let compiled = crate::dsl::source::compile_response(&crate::dsl::compiler::SourceRequest {
        source: source.into(),
        limits: Default::default(),
    });
    assert!(compiled.diagnostics.is_empty());
    let saved: crate::dsl::source::SourceCompileResponse =
        serde_json::from_slice(&serde_json::to_vec(&compiled).unwrap()).unwrap();
    let response = crate::simulation::run_compiled_source(&saved, &Default::default()).unwrap();
    let result = response.execution.unwrap();
    assert!(result.report.success && result.report.execution_completed);
    let mut tampered = saved.clone();
    tampered.compiled.as_mut().unwrap().request.actors[0]
        .initial_resources
        .timepiece = None;
    assert!(crate::simulation::run_compiled_source(&tampered, &Default::default()).is_err());
    for (numerator, denominator) in [(121, 1), (0, 0)] {
        let mut request = result.request.clone();
        request.actors[0].initial_resources.timepiece = Some(crate::formula::Ratio {
            numerator,
            denominator,
        });
        assert!(
            crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err()
        );
    }
    let mut request = result.request;
    request.actors[0].character_id = "character:1019".into();
    assert!(crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err());
}

#[test]
fn source_owner_queries_distinguish_absent_modifiers_and_unknown_names_without_rng_draws() {
    let prefix = "options duration=6 crit=random;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=66;
        target hp=100000000 def=0 resist=0; active hotori;";
    let baseline = run(&format!("{prefix} hotori attack:5;"));
    let observed = run(&format!(
        "{prefix}
        repeat 5 {{
            if .hotori.resources.timepiece!=66 || .hotori.mods.momentary_gaze!=0
                || .hotori.time_stop.active || .hotori.rewind.active {{let assertion_failure=1/0;}}
            if 0 && .hotori.mods.unknown {{let assertion_failure=1/0;}}
        }} hotori attack:5;"
    ));
    assert!(observed.report.success && observed.report.execution_completed);
    super::assert_same_combat(&observed.report, &baseline.report);
    for field in ["resources.unknown", "mods.unknown", "unknown.active"] {
        let failed = run(&format!("{prefix} wait(.hotori.{field});"));
        assert!(!failed.report.execution_completed && !failed.report.success);
        assert!(
            failed
                .diagnostic
                .as_ref()
                .unwrap()
                .message
                .contains("Unknown")
        );
        assert_eq!(failed.diagnostic.as_ref().unwrap().loc.line, 3);
    }
}

#[test]
fn source_resource_inspection_preserves_omission_zero_and_fraction_without_execution() {
    for (parameter, expected) in [
        ("", None),
        ("start_timepiece=0", Some((0, 1))),
        ("start_timepiece=0.1+0.2", Some((3, 10))),
    ] {
        let response =
            crate::dsl::inspection::inspect_response(&crate::dsl::inspection::InspectionRequest {
                source: format!(
                    "options duration=1; hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {parameter};
                    target hp=100000000 def=0 resist=0; active hotori; while 1 {{}}"
                ),
            });
        assert!(
            response.diagnostics.is_empty(),
            "{:?}",
            response.diagnostics
        );
        let actor = &response.party.unwrap().actors[0];
        assert_eq!(
            actor
                .initial_resources
                .timepiece
                .map(|r| (r.numerator, r.denominator)),
            expected
        );
    }
    for (name, parameter) in [
        ("mint", "start_timepiece=0"),
        ("hotori", "start_timepiece=-1"),
        ("hotori", "start_timepiece=120.000001"),
    ] {
        let response =
            crate::dsl::inspection::inspect_response(&crate::dsl::inspection::InspectionRequest {
                source: format!("options duration=1; {name} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {parameter};
                    target hp=100000000 def=0 resist=0; active {name};"),
            });
        assert!(
            response.party.is_none() && response.diagnostics.len() == 1,
            "{name} {parameter}"
        );
        assert_eq!(response.diagnostics[0].loc.line, 1);
        assert!(response.diagnostics[0].loc.column > 50);
    }
}
