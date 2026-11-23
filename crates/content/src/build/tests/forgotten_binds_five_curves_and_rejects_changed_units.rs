use super::*;

#[test]
fn forgotten_binds_five_curves_and_rejects_changed_units() {
    let (mut tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.character_id = "character:1046".into();
    actor.weapon_id = "weapon:fork_wuhuakuang".into();
    actor.gear_procs.clear();
    for tier in 1..=5 {
        actor.weapon_tier = tier;
        let report = crate::weapons::forgotten::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        for field in ["atk_bonus", "def_bonus"] {
            let value: Ratio =
                serde_json::from_value(report["compiled_proc"][field].clone()).unwrap();
            assert_eq!(
                ExactFraction::from_ratio(value).unwrap(),
                ExactFraction::new(16 + u128::from(tier) * 4, 100).unwrap()
            );
        }
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
        // The ATK/DEF threshold modifiers reach damage through the live gear
        // hit-modifier dispatch (gear.rs) in real scenario runs, so the
        // projection honestly claims validated execution. The per-hit live-HP
        // sampling itself is a tested hypothesis.
        assert_eq!(report["runtime_execution_validated"], true);
    }
    tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_wuhuakuang_5"]
        ["DataList"][1]["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::forgotten::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[0]
        )
        .unwrap_err()
        .contains("percentage ratio")
    );
}
