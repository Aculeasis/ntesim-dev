use super::*;
use serde_json::json;

#[test]
fn rose_native_redirect_dot_and_break_extension_keep_two_live_refinement_cycles() {
    for (tier, atk, thorn) in [
        (1, 1140, 60),
        (2, 1175, 75),
        (3, 1210, 90),
        (4, 1245, 105),
        (5, 1280, 120),
    ] {
        let source = include_str!("../../../../../examples/bohemian-rose-break-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        req.actors[0].static_stats.base_atk = 1000;
        req.actors[0].static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
        req.actors[0].static_stats.damage_bonus = Ratio::zero();
        req.actors[0].static_stats.elemental_bonus = Ratio::zero();
        req.actors[0].static_stats.elemental_bonuses.clear();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let ticks: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_Blood_Damage")
            .collect();
        assert_eq!(ticks.len(), 56);
        for cycle in ticks.as_chunks::<28>().0 {
            for (index, pair) in cycle.as_chunks::<2>().0.iter().enumerate() {
                // Five initial stacks decay by one at each 3s boundary:
                // 5,5,4,4,4,3,3,3,2,2,2,1,1,1. Native level1 ratio=.06.
                let stacks = [5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 1, 1, 1][index];
                let noncrit = ExactFraction::new(atk * 6 * stacks, 100).unwrap();
                let critical = noncrit
                    .mul(ExactFraction::new(100 + thorn, 100).unwrap())
                    .unwrap();
                for (target, hit) in pair.iter().enumerate() {
                    assert_eq!(hit["target_index"], target);
                    assert_eq!(hit["actor_index"], 0);
                    assert_eq!(
                        hit["non_crit_damage"],
                        serde_json::to_value(noncrit).unwrap()
                    );
                    assert_eq!(hit["crit_damage"], serde_json::to_value(critical).unwrap());
                }
            }
        }
        let trace = report["trace"].as_array().unwrap();
        let extensions: Vec<_> = trace
            .iter()
            .filter(|e| e["event_kind"] == "target_broken_duration_extended")
            .collect();
        let recoveries: Vec<_> = trace
            .iter()
            .filter(|e| e["event_kind"] == "target_break_recovered")
            .collect();
        assert_eq!(extensions.len(), 4);
        assert_eq!(recoveries.len(), 4);
        for cycle in 0..2 {
            let first_tick = ticks[cycle * 28]["time_us"].as_u64().unwrap();
            for target in 0..2 {
                // Original two-second recovery plus one three-second extension.
                assert_eq!(
                    recoveries[cycle * 2 + target]["time_us"],
                    first_tick + 4_000_000
                );
                assert_eq!(extensions[cycle * 2 + target]["time_us"], first_tick);
            }
        }
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
        assert!(
            crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .all(|buff| buff["id"] != crate::weapons::rose::THORN_BUFF_ID)
        );
        assert_eq!(report["final_actor_states"][0]["spent_energy"], json!(0));
    }
}
