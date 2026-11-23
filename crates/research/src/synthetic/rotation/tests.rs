use super::*;
use ntesim_contracts::Quality;
use serde_json::{Value, json};
const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-rotation.json"
));
const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-rotation.nte"
));
#[test]
fn actor_rotation_preserves_cycle_state_and_command_source_mapping() {
    let result = run(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/synthetic-actions.json"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/synthetic-actions.nte"
        )),
    )
    .unwrap();
    assert!(result.simulation.success);
    assert_eq!(result.simulation.targets["first"].hp(), 170);
    assert_eq!(result.rotation.origins.len(), 3);
    for hit in result.simulation.trace.iter().filter(|e| e.phase == "hit") {
        let origin = &result.rotation.origins[hit.command_index];
        assert_eq!(origin.position.line, 4);
        assert_eq!(origin.iterations, [hit.command_index as u64 + 1]);
        assert_eq!(hit.action_origin.as_ref().unwrap().actor, "hero");
    }
}
#[test]
fn formula_actions_keep_state_across_repeats_and_trace_to_source() {
    let result = run(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/synthetic-formula.json"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/synthetic-formula.nte"
        )),
    )
    .unwrap();
    assert!(result.simulation.success);
    assert_eq!(result.simulation.targets["first"].hp(), 43);
    assert!(result.simulation.targets["first"].shields().is_empty());
    assert_eq!(result.simulation.trace.len(), 8);
    let hits: Vec<_> = result
        .simulation
        .trace
        .iter()
        .filter(|e| e.phase == "hit")
        .collect();
    assert_eq!(hits[0].changes["damage"]["absorbed"][0]["amount"], 5);
    assert_eq!(hits[0].changes["damage"]["hp_lost"], 26);
    assert_eq!(hits[1].changes["damage"]["hp_lost"], 31);
    assert_eq!(result.rotation.origins[2].iterations, [2]);
    assert_eq!(result.rotation.origins[2].position.line, 6);
}
fn fixture() -> Fixture {
    serde_json::from_slice(FIXTURE).unwrap()
}
fn plan(source: &str) -> std::result::Result<Compiled, Error> {
    let f = fixture();
    compile(source, &f.actions, &f.clocks)
}
#[test]
fn repeated_rotation_matches_independent_timestamp_fixture() {
    let result = run(FIXTURE, SOURCE).unwrap();
    let expected = synthetic::run(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/synthetic-state.json"
    )))
    .unwrap();
    assert!(result.simulation.success);
    assert_eq!(result.simulation.quality, Quality::Synthetic);
    for field in [
        "trace",
        "targets",
        "resources",
        "pending_events",
        "real_time_us",
        "mode_time_us",
    ] {
        assert_eq!(
            serde_json::to_value(&result.simulation).unwrap()[field],
            serde_json::to_value(&expected).unwrap()[field],
            "{field}"
        );
    }
    let hits: Vec<_> = result
        .rotation
        .origins
        .iter()
        .filter(|o| o.action == "attack")
        .collect();
    assert_eq!(hits.len(), 6);
    assert_eq!(hits[4].iterations, [3]);
    assert_eq!(hits[4].effect_index, 0);
}
#[test]
fn clock_cursors_and_nested_iteration_positions_are_explicit() {
    let plan = plan("clock 0; wait 3; repeat 2 { repeat 2 { do attack; wait 4; } } clock 1; wait 7; do attack; clock 0; do attack;").unwrap();
    assert_eq!(
        plan.commands
            .iter()
            .map(|c| (c.clock, c.delay_us))
            .collect::<Vec<_>>(),
        [
            (0, 3),
            (0, 3),
            (0, 7),
            (0, 7),
            (0, 11),
            (0, 11),
            (0, 15),
            (0, 15),
            (1, 7),
            (1, 7),
            (0, 19),
            (0, 19)
        ]
    );
    assert_eq!(plan.origins[6].iterations, [2, 2]);
    assert_eq!(
        plan.origins[6].position,
        Position {
            line: 1,
            column: 40
        }
    );
}
#[test]
fn syntax_diagnostics_locate_errors_and_reject_ambiguous_input() {
    let e = plan("# comment\r\nclock 0;\r\nwait -1;").err().unwrap();
    assert_eq!(e.position, Position { line: 3, column: 6 });
    for source in [
        "clock 0",
        "clock 0; }",
        "clock 0; repeat 2 { do attack;",
        "repeat 0 { clock 0; }",
        "repeat 1 {}",
        "clock 0; wait 1s;",
        "clock 0; do 2;",
        "at 10;",
        "clock 0; do attack; garbage",
        "",
    ] {
        assert!(plan(source).is_err(), "{source}");
    }
}
#[test]
fn no_implicit_clock_action_or_timing() {
    for source in [
        "do attack;",
        "wait 0;",
        "clock 9; do attack;",
        "clock 0; do missing;",
        "clock 0; wait 0;",
    ] {
        assert!(plan(source).is_err());
    }
    let mut f = fixture();
    f.actions.insert("empty".into(), vec![]);
    assert!(compile("clock 0; do empty;", &f.actions, &f.clocks).is_err());
    f.clocks.push(f.clocks[0].clone());
    assert!(compile(SOURCE, &f.actions, &f.clocks).is_err());
}
#[test]
fn overflow_and_expansion_limits_fail_before_simulation() {
    for source in [
        "clock 4294967296; do attack;",
        "clock 0; wait 18446744073709551616; do attack;",
        "clock 0; wait 18446744073709551615; wait 1; do attack;",
        "clock 0; repeat 10001 { do attack; }",
        "clock 0; repeat 10000 { repeat 10000 { wait 0; } }",
        "clock 0; repeat 10000 { repeat 6 { do attack; } }",
    ] {
        assert!(plan(source).is_err(), "{source}");
    }
    let deep = format!(
        "{}clock 0; do attack;{}",
        "repeat 1 {".repeat(33),
        "}".repeat(33)
    );
    assert!(plan(&deep).is_err());
    assert!(plan(&" ".repeat(MAX_SOURCE_BYTES + 1)).is_err());
}
#[test]
fn paused_clocks_preserve_local_deadlines_and_source_order_is_not_real_time() {
    let mut value: Value = serde_json::from_slice(FIXTURE).unwrap();
    value["actions"]["pause"] = json!([{"kind":"set_rate","clock":1,"rate_ppm":0}]);
    value["actions"]["resume"] = json!([{"kind":"set_rate","clock":1,"rate_ppm":1000000}]);
    let result = run(
        &serde_json::to_vec(&value).unwrap(),
        "clock 0; do pause; wait 50; do resume; clock 1; wait 10; do attack;",
    )
    .unwrap();
    let times: Vec<_> = result
        .simulation
        .trace
        .iter()
        .filter(|e| e.phase == "hit")
        .map(|e| e.real_time_us)
        .collect();
    assert_eq!(times, [61, 61]);
    assert_eq!(result.simulation.mode_time_us, 50);
}
#[test]
fn two_input_identity_is_deterministic_and_preserves_source_bytes() {
    let first = run(FIXTURE, SOURCE).unwrap();
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&run(FIXTURE, SOURCE).unwrap()).unwrap()
    );
    let comment = run(FIXTURE, &format!("{SOURCE}\n# same timeline")).unwrap();
    assert_ne!(
        first.simulation.input_sha256,
        comment.simulation.input_sha256
    );
    assert_eq!(first.rotation.fixture_sha256, sha256(FIXTURE));
    assert_eq!(first.rotation.source_sha256, sha256(SOURCE.as_bytes()));
    let mut f: Value = serde_json::from_slice(FIXTURE).unwrap();
    f["seed"] = json!(8);
    assert_ne!(
        first.simulation.input_sha256,
        run(&serde_json::to_vec(&f).unwrap(), SOURCE)
            .unwrap()
            .simulation
            .input_sha256
    );
}
#[test]
fn invalid_fixture_and_failed_spend_keep_quality_boundaries() {
    let mut f: Value = serde_json::from_slice(FIXTURE).unwrap();
    f["content_id"] = json!("ntesim-current");
    assert!(run(&serde_json::to_vec(&f).unwrap(), SOURCE).is_err());
    f["content_id"] = json!("synthetic-v1");
    f["actions"]["attack"][0]["target"] = json!("unknown");
    assert!(run(&serde_json::to_vec(&f).unwrap(), SOURCE).is_err());
    f["actions"]["attack"][0]["target"] = json!("first");
    f["actions"]["shortage"] = json!([{"kind":"spend","pool":"energy","amount":100}]);
    let result = run(&serde_json::to_vec(&f).unwrap(), "clock 0; do shortage;").unwrap();
    assert!(!result.simulation.success);
    assert_eq!(result.simulation.resources["energy"].current(), 3);
    assert_eq!(result.rotation.origins[0].action, "shortage");
    assert_eq!(result.simulation.diagnostics[0].path, "commands/0");
    f["commands"] = json!([]);
    assert!(run(&serde_json::to_vec(&f).unwrap(), SOURCE).is_err());
}
