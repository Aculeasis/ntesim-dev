use super::*;

#[test]
fn noros_native_pact_reads_live_base_atk_windows_and_continuing_independent_icd() {
    for tier in 1u32..=5 {
        let script = include_str!("../../../../../examples/first-step-pact-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut request = crate::dsl::compiler::compile_scenario(&script, Default::default())
            .unwrap()
            .request;
        request.actors[0].static_stats.base_atk = 1000;
        request.actors[0].static_stats.atk = Ratio {
            numerator: 2000,
            denominator: 1,
        };
        request.actors[0].static_stats.crit_rate = Ratio::zero();
        let result = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            result["execution_completed"], true,
            "{}",
            result["halt_diagnostic"]
        );
        let grants: Vec<_> = result["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["event_kind"] == "noros_atk")
            .map(|row| row["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(grants, [0, 32_000_000, 52_000_000]);
        let mut boosted_ticks = 0;
        let mut expired_ticks = 0;
        let mut settlements = 0;
        for hit in result["hit_receipts"].as_array().unwrap() {
            let effect = hit["damage_effect_id"].as_str().unwrap();
            let coefficient = match effect {
                "GE_Player_Kuhara_Seed_Damage" => 15,
                "GE_Player_Kuhara_BudEnd_Damage" => {
                    settlements += 1;
                    100
                }
                _ => continue,
            };
            let time = hit["time_us"].as_u64().unwrap();
            let boosted = grants
                .iter()
                .any(|start| time >= *start && time < start + 10_000_000);
            if boosted {
                boosted_ticks += 1;
            } else {
                expired_ticks += 1;
            }
            // ATK_static2000 + base1000 * [.12,.13,.14,.15,.16], never 2000*(1+p).
            let attack = 2000
                + if boosted {
                    10 * (11 + u128::from(tier))
                } else {
                    0
                };
            let expected = ExactFraction::new(attack * coefficient, 100).unwrap();
            let measured: ExactFraction =
                serde_json::from_value(hit["non_crit_damage"].clone()).unwrap();
            assert_eq!(measured, expected, "T{tier}, time {time}, {effect}");
        }
        assert_eq!(settlements, 4);
        assert!(boosted_ticks > 0 && expired_ticks > 0);
        assert_eq!(result["final_buff_state"]["active_actor_index"], 1);
    }
}
