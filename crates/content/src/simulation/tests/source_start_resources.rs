use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

fn assert_replay(result: &SourceScenarioReport) {
    let native =
        crate::simulation::run_game_scenario_report(&result.request, &Default::default()).unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let native = serde_json::to_value(native).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert_eq!(value, &native[key], "native replay mismatch on {key}");
    }
}

#[test]
fn source_adler_start_karma_and_query_consumption() {
    let script = "options duration=12 crit=fixed;
        adler char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_karma=15;
        target hp=100000000 def=0 resist=0; active adler;
        if .adler.resources.karma!=15 {let assertion_failure=1/0;}
        adler skill;
        if .adler.resources.karma!=0 {let assertion_failure=1/0;}";
    let result = run(script);
    assert!(
        result.report.success && result.report.execution_completed,
        "diag: {:?}, halt: {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_replay(&result);
}

#[test]
fn source_adler_start_karma_validation() {
    for valid in ["start_karma=0", "start_karma=20", "start_karma=10"] {
        let script = format!(
            "options duration=1 crit=fixed;
            adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {valid};
            target hp=1000 def=0 resist=0; active adler;"
        );
        assert!(
            PreparedSourceScenario::new(&script, Default::default(), &Default::default()).is_ok()
        );
    }

    for invalid in ["start_karma=21", "start_karma=0.5", "start_karma=-1"] {
        let script = format!(
            "options duration=1 crit=fixed;
            adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {invalid};
            target hp=1000 def=0 resist=0; active adler;"
        );
        assert!(
            PreparedSourceScenario::new(&script, Default::default(), &Default::default()).is_err()
        );
    }

    let script = "options duration=1 crit=fixed;
        adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_karma=21;
        target hp=1000 def=0 resist=0; active adler;";
    let errors = PreparedSourceScenario::new(script, Default::default(), &Default::default())
        .err()
        .unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Adler Karma must be integral in 0..20.")),
        "{:?}",
        errors
    );

    let non_adler = "options duration=1 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_karma=0;
        target hp=1000 def=0 resist=0; active mint;";
    let errors = PreparedSourceScenario::new(non_adler, Default::default(), &Default::default())
        .err()
        .unwrap();
    assert!(
        errors.iter().any(|e| e
            .message
            .contains("start_karma is only supported by Adler.")),
        "{:?}",
        errors
    );
}

#[test]
fn source_edgar_start_keys_and_burst_clearing() {
    let script = "options duration=15 crit=fixed;
        edgar char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_keys=2 start_energy=100;
        target hp=100000000 def=0 resist=0; active edgar;
        if .edgar.resources.keys!=2 {let assertion_failure=1/0;}
        edgar burst;
        wait_until(5);
        if .edgar.resources.keys!=0 {let assertion_failure=1/0;}";
    let result = run(script);
    assert!(
        result.report.success && result.report.execution_completed,
        "diag: {:?}, halt: {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_replay(&result);
}

#[test]
fn source_edgar_start_keys_validation() {
    for valid in ["start_keys=0", "start_keys=3", "start_keys=1"] {
        let script = format!(
            "options duration=1 crit=fixed;
            edgar char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {valid};
            target hp=1000 def=0 resist=0; active edgar;"
        );
        assert!(
            PreparedSourceScenario::new(&script, Default::default(), &Default::default()).is_ok()
        );
    }

    for invalid in ["start_keys=4", "start_keys=1.5", "start_keys=-1"] {
        let script = format!(
            "options duration=1 crit=fixed;
            edgar char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {invalid};
            target hp=1000 def=0 resist=0; active edgar;"
        );
        assert!(
            PreparedSourceScenario::new(&script, Default::default(), &Default::default()).is_err()
        );
    }

    let script = "options duration=1 crit=fixed;
        edgar char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_keys=4;
        target hp=1000 def=0 resist=0; active edgar;";
    let errors = PreparedSourceScenario::new(script, Default::default(), &Default::default())
        .err()
        .unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Edgar Keys must be integral in 0..3.")),
        "{:?}",
        errors
    );

    let non_edgar = "options duration=1 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_keys=0;
        target hp=1000 def=0 resist=0; active mint;";
    let errors = PreparedSourceScenario::new(non_edgar, Default::default(), &Default::default())
        .err()
        .unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("start_keys is only supported by Edgar.")),
        "{:?}",
        errors
    );
}

