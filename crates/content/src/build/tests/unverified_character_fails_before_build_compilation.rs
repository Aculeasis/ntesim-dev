use super::*;

#[test]
fn unverified_character_fails_before_build_compilation() {
    let request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/built-game-research.json"
    )))
    .unwrap();
    let coverage = json!({"entities":[{"id":"character:1019","availability":"unverified"}]});
    let error = run(
        &BTreeMap::new(),
        &coverage,
        &request,
        &ExecutionLimits::default(),
    )
    .unwrap_err();
    assert!(error.contains("released character"));
}
