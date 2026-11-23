use super::*;

fn inputs() -> (BTreeMap<String, Value>, GameScenarioRequest, GearProjection) {
    let tables = weapon_parameter_tables();
    let request: Request = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/native-gear-built-game-research.json"
    )))
    .unwrap();
    let mut scenario = request.scenario;
    scenario.actors[0].character_id = "character:1033".into();
    scenario.actors[0].weapon_id = "weapon:fork_BitterCake".into();
    scenario.actors[0].gear_procs.clear();
    (
        tables,
        scenario,
        GearProjection {
            policy: GearProjectionPolicy::NativeWeaponCurveV1,
            hypothesis: crate::weapons::bittercake::HYPOTHESIS_ID
                .description()
                .into(),
        },
    )
}

#[test]
fn bittercake_binds_all_refinements_and_rejects_changed_units_or_dependencies() {
    let (tables, mut scenario, projection) = inputs();
    assert_eq!(
        ntesim_data::weapon_definitions::lookup(&tables, "fork_BitterCake")
            .unwrap()
            .1["ElementData"]["ApplyGroupType"],
        "ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE"
    );
    for (tier, (numerator, denominator)) in
        (1..=5).zip([(13, 50), (31, 100), (9, 25), (41, 100), (23, 50)])
    {
        scenario.actors[0].weapon_tier = tier;
        let reports = project_gear(&tables, &mut scenario, &projection).unwrap();
        assert_eq!(
            reports[0]["compiled_proc"]["def_bonus"],
            json!(format!("{}/{}", numerator, denominator))
        );
        assert_eq!(reports[0]["compiled_proc"]["duration_us"], 10_000_000);
        assert_eq!(reports[0]["compiled_proc"]["cooldown_us"], 20_000_000);
    }

    let (tables, mut scenario, projection) = inputs();
    let mut wrong_unit = tables.clone();
    *wrong_unit
        .get_mut("DT_ForkUpgradeStarDataTable")
        .unwrap()
        .pointer_mut("/rows/upgradestar_pack_fork_BitterCake_5/DataList/0/bIsPercent")
        .unwrap() = json!(false);
    assert!(project_gear(&wrong_unit, &mut scenario, &projection).is_err());
    let mut missing_curve = tables;
    missing_curve.get_mut("CT_ForkBuff").unwrap()["rows"]
        .as_object_mut()
        .unwrap()
        .remove("buff_BitterCake_CD2");
    assert!(project_gear(&missing_curve, &mut scenario, &projection).is_err());
}
