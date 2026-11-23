use super::*;

#[test]
fn noros_binds_atk_duration_and_separate_cooldown_curves() {
    let (mut tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.weapon_id = "weapon:fork_nonos".into();
    actor.gear_procs.clear();
    for tier in 1..=5 {
        actor.weapon_tier = tier;
        let report = crate::weapons::noros::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let bonus: Ratio =
            serde_json::from_value(report["compiled_proc"]["atk_bonus"].clone()).unwrap();
        assert_eq!(
            ExactFraction::from_ratio(bonus).unwrap(),
            ExactFraction::new(u128::from(11 + tier), 100).unwrap()
        );
        assert_eq!(report["compiled_proc"]["duration_us"], 10_000_000);
        assert_eq!(report["compiled_proc"]["cooldown_us"], 20_000_000);
        assert_eq!(actor.gear_procs.len(), 1);
    }
    tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_nonos_5"]
        ["DataList"][2]["bIsPercent"] = json!(true);
    assert!(
        crate::weapons::noros::projection::project(&tables, 1, "ntesim-current", &mut actor, &[0])
            .unwrap_err()
            .contains("unit hint")
    );
}
