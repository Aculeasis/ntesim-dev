use super::{compiler::SourceRequest, source::compile_response};

const PREFIX: &str = "options duration=20; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint;";

#[test]
fn native_compilation_retains_live_source_without_running_or_unrolling_loops() {
    for rotation in [
        "while .time<10 {mint attack; wait_for_energy(1);}",
        "while 1 {}",
    ] {
        let source = format!("{PREFIX}{rotation}");
        let response = compile_response(&SourceRequest {
            source: source.clone(),
            limits: Default::default(),
        });
        assert!(
            response.diagnostics.is_empty(),
            "{:?}",
            response.diagnostics
        );
        let compiled = response.compiled.unwrap();
        assert_eq!(compiled.input.source, source);
        assert!(compiled.request.actions.is_empty());
        assert_eq!(
            response.party.unwrap().actors[0].character_id,
            compiled.request.actors[0].character_id
        );
        let options = response.options.unwrap();
        assert_eq!(options.trials, compiled.trials);
        assert_eq!(options.relative_duration_us, 20_000_000);
    }
}

#[test]
fn closed_invalid_waits_and_counts_keep_valid_party_and_exact_expression_ranges() {
    for rotation in [
        "wait(-1);",
        "wait();",
        "wait_for_energy(1/2);",
        "repeat (1-0.0000000000000000001) {}",
    ] {
        let response = compile_response(&SourceRequest {
            source: format!("# 🦀\r\n{PREFIX}\r\n{rotation}"),
            limits: Default::default(),
        });
        assert!(response.compiled.is_none());
        assert!(response.party.is_some());
        assert_eq!(response.options.unwrap().seed, 42);
        assert_eq!(response.diagnostics[0].loc.line, 3);
        assert!(response.diagnostics[0].loc.offset > PREFIX.len());
        assert!(response.diagnostics[0].loc.length > 0);
    }
    let response = compile_response(&SourceRequest {
        source: PREFIX.replace("hp=100000000", "hp=-1"),
        limits: Default::default(),
    });
    assert!(response.compiled.is_none() && response.party.is_none() && response.options.is_none());
}

#[test]
fn native_scalar_preflight_respects_lexical_shadowing_and_for_scope() {
    let prefix = PREFIX.replacen(
        "options duration=20;",
        "options duration=20; const n=-1;",
        1,
    );
    let source =
        format!("{prefix}repeat 1 {{let n=1; wait(n);}} for let n=0; n<1; n=n+1 {{wait(n);}}");
    let response = compile_response(&SourceRequest {
        source,
        limits: Default::default(),
    });
    assert!(
        response.diagnostics.is_empty(),
        "{:?}",
        response.diagnostics
    );
    let source = format!("{prefix}repeat 1 {{let n=1; wait(n);}} wait(n);");
    let response = compile_response(&SourceRequest {
        source,
        limits: Default::default(),
    });
    assert!(response.compiled.is_none() && response.party.is_some());
}
