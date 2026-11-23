use super::*;

#[test]
fn boxing_candy_binds_both_exact_refinement_curves_and_checks_units() {
    let (mut tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.weapon_id = "weapon:fork_BoxingCandy".into();
    actor.gear_procs.clear();
    for tier in 1..=5 {
        actor.weapon_tier = tier;
        let report = crate::weapons::boxing_candy::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        for (field, numerator) in [
            ("damage_bonus", 22 + (tier - 1) * 4),
            ("low_hp_damage_bonus", 28 + (tier - 1) * 6),
        ] {
            let value: Ratio =
                serde_json::from_value(report["compiled_proc"][field].clone()).unwrap();
            assert_eq!(
                ExactFraction::from_ratio(value).unwrap(),
                ExactFraction::new(numerator.into(), 100).unwrap()
            );
        }
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
    }
    tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_BoxingCandy_5"]
        ["DataList"][1]["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::boxing_candy::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[0]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}
