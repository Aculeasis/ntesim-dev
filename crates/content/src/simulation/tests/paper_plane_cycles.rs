use super::*;

#[test]
fn paper_plane_native_two_cycle_packets_use_only_wearer_anima_e_and_q() {
    for (tier, bonus) in [(1, 160), (2, 190), (3, 220), (4, 250), (5, 280)] {
        let source = include_str!("../../../../../examples/clear-skies-mint-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        req.actors[0].static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        let mut baseline = req.clone();
        baseline.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::PaperPlaneDamage { .. }));
        let plain = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert_eq!(plain["execution_completed"], true);
        let hits = report["hit_receipts"].as_array().unwrap();
        let bare = plain["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), bare.len());
        let mut counts = [0; 3];
        for (hit, base) in hits.iter().zip(bare) {
            for key in [
                "time_us",
                "command_index",
                "target_index",
                "damage_effect_id",
            ] {
                assert_eq!(hit[key], base[key]);
            }
            let effect = hit["damage_effect_id"].as_str().unwrap();
            let eligible = hit["actor_index"] == 0
                && (effect == "GE_Player_Mint_Skill1_Damage_New"
                    || effect.starts_with("GE_Player_Mint_UtraSkill"));
            counts[if eligible {
                0
            } else if hit["actor_index"] == 0 {
                1
            } else {
                2
            }] += 1;
            let expected: ExactFraction =
                serde_json::from_value(base["non_crit_damage"].clone()).unwrap();
            // The existing .25 bonus and weapon bonus sum; do not multiply 1.25 by 1+p.
            let expected = expected
                .mul(ExactFraction::new(1000 + if eligible { bonus } else { 0 }, 1000).unwrap())
                .unwrap();
            let actual: ExactFraction =
                serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            assert_eq!(actual, expected, "T{tier}, {effect}");
        }
        assert!(counts.iter().all(|count| *count > 0));
        assert_eq!(counts[0], 36);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
    }
}
