use super::*;

#[test]
fn motor_candy_native_field_cycles_rebuild_capped_base_atk_without_buffing_teammates() {
    for (tier, per_stack) in [(1, 5), (2, 6), (3, 7), (4, 8), (5, 9)] {
        let source = include_str!("../../../../../examples/motor-candy-field-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        let stats = &mut req.actors[0].static_stats;
        stats.base_atk = 1000;
        stats.atk = Ratio {
            numerator: 2000,
            denominator: 1,
        };
        stats.crit_rate = Ratio::zero();
        let mut baseline = req.clone();
        baseline.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::MotorCandyAtk { .. }));
        let plain = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
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
        let entries = times("motor_candy_field_entry");
        let departures = times("motor_candy_reset");
        assert_eq!(entries.len(), 2);
        assert_eq!(departures.len(), 2);
        assert_eq!(entries[0], 0);
        let ticks = times("motor_candy_atk_stack");
        let expected_ticks: Vec<_> = entries
            .iter()
            .flat_map(|entry| (1..=5).map(move |stack| entry + stack * 1_000_000))
            .collect();
        assert_eq!(ticks, expected_ticks);
        let hits = report["hit_receipts"].as_array().unwrap();
        let reference = plain["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), reference.len());
        let mut seen = [0; 6];
        let mut peer = 0;
        for (hit, base) in hits.iter().zip(reference) {
            assert_eq!(hit["time_us"], base["time_us"]);
            let time = hit["time_us"].as_u64().unwrap();
            let wearer = hit["actor_index"] == 0;
            let stacks = if wearer {
                entries
                    .iter()
                    .zip(&departures)
                    .find_map(|(entry, departure)| {
                        (time >= *entry && time < *departure)
                            .then(|| ((time - entry) / 1_000_000).min(5))
                    })
                    .unwrap_or(0)
            } else {
                peer += 1;
                0
            };
            if wearer {
                seen[stacks as usize] += 1;
            }
            let fraction = |value: &serde_json::Value| -> ExactFraction {
                let ratio: Ratio = serde_json::from_value(value.clone()).unwrap();
                ExactFraction::from_ratio(ratio).unwrap()
            };
            // Each stack adds base1000*ratio; the compiled2000 is not its base.
            let expected = fraction(&base["non_crit_damage"])
                .mul(ExactFraction::new(2000 + u128::from(stacks) * per_stack * 10, 2000).unwrap())
                .unwrap();
            assert_eq!(
                fraction(&hit["non_crit_damage"]),
                expected,
                "tier {tier}, time {time}"
            );
        }
        assert!(seen[0] > 0 && seen[1] > 0 && seen[5] > 0);
        assert!(peer > 0);
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
        assert!(
            !crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .any(|buff| buff["source"] == "weapon:fork_MotorCandy")
        );
    }
}
