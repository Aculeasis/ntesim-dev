use super::*;

#[test]
fn boxing_candy_native_chains_select_each_live_target_hp_branch_without_stacking() {
    for (tier, ordinary, low) in [
        (1, 22, 28),
        (2, 26, 34),
        (3, 30, 40),
        (4, 34, 46),
        (5, 38, 52),
    ] {
        let source = include_str!("../../../../../examples/boxing-candy-hp-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        let stats = &mut req.actors[0].static_stats;
        stats.base_atk = 1000;
        stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        stats.crit_rate = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
        stats.elemental_bonuses.clear();
        // The first recipient starts at exactly half; the second stays above
        // half throughout both periods. HP keeps advancing between inputs.
        req.targets[0].hp_current = 500000;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let mut first = true;
        let mut counts = [0, 0];
        for hit in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["actor_index"] == 0)
        {
            let coefficient = match hit["damage_effect_id"].as_str().unwrap() {
                "GE_Player_Nanally_Melee1_Damage" => 296,
                "GE_Player_Nanally_Melee2_Damage" => 388,
                "GE_Player_Nanally_Melee3_Damage" => 122,
                "GE_Player_Nanally_Melee4_Damage" => 210,
                "GE_Player_Nanally_Melee5_Damage" => 1270,
                "GE_Player_Nanally_Melee6_Damage" => 69,
                "GE_Player_Nanally_Melee9_Damage" => 771,
                "GE_Player_Nanally_Melee10_Damage" => 773,
                "GE_Player_Nanally_QTE1_Damage" => 2000,
                effect => panic!("unexpected native packet {effect}"),
            };
            let target = hit["target_index"].as_u64().unwrap() as usize;
            let bonus = if target == 0 && !first { low } else { ordinary };
            if target == 0 {
                first = false;
            }
            let expected = ExactFraction::new(coefficient * (125 + bonus), 100).unwrap();
            let actual: Ratio = serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            assert_eq!(
                ExactFraction::from_ratio(actual).unwrap(),
                expected,
                "tier {tier}, {hit}"
            );
            counts[target] += 1;
        }
        // Both ordinary melee and automatic Support hit both living dummies.
        assert_eq!(counts, [29, 29]);
        assert!(
            report["final_target_states"][0]["hp_current"]
                .as_u64()
                .unwrap()
                < 500000
        );
        assert!(
            report["final_target_states"][1]["hp_current"]
                .as_u64()
                .unwrap()
                > 500000
        );
        assert!(
            report["final_actor_states"][0]["granted_energy"]
                .as_u64()
                .unwrap()
                > 0
        );
    }
}
