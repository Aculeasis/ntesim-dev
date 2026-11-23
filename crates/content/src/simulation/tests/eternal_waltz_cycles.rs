use super::*;

#[test]
fn eternal_waltz_native_mental_bursts_keep_hp_scaling_and_wearer_scope_for_two_paid_cycles() {
    for (tier, mental) in [(1, 10), (2, 12), (3, 14), (4, 16), (5, 18)] {
        let source = include_str!("../../../../../examples/eternal-waltz-mental-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = live_source_request(&source);
        let actor = &mut req.actors[0];
        // Character HP17089; weapon secondary41.25% and passive20..32% add once.
        let expected_hp = ExactFraction::new(17089 * (16125 + (tier - 1) * 300), 10000)
            .unwrap()
            .round_u64()
            .unwrap();
        assert_eq!(actor.static_stats.hp, expected_hp);
        assert_eq!(actor.initial_hp_max, expected_hp);
        actor.static_stats.hp = 10_000;
        actor.static_stats.base_hp = Some(10_000);
        actor.static_stats.hp_percent = Ratio::zero();
        actor.static_stats.hp_flat = Ratio::zero();
        actor.initial_hp = 10_000;
        actor.initial_hp_max = 10_000;
        actor.static_stats.crit_rate = Ratio::one();
        actor.static_stats.crit_damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
        let mut baseline = req.clone();
        baseline.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::EternalWaltz { .. }));
        let plain = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(plain["execution_completed"], true);
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "eternal_waltz_mental")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(grants.len(), 2);
        assert!(grants[1] > grants[0] + 10_000_000);
        let mut mental_hits = 0;
        let hits = report["hit_receipts"].as_array().unwrap();
        let references = plain["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), references.len());
        for (hit, reference) in hits.iter().zip(references) {
            let coefficient = match hit["damage_effect_id"].as_str().unwrap() {
                "GE_Player_Fadia_UltraSkill1_Damage" => 67,
                "GE_Player_Fadia_UltraSkill2_Damage" => 21,
                "GE_Player_Fadia_UltraSkill3_Damage" => 108,
                _ => {
                    assert_eq!(hit["non_crit_damage"], reference["non_crit_damage"]);
                    assert_eq!(hit["crit_damage"], reference["crit_damage"]);
                    continue;
                }
            };
            mental_hits += 1;
            // B4 adds .10 of base HP10000 once at combat membership:11000.
            let expected = ExactFraction::new(coefficient * (125 + mental) * 11, 100).unwrap();
            let actual: ExactFraction =
                serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            assert_eq!(actual, expected, "tier {tier}: {hit}");
            assert_eq!(hit["scaling_stat"], "hp");
            assert_eq!(hit["critical"], true);
            assert_eq!(
                hit["rounded_damage"],
                expected
                    .mul(ExactFraction::new(3, 2).unwrap())
                    .unwrap()
                    .round_u64()
                    .unwrap()
            );
        }
        assert_eq!(mental_hits, 12);
        assert_eq!(report["final_actor_states"][0]["hp_max"], 11000);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
        assert!(
            !crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .any(|buff| buff["source"] == "weapon:fork_Arachne")
        );
    }
}
