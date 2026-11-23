use super::*;

#[test]
fn blast_candy_native_paid_bursts_use_base_atk_and_preserve_expiry_between_cycles() {
    for (tier, active_atk) in [(1, 2250), (2, 2300), (3, 2350), (4, 2400), (5, 2450)] {
        let source = include_str!("../../../../../examples/blast-candy-burst-cycles.nte")
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
        stats.damage_bonus = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
        stats.elemental_bonuses.clear();
        let mut baseline = req.clone();
        baseline.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::BlastCandyAtk { .. }));
        let plain = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(plain["execution_completed"], true);
        let starts: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "blast_candy_atk")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(starts.len(), 2);
        assert!(starts[1] >= 20_000_000);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
        let hits = report["hit_receipts"].as_array().unwrap();
        let reference = plain["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), reference.len());
        let mut counts = [0, 0];
        for (hit, base) in hits.iter().zip(reference) {
            assert_eq!(hit["time_us"], base["time_us"]);
            if hit["actor_index"] != 0 {
                continue;
            }
            let time = hit["time_us"].as_u64().unwrap();
            let active = starts
                .iter()
                .any(|start| time >= *start && time < start + 10_000_000);
            counts[usize::from(active)] += 1;
            let ratio: Ratio = serde_json::from_value(base["non_crit_damage"].clone()).unwrap();
            let expected = ExactFraction::from_ratio(ratio)
                .unwrap()
                .mul(ExactFraction::new(if active { active_atk } else { 2000 }, 2000).unwrap())
                .unwrap();
            let actual: Ratio = serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            assert_eq!(
                ExactFraction::from_ratio(actual).unwrap(),
                expected,
                "tier {tier}, time {time}"
            );
        }
        assert!(counts.iter().all(|count| *count > 0));
        assert_eq!(
            report["final_actor_states"][0]["spent_energy"],
            plain["final_actor_states"][0]["spent_energy"]
        );
    }
}
