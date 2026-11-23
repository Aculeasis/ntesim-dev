use super::*;
use serde_json::json;

#[test]
fn danzaburou_native_break_cycles_bind_refinement_aura_roster_gate_and_living_carrier() {
    for (tier, aura) in [(1, 70), (2, 84), (3, 98), (4, 112), (5, 126)] {
        for (qualified, defeated) in [(true, false), (false, false), (true, true)] {
            let mut source = include_str!("../../../../../examples/danzaburou-break-cycles.nte")
                .replace("refine=1", &format!("refine={tier}"));
            if !qualified {
                source = source.replace("aurelia char", "zero char");
            }
            if defeated {
                // Fadia's selected sharing model preserves her last HP while
                // allies live. Use a legal Incantation party and Adler carrier
                // for the actual-death counterexample instead of disabling it.
                source = source
                    .replace("fadia", "adler")
                    .replace("haniel", "sakiri")
                    .replace("aurelia", "baicang");
                source = source.replace(
                    "active sakiri;",
                    "incoming adler at=5 damage=100000000; active sakiri;",
                );
            }
            let req = live_source_request(&source);
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{}",
                report["rejected_action"]
            );
            let breaks: Vec<_> = report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["break_contribution"].is_object())
                .collect();
            if defeated {
                assert_eq!(report["final_actor_states"][0]["hp"], 0);
            }
            assert_eq!(breaks.len(), if defeated { 10 } else { 12 });
            for hit in breaks {
                let owner = hit["actor_index"].as_u64().unwrap() as usize;
                let time = hit["time_us"].as_u64().unwrap();
                let matching = owner < 2 || (owner == 2 && qualified);
                let bonus = if qualified && matching && (!defeated || time < 5_000_000) {
                    aura
                } else {
                    0
                };
                // The weapon's secondary stat is 120 BI points, independent
                // of the 70..126 point aura; the Anima peer gets neither.
                let bi = bonus + if owner == 0 { 120 } else { 0 };
                assert_eq!(
                    hit["break_contribution"]["intensity"],
                    json!(format!("{}/{}", bi, 1))
                );
                // 3603 * original capacity.1/3 * (1 + BI/300).
                let expected = ExactFraction::new(1201 * (300 + bi), 3000).unwrap();
                assert_eq!(hit["damage"], serde_json::to_value(expected).unwrap());
            }
            assert_eq!(report["final_buff_state"]["active_actor_index"], 3);
            assert_eq!(
                report["trace"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["event_kind"] == "target_break_recovered")
                    .count(),
                3
            );
        }
    }
}
