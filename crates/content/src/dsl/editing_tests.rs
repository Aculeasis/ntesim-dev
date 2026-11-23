use super::{editing::*, *};
use serde_json::json;

#[test]
fn numeric_form_inputs_preserve_exact_integer_and_microsecond_boundaries() {
    for (name, input, expected) in [
        ("duration", "1e-6", json!(0.000001)),
        ("duration", ".5", json!(0.5)),
        ("iteration", "1e3", json!(1000)),
        ("seed", "9007199254740991", json!(9_007_199_254_740_991u64)),
        ("start_fons", " ", json!(null)),
    ] {
        assert_eq!(parse_option_input(name, input).unwrap(), expected);
    }
    for (name, input) in [
        ("duration", "0.0000001"),
        ("seed", "1.00000000000000001"),
        ("seed", "-1"),
        ("iteration", ""),
        ("seed", "9007199254740992"),
    ] {
        assert!(parse_option_input(name, input).is_err(), "{name}={input}");
    }
}

#[test]
fn option_edits_preserve_comments_crlf_rotation_and_unicode_ranges() {
    let source = format!(
        "# 😀\r\n{}",
        include_str!("../../../../presets/studio/s1.nte").replace('\n', "\r\n")
    );
    let changes =
        serde_json::from_value(json!({"duration":21.000001,"iteration":7,"start_fons":0})).unwrap();
    let edited = edit_options(&source, &changes).unwrap();
    assert_eq!(
        edited,
        source
            .replace("iteration=1000", "iteration=7")
            .replace("duration=20", "duration=21.000001")
            .replace("seed=42;", "seed=42 start_fons=0;")
    );
    let removed = edit_options(
        &edited,
        &serde_json::from_value(json!({"start_fons":null})).unwrap(),
    )
    .unwrap();
    assert!(!removed.contains("start_fons="));
}

#[test]
fn edited_microsecond_options_stay_decimal_source_literals() {
    let source = include_str!("../../../../presets/studio/s1.nte");
    // Keep a sufficiently long horizon for this preset; the seed exercises exponent normalization.
    let changes = serde_json::from_value(json!({"seed":1e3})).unwrap();
    let edited = edit_options(source, &changes).unwrap();
    assert!(edited.contains("seed=1000"));
    assert!(parse_nte(&edited).is_ok());
}

#[test]
fn settings_edit_live_and_invalid_rotation_drafts_without_execution() {
    let prefix = "# 😀\r\noptions duration=(10+10) seed=1+1; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=1000000 def=0 resist=0; active mint;";
    for rotation in ["while 1 {}", "wait(-1);", "while .time<10 {mint attack;}"] {
        let source = format!("{prefix}\r\n{rotation}");
        let changes =
            serde_json::from_value(json!({"duration":20,"seed":3,"start_fons":0})).unwrap();
        let edited = edit_options(&source, &changes).unwrap();
        assert_eq!(edited, source.replace("seed=1+1", "seed=3 start_fons=0"));
        let changes = serde_json::from_value(json!({"duration":null,"stop":"victory"})).unwrap();
        let victory = edit_options(&edited, &changes).unwrap();
        assert!(victory.contains("stop=victory"));
        assert!(!victory.contains("duration="));
        assert!(victory.ends_with(rotation));
        let (_, config) = super::inspection::prepare(&victory).unwrap();
        assert!(!config.options.duration_specified);
        // Removing duration while retaining time-stop is still invalid.
        assert!(
            edit_options(
                &source,
                &serde_json::from_value(json!({"duration":null})).unwrap()
            )
            .is_err()
        );
    }
}

#[test]
fn settings_edit_allows_switching_stop_to_victory_with_duration_preserved() {
    let source = "options duration=120 stop=time; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=1000 def=0 resist=0; active mint;";
    let changes = serde_json::from_value(json!({"duration": 120, "stop": "victory"})).unwrap();
    let edited = edit_options(source, &changes).unwrap();
    assert!(edited.contains("stop=victory"));
    assert!(edited.contains("duration=120"));
    let (_, config) = super::inspection::prepare(&edited).unwrap();
    assert_eq!(
        config.options.stop,
        crate::simulation::GameStopPolicy::Victory
    );
    assert_eq!(config.options.relative_duration_us, 120_000_000);
    assert!(config.options.duration_specified);
}
