use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/worldrain-cycle-intensity.nte");

#[test]
fn worldrain_invalid_ci_and_duration_fail_before_execution() {
    let req = crate::dsl::compiler::compile_scenario(SOURCE, Default::default())
        .unwrap()
        .request;
    for (duration, denominator) in [(0, 1), (15_000_000, 0), (u64::MAX, 1)] {
        let mut invalid = req.clone();
        let GearProcConfig::WorldrainDamage {
            duration_us,
            cycle_intensity_bonus,
            ..
        } = &mut invalid.actors[0].gear_procs[0]
        else {
            unreachable!()
        };
        *duration_us = duration;
        cycle_intensity_bonus.denominator = denominator;
        assert!(run_game_scenario(&invalid, &ExecutionLimits::default()).is_err());
    }
}
fn exact(v: &Value) -> ExactFraction {
    serde_json::from_value(v.clone()).unwrap()
}

#[test]
fn worldrain_native_refinement_ci_refresh_changes_real_blossom_ownership_and_damage() {
    for tier in 1u32..=5 {
        for expired in [false, true] {
            let mut source = SOURCE.replace("refine=1", &format!("refine={tier}"));
            if expired {
                // The 15s buff from the 26.65s Skill is half-open.
                source = source.replace("wait_until(33)", "wait_until(41.65)");
            }
            let req = crate::dsl::compiler::compile_scenario(&source, Default::default())
                .unwrap()
                .request;
            let GearProcConfig::WorldrainDamage {
                damage_bonus,
                cycle_intensity_bonus,
                duration_us,
            } = req.actors[0].gear_procs[0]
            else {
                panic!("Worldrain projection missing");
            };
            assert_eq!(
                ExactFraction::from_ratio(damage_bonus).unwrap(),
                ExactFraction::new(u128::from(27 + 3 * tier), 100).unwrap()
            );
            assert_eq!(
                ExactFraction::from_ratio(cycle_intensity_bonus).unwrap(),
                ExactFraction::new(u128::from(27 + 9 * tier), 1).unwrap()
            );
            assert_eq!(duration_us, 15_000_000);
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{}",
                report["rejected_action"]
            );
            let mut control = req.clone();
            let GearProcConfig::WorldrainDamage {
                cycle_intensity_bonus,
                ..
            } = &mut control.actors[0].gear_procs[0]
            else {
                unreachable!()
            };
            *cycle_intensity_bonus = Ratio::zero();
            let plain = run_game_scenario(&control, &ExecutionLimits::default()).unwrap();
            let reactions = |report: &Value| -> Vec<Value> {
                report["hit_receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|hit| {
                        hit["damage_effect_id"]
                            .as_str()
                            .unwrap()
                            .starts_with("ntesim.reaction.blossom")
                    })
                    .cloned()
                    .collect()
            };
            let hits = reactions(&report);
            let bare = reactions(&plain);
            assert_eq!(hits.len(), 15);
            assert_eq!(bare.len(), 15);
            for (index, (hit, base)) in hits.iter().zip(&bare).enumerate() {
                let boosted = index < 10 || !expired;
                let factor = if boosted {
                    ExactFraction::new(u128::from(627 + 9 * tier), 600).unwrap()
                } else {
                    ExactFraction::new(1, 1).unwrap()
                };
                assert_eq!(
                    exact(&hit["non_crit_damage"]),
                    exact(&base["non_crit_damage"]).mul(factor).unwrap(),
                    "T{tier}, packet{index}, expired={expired}"
                );
                assert_eq!(hit["actor_index"], if boosted { 0 } else { 2 });
            }
            assert!(
                crate::simulation::tests::support::final_buffs(&report)
                    .iter()
                    .all(|buff| buff["id"] != "worldrain.cycle_intensity")
            );
            let replay: GameScenarioRequest =
                serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
            assert_eq!(
                report,
                run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
            );
        }
    }
}
