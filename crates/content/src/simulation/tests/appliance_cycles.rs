use super::*;

#[test]
fn appliance_native_redirect_hits_add_bonus_without_boosting_paid_bursts() {
    for (tier, skill_damage) in [(1, 1233), (2, 1242), (3, 1251), (4, 1260), (5, 1269)] {
        let text = include_str!("../../../../../examples/real-music-redirect-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut compiled = crate::dsl::compiler::compile_scenario(&text, Default::default())
            .unwrap()
            .request;
        compiled.actors[0].static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        compiled.actors[0].static_stats.crit_rate = Ratio::zero();
        let result = run_game_scenario(&compiled, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            result["execution_completed"], true,
            "{}",
            result["halt_diagnostic"]
        );
        let mut redirect_hits = 0;
        let mut ultimate_hits = 0;
        for receipt in result["hit_receipts"].as_array().unwrap() {
            if receipt["actor_index"] != 0 {
                continue;
            }
            let effect = receipt["damage_effect_id"].as_str().unwrap();
            let expected = if effect == "GE_Player_Mint_Skill1_Damage_New" {
                redirect_hits += 1;
                // .900 * 1000 * (1 + .25 + [.12,.13,.14,.15,.16]).
                ExactFraction::new(skill_damage, 1).unwrap()
            } else if effect.starts_with("GE_Player_Mint_UtraSkill") {
                ultimate_hits += 1;
                let coefficient = match effect {
                    "GE_Player_Mint_UtraSkill1_Damage_New" => 1326,
                    "GE_Player_Mint_UtraSkill7_Damage_New" => 3735,
                    _ => 588,
                };
                ExactFraction::new(coefficient * 5, 4).unwrap()
            } else {
                continue;
            };
            let measured: ExactFraction =
                serde_json::from_value(receipt["non_crit_damage"].clone()).unwrap();
            assert_eq!(measured, expected, "refinement {tier}, {effect}");
        }
        assert_eq!((redirect_hits, ultimate_hits), (8, 28));
        assert_eq!(result["final_actor_states"][0]["spent_energy"], 200);
    }
}
