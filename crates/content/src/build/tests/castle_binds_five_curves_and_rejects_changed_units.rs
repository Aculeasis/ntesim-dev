use super::*;

fn inputs() -> (BTreeMap<String, Value>, GameScenarioRequest, GearProjection) {
    let tables = weapon_parameter_tables();
    let request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/native-gear-built-game-research.json"
    )))
    .unwrap();
    let mut scenario = request.scenario;
    scenario.actors[0].character_id = "character:1021".into();
    scenario.actors[0].weapon_id = "weapon:fork_Castle".into();
    scenario.actors[0].gear_procs.clear();
    (
        tables,
        scenario,
        GearProjection {
            policy: GearProjectionPolicy::NativeWeaponCurveV1,
            hypothesis: crate::weapons::castle::HYPOTHESIS_ID.description().into(),
        },
    )
}

#[test]
fn castle_binds_five_curves_and_rejects_changed_units() {
    let (tables, mut scenario, projection) = inputs();
    for (tier, numerator, denominator) in
        [(1, 3, 25), (2, 7, 50), (3, 4, 25), (4, 9, 50), (5, 1, 5)]
    {
        scenario.actors[0].weapon_tier = tier;
        let receipts = project_gear(&tables, &mut scenario, &projection).unwrap();
        assert_eq!(
            receipts[0]["compiled_proc"]["healing_bonus"],
            json!(format!("{}/{}", numerator, denominator))
        );
        assert_eq!(receipts[0]["compiled_proc"]["duration_us"], 10_000_000);
    }
    let (tables, mut scenario, projection) = inputs();
    let mut wrong_unit = tables.clone();
    *wrong_unit
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()
        .pointer_mut("/rows/upgradestar_pack_fork_Castle_5/DataList/0/bIsPercent")
        .unwrap() = json!(false);
    assert!(project_gear(&wrong_unit, &mut scenario, &projection).is_err());
    let mut missing_curve = tables;
    missing_curve.get_mut("CT_ForkBuff").unwrap()["rows"]
        .as_object_mut()
        .unwrap()
        .remove("buff_Castle_HealUp");
    assert!(project_gear(&missing_curve, &mut scenario, &projection).is_err());

    let (tables, _scenario, _projection) = inputs();
    let mut incompatible = scenario.actors[0].clone();
    incompatible.character_id = "character:1003".into();
    assert!(
        crate::weapons::castle::projection::compile_native(
            &tables,
            1,
            "ntesim-current",
            &incompatible
        )
        .unwrap_err()
        .contains("group is incompatible")
    );

    let (tables, mut scenario, projection) = inputs();
    scenario.actors[0].gear_procs = vec![
        GearProcConfig::CastleHealingBonus {
            healing_bonus: Ratio {
                numerator: 1,
                denominator: 5
            },
            duration_us: 10_000_000,
        };
        2
    ];
    assert!(project_gear(&tables, &mut scenario, &projection).is_err());
}
