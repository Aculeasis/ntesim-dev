use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_malice_seeds_before_entry_and_keeps_two_skill_grants_and_overflow() {
    let source = "options duration=20 crit=random;
        blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_malice=0.1;
        target hp=100000000 def=0 resist=0; active blackbird;
        if .blackbird.resources.malice!=20.1 || .blackbird.transformed.active
            || .blackbird.out_of_combat.active || .blackbird.mods.skill_permission!=0 {let assertion_failure=1/0;}
        blackbird skill;
        if .blackbird.resources.malice!=60 {let assertion_failure=1/0;}
        wait_until(16);
        blackbird skill;
        if .blackbird.resources.malice!=60 {let assertion_failure=1/0;}";
    let result = run(source);
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let actor = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
    let pool = &actor["fractional_resources"]["blackbird.malice"];
    for (field, n, d) in [
        ("initial", 1, 10),
        ("granted", 100, 1),
        ("current", 60, 1),
        ("overflow", 401, 10),
    ] {
        assert_eq!(
            pool[field],
            serde_json::json!(format!("{}/{}", n, d)),
            "{field}"
        );
    }
    assert_eq!(pool["conserved"], true);
    let baseline = run(&source.replace("if .blackbird.resources.malice!=20.1 || .blackbird.transformed.active\n            || .blackbird.out_of_combat.active || .blackbird.mods.skill_permission!=0 {let assertion_failure=1/0;}", "")
        .replace("if .blackbird.resources.malice!=60 {let assertion_failure=1/0;}", ""));
    super::assert_same_combat(&result.report, &baseline.report);
    replay(&result);
}

#[test]
fn source_crime_preserves_two_chains_and_per_packet_recipient_deduplication() {
    for targets in [1, 3] {
        let target = "target hp=100000000 def=0 resist=0;".repeat(targets);
        let result = run(&format!(
            "options duration=12 crit=fixed;
            chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_crime=0;
            {target} active chaos;
            if .chaos.resources.crime!=0 || .chaos.dread_echo.active {{let assertion_failure=1/0;}}
            chaos attack:5;
            if .chaos.resources.crime!=382 {{let assertion_failure=1/0;}}
            chaos attack:5;
            if .chaos.resources.crime!=764 {{let assertion_failure=1/0;}}"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        assert_eq!(result.report.hits_settled, 24 * targets);
        let actor = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
        assert_eq!(
            actor["fractional_resources"]["chaos.crime"]["granted"],
            serde_json::json!("764/1")
        );
        replay(&result);
    }
    let capped = run("options duration=12 crit=fixed;
        chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_crime=600;
        target hp=100000000 def=0 resist=0; active chaos;
        chaos attack:5; if .chaos.resources.crime!=982 {let assertion_failure=1/0;}
        chaos attack:5; if .chaos.resources.crime!=1000 {let assertion_failure=1/0;}");
    assert!(
        capped.report.success && capped.report.execution_completed,
        "{:?}",
        capped.diagnostic
    );
    let actor = serde_json::to_value(&capped.report.final_actor_states[0]).unwrap();
    assert_eq!(
        actor["fractional_resources"]["chaos.crime"]["overflow"],
        serde_json::json!("364/1")
    );
}

#[test]
fn source_malice_crime_caps_owners_and_required_native_models_validate() {
    for (name, parameter) in [
        ("mint", "start_malice=0"),
        ("mint", "start_crime=0"),
        ("blackbird", "start_crime=0"),
        ("chaos", "start_malice=0"),
        ("blackbird", "start_malice=60.000001"),
        ("chaos", "start_crime=1000.000001"),
        ("blackbird", "start_malice=-1"),
        ("chaos", "start_crime=-1"),
    ] {
        let source = format!(
            "options duration=1; {name} char lvl=80/80 awakening=0
            talent=1,1,1 qte=1 {parameter}; target hp=100000000 def=0 resist=0; active {name};"
        );
        assert!(
            PreparedSourceScenario::new(&source, Default::default(), &Default::default()).is_err(),
            "{name} {parameter}"
        );
    }
    for (name, parameter, field, model) in [
        (
            "blackbird",
            "start_malice=60",
            "malice",
            crate::characters::blackbird::malice::MODEL,
        ),
        (
            "chaos",
            "start_crime=1000",
            "crime",
            crate::characters::chaos::resources::MODEL,
        ),
    ] {
        let result = run(&format!(
            "options duration=1; {name} char lvl=80/80 awakening=0
            talent=1,1,1 qte=1 {parameter}; target hp=100000000 def=0 resist=0; active {name};"
        ));
        assert!(result.report.success && result.report.execution_completed);
        if name == "blackbird" {
            let actor = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
            assert_eq!(
                actor["fractional_resources"]["blackbird.malice"]["overflow"],
                serde_json::json!("20/1")
            );
        }
        let mut request = result.request;
        request.model_assumptions.remove(model.as_str());
        assert!(
            crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err(),
            "{field}"
        );
    }
}

fn replay(result: &SourceScenarioReport) {
    let report =
        crate::simulation::run_game_scenario_report(&result.request, &Default::default()).unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let report = serde_json::to_value(report).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert!(
            value == &report[key],
            "Malice/Crime source replay field {key}"
        );
    }
}
