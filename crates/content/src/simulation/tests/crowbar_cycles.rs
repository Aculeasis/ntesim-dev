use super::*;
use serde_json::json;

#[test]
fn crowbar_native_redirect_cycles_capture_live_self_bi_without_refresh_or_break_bonus_leakage() {
    for (tier, intensity) in [(1, 90), (2, 108), (3, 126), (4, 144), (5, 162)] {
        let source = include_str!("../../../../../examples/time-bandit-break-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let (script, mut config) = crate::dsl::inspection::prepare(&source).unwrap();
        for compiled in &mut config.actors {
            let actor = &mut compiled.actor;
            actor.static_stats.crit_rate = Ratio::zero();
            actor.static_stats.damage_bonus = Ratio {
                numerator: 9,
                denominator: 1,
            };
            compiled.projection.static_stats = actor.static_stats.clone();
        }
        let result = crate::simulation::PreparedSourceScenario::from_configuration(
            &source,
            config,
            Default::default(),
            &ExecutionLimits::default(),
            &script.options[0].loc,
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        super::assert_native_replay(&result);
        let report = serde_json::to_value(result.report).unwrap();
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
        assert_eq!(
            breaks.len(),
            12,
            "break times {:?}",
            breaks.iter().map(|h| &h["time_us"]).collect::<Vec<_>>()
        );
        let mut boosted = 0;
        let mut expired = 0;
        for hit in breaks {
            let time = hit["time_us"].as_u64().unwrap();
            let active = time < 50_000_000 || (52_000_000..102_000_000).contains(&time);
            let bi = if hit["actor_index"] == 0 && active {
                intensity
            } else {
                0
            };
            assert_eq!(
                hit["break_contribution"]["intensity"],
                json!(format!("{}/{}", bi, 1))
            );
            // Lv80 base3603 * original capacity.1/3 * (1 + BI/300).
            // The ordinary +900% damage zone must never enter Break HP.
            let expected = ExactFraction::new(1201 * (300 + bi), 3000).unwrap();
            assert_eq!(hit["damage"], serde_json::to_value(expected).unwrap());
            if hit["actor_index"] == 0 {
                if active {
                    boosted += 1;
                } else {
                    expired += 1;
                }
            }
        }
        // The four Skills break during active windows; the two authored
        // attacks after expiry break without the weapon's intensity.
        assert_eq!((boosted, expired), (4, 2));
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["event_kind"] == "target_break_recovered")
                .count(),
            6
        );
    }
}
