use super::*;

#[test]
fn blast_candy_native_refinements_bind_exact_ratios_and_reject_changed_units() {
    let (tables, _, request) = fireflies_fixture();
    for (tier, numerator) in [(1, 25), (2, 30), (3, 35), (4, 40), (5, 45)] {
        let mut actor = request.scenario.actors[0].clone();
        actor.weapon_id = "weapon:fork_BlastCandy".into();
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let projected = crate::weapons::blast_candy::projection::project(
            &tables,
            1,
            "ntesim-current",
            20_000_000,
            &mut actor,
            &[],
        )
        .unwrap();
        assert_eq!(
            ExactFraction::from_ratio(
                serde_json::from_value(projected["compiled_proc"]["atk_bonus"].clone()).unwrap()
            )
            .unwrap(),
            ExactFraction::new(numerator, 100).unwrap()
        );
        assert_eq!(projected["compiled_proc"]["duration_us"], 10_000_000);
        assert_eq!(
            projected["native_parameter_report"]["selected_refinement_row"],
            format!("upgradestar_pack_fork_BlastCandy_{tier}")
        );
    }
    let mut actor = request.scenario.actors[0].clone();
    actor.weapon_id = "weapon:fork_BlastCandy".into();
    actor.weapon_tier = 1;
    let mut wrong = tables;
    wrong.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_BlastCandy_1"]
        ["DataList"][0]["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::blast_candy::projection::project(
            &wrong,
            1,
            "ntesim-current",
            20_000_000,
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}
