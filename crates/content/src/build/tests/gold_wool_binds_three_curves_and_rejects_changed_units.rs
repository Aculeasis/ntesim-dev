use super::*;

#[test]
fn gold_wool_binds_three_curves_and_rejects_changed_units() {
    let (mut tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.weapon_id = "weapon:fork_GoldWool".into();
    actor.gear_procs.clear();
    for tier in 1..=5 {
        actor.weapon_tier = tier;
        let report = crate::weapons::gold_wool::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let lakshana: Ratio = serde_json::from_value(proc["lakshana_bonus"].clone()).unwrap();
        let crit: Ratio = serde_json::from_value(proc["crit_damage_bonus"].clone()).unwrap();
        assert_eq!(
            ExactFraction::from_ratio(lakshana).unwrap(),
            ExactFraction::new(u128::from(3 + tier), 20).unwrap()
        );
        assert_eq!(
            ExactFraction::from_ratio(crit).unwrap(),
            ExactFraction::new(u128::from(3 + tier), 10).unwrap()
        );
        assert_eq!(proc["duration_us"], 20_000_000);
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
    }
    let row = &mut tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_GoldWool_5"]
        ["DataList"];
    row[2]["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::gold_wool::projection::project(
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
