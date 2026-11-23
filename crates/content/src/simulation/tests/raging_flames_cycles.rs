use super::*;

#[test]
fn raging_flames_native_hathor_cycles_keep_additive_zones_and_reset_both_stacks() {
    for (tier, lakshana, window, stack) in [(1, 150, 100, 50), (5, 250, 200, 90)] {
        let source = include_str!("../../../../../examples/raging-flames-redirect-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let executed =
            PreparedSourceScenario::new(&source, Default::default(), &Default::default())
                .unwrap()
                .run(42)
                .unwrap();
        assert!(
            executed.report.success && executed.report.execution_completed,
            "{:?}",
            executed.diagnostic
        );
        super::assert_native_replay(&executed);
        let mut req = executed.request;
        req.actors[0].static_stats.base_atk = 1000;
        req.actors[0].static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        let mut control = req.clone();
        control.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::RagingFlames { .. }));
        let plain = run_game_scenario(&control, &ExecutionLimits::default()).unwrap();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}; {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(plain["execution_completed"], true);
        let times = |kind: &str| -> Vec<u64> {
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == kind)
                .map(|event| event["time_us"].as_u64().unwrap())
                .collect()
        };
        let grants = times("raging_flames_window");
        let stacks = times("raging_flames_skill_stack");
        assert_eq!(grants.len(), 2);
        assert_eq!(stacks.len(), 4);
        assert!(grants[1] > grants[0] + 20_000_000);
        let hits = report["hit_receipts"].as_array().unwrap();
        let references = plain["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), references.len());
        let mut skill_packets = 0;
        for (hit, reference) in hits.iter().zip(references) {
            let time = hit["time_us"].as_u64().unwrap();
            let mut bonus = 0;
            if hit["actor_index"] == 0 {
                bonus += lakshana;
                let category = super::support::hit_metadata(&report, hit)["packet"]
                    ["native_damage_source_category"]
                    .as_str()
                    .unwrap();
                if let Some((generation, start)) = grants.iter().enumerate().find(|(_, start)| {
                    support::world_time_at(&report, **start)
                        <= support::world_time_at(&report, time)
                        && support::world_time_at(&report, time)
                            < support::world_time_at(&report, **start) + 10_000_000
                }) && (category.ends_with("_E") || category.ends_with("_Q"))
                {
                    bonus += window;
                    if category.ends_with("_E") {
                        let count = stacks
                            .iter()
                            .filter(|cast| **cast >= *start && **cast <= time)
                            .count();
                        assert!((1..=2).contains(&count));
                        bonus += stack * count as u128;
                        // Full recovery resets the combo: two legal first stages,
                        // seven level-one .429 ATK contacts each. Snapshot Power
                        //4 then6 supplies +60%/+90% base ATK across the periods.
                        assert_eq!(hit["damage_effect_id"], "GE_Player_Hathor_Skill5_Damage");
                        let atk = if generation == 0 { 1600 } else { 1900 };
                        let expected =
                            ExactFraction::new(429 * atk * (1250 + bonus), 1_000_000).unwrap();
                        let actual: ExactFraction =
                            serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
                        assert_eq!(actual, expected);
                        skill_packets += 1;
                    }
                }
            }
            let base: ExactFraction =
                serde_json::from_value(reference["non_crit_damage"].clone()).unwrap();
            let actual: ExactFraction =
                serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            let denominator = if hit["actor_index"] == 0 { 1250 } else { 1000 };
            assert_eq!(
                actual,
                base.mul(ExactFraction::new(denominator + bonus, denominator).unwrap())
                    .unwrap(),
                "T{tier}: {hit}"
            );
        }
        assert_eq!(skill_packets, 28);
        let pool = &report["final_actor_states"][0]["fractional_resources"]["hathor.express_delivery_power"];
        assert_eq!(pool["granted"], serde_json::json!("8/1"));
        assert_eq!(pool["spent"], serde_json::json!("4/1"));
        assert_eq!(pool["current"], serde_json::json!("4/1"));
        assert_eq!(pool["conserved"], true);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
        assert!(
            !crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .any(|buff| buff["source"] == "weapon:fork_wushoutieyu")
        );
    }
}
