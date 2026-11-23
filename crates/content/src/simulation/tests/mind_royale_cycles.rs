use super::*;
use serde_json::json;

#[test]
fn mind_royale_native_mint_cycles_keep_icd_spending_overflow_and_peer_energy_at_each_rank() {
    for (tier, amount) in [(1, 10), (2, 12), (3, 14), (4, 16), (5, 18)] {
        let source = include_str!("../../../../../examples/mind-royale-energy-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let result = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
        super::assert_native_replay(&result);
        let req = result.request;
        assert_eq!(
            serde_json::to_value(req.actors[0].static_stats.break_intensity).unwrap(),
            json!("60/1")
        );
        let report = serde_json::to_value(result.report).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "mind_royale_energy")
            .collect();
        assert_eq!(grants.len(), 3);
        for (event, time) in grants.iter().zip([0, 20_000_000, 40_000_000]) {
            assert_eq!(
                support::world_time_at(&report, event["time_us"].as_u64().unwrap()),
                time
            );
            let grant = &event["energy_change"];
            assert_eq!(grant["source_actor_index"], 0);
            assert_eq!(grant["recipient_actor_index"], 0);
            assert_eq!(grant["gross"], amount);
            assert_eq!(grant["charge_basis"], json!(format!("{}/{}", amount, 1)));
        }
        assert_eq!(grants[0]["energy_change"]["retained"], 0);
        assert_eq!(grants[0]["energy_change"]["overflow"], amount);
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
        let breaks: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["actor_index"] == 0 && hit.get("break_contribution").is_some())
            .collect();
        assert!(!breaks.is_empty());
        // Level80 base3603 * (1 + BI60/300) * capacity30/3, zero DEF/RES.
        for hit in breaks {
            assert_eq!(hit["damage"], json!("43236/1"));
        }
        assert!(
            report["final_actor_states"][1]["granted_energy"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(
            report,
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
        );
    }
}
