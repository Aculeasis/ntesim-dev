use super::*;

#[test]
fn paper_plane_binds_five_exact_refinements_and_rejects_extra_parameters() {
    let (mut tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.weapon_id = "weapon:fork_PaperPlane".into();
    actor.gear_procs.clear();
    for (tier, numerator) in [(1, 16), (2, 19), (3, 22), (4, 25), (5, 28)] {
        actor.weapon_tier = tier;
        let report = crate::weapons::paper_plane::projection::project(
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
            ExactFraction::new(numerator, 80).unwrap()
        );
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
    }
    let row = &mut tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_PaperPlane_5"]
        ["DataList"];
    row.as_array_mut()
        .unwrap()
        .push(json!({"NameID":"buff_BlastCandy_CD","bIsPercent":false}));
    assert!(
        crate::weapons::paper_plane::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[0]
        )
        .unwrap_err()
        .contains("parameters or unit hint")
    );
}
