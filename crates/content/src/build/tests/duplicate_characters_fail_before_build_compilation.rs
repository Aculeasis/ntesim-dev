use super::*;

#[test]
fn duplicate_characters_fail_before_build_compilation() {
    let mut request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/built-game-research.json"
    )))
    .unwrap();
    let mut actor = request.scenario.actors[0].clone();
    actor.slot = 1;
    request.scenario.actors.push(actor);
    let mut build = request.builds[0].clone();
    build.slot = 1;
    request.builds.push(build);
    let error = run(
        &BTreeMap::new(),
        &json!({}),
        &request,
        &ExecutionLimits::default(),
    )
    .unwrap_err();
    assert!(error.contains("duplicate actor slot or character"));
}
