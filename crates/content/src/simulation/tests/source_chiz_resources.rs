use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_chiz_initial_savings_compound_exactly_beyond_u128_without_query_rng() {
    let result = run(include_str!(
        "../../../../../presets/native/finance-control.nte"
    ));
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let actor = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
    let pool = &actor["fractional_resources"]["chiz.grain"];
    assert_eq!(pool["conserved"], true);
    let (_, current_den) = pool["current"].as_str().unwrap().split_once('/').unwrap();
    assert!(current_den.len() > 39);
    assert!(result.report.reaction_rng.is_none());
}

#[test]
fn source_chiz_seeded_loans_suppress_interest_and_market_repays_before_savings() {
    let prefix = "options duration=20 crit=random start_fons=123456;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_grains=500 start_loans=100;
        target hp=100000000 def=0 resist=0; active chiz;";
    let debt = run(&format!(
        "{} wait(3); if .chiz.resources.grains!=500
        || .chiz.resources.loans!=100 {{let assertion_failure=1/0;}}",
        prefix.replace("awakening=0", "awakening=2")
    ));
    assert!(
        debt.report.success && debt.report.execution_completed,
        "{:?}",
        debt.diagnostic
    );
    let result = run(&format!(
        "{prefix}
        if .chiz.resources.grains!=500 || .chiz.resources.loans!=100
            || !.chiz.debt.active || .chiz.surplus.active {{let assertion_failure=1/0;}}
        chiz burst;
        if !.chiz.market.active || .chiz.debt.active || !.chiz.surplus.active
            || .chiz.mods.market<=0 || .chiz.mods.market>7 {{let assertion_failure=1/0;}}
        while .chiz.mods.market>0 {{wait(.chiz.mods.market);}}
        if .chiz.market.active || .chiz.mods.market!=0
            || .chiz.resources.grains!=400 || .chiz.resources.loans!=0
            || .team.fons!=123456 {{let assertion_failure=1/0;}}"
    ));
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let entry = result
        .report
        .trace
        .iter()
        .find(|event| event.event_kind == "chiz_market_entry")
        .unwrap();
    let wait = result
        .occurrences
        .iter()
        .rfind(|occurrence| occurrence.kind == crate::simulation::SourceOccurrenceKind::Wait)
        .unwrap();
    // The market is born inside the cinematic and retains all seven World seconds.
    assert!(entry.time_us < 4_250_937);
    assert_eq!(wait.completed_us.unwrap(), 4_250_937 + 7_000_000);
    let replay =
        crate::simulation::run_game_scenario_report(&result.request, &Default::default()).unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let replay = serde_json::to_value(replay).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert!(
            value == &replay[key],
            "Chiz seeded resource replay field {key}"
        );
    }
    let baseline = run(&format!("{prefix} chiz burst;"));
    // Read-only probes and an explicit wait preserve gameplay. Input-time
    // provenance deliberately distinguishes the wait from an exhausted script.
    let mut observed = serde_json::to_value(&result.report).unwrap();
    let mut baseline = serde_json::to_value(&baseline.report).unwrap();
    observed.as_object_mut().unwrap().remove("input_time");
    baseline.as_object_mut().unwrap().remove("input_time");
    assert_eq!(observed, baseline);
}

