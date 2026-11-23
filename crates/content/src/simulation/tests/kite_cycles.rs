use super::*;

#[test]
fn kite_overlapping_stain_and_remora_add_one_lakshana_bonus() {
    let (header, _) = include_str!("../../../../../examples/kite-marked-cycles.nte")
        .split_once("active aurelia;")
        .unwrap();
    let source = format!(
        "{header} active aurelia; skia swap; wait_until(0.6); skia skill; wait(1.5); zero swap; wait(0.6); skia swap; wait(0.35); skia attack;"
    );
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    req.actors[1].static_stats.base_atk = 1000;
    req.actors[1].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[1].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    req.actors[1].static_stats.elemental_bonus = Ratio::zero();
    req.actors[1].static_stats.elemental_bonuses.clear();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    let index = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .rposition(|hit| hit["actor_index"] == 1)
        .unwrap();
    let time = report["hit_receipts"][index]["time_us"].as_u64().unwrap();
    assert!(time < 5_000_000);
    for kind in ["reaction_stain_applied", "reaction_remora_applied"] {
        assert!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == kind
                    && event["time_us"].as_u64().unwrap() < time)
        );
    }
    req.actors[1].gear_procs.clear();
    let plain = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let damage: ExactFraction =
        serde_json::from_value(plain["hit_receipts"][index]["non_crit_damage"].clone()).unwrap();
    assert_eq!(
        report["hit_receipts"][index]["non_crit_damage"],
        serde_json::to_value(damage.mul(ExactFraction::new(88, 75).unwrap()).unwrap()).unwrap()
    );
}

#[test]
fn kite_continuing_native_skia_cycles_observe_live_stain_remora_and_expiry_at_all_ranks() {
    for (tier, atk_n, atk_d, marked_n, marked_d) in [
        (1, 11, 10, 88, 75),
        (2, 28, 25, 756, 625),
        (3, 57, 50, 779, 625),
        (4, 29, 25, 2407, 1875),
        (5, 59, 50, 826, 625),
    ] {
        let source = include_str!("../../../../../examples/kite-marked-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        for actor in &mut req.actors {
            actor.static_stats.base_atk = 1000;
            actor.static_stats.atk = Ratio {
                numerator: 1000,
                denominator: 1,
            };
            actor.static_stats.crit_rate = Ratio::zero();
            actor.static_stats.damage_bonus = Ratio {
                numerator: 1,
                denominator: 2,
            };
            actor.static_stats.elemental_bonus = Ratio::zero();
            actor.static_stats.elemental_bonuses.clear();
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let mut bare = req.clone();
        bare.actors[1].gear_procs.clear();
        let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
        let own = |value: &serde_json::Value| {
            value["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["actor_index"] == 1)
                .cloned()
                .collect::<Vec<_>>()
        };
        let actual = own(&report);
        let plain = own(&baseline);
        assert_eq!(actual.len(), plain.len());
        let mut witnessed = [false; 3];
        for (hit, base) in actual.iter().zip(&plain) {
            let time = hit["time_us"].as_u64().unwrap();
            let window =
                (600_000..15_600_000).contains(&time) || (20_600_000..35_600_000).contains(&time);
            let marked = time < 12_000_000 || (20_000_000..25_000_000).contains(&time);
            let (n, d, index) = if !window {
                (1, 1, 0)
            } else if marked {
                (marked_n, marked_d, 2)
            } else {
                (atk_n, atk_d, 1)
            };
            witnessed[index] = true;
            let value: ExactFraction =
                serde_json::from_value(base["non_crit_damage"].clone()).unwrap();
            assert_eq!(
                hit["non_crit_damage"],
                serde_json::to_value(value.mul(ExactFraction::new(n, d).unwrap()).unwrap())
                    .unwrap(),
                "tier {tier} at {time}"
            );
        }
        assert_eq!(witnessed, [true; 3]);
        assert_eq!(report["final_actor_states"], baseline["final_actor_states"]);
        for kind in ["reaction_stain_applied", "reaction_remora_applied"] {
            assert!(
                report["trace"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|event| event["event_kind"] == kind)
            );
        }
    }
}
