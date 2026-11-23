use super::*;

#[test]
fn knight_candy_binds_exact_refinements_and_checks_both_units() {
    let (mut tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.weapon_id = "weapon:fork_KnightCandy".into();
    actor.gear_procs.clear();
    for tier in 1..=5 {
        actor.weapon_tier = tier;
        let report = crate::weapons::knight_candy::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let value: Ratio =
            serde_json::from_value(report["compiled_proc"]["crit_damage_per_stack"].clone())
                .unwrap();
        assert_eq!(
            ExactFraction::from_ratio(value).unwrap(),
            ExactFraction::new((40 + (tier - 1) * 8).into(), 1000).unwrap()
        );
        assert_eq!(report["compiled_proc"]["duration_us"], 10_000_000);
        assert_eq!(actor.gear_procs.len(), 1);
    }
    tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_KnightCandy_5"]
        ["DataList"][1]["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::knight_candy::projection::project(
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
