use super::*;

#[test]
fn lunarphase_native_shinku_cycles_separate_base_atk_cosmos_and_defense_zones() {
    for (tier, atk_up, cosmos, ignore) in [
        (1, 200, 320, 1200),
        (2, 230, 368, 1380),
        (3, 260, 416, 1560),
        (4, 290, 464, 1740),
        (5, 320, 512, 1920),
    ] {
        let source = include_str!("../../../../../examples/blushing-mirage-cosmos-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        let stats = &mut req.actors[0].static_stats;
        stats.base_atk = 1000;
        stats.atk = Ratio {
            numerator: 1500,
            denominator: 1,
        };
        stats.crit_rate = Ratio::one();
        let mut control = req.clone();
        control.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::LunarPhase { .. }));
        let plain = run_game_scenario(&control, &ExecutionLimits::default()).unwrap();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true, "{report}");
        assert_eq!(plain["execution_completed"], true);
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "lunarphase_proc")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(grants.len(), 2);
        assert!(grants[1] > grants[0] + 20_000_000);
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), plain["hit_receipts"].as_array().unwrap().len());
        let mut initial_bursts = 0;
        for (hit, reference) in hits.iter().zip(plain["hit_receipts"].as_array().unwrap()) {
            let actual: ExactFraction =
                serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            let base: ExactFraction =
                serde_json::from_value(reference["non_crit_damage"].clone()).unwrap();
            if hit["actor_index"] != 0 {
                assert_eq!(actual, base);
                continue;
            }
            let time = hit["time_us"].as_u64().unwrap();
            let active = grants
                .iter()
                .any(|start| *start <= time && time < *start + 20_000_000);
            let surging = grants
                .iter()
                .any(|start| *start <= time && time < *start + 13_000_000);
            let damage_zone = if surging { 1550 } else { 1250 };
            // The unconditional ATK is base1000*p, not static1500*p.
            let mut expected = base
                .mul(ExactFraction::new(1500 + atk_up, 1500).unwrap())
                .unwrap();
            if active {
                expected = expected
                    .mul(ExactFraction::new(damage_zone + cosmos, damage_zone).unwrap())
                    .unwrap()
                    // DEF1080 vs level80: 1/(2-ignore), formerly 1/2.
                    .mul(ExactFraction::new(20_000, 20_000 - ignore).unwrap())
                    .unwrap();
            }
            assert_eq!(actual, expected, "T{tier}: {hit}");
            if hit["damage_effect_id"] == "GE_Player_Shinku_UltraSkill1_Damage" {
                initial_bursts += 1;
                assert_eq!(
                    actual,
                    ExactFraction::new(
                        185 * (1500 + atk_up) * (1550 + cosmos) * 10_000,
                        1000 * 1000 * (20_000 - ignore),
                    )
                    .unwrap()
                );
            }
        }
        assert_eq!(initial_bursts, 2);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
        assert!(
            !crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .any(|buff| buff["source"] == "weapon:fork_lunarphase")
        );
    }
}
