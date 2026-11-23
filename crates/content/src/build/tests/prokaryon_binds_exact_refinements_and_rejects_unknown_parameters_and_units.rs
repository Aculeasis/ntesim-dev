use super::*;

#[test]
fn prokaryon_binds_exact_refinements_and_rejects_unknown_parameters_and_units() {
    let (mut tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.weapon_id = "weapon:fork_Prokaryon".into();
    actor.gear_procs.clear();
    for tier in 1..=5 {
        actor.weapon_tier = tier;
        let report = crate::weapons::prokaryon::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let value: Ratio =
            serde_json::from_value(report["compiled_proc"]["damage_bonus"].clone()).unwrap();
        assert_eq!(
            ExactFraction::from_ratio(value).unwrap(),
            ExactFraction::new(u128::from(11 + tier), 100).unwrap()
        );
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
    }
    let row = &mut tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_Prokaryon_5"]
        ["DataList"];
    row[0]["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::prokaryon::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[0]
        )
        .unwrap_err()
        .contains("unit hint")
    );
    let row = &mut tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_Prokaryon_5"]
        ["DataList"];
    row[0]["bIsPercent"] = json!(true);
    row.as_array_mut()
        .unwrap()
        .push(json!({"NameID":"buff_BlastCandy_CD","bIsPercent":false}));
    assert!(
        crate::weapons::prokaryon::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[0]
        )
        .unwrap_err()
        .contains("parameters")
    );
}
