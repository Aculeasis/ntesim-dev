use crate::dsl::{
    compiler::SourceRequest,
    source::{SourceCompileResponse, compile_response},
};
use crate::simulation::{run_compiled_source, run_source};

fn artifact() -> SourceCompileResponse {
    compile_response(&SourceRequest {
        source: include_str!("../../../../../presets/native/live-control.nte").into(),
        limits: Default::default(),
    })
}

#[test]
fn compiled_source_roundtrip_preserves_the_full_native_trial() {
    let artifact = artifact();
    assert!(artifact.diagnostics.is_empty());
    let decoded = serde_json::from_slice(&serde_json::to_vec(&artifact).unwrap()).unwrap();
    let actual = run_compiled_source(&decoded, &Default::default()).unwrap();
    let expected = run_source(&artifact.compiled.unwrap().input, &Default::default()).unwrap();
    assert!(serde_json::to_value(actual).unwrap() == serde_json::to_value(expected).unwrap());
}

#[test]
fn compiled_source_rejects_engine_configuration_and_inspection_mismatches() {
    for variant in 0..8 {
        let mut artifact = artifact();
        let program = artifact.compiled.as_mut().unwrap();
        match variant {
            0 => program.schema_version = 0,
            1 => program.engine_source_sha256.push('0'),
            2 => program.request.relative_duration_us -= 1,
            3 => program.trials += 1,
            4 => program.input.source = program.input.source.replace("duration=30", "duration=31"),
            5 => program.input.limits.max_steps = 0,
            6 => artifact.party.as_mut().unwrap().actors[0].initial_energy += 1,
            _ => artifact.options.as_mut().unwrap().seed += 1,
        }
        assert!(
            run_compiled_source(&artifact, &Default::default()).is_err(),
            "variant {variant}"
        );
    }
}