#[test]
fn source_chiz_a5_entry_grant_remains_separate_from_authored_initial_savings() {
    let result = run("options duration=1 crit=fixed;
        chiz char lvl=80/80 awakening=[5] talent=1,1,1 qte=1 start_grains=1;
        target hp=100000000 def=0 resist=0; active chiz;
        if .chiz.resources.grains!=15000 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let actor = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
    let pool = &actor["fractional_resources"]["chiz.grain"];
    for (field, expected) in [("initial", 1), ("granted", 15000), ("overflow", 1)] {
        assert_eq!(
            pool[field],
            serde_json::json!(format!("{}/{}", expected, 1)),
            "{field}"
        );
    }
    assert_eq!(pool["conserved"], true);
    let paid = run("options duration=1 crit=fixed;
        chiz char lvl=80/80 awakening=[5] talent=1,1,1 qte=1 start_grains=1 start_loans=100;
        target hp=100000000 def=0 resist=0; active chiz;
        if .chiz.resources.grains!=14901 || .chiz.resources.loans!=0 {let assertion_failure=1/0;}");
    assert!(
        paid.report.success && paid.report.execution_completed,
        "{:?}",
        paid.diagnostic
    );
    let actor = serde_json::to_value(&paid.report.final_actor_states[0]).unwrap();
    assert_eq!(
        actor["fractional_resources"]["chiz.loan_debt"]["spent"],
        serde_json::json!("100/1")
    );
}

#[test]
fn source_chiz_initial_resources_validate_owner_level_awakening_and_native_models() {
    for (name, awakening, parameter) in [
        ("mint", "0", "start_grains=0"),
        ("chiz", "0", "start_grains=15000.000001"),
        ("chiz", "0", "start_loans=15001"),
        ("chiz", "1", "start_grains=16500.1"),
        ("chiz", "0", "start_loans=-1"),
    ] {
        let source = format!(
            "options duration=1; {name} char lvl=80/80 awakening={awakening}
            talent=1,1,1 {parameter}; target hp=100000000 def=0 resist=0; active {name};"
        );
        assert!(
            PreparedSourceScenario::new(&source, Default::default(), &Default::default()).is_err(),
            "{parameter}"
        );
    }
    let result = run("options duration=1 crit=fixed;
        chiz char lvl=80/80 awakening=1 talent=1,1,1 qte=1 start_grains=16500 start_loans=16500;
        target hp=100000000 def=0 resist=0; active chiz;
        if .chiz.resources.grains!=16500 || .chiz.resources.loans!=16500 {let assertion_failure=1/0;}");
    assert!(result.report.success && result.report.execution_completed);
    let mut request = result.request;
    request
        .model_assumptions
        .remove(crate::characters::chiz::finance::MODEL.as_str());
    assert!(crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err());
    for (amount, valid) in [(500, true), (501, false)] {
        let source = format!(
            "options duration=1 crit=fixed;
            chiz char lvl=1/20 awakening=0 talent=1,1,1 qte=1 start_grains={amount} start_loans=500;
            target hp=100000000 def=0 resist=0; active chiz;
            if .chiz.resources.grains!=500 || .chiz.resources.loans!=500 {{let assertion_failure=1/0;}}"
        );
        let program = PreparedSourceScenario::new(&source, Default::default(), &Default::default());
        if valid {
            let result = program.unwrap().run(42).unwrap();
            assert!(result.report.success && result.report.execution_completed);
        } else {
            assert!(program.is_err());
        }
    }
}

#[test]
fn source_chiz_saved_identity_retains_explicit_zero_principal() {
    let source = "options duration=1 crit=fixed;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_grains=0 start_loans=0;
        target hp=100000000 def=0 resist=0; active chiz;";
    let mut compiled = crate::dsl::source::compile_response(&crate::dsl::compiler::SourceRequest {
        source: source.into(),
        limits: Default::default(),
    });
    assert!(compiled.diagnostics.is_empty());
    assert_eq!(
        compiled.party.as_ref().unwrap().actors[0]
            .initial_resources
            .loans
            .unwrap()
            .numerator,
        0
    );
    let roundtrip = serde_json::from_slice(&serde_json::to_vec(&compiled).unwrap()).unwrap();
    let result = crate::simulation::run_compiled_source(&roundtrip, &Default::default()).unwrap();
    assert!(result.execution.unwrap().report.execution_completed);
    compiled.compiled.as_mut().unwrap().request.actors[0]
        .initial_resources
        .loans = None;
    assert!(crate::simulation::run_compiled_source(&compiled, &Default::default()).is_err());
}

#[test]
fn source_chiz_stocks_refill_on_native_deadlines_and_hoarding_follows_field_control() {
    let result = run("options duration=12 crit=random;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_grains=0;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint;
        if .chiz.resources.skill_stocks!=3 || .chiz.hoarding.active || .chiz.out_of_combat.active {let assertion_failure=1/0;}
        for let i=0; i<2; i=i+1 {
            wait_until(i*6);
            chiz swap;
            if !.chiz.hoarding.active || .chiz.resources.skill_stocks!=3 {let assertion_failure=1/0;}
            let start=.time;
            chiz skill;
            if .chiz.resources.skill_stocks!=2 {let assertion_failure=1/0;}
            mint swap;
            if .chiz.hoarding.active || .chiz.resources.skill_stocks!=2 {let assertion_failure=1/0;}
            wait_until(start+2.999999);
            if .chiz.resources.skill_stocks!=2 {let assertion_failure=1/0;}
            wait(0.000001);
            if .chiz.resources.skill_stocks!=3 {let assertion_failure=1/0;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(
        result
            .report
            .trace
            .iter()
            .filter(|event| event.event_kind == "chiz_skill_stock_refill")
            .count(),
        2
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_chiz_stock_query_requires_the_selected_model_and_rejects_unknown_names() {
    let prefix = "options duration=1 crit=random;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active chiz;";
    for (field, message) in [
        ("resources.skill_stocks", "require the pressed Skill model"),
        ("resources.unknown", "Unknown resource"),
        ("unknown.active", "Unknown state"),
    ] {
        let result = run(&format!("{prefix} wait(.chiz.{field});"));
        let diagnostic = result.diagnostic.unwrap();
        assert!(diagnostic.message.contains(message), "{diagnostic:?}");
        assert!(diagnostic.loc.length > 0);
        assert!(!result.report.execution_completed);
    }
}
