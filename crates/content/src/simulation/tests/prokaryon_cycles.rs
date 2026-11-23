use super::*;

#[test]
fn prokaryon_native_chains_preserve_support_exclusion_at_every_refinement() {
    let coefficients = [
        ("GE_Player_Nanally_Melee1_Damage", 296),
        ("GE_Player_Nanally_Melee2_Damage", 388),
        ("GE_Player_Nanally_Melee3_Damage", 122),
        ("GE_Player_Nanally_Melee4_Damage", 210),
        ("GE_Player_Nanally_Melee5_Damage", 1270),
        ("GE_Player_Nanally_Melee6_Damage", 69),
        ("GE_Player_Nanally_Melee9_Damage", 771),
        ("GE_Player_Nanally_Melee10_Damage", 773),
    ];
    for tier in 1u32..=5 {
        let source = include_str!("../../../../../examples/prokaryon-normal-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut request = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        let stats = &mut request.actors[0].static_stats;
        stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        stats.crit_rate = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
        stats.elemental_bonuses.clear();
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let mut normals = 0;
        let mut supports = 0;
        for hit in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["actor_index"] == 0)
        {
            let effect = hit["damage_effect_id"].as_str().unwrap();
            let expected = if effect == "GE_Player_Nanally_QTE1_Damage" {
                supports += 1;
                ExactFraction::new(2500, 1).unwrap()
            } else {
                normals += 1;
                let coefficient = coefficients.iter().find(|(id, _)| *id == effect).unwrap().1;
                // T1..T5 .12/.13/.14/.15/.16 sum with the authored .25.
                ExactFraction::new(coefficient * (136 + u128::from(tier)), 100).unwrap()
            };
            let damage: ExactFraction =
                serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            assert_eq!(damage, expected, "T{tier}, {effect}");
        }
        assert_eq!((normals, supports), (56, 2));
        assert!(
            report["final_actor_states"][0]["granted_energy"]
                .as_u64()
                .unwrap()
                > 0
        );
    }
}
