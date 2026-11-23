use super::*;

#[test]
fn paper_plane_native_build_compiler_preserves_equipped_set_and_executes_its_passive() {
    let (tables, coverage, mut request) = fireflies_fixture();
    let limits = ExecutionLimits::default();
    // Preserve the already source-bound Fireflies handler while changing the weapon.
    request.scenario = compile(&tables, &coverage, &request, &limits)
        .unwrap()
        .scenario;
    request.scenario.actors[0].weapon_id = "weapon:fork_PaperPlane".into();
    request.scenario.actors[0]
        .gear_procs
        .retain(|proc| matches!(proc, GearProcConfig::FirefliesCritDamage { .. }));
    request.builds[0].build.weapon_id = "weapon:fork_PaperPlane".into();
    request
        .scenario
        .model_assumptions
        .remove("build.gear_projection_v1");
    request.gear_projection = Some(GearProjection {
        policy: GearProjectionPolicy::NativeWeaponCurveV1,
        hypothesis: crate::weapons::paper_plane::HYPOTHESIS_ID
            .description()
            .into(),
    });
    let compiled = compile(&tables, &coverage, &request, &limits).unwrap();
    assert_eq!(
        compiled.receipt["gear_receipts"][0]["compiled_proc"]["kind"],
        "paper_plane_damage"
    );
    let report = run_game_scenario(&compiled.scenario, &limits).unwrap();
    assert_eq!(report["execution_completed"], true);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::paper_plane::HYPOTHESIS_ID.as_str()],
        crate::weapons::paper_plane::HYPOTHESIS_ID.description()
    );
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_kind"] == "fireflies_crit_stack")
    );
    let mut baseline = compiled.scenario;
    baseline.actors[0]
        .gear_procs
        .retain(|proc| !matches!(proc, GearProcConfig::PaperPlaneDamage { .. }));
    let bare = run_game_scenario(&baseline, &limits).unwrap();
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        bare["hit_receipts"][0]["non_crit_damage"]
    );
    assert_ne!(
        report["hit_receipts"][2]["non_crit_damage"],
        bare["hit_receipts"][2]["non_crit_damage"]
    );
}