#[test]
fn source_chiz_start_stocks_and_time_refill() {
    let script = "options duration=8 crit=fixed start_fons=500000;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_stocks=1;
        target hp=100000000 def=0 resist=0; active chiz;
        if .chiz.resources.skill_stocks!=1 || .chiz.resources.stocks!=1 || .team.fons!=500000 {let assertion_failure=1/0;}
        wait_until(3);
        if .chiz.resources.skill_stocks!=2 {let assertion_failure=1/0;}
        wait_until(6);
        if .chiz.resources.skill_stocks!=3 {let assertion_failure=1/0;}";
    let result = run(script);
    assert!(
        result.report.success && result.report.execution_completed,
        "diag: {:?}, halt: {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_replay(&result);

    let alias_script = "options duration=2 crit=fixed start_fons=0;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_skill_stocks=2;
        target hp=1000 def=0 resist=0; active chiz;
        if .chiz.resources.skill_stocks!=2 {let assertion_failure=1/0;}";
    let alias_result = run(alias_script);
    assert!(
        alias_result.report.success && alias_result.report.execution_completed,
        "diag: {:?}, halt: {:?}",
        alias_result.diagnostic,
        alias_result.report.halt_diagnostic
    );
}

#[test]
fn source_chiz_start_stocks_validation() {
    for valid in ["start_stocks=0", "start_stocks=3", "start_skill_stocks=2"] {
        let script = format!(
            "options duration=1 crit=fixed start_fons=0;
            chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {valid};
            target hp=1000 def=0 resist=0; active chiz;"
        );
        assert!(
            PreparedSourceScenario::new(&script, Default::default(), &Default::default()).is_ok()
        );
    }

    for invalid in ["start_stocks=4", "start_stocks=0.5", "start_stocks=-1"] {
        let script = format!(
            "options duration=1 crit=fixed start_fons=0;
            chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {invalid};
            target hp=1000 def=0 resist=0; active chiz;"
        );
        assert!(
            PreparedSourceScenario::new(&script, Default::default(), &Default::default()).is_err()
        );
    }

    let script = "options duration=1 crit=fixed start_fons=0;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_stocks=4;
        target hp=1000 def=0 resist=0; active chiz;";
    let errors = PreparedSourceScenario::new(script, Default::default(), &Default::default())
        .err()
        .unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Chiz stocks must be integral in 0..3.")),
        "{:?}",
        errors
    );

    let non_chiz = "options duration=1 crit=fixed start_fons=0;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_stocks=0;
        target hp=1000 def=0 resist=0; active mint;";
    let errors = PreparedSourceScenario::new(non_chiz, Default::default(), &Default::default())
        .err()
        .unwrap();
    assert!(
        errors.iter().any(|e| e
            .message
            .contains("start_stocks/start_skill_stocks is only supported by Chiz.")),
        "{:?}",
        errors
    );
}

#[test]
fn source_zankou_start_kindling_and_burst_consumption() {
    let script = "options duration=35 crit=fixed;
        zankou char lvl=80/80 awakening=2 talent=8,8,8 qte=8 start_kindling=1 start_energy=120;
        target hp=100000000 def=0 resist=0; active zankou;
        if .zankou.resources.kindling!=1 {let assertion_failure=1/0;}
        zankou burst;
        wait_until(3);
        if .zankou.resources.kindling!=0 {let assertion_failure=1/0;}";
    let result = run(script);
    assert!(
        result.report.success && result.report.execution_completed,
        "diag: {:?}, halt: {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_replay(&result);
}

#[test]
fn source_zankou_start_kindling_validation() {
    for valid in ["start_kindling=0", "start_kindling=1"] {
        let script = format!(
            "options duration=1 crit=fixed;
            zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {valid};
            target hp=1000 def=0 resist=0; active zankou;"
        );
        assert!(
            PreparedSourceScenario::new(&script, Default::default(), &Default::default()).is_ok()
        );
    }

    for invalid in [
        "start_kindling=0.5",
        "start_kindling=-1",
        "start_kindling=2",
    ] {
        let script = format!(
            "options duration=1 crit=fixed;
            zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1 {invalid};
            target hp=1000 def=0 resist=0; active zankou;"
        );
        assert!(
            PreparedSourceScenario::new(&script, Default::default(), &Default::default()).is_err()
        );
    }

    let script = "options duration=1 crit=fixed;
        zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_kindling=0.5;
        target hp=1000 def=0 resist=0; active zankou;";
    let errors = PreparedSourceScenario::new(script, Default::default(), &Default::default())
        .err()
        .unwrap();
    assert!(
        errors.iter().any(|e| e
            .message
            .contains("Zankou Kindling must be integral in0..1.")),
        "{:?}",
        errors
    );

    let non_zankou = "options duration=1 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_kindling=0;
        target hp=1000 def=0 resist=0; active mint;";
    let errors = PreparedSourceScenario::new(non_zankou, Default::default(), &Default::default())
        .err()
        .unwrap();
    assert!(
        errors.iter().any(|e| e
            .message
            .contains("start_kindling is only supported by Zankou.")),
        "{:?}",
        errors
    );
}

#[test]
fn source_fons_shared_consumer_and_distinct_separation() {
    // Missing start_fons when queried diagnoses
    let query_missing = "options duration=1 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=1000 def=0 resist=0; active mint;
        if .team.fons!=0 {let assertion_failure=1/0;}";
    let result =
        PreparedSourceScenario::new(query_missing, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
    assert!(!result.report.success);
    assert!(
        result
            .diagnostic
            .unwrap()
            .message
            .contains("Missing options start_fons required by .team.fons.")
    );

    // Explicit zero: start_fons=0 retains zero and does not fail
    let query_zero = "options duration=1 crit=fixed start_fons=0;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=1000 def=0 resist=0; active mint;
        if .team.fons!=0 {let assertion_failure=1/0;}";
    let zero_res = run(query_zero);
    assert!(zero_res.report.success && zero_res.report.execution_completed);

    // Contemplative Cat with Chiz: Grains and Loans do not mutate team Fons
    let script = "options duration=5 crit=fixed start_fons=250000;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_grains=500 start_loans=100;
        chiz add weapon=fork_mamen refine=1;
        target hp=100000000 def=0 resist=0; active chiz;
        if .team.fons!=250000 || .chiz.resources.grains!=500 || .chiz.resources.loans!=100 {let assertion_failure=1/0;}
        chiz attack:3;
        wait_until(2);
        if .team.fons!=250000 {let assertion_failure=1/0;}";
    let cat_res = run(script);
    assert!(
        cat_res.report.success && cat_res.report.execution_completed,
        "diag: {:?}, halt: {:?}",
        cat_res.diagnostic,
        cat_res.report.halt_diagnostic
    );
    assert_replay(&cat_res);
}
