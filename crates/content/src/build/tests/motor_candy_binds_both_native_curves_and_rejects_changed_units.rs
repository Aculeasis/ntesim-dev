use super::*;

#[test]
fn motor_candy_binds_both_native_curves_and_rejects_changed_units() {
    let (mut tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.weapon_id = "weapon:fork_MotorCandy".into();
    actor.gear_procs.clear();
    for tier in 1..=5 {
        actor.weapon_tier = tier;
        let report = crate::weapons::motor_candy::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let ratio: Ratio =
            serde_json::from_value(report["compiled_proc"]["atk_per_stack"].clone()).unwrap();
        assert_eq!(
            ExactFraction::from_ratio(ratio).unwrap(),
            ExactFraction::new(u128::from(4 + tier), 100).unwrap()
        );
        assert_eq!(report["compiled_proc"]["interval_us"], 1_000_000);
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
    }
    tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_MotorCandy_5"]
        ["DataList"][0]["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::motor_candy::projection::project(
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
