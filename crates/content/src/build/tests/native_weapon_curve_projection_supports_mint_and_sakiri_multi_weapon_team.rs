use super::*;

#[test]
fn native_weapon_curve_projection_supports_mint_and_sakiri_multi_weapon_team() {
    let (mut tables, coverage, _) = skill_fixture();
    for id in ["1003", "1046", "1075"] {
        tables.insert(
            format!("{id}AwakenEffect"),
            ntesim_data::resonance_levels::overlay(id).unwrap(),
        );
    }

    let mut request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/universal-skill-level-built-game-research.json"
    )))
    .unwrap();

    request.gear_projection = Some(GearProjection {
        policy: GearProjectionPolicy::NativeWeaponCurveV1,
        hypothesis: "Equipped native weapon parameters select exact refinement values: Mind Royale grants energy on Skill; Good Boy grants team ATK percentage on Ultimate.".into(),
    });

    let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
    let gear_receipts = compiled.receipt["gear_receipts"].as_array().unwrap();
    assert_eq!(gear_receipts.len(), 2);

    assert_eq!(
        gear_receipts[0]["weapon_id"],
        "weapon:fork_lingganzhongjiezhe"
    );
    assert_eq!(gear_receipts[0]["refinement"], 5);
    assert_eq!(
        gear_receipts[0]["compiled_proc"],
        json!({
            "kind": "mind_royale_energy",
            "energy_grant": 18,
            "cooldown_us": 20_000_000
        })
    );

    assert_eq!(gear_receipts[1]["weapon_id"], "weapon:fork_mofeikesi");
    assert_eq!(gear_receipts[1]["refinement"], 1);
    assert_eq!(
        gear_receipts[1]["compiled_proc"],
        json!({
            "kind": "good_boy_team_atk",
            "control_atk_bonus":"3/50",
            "team_atk_bonus": "1/10",
            "duration_us": 20_000_000
        })
    );

    assert!(
        compiled
            .scenario
            .model_assumptions
            .contains_key("weapons.native_curve_v1")
    );

    let execution = run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap();
    assert_eq!(execution["execution_completed"], true);

    let mut request_r5 = request.clone();
    request_r5.scenario.actors[1].weapon_tier = 5;
    request_r5.builds[1].build.weapon_tier = 5;
    let compiled_r5 =
        compile(&tables, &coverage, &request_r5, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        compiled_r5.receipt["gear_receipts"][1]["compiled_proc"],
        json!({
            "kind": "good_boy_team_atk",
            "control_atk_bonus":"1/10",
            "team_atk_bonus": "4/25",
            "duration_us": 20_000_000
        })
    );

    let mut request_unsupported = request.clone();
    request_unsupported.scenario.actors[1].weapon_id = "weapon:fork_unsupported".into();
    request_unsupported.builds[1].build.weapon_id = "weapon:fork_unsupported".into();
    assert!(
        compile(
            &tables,
            &coverage,
            &request_unsupported,
            &ExecutionLimits::default()
        )
        .is_err()
    );
}
