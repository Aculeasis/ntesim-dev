use super::*;

#[test]
fn gold_wool_native_lakshana_packets_keep_one_refreshed_crit_window_through_switches() {
    for (tier, elemental, critical) in [
        (1, 20, 40),
        (2, 25, 50),
        (3, 30, 60),
        (4, 35, 70),
        (5, 40, 80),
    ] {
        let source = include_str!("../../../../../examples/gold-wool-refresh-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let (script, mut config) = crate::dsl::inspection::prepare(&source).unwrap();
        let stats = &mut config.actors[0].actor.static_stats;
        stats.base_atk = 1000;
        stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        stats.crit_rate = Ratio::one();
        stats.crit_damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
        stats.elemental_bonus = Ratio::zero();
        stats.elemental_bonuses.clear();
        config.actors[0].projection.static_stats = stats.clone();
        let result = PreparedSourceScenario::from_configuration(
            &source,
            config,
            Default::default(),
            &ExecutionLimits::default(),
            &script.options[0].loc,
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        super::assert_native_replay(&result);
        let req = result.request;
        let mut baseline = req.clone();
        baseline.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::GoldWool { .. }));
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
            .filter(|event| event["event_kind"] == "gold_wool_crit_damage")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(starts.len(), 3);
        assert_eq!(starts[0], 1_054_994);
        assert_eq!(starts[2], 31_054_994);
        assert!(starts[1] > 0 && starts[1] < 20_000_000);
        let mut counts = [0, 0];
        let hits = report["hit_receipts"].as_array().unwrap();
        let reference = plain["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), reference.len());
        for (hit, base) in hits.iter().zip(reference) {
            if hit["actor_index"] != 0 {
                continue;
            }
            let effect = hit["damage_effect_id"].as_str().unwrap();
            let coefficient = match effect.strip_suffix("_Ultra").unwrap_or(effect) {
                "GE_Player_Chaos_Melee1_Damage" => 182,
                "GE_Player_Chaos_Skill1_Damage" => 825,
                "GE_Player_Chaos_Skill2_Damage" => 2175,
                "GE_Player_Chaos_UltraSkill1_Damage" => 1273,
                "GE_Player_Chaos_UltraSkill2_Damage" => 2909,
                effect => panic!("unexpected native packet {effect}"),
            };
            let fraction = |value: &serde_json::Value| -> ExactFraction {
                let ratio: Ratio = serde_json::from_value(value.clone()).unwrap();
                ExactFraction::from_ratio(ratio).unwrap()
            };
            let expected = fraction(&base["non_crit_damage"])
                .add(ExactFraction::new(coefficient * elemental, 100).unwrap())
                .unwrap();
            assert_eq!(
                fraction(&hit["non_crit_damage"]),
                expected,
                "tier {tier}, {hit}"
            );
            let time = hit["time_us"].as_u64().unwrap();
            let active = starts.iter().any(|start| {
                support::world_time_at(&report, time) >= support::world_time_at(&report, *start)
                    && support::world_time_at(&report, time)
                        < support::world_time_at(&report, *start) + 20_000_000
            });
            counts[usize::from(active)] += 1;
            let expected_crit = expected
                .mul(ExactFraction::new(150 + if active { critical } else { 0 }, 100).unwrap())
                .unwrap();
            assert_eq!(fraction(&hit["crit_damage"]), expected_crit);
            assert_eq!(hit["critical"], true);
            assert_eq!(hit["rounded_damage"], expected_crit.round_u64().unwrap());
        }
        assert!(counts.iter().all(|count| *count > 0));
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    }
}
