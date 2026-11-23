use super::*;

#[test]
fn good_boy_native_cycles_add_charge_efficiency_and_keep_off_field_energy_at_all_ranks() {
    for (tier, numerator) in [(1, 168), (2, 171), (3, 174), (4, 177), (5, 180)] {
        let source = include_str!("../../../../../examples/good-boy-energy-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        // 1 + authored .50 + weapon .18/.21/.24/.27/.30, never 1.5 * 1.18.
        let multiplier = req.actors[0]
            .gear_procs
            .iter()
            .find_map(|p| match p {
                GearProcConfig::ChargeEfficiency { multiplier } => Some(*multiplier),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            ExactFraction::from_ratio(multiplier).unwrap(),
            ExactFraction::new(numerator, 100).unwrap()
        );
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
        assert!(
            report["model_assumptions"]
                .get("weapons.good_boy_v3")
                .is_some()
        );
        let events = report["trace"].as_array().unwrap();
        let fanout_count = events
            .iter()
            .filter(|event| event.get("energy_fanout").is_some())
            .count();
        assert!(fanout_count > 0);
        let grants: Vec<_> = events
            .iter()
            .flat_map(|event| {
                serde_json::from_value::<crate::simulation::model::GameEventReceipt>(event.clone())
                    .unwrap()
                    .energy_grants()
            })
            .collect();
        assert!(!grants.is_empty());
        let mut off_field = 0;
        for grant in grants {
            let carry = grant
                .charge_remainder_before
                .unwrap_or(ExactFraction::new(0, 1).unwrap());
            let scaled = grant
                .charge_basis
                .unwrap()
                .mul(if grant.recipient_actor_index == 0 {
                    ExactFraction::new(numerator, 100).unwrap()
                } else {
                    ExactFraction::new(1, 1).unwrap()
                })
                .unwrap()
                .add(carry)
                .unwrap();
            assert_eq!(grant.gross, (scaled.numerator / scaled.denominator) as u64);
            if grant.recipient_actor_index == 0 && grant.source_actor_index == 1 {
                off_field += 1;
            }
        }
        assert!(off_field > 0);
        assert_eq!(
            report,
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
        );
    }
}

#[test]
fn good_boy_static_charge_rejects_changed_units_missing_and_extra_dependencies() {
    for case in 0..3 {
        let mut tables = super::support::weapon_projection_tables();
        let list = tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_mofeikesi_1"]["DataList"].as_array_mut().unwrap();
        match case {
            0 => list[0]["bIsPercent"] = serde_json::json!(false),
            1 => {
                list.pop();
            }
            _ => {
                let mut extra = list[0].clone();
                extra["NameID"] = serde_json::json!("buff_mamen_CosmosUp");
                list.push(extra);
            }
        }
        assert!(
            crate::weapons::good_boy::projection::static_charge(
                &tables,
                "ntesim-current",
                Some("weapon:fork_mofeikesi"),
                1
            )
            .is_err()
        );
    }
}
