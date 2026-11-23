mod source;

use serde_json::json;
use std::io::{self, Write};

#[test]
fn usage_lists_resumable_research_commands() {
    let usage = crate::commands::execute(&[]).unwrap_err();
    assert!(usage.contains("pilot [REQUEST.json [CHECKPOINT.json]]"));
    assert!(usage.contains("acceptance-run [REQUEST.json [CHECKPOINT.json"));
}

#[test]
fn json_report_write_returns_broken_pipe() {
    struct ClosedPipe;

    impl Write for ClosedPipe {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let error = crate::write_report(&mut ClosedPipe, &json!({"report": true})).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn json_report_write_preserves_pretty_json_and_trailing_newline() {
    let mut output = Vec::new();
    crate::write_report(&mut output, &json!({"report": true})).unwrap();
    assert_eq!(output, b"{\n  \"report\": true\n}\n");
}

#[test]
fn parse_command_recognizes_structured_commands() {
    use crate::commands::{CliCommand, parse_command};

    let args = vec!["run-game".into(), "scenario.json".into()];
    assert_eq!(
        parse_command(&args).unwrap(),
        CliCommand::RunGame {
            input: "scenario.json"
        }
    );
    assert!(parse_command(&["audit".into()]).is_err());
    assert!(
        crate::reports::composition_enumeration_report(Some(std::path::Path::new(
            "missing-research-input.csv"
        )))
        .unwrap_err()
        .contains("missing-research-input.csv")
    );

    let args = vec![
        "run-rotation-synthetic".into(),
        "fixture.json".into(),
        "rot.nte".into(),
        "--seed".into(),
        "42".into(),
    ];
    assert_eq!(
        parse_command(&args).unwrap(),
        CliCommand::RunRotationSynthetic {
            input: "fixture.json",
            rotation: "rot.nte",
            seed: Some(42),
        }
    );

    let invalid = vec!["unknown-command".into()];
    assert!(parse_command(&invalid).is_err());
}

#[test]
fn clap_cli_builds_and_renders_help() {
    let mut cmd = crate::clap_app::build_cli();
    let help = cmd.render_help().to_string();
    assert!(help.contains("ntesim"));
    assert!(help.contains("run-game"));
    assert!(help.contains("run-synthetic"));
    assert!(help.contains("pilot"));
}
