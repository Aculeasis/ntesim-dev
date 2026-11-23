use super::{options::*, validation::DiagnosticCode, *};
use crate::simulation::GameStopPolicy as StopPolicy;

#[test]
fn options_compile_exact_expressions_and_documented_defaults() {
    let script = parse_nte("options duration=0.1+0.2;").unwrap();
    let options = compile_options(&script).unwrap();
    assert_eq!(
        options,
        CompiledOptions {
            trials: 1000,
            relative_duration_us: 300_000,
            seed: 42,
            crit: CriticalPolicy::Random,
            start_fons: None,
            combat_stamina: None,
            stop: StopPolicy::Time,
            duration_specified: true,
        }
    );
    let script = parse_nte("options duration=100ms+3us iteration=max(1,2+3) seed=9007199254740991 crit=random start_fons=0;").unwrap();
    let options = compile_options(&script).unwrap();
    assert_eq!(
        options,
        CompiledOptions {
            trials: 5,
            relative_duration_us: 100_003,
            seed: 9_007_199_254_740_991,
            crit: CriticalPolicy::Random,
            start_fons: Some(0),
            combat_stamina: None,
            stop: StopPolicy::Time,
            duration_specified: true,
        }
    );
    assert_eq!(
        serde_json::from_str::<CompiledOptions>(&serde_json::to_string(&options).unwrap()).unwrap(),
        options
    );
}

#[test]
fn invalid_options_fail_without_coercion_rounding_or_silent_defaults() {
    for fields in [
        "duration=0",
        "duration=-1",
        "duration=0.0000001s",
        "duration=true",
        "duration=1 iteration=0",
        "duration=1 iteration=1.5",
        "duration=1 seed=-1",
        "duration=1 seed=9007199254740992",
        "duration=1 seed=1.00000000000000001",
        "duration=1 start_fons=-1",
        "duration=1 start_fons=false",
        "duration=1 mode=unknown",
        "duration=1 mode=research",
        "duration=1 mode=strict",
        "duration=1 crit=expected",
        "duration=1 crit=1",
        "duration=1 unexpected=1",
        "duration=1 stop=unknown",
        "duration=1 stop=1",
        "stop=time",
        "duration=.time",
        "duration=unknown",
        "duration=1/0",
        "seed=42",
    ] {
        let script = parse_nte(&format!("options {fields};")).unwrap();
        assert!(compile_options(&script).is_err(), "{fields}");
    }
    assert!(compile_options(&parse_nte("").unwrap()).is_err());
}

#[test]
fn option_diagnostics_accumulate_and_preserve_the_failing_expression_range() {
    let script = parse_nte("# 😀\r\noptions duration=1 seed=1/0 iteration=0;").unwrap();
    let diagnostics = compile_options(&script).unwrap_err();
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].code, DiagnosticCode::Arithmetic);
    assert_eq!(
        diagnostics[0].loc,
        SourceLocation {
            line: 2,
            column: 26,
            offset: 31,
            length: 1
        }
    );
    assert_eq!(diagnostics[1].code, DiagnosticCode::Configuration);
    assert_eq!(diagnostics[1].message, "iteration must be positive.");
}

#[test]
fn compilation_also_rejects_duplicate_options_in_direct_ast_inputs() {
    let mut script = parse_nte("options duration=1;").unwrap();
    script.options.push(script.options[0].clone());
    let diagnostics = compile_options(&script).unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, DiagnosticCode::Configuration);
}

#[test]
fn all_public_preset_options_compile_in_rust() {
    for source in [
        include_str!("../../../../presets/studio/s1.nte"),
        include_str!("../../../../presets/studio/s2.nte"),
        include_str!("../../../../presets/studio/s3.nte"),
        include_str!("../../../../presets/studio/s4.nte"),
        include_str!("../../../../presets/studio/s5.nte"),
        include_str!("../../../../presets/studio/language-reference.nte"),
    ] {
        assert!(compile_options(&parse_nte(source).unwrap()).is_ok());
    }
}

#[test]
fn options_compile_stop_policies() {
    let script = parse_nte("options duration=10 stop=time;").unwrap();
    let options = compile_options(&script).unwrap();
    assert_eq!(options.stop, StopPolicy::Time);
    assert_eq!(options.relative_duration_us, 10_000_000);
    assert!(options.duration_specified);

    let script = parse_nte("options duration=5 stop=victory;").unwrap();
    let options = compile_options(&script).unwrap();
    assert_eq!(options.stop, StopPolicy::Victory);
    assert_eq!(options.relative_duration_us, 5_000_000);
    assert!(options.duration_specified);

    let script = parse_nte("options stop=victory;").unwrap();
    let options = compile_options(&script).unwrap();
    assert_eq!(options.stop, StopPolicy::Victory);
    assert_eq!(
        options.relative_duration_us,
        crate::execution::ExecutionLimits::default().max_horizon_us
    );
    assert!(!options.duration_specified);
}
