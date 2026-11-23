use super::*;
use serde_json::{Value, json};

#[test]
fn mamen_native_cycles_keep_fons_separate_from_grain_and_bonus_on_the_wearer() {
    // Two stacks add to the authored 1/10 damage zone, never multiply it.
    for (tier, numerator, denominator) in [
        (1, 23, 22),
        (2, 58, 55),
        (3, 117, 110),
        (4, 59, 55),
        (5, 119, 110),
    ] {
        let source = include_str!("../../../../../examples/contemplative-cat-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let req = live_source_request(&source);
        let mut zero_fons = req.clone();
        zero_fons.start_fons = Some(0);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let baseline = run_game_scenario(&zero_fons, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_eq!(baseline["execution_completed"], true);
        assert_eq!(req.start_fons, Some(250_000));
        assert_eq!(report["final_actor_states"][1]["spent_energy"], 200);
        // Two real five-stage Chiz chains grant 2 * 5 * 450 Grain.
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
            json!("4500/1")
        );
        assert_eq!(report["final_actor_states"], baseline["final_actor_states"]);
        let hits = report["hit_receipts"].as_array().unwrap();
        let bare = baseline["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), bare.len());
        let mut wearer_cosmos = 0;
        let mut peer_cosmos = 0;
        for (hit, base) in hits.iter().zip(bare) {
            assert_eq!(hit["time_us"], base["time_us"]);
            assert_eq!(hit["actor_index"], base["actor_index"]);
            let cosmos =
                super::support::hit_metadata(&report, hit)["packet"]["element"] == "cosmos";
            if hit["actor_index"] == 0 && cosmos {
                wearer_cosmos += 1;
                let fraction = |value: &Value| {
                    ExactFraction::from_ratio(serde_json::from_value(value.clone()).unwrap())
                        .unwrap()
                };
                assert_eq!(
                    fraction(&hit["non_crit_damage"]),
                    fraction(&base["non_crit_damage"])
                        .mul(ExactFraction::new(numerator, denominator).unwrap())
                        .unwrap()
                );
            } else {
                if hit["actor_index"] == 1 && cosmos {
                    peer_cosmos += 1;
                }
                assert_eq!(hit["non_crit_damage"], base["non_crit_damage"]);
            }
        }
        assert!(wearer_cosmos > 0 && peer_cosmos > 0);
        assert_eq!(
            report,
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
        );
    }
}
