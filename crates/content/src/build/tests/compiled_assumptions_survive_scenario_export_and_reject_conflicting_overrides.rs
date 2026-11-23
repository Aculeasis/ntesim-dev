use super::*;

#[test]
fn compiled_assumptions_survive_scenario_export_and_reject_conflicting_overrides() {
    let (tables, coverage, mut request) = skill_fixture();
    request.scenario.model_assumptions.insert(
        "caller.target_geometry_v1".into(),
        "Tightly grouped targets".into(),
    );
    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    assert_eq!(compiled.scenario.model_assumptions.len(), 8);
    assert_eq!(request.scenario.model_assumptions.len(), 1);
    let exported = serde_json::to_vec(&compiled.scenario).unwrap();
    let exported: GameScenarioRequest = serde_json::from_slice(&exported).unwrap();
    let report = run_game_scenario(&exported, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["model_assumptions"],
        compiled.receipt["compiled_scenario"]["model_assumptions"]
    );
    request.scenario.model_assumptions.insert(
        "mint.resonance_3_v1".into(),
        "Ignore the native bonus".into(),
    );
    assert!(
        compile(&tables, &coverage, &request, &ExecutionLimits::default())
            .unwrap_err()
            .contains("conflicting compiled")
    );
}

#[test]
fn native_build_and_runtime_talents_use_identical_typed_hypotheses() {
    let (tables, coverage, mut request) = skill_fixture();
    request.action_projection.as_mut().unwrap().policy = ActionProjectionPolicy::NativeSkillLevelV1;
    request.scenario.actors[0].talent_levels = Some([8; 3]);
    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let report = run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    for id in [
        crate::HypothesisId::SkillLevelsCoordinatesV1,
        crate::HypothesisId::SkillLevelsResonance3V1,
    ] {
        assert_eq!(report["model_assumptions"][id.as_str()], id.description());
        assert_eq!(
            compiled.scenario.model_assumptions[id.as_str()],
            id.description()
        );
    }
    let mut forged = compiled.scenario;
    forged.model_assumptions.insert(
        crate::HypothesisId::SkillLevelsResonance3V1.as_str().into(),
        "different selection rule".into(),
    );
    assert!(
        run_game_scenario(&forged, &ExecutionLimits::default())
            .unwrap_err()
            .contains("conflicting compiled model assumption")
    );
}
